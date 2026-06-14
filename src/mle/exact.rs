//! Exact set-algebra decomposition of the joint sketch when every operand is still a hash list.
//!
//! A hash list stores a near-exact sample of the element hashes (the composite hash encodes the
//! register index, the rank, and leading bits of the original hash), so the disjoint-cell
//! cardinalities can be counted directly from the stored hashes instead of being reconstructed from
//! register collisions by MLE. This mirrors the crate's existing hash-list union (the `(true, true)`
//! branch of [`HyperLogLog::estimate_union_cardinality`] and of `merge`): two hashes are the same
//! element iff their encodings are equal once both are downgraded to a common hash size. Accuracy is
//! near-exact at high hash sizes and degrades gracefully toward `SMALLEST_VIABLE_HASH_BITS`, where a
//! composite hash is essentially just `(index, rank)`.

use crate::composite_hash::GapHash;
use crate::prelude::*;
use crate::utils::Zero;
use std::collections::HashMap;

/// Exact joint sketch over the disjoint-region model, assuming all counters are in hash-list mode.
///
/// Returns `(overlap[M][N], left_diff[M], right_diff[N])` with the same cell layout as
/// [`super::sketch::joint_sketch_mle_from_registers`]: `overlap[i][j] = |L_i intersect R_j|`,
/// `left_diff[i] = |L_i \ B_{N-1}|`, `right_diff[j] = |R_j \ A_{M-1}|`, where `L_i = A_i \ A_{i-1}`
/// and `R_j = B_j \ B_{j-1}` are the left/right shells of the nested inputs.
///
/// The cells are raw distinct-hash counts: each distinct downgraded composite hash is assigned to
/// the smallest left shell that contains it and the smallest right shell that contains it, then
/// classified into exactly one cell. They are non-negative and disjoint by construction, so they sum
/// to the distinct-hash count of `A_{M-1} union B_{N-1}`.
pub(crate) fn joint_sketch_exact_from_hash_lists<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
    const M: usize,
    const N: usize,
>(
    lefts: &[HyperLogLog<P, B, R, H>; M],
    rights: &[HyperLogLog<P, B, R, H>; N],
) -> ([[f64; N]; M], [f64; M], [f64; N]) {
    debug_assert!(
        lefts.iter().all(HyperLogLog::is_hash_list) && rights.iter().all(HyperLogLog::is_hash_list),
        "joint_sketch_exact_from_hash_lists requires every operand to be a hash list",
    );

    // The common hash size is the minimum across all operands: a stored hash can only be downgraded,
    // never upgraded, so all hashes must be brought down to the coarsest level to be comparable.
    let common = lefts
        .iter()
        .chain(rights.iter())
        .map(|counter| counter.get_hash_bits().unwrap())
        .min()
        .unwrap();

    // Per distinct downgraded hash, the 1-based smallest containing left and right shell indices
    // (0 means the hash is absent from that side). Because the inputs are nested, the smallest index
    // that contains a hash is its shell index, and the first sighting over increasing index wins.
    let mut membership: HashMap<u32, (u8, u8)> = HashMap::new();
    for (i, left) in lefts.iter().enumerate() {
        let shell = (i + 1) as u8;
        let hash_bits = left.get_hash_bits().unwrap();
        for encoded_hash in GapHash::<P, B>::downgraded(
            left.registers.as_ref(),
            left.get_number_of_hashes().unwrap(),
            hash_bits,
            left.get_writer_tell(),
            hash_bits - common,
        ) {
            let entry = membership.entry(encoded_hash).or_insert((0, 0));
            if entry.0 == 0 {
                entry.0 = shell;
            }
        }
    }
    for (j, right) in rights.iter().enumerate() {
        let shell = (j + 1) as u8;
        let hash_bits = right.get_hash_bits().unwrap();
        for encoded_hash in GapHash::<P, B>::downgraded(
            right.registers.as_ref(),
            right.get_number_of_hashes().unwrap(),
            hash_bits,
            right.get_writer_tell(),
            hash_bits - common,
        ) {
            let entry = membership.entry(encoded_hash).or_insert((0, 0));
            if entry.1 == 0 {
                entry.1 = shell;
            }
        }
    }

    let mut overlap = [[f64::ZERO; N]; M];
    let mut left_diff = [f64::ZERO; M];
    let mut right_diff = [f64::ZERO; N];
    for &(left_shell, right_shell) in membership.values() {
        match (left_shell, right_shell) {
            (0, 0) => unreachable!("every recorded hash belongs to at least one side"),
            (li, 0) => left_diff[usize::from(li) - 1] += 1.0,
            (0, rj) => right_diff[usize::from(rj) - 1] += 1.0,
            (li, rj) => overlap[usize::from(li) - 1][usize::from(rj) - 1] += 1.0,
        }
    }

    (overlap, left_diff, right_diff)
}
