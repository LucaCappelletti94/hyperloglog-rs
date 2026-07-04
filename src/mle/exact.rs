//! Exact set-algebra decomposition of the joint sketch when every operand is still a sorted value
//! list. A value list stores its elements verbatim, so each distinct literal value is classified
//! into exactly one disjoint cell with no hashing and no collisions, giving truly exact cells. (The
//! all-hash-list case is NOT handled here: routing it to the raw distinct-hash decomposition drifts
//! 15-30% once the common hash size narrows, so `joint_sketch_mle` sends hash-list operands to the
//! corrected, allocation-free inclusion-exclusion path instead.)
//!
//! Because every value list is sorted and the inputs are nested (`A_0 subseteq A_1 subseteq ...`),
//! the decomposition is a single `M + N`-way merge over the lists' iterators: no map, no heap
//! allocation. `ValueIter` yields values in DESCENDING order, so the merge repeatedly takes the
//! largest current head. For each distinct value we read its left shell (the smallest left list that
//! contains it) and right shell directly off the merge cursors.

use crate::prelude::*;
use crate::utils::Zero;
use sketching_core::sparse_value_list::{SparseValueCodec, ValueIter, BE};

/// Exact joint sketch when every operand is in the sorted value list: classify each distinct literal
/// value directly (no hashing, no collisions), giving truly exact disjoint cells. `overlap[i][j] =
/// |L_i intersect R_j|`, `left_diff[i] = |L_i \ B_{N-1}|`, `right_diff[j] = |R_j \ A_{M-1}|`, where
/// `L_i = A_i \ A_{i-1}` and `R_j = B_j \ B_{j-1}` are the left/right shells of the nested inputs.
pub(crate) fn joint_sketch_exact_from_values<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
    C: SparseValueCodec,
    const M: usize,
    const N: usize,
>(
    lefts: &[HyperLogLog<P, B, R, H, C>; M],
    rights: &[HyperLogLog<P, B, R, H, C>; N],
) -> JointSketch<M, N> {
    debug_assert!(
        lefts.iter().all(HyperLogLog::is_sorted_value_list)
            && rights.iter().all(HyperLogLog::is_sorted_value_list),
        "joint_sketch_exact_from_values requires every operand to be in sorted value list",
    );

    // One descending cursor per input list, with its current head value buffered. The lists are
    // sorted, so this is a standard multi-way merge over `M + N` streams (largest value first).
    let mut left_iters: [ValueIter<'_, BE, C>; M] = core::array::from_fn(|i| {
        ValueIter::<BE, _>::new(
            lefts[i].registers.as_ref(),
            0,
            lefts[i].get_number_of_values(),
            C::default(),
        )
    });
    let mut right_iters: [ValueIter<'_, BE, C>; N] = core::array::from_fn(|j| {
        ValueIter::<BE, _>::new(
            rights[j].registers.as_ref(),
            0,
            rights[j].get_number_of_values(),
            C::default(),
        )
    });
    let mut left_head: [Option<u64>; M] = core::array::from_fn(|i| left_iters[i].next());
    let mut right_head: [Option<u64>; N] = core::array::from_fn(|j| right_iters[j].next());

    let mut overlap = [[f64::ZERO; N]; M];
    let mut left_diff = [f64::ZERO; M];
    let mut right_diff = [f64::ZERO; N];

    loop {
        // The cursors descend, so the next distinct value is the largest head across all live ones.
        let mut value: Option<u64> = None;
        for head in left_head.iter().chain(right_head.iter()).copied().flatten() {
            value = Some(value.map_or(head, |current| current.max(head)));
        }
        let Some(value) = value else { break };

        // The left shell of `value` is the smallest left list that contains it. The inputs are
        // nested, so every left list from that index onward also contains it; advance all matching
        // cursors. The right shell is read the same way. A shell index of 0 means "in no list on
        // that side".
        let mut left_shell = 0u8;
        for i in 0..M {
            if left_head[i] == Some(value) {
                if left_shell == 0 {
                    left_shell = (i + 1) as u8;
                }
                left_head[i] = left_iters[i].next();
            }
        }
        let mut right_shell = 0u8;
        for j in 0..N {
            if right_head[j] == Some(value) {
                if right_shell == 0 {
                    right_shell = (j + 1) as u8;
                }
                right_head[j] = right_iters[j].next();
            }
        }

        match (left_shell, right_shell) {
            (0, 0) => unreachable!("every merged value belongs to at least one side"),
            (li, 0) => left_diff[usize::from(li) - 1] += 1.0,
            (0, rj) => right_diff[usize::from(rj) - 1] += 1.0,
            (li, rj) => overlap[usize::from(li) - 1][usize::from(rj) - 1] += 1.0,
        }
    }

    JointSketch {
        overlap,
        left_diff,
        right_diff,
    }
}
