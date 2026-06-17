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
