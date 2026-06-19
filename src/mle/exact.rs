//! Exact set-algebra decomposition of the joint sketch when every operand is still a sorted value
//! list. A value list stores its elements verbatim, so each distinct literal value is classified
//! into exactly one disjoint cell with no hashing and no collisions, giving truly exact cells. (The
//! all-hash-list case is NOT handled here: routing it to the raw distinct-hash decomposition drifts
//! 15-30% once the common hash size narrows, so `joint_sketch_mle` sends hash-list operands to the
//! corrected, allocation-free inclusion-exclusion path instead.)

use super::PatternMap;
use crate::prelude::*;
use crate::utils::Zero;

/// Exact joint sketch when every operand is in the sorted value list: classify each distinct literal
/// value directly (no hashing, no collisions), giving truly exact disjoint cells. `overlap[i][j] =
/// |L_i intersect R_j|`, `left_diff[i] = |L_i \ B_{N-1}|`, `right_diff[j] = |R_j \ A_{M-1}|`, where
/// `L_i = A_i \ A_{i-1}` and `R_j = B_j \ B_{j-1}` are the left/right shells of the nested inputs.
pub(crate) fn joint_sketch_exact_from_values<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
    const M: usize,
    const N: usize,
>(
    lefts: &[HyperLogLog<P, B, R, H>; M],
    rights: &[HyperLogLog<P, B, R, H>; N],
) -> JointSketch<M, N> {
    use crate::composite_hash::gaps::value_list::ValueIter;

    debug_assert!(
        lefts.iter().all(HyperLogLog::is_sorted_value_list)
            && rights.iter().all(HyperLogLog::is_sorted_value_list),
        "joint_sketch_exact_from_values requires every operand to be in sorted value list",
    );

    let mut membership: PatternMap<u64, (u8, u8)> = PatternMap::new();
    for (i, left) in lefts.iter().enumerate() {
        let shell = (i + 1) as u8;
        for value in ValueIter::new(left.registers.as_ref(), left.get_number_of_values()) {
            let entry = membership.entry(value).or_insert((0, 0));
            if entry.0 == 0 {
                entry.0 = shell;
            }
        }
    }
    for (j, right) in rights.iter().enumerate() {
        let shell = (j + 1) as u8;
        for value in ValueIter::new(right.registers.as_ref(), right.get_number_of_values()) {
            let entry = membership.entry(value).or_insert((0, 0));
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
            (0, 0) => unreachable!("every recorded value belongs to at least one side"),
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
