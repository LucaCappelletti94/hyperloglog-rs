//! Orchestration of the generalized joint sketch MLE: warm start from the pairwise sketch, the
//! marginal-anchor prior, and the optimizer-driven refinement of the disjoint-cell cardinalities.

use super::likelihood::{joint_pattern_ll_and_gradient_poly, tabulate_joint_value_patterns};
use super::optimizers::{Adam, Chain, JointOptimizer, Lbfgs};
use crate::prelude::*;
use crate::utils::{FloatOps, Zero};
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use num_traits::Float;

/// Generalized joint MLE over the disjoint-region model, assuming all counters are in register
/// mode. Returns `(overlap[M][N], left_diff[M], right_diff[N])`. Uses the polynomial per-pattern
/// log-likelihood gradient and the default optimizer `Chain<Adam, Lbfgs>`: an Adam warmup (whose
/// momentum escapes poor local optima) followed by L-BFGS for fast final convergence. In the
/// `experiment_optimizers` comparison this matches a long Adam run's accuracy while being several
/// times faster, and beats plain L-BFGS on accuracy.
pub(crate) fn joint_sketch_mle_from_registers<
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
    joint_sketch_mle_from_registers_with::<P, B, R, H, Chain<Adam, Lbfgs>, M, N>(lefts, rights)
}

/// Generalized joint MLE with a caller-chosen optimizer type (compile-time generic composition). The
/// default path [`joint_sketch_mle_from_registers`] uses `Chain<Adam, Lbfgs>`.
pub(crate) fn joint_sketch_mle_from_registers_with<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
    O: JointOptimizer,
    const M: usize,
    const N: usize,
>(
    lefts: &[HyperLogLog<P, B, R, H>; M],
    rights: &[HyperLogLog<P, B, R, H>; N],
) -> ([[f64; N]; M], [f64; M], [f64; N]) {
    let value_patterns = tabulate_joint_value_patterns::<P, B, R, H, M, N>(lefts, rights);
    let p_exponent = P::EXPONENT;
    let q_plus_one: u8 = (1 << B::NUMBER_OF_BITS) - 1;

    joint_sketch_mle_core::<P, B, R, H, O, M, N>(lefts, rights, |phis, gradient| {
        let ephi: Vec<f64> = phis.iter().map(|phi| phi.exp()).collect();
        let mut log_likelihood = f64::ZERO;
        for (a_pat, b_pat, count) in &value_patterns {
            log_likelihood += joint_pattern_ll_and_gradient_poly::<M, N>(
                a_pat, b_pat, &ephi, p_exponent, q_plus_one, *count, gradient,
            );
        }
        log_likelihood
    })
}

/// Runs the warm-started, marginal-anchored optimization of the disjoint-region model with the
/// chosen `optimizer` and returns the cell matrices. The log-likelihood term is supplied by
/// `log_likelihood_gradient`, which receives the current `phis` and a pre-zeroed gradient buffer,
/// returns the log-likelihood value, and accumulates its ascent gradient into the buffer. Production
/// passes the polynomial evaluation; tests pass the exponential `2^(M+N)` oracle for cross-validation.
/// The objective being maximized is the MAP log-posterior (log-likelihood plus the marginal-anchor
/// log-prior).
#[allow(clippy::needless_range_loop)]
pub(crate) fn joint_sketch_mle_core<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
    O: JointOptimizer,
    const M: usize,
    const N: usize,
>(
    lefts: &[HyperLogLog<P, B, R, H>; M],
    rights: &[HyperLogLog<P, B, R, H>; N],
    mut log_likelihood_gradient: impl FnMut(&[f64], &mut [f64]) -> f64,
) -> ([[f64; N]; M], [f64; M], [f64; N]) {
    let n_overlap = M * N;
    let k = n_overlap + M + N;

    // Warm start from the pairwise hypersphere sketch: its differential overlaps and margin
    // differences are exactly the disjoint regions we optimize.
    let (overlap0, left0, right0) =
        <HyperLogLog<P, B, R, H> as HyperSpheresSketch>::overlap_and_differences_cardinality_matrices(
            lefts, rights,
        );

    let mut phis = vec![f64::ZERO; k];
    for i in 0..M {
        for j in 0..N {
            phis[i * N + j] = overlap0[i][j].max(f64::EPSILON).ln();
        }
    }
    for i in 0..M {
        phis[n_overlap + i] = left0[i].max(f64::EPSILON).ln();
    }
    for j in 0..N {
        phis[n_overlap + M + j] = right0[j].max(f64::EPSILON).ln();
    }

    // Marginal anchors. The deep overlap cells (contained only in the largest counters) are weakly
    // identified at high load: `x = n * 2^-(P + level)` is negligible at the high register levels
    // those counters reach, so the register likelihood barely constrains them and the free MLE
    // inflates them. We anchor each counter's cumulative cardinality to its HyperLogLog++ estimate
    // (the most reliable single-counter estimate) with a Gaussian log-space prior, which pins the
    // cell sums at every nesting level while the register likelihood still distributes mass among
    // the cells. See docs/joint_mle_math.md.
    let mut anchors: Vec<(Vec<usize>, f64, f64)> = Vec::with_capacity(M + N);
    // A HyperLogLog++ relative error of ~1.04/sqrt(m) corresponds, in log-space, to a Gaussian of
    // that standard deviation, hence a precision (weight) of m / 1.04^2.
    let m_registers = f64::integer_exp2(P::EXPONENT);
    let anchor_weight = m_registers / 1.04_f64.powi(2);
    for i in 0..M {
        let mut regions = Vec::new();
        for ii in 0..=i {
            for j in 0..N {
                regions.push(ii * N + j);
            }
            regions.push(n_overlap + ii);
        }
        anchors.push((
            regions,
            lefts[i].estimate_cardinality().max(f64::EPSILON).ln(),
            anchor_weight,
        ));
    }
    for j in 0..N {
        let mut regions = Vec::new();
        for jj in 0..=j {
            for i in 0..M {
                regions.push(i * N + jj);
            }
            regions.push(n_overlap + M + jj);
        }
        anchors.push((
            regions,
            rights[j].estimate_cardinality().max(f64::EPSILON).ln(),
            anchor_weight,
        ));
    }

    // The expected statistical error scales like 1/sqrt(m), so the optimizer stops once the
    // parameter step falls below that scale (the convergence threshold Ertl uses for the 2-set
    // joint MLE).
    let step_tolerance = 10.0_f64.powi(-2) / f64::integer_exp2(P::EXPONENT).sqrt();

    // The MAP objective: log-likelihood plus the marginal-anchor log-prior, with its ascent gradient.
    let objective = |phis: &[f64], gradient: &mut [f64]| {
        let log_likelihood = log_likelihood_gradient(phis, gradient);
        add_marginal_anchor_gradient(&anchors, phis, gradient);
        let mut log_prior = f64::ZERO;
        for (regions, log_estimate, weight) in &anchors {
            let sum: f64 = regions.iter().map(|&rho| phis[rho].exp()).sum();
            let residual = sum.max(f64::EPSILON).ln() - log_estimate;
            log_prior -= 0.5 * weight * residual * residual;
        }
        log_likelihood + log_prior
    };
    let phis = O::maximize(phis, objective, step_tolerance);

    let mut overlap = [[f64::ZERO; N]; M];
    for i in 0..M {
        for j in 0..N {
            overlap[i][j] = phis[i * N + j].exp();
        }
    }
    let mut left_diff = [f64::ZERO; M];
    for i in 0..M {
        left_diff[i] = phis[n_overlap + i].exp();
    }
    let mut right_diff = [f64::ZERO; N];
    for j in 0..N {
        right_diff[j] = phis[n_overlap + M + j].exp();
    }

    (overlap, left_diff, right_diff)
}

/// Adds the gradient of the marginal-anchor log-prior to `gradient` (which already holds the
/// log-likelihood gradient), forming the gradient of the MAP objective being maximized.
///
/// Each anchor is `(region indices summing to a counter, ln of that counter's HLL++ estimate,
/// weight)`. The prior is `-(weight/2) * (ln(sum n_rho) - ln(estimate))^2`, whose derivative with
/// respect to `phi_rho` (for `rho` in the counter) is `-weight * (ln S - ln estimate) * n_rho / S`,
/// with `S = sum over the counter of n_rho` and `n_rho = e^{phi_rho}`.
pub(crate) fn add_marginal_anchor_gradient(
    anchors: &[(Vec<usize>, f64, f64)],
    phis: &[f64],
    gradient: &mut [f64],
) {
    for (regions, log_estimate, weight) in anchors {
        let sum: f64 = regions.iter().map(|&rho| phis[rho].exp()).sum();
        let residual = sum.max(f64::EPSILON).ln() - log_estimate;
        let factor = -weight * residual / sum.max(f64::EPSILON);
        for &rho in regions {
            gradient[rho] += factor * phis[rho].exp();
        }
    }
}
