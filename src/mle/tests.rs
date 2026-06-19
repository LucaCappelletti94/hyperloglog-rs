//! Tests for the MLE estimators: the 2-set union MLE's Adam optimizer and the joint sketch built by
//! inclusion-exclusion over the 2-set union MLE.

use super::union::*;
use crate::prelude::*;

// Test function: f(x) = -(x1 - 1)^2 - (x2 + 2)^2
fn quadratic_function(phis: &[f64; 2]) -> (f64, [f64; 2]) {
    let value = -(phis[0] - 1.0).powi(2) - (phis[1] + 2.0).powi(2);
    let gradients = [-2.0 * (phis[0] - 1.0), -2.0 * (phis[1] + 2.0)];
    (value, gradients)
}

#[test]
fn test_adam_optimizer() {
    let mut phis = [0.0, 0.0]; // Initial guess
    let mut adam = ArrayAdam::<2>::default();

    for _ in 0..10_000 {
        let (_value, gradients) = quadratic_function(&phis);
        adam.apply(&mut gradients.clone(), &mut phis);
    }

    assert!((phis[0] - 1.0).abs() < 1e-4);
    assert!((phis[1] + 2.0).abs() < 1e-4);
}

/// The `M = N = 1` joint sketch over MLE views must recover the union, intersection and differences
/// of two overlapping sets through the 2-set union MLE.
#[test]
fn test_joint_sketch_mle_2set_recovers_regions() {
    type Hll = HyperLogLog<Precision12, Bits6>;
    let mut a = Hll::default();
    let mut b = Hll::default();
    for x in 0u64..30_000 {
        a.insert(&x); // A = [0, 30000)
    }
    for x in 20_000u64..50_000 {
        b.insert(&x); // B = [20000, 50000): union 50000, intersection 10000
    }

    let (overlap, left_diff, right_diff) =
        JointSketch::estimate(&[a.mle()], &[b.mle()]).into_parts();
    let union = overlap[0][0] + left_diff[0] + right_diff[0];

    assert!((union - 50_000.0).abs() / 50_000.0 < 0.05, "union {union}");
    assert!(
        (overlap[0][0] - 10_000.0).abs() / 10_000.0 < 0.2,
        "intersection {}",
        overlap[0][0]
    );
    assert!(
        (left_diff[0] - 20_000.0).abs() / 20_000.0 < 0.1,
        "left difference {}",
        left_diff[0]
    );
    assert!(
        (right_diff[0] - 20_000.0).abs() / 20_000.0 < 0.1,
        "right difference {}",
        right_diff[0]
    );
}

/// The `M = N = 2` nested joint sketch over MLE views must decompose into non-negative cells whose
/// sum recovers the overall union.
#[test]
fn test_joint_sketch_mle_2x2_cells_sum_to_union() {
    type Hll = HyperLogLog<Precision12, Bits6>;

    let build = |range: core::ops::Range<u64>| {
        let mut h = Hll::default();
        for x in range {
            h.insert(&x);
        }
        h
    };

    // Nested left chain A0 subset A1, nested right chain B0 subset B1.
    let a0 = build(0..10_000);
    let a1 = build(0..20_000);
    let b0 = build(5_000..15_000);
    let b1 = build(5_000..30_000);

    let sketch = JointSketch::estimate(&[a0.mle(), a1.mle()], &[b0.mle(), b1.mle()]);
    let (overlap, left_diff, right_diff) = sketch.into_parts();

    let mut total = 0.0;
    for row in &overlap {
        for &cell in row {
            assert!(cell >= 0.0, "negative overlap cell {cell}");
            total += cell;
        }
    }
    for &d in left_diff.iter().chain(right_diff.iter()) {
        assert!(d >= 0.0, "negative margin {d}");
        total += d;
    }

    // The full union is A1 | B1 = [0, 30000).
    assert!(
        (total - 30_000.0).abs() / 30_000.0 < 0.1,
        "cells sum {total} vs union 30000"
    );
}

#[cfg(test)]
mod hash_list_sketch_not_degraded {
    //! Guards that the all-hash-list MLE joint sketch stays accurate when the common hash size
    //! narrows. It is now routed to inclusion-exclusion over the corrected hash-list union estimates;
    //! the previous raw distinct-hash decomposition drifted to ~30%+ here (measured: card 1100 went
    //! from 32.2% with the exact path to 2.3% with inclusion-exclusion). Truth is the disjoint-pool
    //! construction where every one of the eight differential cells is exactly `card`.
    use crate::prelude::*;

    type Hll = HyperLogLog<Precision12, Bits6>;

    fn smix(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn pools(card: u64, seed: u64) -> [std::vec::Vec<u64>; 8] {
        let mut state = seed;
        core::array::from_fn(|_| (0..card).map(|_| smix(&mut state)).collect())
    }

    fn build(pools: &[std::vec::Vec<u64>; 8], idx: &[usize]) -> Hll {
        let mut hll = Hll::default();
        for &k in idx {
            for &v in &pools[k] {
                hll.insert(&v);
            }
        }
        hll
    }

    fn cell_error(parts: ([[f64; 2]; 2], [f64; 2], [f64; 2]), card: f64) -> f64 {
        let (overlap, left_diff, right_diff) = parts;
        let mut error = 0.0;
        for row in &overlap {
            for &cell in row {
                error += (cell - card).abs();
            }
        }
        for &margin in left_diff.iter().chain(right_diff.iter()) {
            error += (margin - card).abs();
        }
        error / (8.0 * card)
    }

    #[test]
    fn mle_hash_list_joint_sketch_stays_corrected() {
        // Several cardinalities, all in the narrow-common-width band where the exact decomposition
        // used to blow up; the routed inclusion-exclusion path must keep every config well-bounded.
        for &card in &[700u64, 900, 1100, 1250] {
            let p = pools(card, 0xD06);
            // overlap[i][j] = pool i*2+j; left_diff[i] = pool 4+i; right_diff[j] = pool 6+j.
            let l0 = build(&p, &[0, 1, 4]);
            let l1 = build(&p, &[0, 1, 4, 2, 3, 5]);
            let r0 = build(&p, &[0, 2, 6]);
            let r1 = build(&p, &[0, 2, 6, 1, 3, 7]);
            assert!(
                [&l0, &l1, &r0, &r1].iter().all(|c| c.is_sorted_hash_list()),
                "card {card}: operands must stay hash lists to exercise the path under test",
            );

            let sketch =
                JointSketch::estimate(&[l0.mle(), l1.mle()], &[r0.mle(), r1.mle()]).into_parts();
            let error = cell_error(sketch, card as f64);
            assert!(
                error < 0.05,
                "card {card}: MLE hash-list joint sketch cell error {error} exceeds 5% (the \
                 narrow-width degradation appears to have returned)",
            );
        }
    }
}

#[cfg(test)]
mod mixed_value_hash_sketch {
    //! A nested operand set straddling the value-list/hash-list boundary (value-list inner shells,
    //! hash-list outer shells, none dense) must be routed by `joint_sketch_mle` to inclusion-exclusion,
    //! identical to the default joint sketch, rather than materialized to registers. Before the fix the
    //! MLE path register-ized these near-exact operands and drifted in the transition band, the spike
    //! the regime figure's hybrid MLE line exposed.
    use crate::prelude::*;

    type Hll = HyperLogLog<Precision12, Bits6>;

    #[test]
    fn mle_joint_sketch_matches_default_on_value_hash_mix() {
        // Inner shells are value lists, outer shells the same elements plus more inserted hashed, so
        // they are hash lists; the chains are nested by content and the two sides overlap.
        let value_list = |range: core::ops::Range<u64>| {
            let mut hll = Hll::default();
            for value in range {
                hll.insert_value(value);
            }
            hll
        };
        let hash_list = |range: core::ops::Range<u64>| {
            let mut hll = Hll::default();
            for value in range {
                hll.insert(&value);
            }
            hll
        };
        let a0 = value_list(0..20);
        let a1 = hash_list(0..120);
        let b0 = value_list(10..30);
        let b1 = hash_list(10..130);

        assert!(
            a0.is_sorted_value_list() && b0.is_sorted_value_list(),
            "inner shells must be value lists to exercise the value/hash mix",
        );
        assert!(
            a1.is_sorted_hash_list() && b1.is_sorted_hash_list(),
            "outer shells must be hash lists to exercise the value/hash mix",
        );
        assert!(
            ![&a0, &a1, &b0, &b1].iter().any(|c| c.is_hyperloglog()),
            "no operand may be dense (the materialize-to-registers branch is for dense operands)",
        );

        // The MLE joint sketch must use the same inclusion-exclusion decomposition as the default
        // (over the very same operands), so the cells are bit-identical. Computed first so the `.mle()`
        // borrows end before the default consumes the operands.
        let mle = JointSketch::estimate(&[a0.mle(), a1.mle()], &[b0.mle(), b1.mle()]).into_parts();
        let default = JointSketch::estimate(&[a0, a1], &[b0, b1]).into_parts();
        assert_eq!(
            mle, default,
            "the MLE joint sketch on a no-dense value/hash mix must equal the default \
             inclusion-exclusion sketch, not be materialized to registers",
        );
    }
}
