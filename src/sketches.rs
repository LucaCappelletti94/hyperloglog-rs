//! `HyperLogLog` sketching algorithms.
//!
//! Re-exports [`JointSketch`], [`JointSketchError`], and [`HyperSpheresSketch`] from
//! [`sketching_core`] and provides the HyperLogLog-specific [`HyperSpheresSketch`] implementation
//! which normalizes operand representations before computing the joint sketch.

pub use sketching_core::{HyperSpheresSketch, JointSketch, JointSketchError};

use crate::prelude::{Bits, HasherType, HyperLogLog, Precision, Registers};

/// Wires `HyperLogLog` to the approximate sketching algorithms. The required cardinality and union
/// estimators come from its [`CardinalityEstimator`](crate::estimator::CardinalityEstimator) implementation.
impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperSpheresSketch
    for HyperLogLog<P, B, R, H>
{
    #[inline]
    fn joint_sketch<const L: usize, const N: usize>(
        lefts: &[Self; L],
        rights: &[Self; N],
    ) -> JointSketch<L, N> {
        // If the nested operands span different representations (some still hash lists, some already
        // registers, as happens when a chain straddles the saturation transition), the differential
        // decomposition mixes near-exact and probabilistic cell estimates and amplifies the mismatch
        // into a spike. Materialize everything to registers first so all cells are consistent. This
        // is a no-op once every operand is already dense, and is skipped entirely while they are all
        // still hash lists (where the set algebra is near-exact).
        let any_dense = lefts
            .iter()
            .chain(rights.iter())
            .any(super::hyperloglog::HyperLogLog::is_hyperloglog);
        if any_dense {
            let lefts: [Self; L] = core::array::from_fn(|i| lefts[i].clone().into_hll());
            let rights: [Self; N] = core::array::from_fn(|j| rights[j].clone().into_hll());
            sketching_core::inclusion_exclusion_joint_sketch(&lefts, &rights)
        } else {
            sketching_core::inclusion_exclusion_joint_sketch(lefts, rights)
        }
    }
}

#[cfg(test)]
mod normalize_tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_normalize_known_cells() {
        let s = JointSketch::<2, 2> {
            overlap: [[40.0, 10.0], [20.0, 30.0]],
            left_diff: [100.0, 50.0],
            right_diff: [80.0, 60.0],
        };
        let n = s.normalize();
        for v in n
            .overlap
            .iter()
            .flatten()
            .chain(&n.left_diff)
            .chain(&n.right_diff)
        {
            assert!((0.0..=1.0).contains(v), "fraction out of range: {v}");
        }
        let close = |a: f64, b: f64| (a - b).abs() < 1e-12;
        assert!(close(n.overlap[0][0], 40.0 / 250.0));
        assert!(close(n.overlap[1][1], 30.0 / 150.0));
        assert!(close(n.left_diff[0], 100.0 / 150.0));
        assert!(close(n.left_diff[1], 50.0 / 100.0));
        assert!(close(n.right_diff[0], 80.0 / 140.0));
        assert!(close(n.right_diff[1], 60.0 / 100.0));
    }

    #[test]
    fn test_normalize_matches_normalized_joint_sketch() {
        type Hll = HyperLogLog<Precision10, Bits6>;
        let build = |ranges: &[core::ops::Range<u64>]| {
            let mut h = Hll::default();
            for r in ranges {
                for v in r.clone() {
                    h.insert(&v);
                }
            }
            h
        };
        let a0 = build(&[0..30]);
        let a1 = build(&[0..30, 30..55, 100..115]);
        let b0 = build(&[20..50]);
        let b1 = build(&[20..50, 30..70, 200..210]);
        assert!(!a1.is_hyperloglog() && !b1.is_hyperloglog());
        let lefts = [a0, a1];
        let rights = [b0, b1];

        let reconstructed = Hll::joint_sketch(&lefts, &rights).normalize();
        let reference = Hll::normalized_joint_sketch(&lefts, &rights);
        let close = |a: f64, b: f64| (a - b).abs() <= 1e-5 * a.abs().max(b.abs()).max(1.0);
        for i in 0..2 {
            for j in 0..2 {
                assert!(
                    close(reconstructed.overlap[i][j], reference.overlap[i][j]),
                    "overlap[{i}][{j}]: {} vs {}",
                    reconstructed.overlap[i][j],
                    reference.overlap[i][j]
                );
            }
            assert!(close(reconstructed.left_diff[i], reference.left_diff[i]));
            assert!(close(reconstructed.right_diff[i], reference.right_diff[i]));
        }
    }
}
