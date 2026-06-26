//! Orchestration of the generalized joint sketch MLE: warm start from the repeated 2-set MLE, the
//! marginal-anchor prior, and the optimizer-driven refinement of the disjoint-cell cardinalities.

use super::likelihood::{joint_pattern_ll_and_gradient_poly, joint_pattern_ll_grad_hess_poly};
use super::optimizers::{DampedNewton, MAX_K};
use crate::prelude::*;
use crate::utils::{FloatOps, Zero};

/// One marginal anchor: which counter it pins (a left counter `A_i` or a right counter `B_j`), the log
/// of that counter's HyperLogLog++ cardinality estimate, and the prior weight. The region index set is
/// reconstructed structurally (see [`for_each_anchor_region`]) rather than materialized, so the anchors
/// need no heap.
#[derive(Clone, Copy)]
pub(crate) enum Anchor {
    Left {
        i: usize,
        log_estimate: f64,
        weight: f64,
    },
    Right {
        j: usize,
        log_estimate: f64,
        weight: f64,
    },
}

/// Calls `f(rho)` for every disjoint region `rho` contained in the counter that `anchor` pins. A left
/// anchor `A_i` contains every overlap cell in rows `<= i` (all columns) and every left margin `<= i`.
/// A right anchor `B_j` contains every overlap cell in columns `<= j` (all rows) and every right margin
/// `<= j`. This mirrors the cumulative-cardinality structure of the nested counters.
#[inline]
fn for_each_anchor_region<const M: usize, const N: usize>(
    anchor: &Anchor,
    mut f: impl FnMut(usize),
) {
    let n_overlap = M * N;
    match *anchor {
        Anchor::Left { i, .. } => {
            for ii in 0..=i {
                for j in 0..N {
                    f(ii * N + j);
                }
                f(n_overlap + ii);
            }
        }
        Anchor::Right { j, .. } => {
            for jj in 0..=j {
                for i in 0..M {
                    f(i * N + jj);
                }
                f(n_overlap + M + jj);
            }
        }
    }
}

/// The log estimate and weight of an anchor.
#[inline]
fn anchor_target(anchor: &Anchor) -> (f64, f64) {
    match *anchor {
        Anchor::Left {
            log_estimate,
            weight,
            ..
        }
        | Anchor::Right {
            log_estimate,
            weight,
            ..
        } => (log_estimate, weight),
    }
}

/// Generalized joint MLE over the disjoint-region model, assuming all counters are in register
/// mode. Returns `(overlap[M][N], left_diff[M], right_diff[N])`. Uses the polynomial per-pattern
/// log-likelihood gradient and analytic Hessian, maximized by Levenberg-damped Newton: the robust
/// second-order optimizer that converges in few objective evaluations on the strongly anisotropic
/// deep-cell instances. The single pair (`M = N = 1`) is short-circuited to the analytic 2-set union
/// MLE.
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
) -> JointSketch<M, N> {
    // The single-pair case is exactly Ertl's 2-set joint MLE, which has a fast analytic solver over
    // the register multiplicity arrays. Use it instead of the generalized pattern-based optimizer,
    // which is orders of magnitude slower for the same three-region problem. The `_full` variant skips
    // this short-circuit and runs the optimizer even at the single pair.
    if M == 1 && N == 1 {
        let [left_difference, right_difference, intersection] =
            lefts[0].mle_union_regions_from_registers(&rights[0]);
        let mut overlap = [[f64::ZERO; N]; M];
        overlap[0][0] = intersection;
        let mut left_diff = [f64::ZERO; M];
        left_diff[0] = left_difference;
        let mut right_diff = [f64::ZERO; N];
        right_diff[0] = right_difference;
        return JointSketch {
            overlap,
            left_diff,
            right_diff,
        };
    }

    joint_sketch_mle_from_registers_full::<P, B, R, H, M, N>(lefts, rights)
}

/// Generalized joint MLE that always runs the full damped-Newton optimizer, with no `M = N = 1`
/// short-circuit. The default path [`joint_sketch_mle_from_registers`] short-circuits the single pair
/// to the analytic 2-set union MLE, so this variant exists for the benchmark hook and the tests that
/// need the optimizer exercised at every shape.
///
/// With `alloc` it first reduces the registers to their sufficient statistic: the distinct observed
/// `(a_pat, b_pat)` patterns and their multiplicities, tabulated once by collecting the `m` patterns,
/// sorting, and run-length encoding. Each Newton objective and Hessian evaluation then iterates the
/// distinct patterns, so the per-evaluation cost is `O(distinct patterns)` rather than `O(m)`. The
/// register values concentrate around `log2(n/m)`, so the distinct-pattern count is far below `m` for
/// the small-to-moderate grids and high precisions where this matters most (it approaches `m` only on
/// large grids, where the speedup tapers to nothing but the result is unchanged).
#[cfg(feature = "alloc")]
pub(crate) fn joint_sketch_mle_from_registers_full<
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
    let m_registers = 1_usize << P::EXPONENT;
    // Collect every register's monotone pattern, sort, then run-length encode into the distinct
    // (pattern, count) sufficient statistic. The resulting Vec is contiguous, so the repeated
    // per-evaluation sweeps walk flat memory.
    let mut keys: alloc::vec::Vec<([u8; M], [u8; N])> = alloc::vec::Vec::with_capacity(m_registers);
    for r in 0..m_registers {
        keys.push(register_pattern::<P, B, R, H, M, N>(lefts, rights, r));
    }
    keys.sort_unstable();
    let mut patterns: alloc::vec::Vec<([u8; M], [u8; N], f64)> = alloc::vec::Vec::new();
    for (a, b) in keys {
        match patterns.last_mut() {
            Some((pa, pb, count)) if *pa == a && *pb == b => *count += 1.0,
            _ => patterns.push((a, b, 1.0)),
        }
    }
    joint_sketch_mle_from_patterns::<P, B, R, H, M, N>(lefts, rights, &patterns)
}

/// Runs the generalized joint MLE over the distinct `(a_pat, b_pat, count)` patterns (the sufficient
/// statistic), so each Newton objective and Hessian evaluation costs `O(distinct patterns)`. Shared by
/// the `alloc` build of [`joint_sketch_mle_from_registers_full`].
#[cfg(feature = "alloc")]
fn joint_sketch_mle_from_patterns<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
    const M: usize,
    const N: usize,
>(
    lefts: &[HyperLogLog<P, B, R, H>; M],
    rights: &[HyperLogLog<P, B, R, H>; N],
    patterns: &[([u8; M], [u8; N], f64)],
) -> JointSketch<M, N> {
    let p_exponent = P::EXPONENT;
    let q_plus_one: u8 = (1 << B::NUMBER_OF_BITS) - 1;
    let k = M * N + M + N;
    joint_sketch_mle_core::<P, B, R, H, M, N>(
        lefts,
        rights,
        |phis, gradient| {
            let mut ephi = [f64::ZERO; MAX_K];
            for (slot, phi) in ephi[..k].iter_mut().zip(phis) {
                *slot = FloatOps::exp(*phi);
            }
            let mut log_likelihood = f64::ZERO;
            for (a_pat, b_pat, count) in patterns {
                log_likelihood += joint_pattern_ll_and_gradient_poly::<M, N>(
                    a_pat,
                    b_pat,
                    &ephi[..k],
                    p_exponent,
                    q_plus_one,
                    *count,
                    gradient,
                );
            }
            log_likelihood
        },
        |phis, hessian| {
            let mut ephi = [f64::ZERO; MAX_K];
            for (slot, phi) in ephi[..k].iter_mut().zip(phis) {
                *slot = FloatOps::exp(*phi);
            }
            let mut scratch_gradient = [f64::ZERO; MAX_K];
            for (a_pat, b_pat, count) in patterns {
                joint_pattern_ll_grad_hess_poly::<M, N>(
                    a_pat,
                    b_pat,
                    &ephi[..k],
                    p_exponent,
                    q_plus_one,
                    *count,
                    &mut scratch_gradient[..k],
                    hessian,
                );
            }
        },
    )
}

/// No-alloc fallback: with no heap to tabulate the sufficient statistic, sum the per-register patterns
/// on the fly, so each Newton evaluation is `O(m)`. It optimizes the same objective as the `alloc`
/// deduped path and reaches the same optimum up to floating-point summation order (the two sum the
/// per-pattern terms in different orders, so converged cells can differ at the 1e-12 level).
#[cfg(not(feature = "alloc"))]
pub(crate) fn joint_sketch_mle_from_registers_full<
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
    let p_exponent = P::EXPONENT;
    let q_plus_one: u8 = (1 << B::NUMBER_OF_BITS) - 1;
    let m_registers = 1_usize << P::EXPONENT;
    let k = M * N + M + N;

    joint_sketch_mle_core::<P, B, R, H, M, N>(
        lefts,
        rights,
        |phis, gradient| {
            let mut ephi = [f64::ZERO; MAX_K];
            for (slot, phi) in ephi[..k].iter_mut().zip(phis) {
                *slot = FloatOps::exp(*phi);
            }
            let mut log_likelihood = f64::ZERO;
            for r in 0..m_registers {
                let (a_pat, b_pat) = register_pattern::<P, B, R, H, M, N>(lefts, rights, r);
                log_likelihood += joint_pattern_ll_and_gradient_poly::<M, N>(
                    &a_pat,
                    &b_pat,
                    &ephi[..k],
                    p_exponent,
                    q_plus_one,
                    1.0,
                    gradient,
                );
            }
            log_likelihood
        },
        |phis, hessian| {
            let mut ephi = [f64::ZERO; MAX_K];
            for (slot, phi) in ephi[..k].iter_mut().zip(phis) {
                *slot = FloatOps::exp(*phi);
            }
            let mut scratch_gradient = [f64::ZERO; MAX_K];
            for r in 0..m_registers {
                let (a_pat, b_pat) = register_pattern::<P, B, R, H, M, N>(lefts, rights, r);
                joint_pattern_ll_grad_hess_poly::<M, N>(
                    &a_pat,
                    &b_pat,
                    &ephi[..k],
                    p_exponent,
                    q_plus_one,
                    1.0,
                    &mut scratch_gradient[..k],
                    hessian,
                );
            }
        },
    )
}

/// Reads the monotone observed register pattern at index `r`: the cumulative max of the left chain into
/// `a_pat` (so `a_pat[i] = max over ii <= i of A_ii's register r`) and of the right chain into `b_pat`.
/// The cumulative max enforces the nesting `A_0 subset ... subset A_{M-1}` at the register level.
#[inline]
fn register_pattern<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
    const M: usize,
    const N: usize,
>(
    lefts: &[HyperLogLog<P, B, R, H>; M],
    rights: &[HyperLogLog<P, B, R, H>; N],
    r: usize,
) -> ([u8; M], [u8; N]) {
    let mut a_pat = [0u8; M];
    let mut acc = 0u8;
    for i in 0..M {
        acc = acc.max(lefts[i].registers.get_register(r));
        a_pat[i] = acc;
    }
    let mut b_pat = [0u8; N];
    acc = 0u8;
    for j in 0..N {
        acc = acc.max(rights[j].registers.get_register(r));
        b_pat[j] = acc;
    }
    (a_pat, b_pat)
}

/// Runs the warm-started, marginal-anchored damped-Newton optimization of the disjoint-region model
/// and returns the cell matrices. The log-likelihood term is supplied by
/// `log_likelihood_gradient`, which receives the current `phis` and a pre-zeroed gradient buffer,
/// returns the log-likelihood value, and accumulates its ascent gradient into the buffer. Production
/// passes the polynomial evaluation, and tests pass the exponential `2^(M+N)` oracle for cross-validation.
/// The objective being maximized is the MAP log-posterior (the log-likelihood plus the
/// marginal-anchor log-prior).
///
/// `log_likelihood_hessian` accumulates the analytic log-likelihood Hessian (row-major `K x K`) into
/// the pre-zeroed buffer it is given. The core adds the marginal-anchor prior Hessian to form the full
/// MAP Hessian and hands it to damped Newton through `maximize_map`. Callers with no analytic Hessian
/// (the test cross-checking the exponential oracle) pass a finite-difference Hessian of their gradient.
#[allow(clippy::needless_range_loop)]
pub(crate) fn joint_sketch_mle_core<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
    const M: usize,
    const N: usize,
>(
    lefts: &[HyperLogLog<P, B, R, H>; M],
    rights: &[HyperLogLog<P, B, R, H>; N],
    mut log_likelihood_gradient: impl FnMut(&[f64], &mut [f64]) -> f64,
    log_likelihood_hessian: impl Fn(&[f64], &mut [f64]),
) -> JointSketch<M, N> {
    let n_overlap = M * N;
    let k = n_overlap + M + N;

    // Warm start from the repeated 2-set MLE decomposition (pairwise inclusion-exclusion over the
    // `.mle()` views, i.e. each pair's union from Ertl's 2-set MLE), NOT the HLL++ pairwise sketch.
    // Its differential overlaps and margin differences are exactly the disjoint regions we optimize.
    // The seed matters: from the HLL++ pairwise sketch the optimizer is trapped in a worse local
    // basin on weakly identified deep cells, whereas from the 2-set MLE solution it starts at a point
    // the joint likelihood can only refine. The seed costs M*N 2-set MLE solves and does not recurse
    // (inclusion-exclusion over `Mle` views uses only the pairwise union, never the joint optimizer).
    let left_views: [Mle<&HyperLogLog<P, B, R, H>>; M] = core::array::from_fn(|i| lefts[i].mle());
    let right_views: [Mle<&HyperLogLog<P, B, R, H>>; N] = core::array::from_fn(|j| rights[j].mle());
    let (overlap0, left0, right0) =
        crate::sketches::inclusion_exclusion_joint_sketch(&left_views, &right_views).into_parts();

    let mut phis_buf = [f64::ZERO; MAX_K];
    let phis = &mut phis_buf[..k];
    for i in 0..M {
        for j in 0..N {
            phis[i * N + j] =
                FloatOps::natural_log(FloatOps::maximum(overlap0[i][j], f64::EPSILON));
        }
    }
    for i in 0..M {
        phis[n_overlap + i] = FloatOps::natural_log(FloatOps::maximum(left0[i], f64::EPSILON));
    }
    for j in 0..N {
        phis[n_overlap + M + j] = FloatOps::natural_log(FloatOps::maximum(right0[j], f64::EPSILON));
    }

    // Marginal anchors. The deep overlap cells (contained only in the largest counters) are weakly
    // identified at high load: `x = n * 2^-(P + level)` is negligible at the high register levels
    // those counters reach, so the register likelihood barely constrains them and the free MLE
    // inflates them. We anchor each counter's cumulative cardinality to its HyperLogLog++ estimate
    // (the most reliable single-counter estimate) with a Gaussian log-space prior, which pins the
    // cell sums at every nesting level while the register likelihood still distributes mass among
    // the cells. See docs/joint_mle_math.md. The anchors are kept in a fixed `M + N` stack array and
    // their region sets are reconstructed structurally (no heap).
    //
    // A HyperLogLog++ relative error of ~1.04/sqrt(m) corresponds, in log-space, to a Gaussian of
    // that standard deviation, hence a precision (weight) of m / 1.04^2.
    let anchor_weight = f64::integer_exp2(P::EXPONENT) / FloatOps::powi(1.04_f64, 2);
    // At most M + N anchors, M, N <= 8, so 16 fits the fixed cap.
    const MAX_ANCHORS: usize = 16;
    debug_assert!(M + N <= MAX_ANCHORS, "anchor cap exceeded for M={M} N={N}");
    let mut anchors = [Anchor::Left {
        i: 0,
        log_estimate: 0.0,
        weight: 0.0,
    }; MAX_ANCHORS];
    let mut anchor_count = 0usize;
    for i in 0..M {
        anchors[anchor_count] = Anchor::Left {
            i,
            log_estimate: FloatOps::natural_log(FloatOps::maximum(
                lefts[i].estimate_cardinality(),
                f64::EPSILON,
            )),
            weight: anchor_weight,
        };
        anchor_count += 1;
    }
    for j in 0..N {
        anchors[anchor_count] = Anchor::Right {
            j,
            log_estimate: FloatOps::natural_log(FloatOps::maximum(
                rights[j].estimate_cardinality(),
                f64::EPSILON,
            )),
            weight: anchor_weight,
        };
        anchor_count += 1;
    }
    let anchors = &anchors[..anchor_count];

    // The expected statistical error scales like 1/sqrt(m), so the optimizer stops once the
    // parameter step falls below that scale (the convergence threshold Ertl uses for the 2-set
    // joint MLE).
    let step_tolerance =
        FloatOps::powi(10.0_f64, -2) / FloatOps::sqrt(f64::integer_exp2(P::EXPONENT));

    // The MAP objective: log-likelihood plus the marginal-anchor log-prior, with its ascent gradient.
    let objective = |phis: &[f64], gradient: &mut [f64]| {
        let log_likelihood = log_likelihood_gradient(phis, gradient);
        add_marginal_anchor_gradient::<M, N>(anchors, phis, gradient);
        let mut log_prior = f64::ZERO;
        for anchor in anchors {
            let mut sum = f64::ZERO;
            for_each_anchor_region::<M, N>(anchor, |rho| sum += FloatOps::exp(phis[rho]));
            let (log_estimate, weight) = anchor_target(anchor);
            let residual =
                FloatOps::natural_log(FloatOps::maximum(sum, f64::EPSILON)) - log_estimate;
            log_prior -= 0.5 * weight * residual * residual;
        }
        log_likelihood + log_prior
    };
    // The MAP Hessian provider: the analytic log-likelihood Hessian plus the marginal-anchor prior
    // Hessian (see docs/joint_mle_math.md section 11).
    let hessian = |phis: &[f64], out: &mut [f64]| {
        log_likelihood_hessian(phis, out);
        add_marginal_anchor_hessian::<M, N>(anchors, phis, out, k);
    };
    DampedNewton::maximize_map(phis, objective, hessian, step_tolerance);

    let mut overlap = [[f64::ZERO; N]; M];
    for i in 0..M {
        for j in 0..N {
            overlap[i][j] = FloatOps::exp(phis[i * N + j]);
        }
    }
    let mut left_diff = [f64::ZERO; M];
    for i in 0..M {
        left_diff[i] = FloatOps::exp(phis[n_overlap + i]);
    }
    let mut right_diff = [f64::ZERO; N];
    for j in 0..N {
        right_diff[j] = FloatOps::exp(phis[n_overlap + M + j]);
    }

    JointSketch {
        overlap,
        left_diff,
        right_diff,
    }
}

/// Adds the gradient of the marginal-anchor log-prior to `gradient` (which already holds the
/// log-likelihood gradient), forming the gradient of the MAP objective being maximized.
///
/// The prior for one anchor is `-(weight/2) * (ln(sum n_rho) - ln(estimate))^2`, whose derivative with
/// respect to `phi_rho` (for `rho` in the anchored counter) is `-weight * (ln S - ln estimate) * n_rho
/// / S`, with `S = sum over the counter of n_rho` and `n_rho = e^{phi_rho}`.
pub(crate) fn add_marginal_anchor_gradient<const M: usize, const N: usize>(
    anchors: &[Anchor],
    phis: &[f64],
    gradient: &mut [f64],
) {
    for anchor in anchors {
        let mut sum = f64::ZERO;
        for_each_anchor_region::<M, N>(anchor, |rho| sum += FloatOps::exp(phis[rho]));
        let (log_estimate, weight) = anchor_target(anchor);
        let residual = FloatOps::natural_log(FloatOps::maximum(sum, f64::EPSILON)) - log_estimate;
        let factor = -weight * residual / FloatOps::maximum(sum, f64::EPSILON);
        for_each_anchor_region::<M, N>(anchor, |rho| {
            gradient[rho] += factor * FloatOps::exp(phis[rho]);
        });
    }
}

/// Adds the Hessian of the marginal-anchor log-prior to `hessian` (row-major `K x K`), forming the
/// full MAP Hessian once the log-likelihood Hessian is already in place. See `docs/joint_mle_math.md`
/// section 11.5: for one anchor with `S = sum_{rho in regions} e^{phi_rho}`, `n_rho = e^{phi_rho}`, and
/// residual `r = ln S - ln estimate`, the second derivative for two regions `sigma, tau` in the anchor
/// is `-weight ((1 - r)(n_sigma n_tau / S^2) + r (n_sigma / S)[sigma == tau])`. Each anchor therefore
/// contributes a dense block over its own region set. The region pairs are enumerated into a fixed
/// stack buffer (bounded by `K`) so no heap is needed.
pub(crate) fn add_marginal_anchor_hessian<const M: usize, const N: usize>(
    anchors: &[Anchor],
    phis: &[f64],
    hessian: &mut [f64],
    k: usize,
) {
    let mut regions = [0usize; MAX_K];
    for anchor in anchors {
        let mut len = 0usize;
        let mut sum = f64::ZERO;
        for_each_anchor_region::<M, N>(anchor, |rho| {
            regions[len] = rho;
            len += 1;
            sum += FloatOps::exp(phis[rho]);
        });
        let safe_sum = FloatOps::maximum(sum, f64::EPSILON);
        let (log_estimate, weight) = anchor_target(anchor);
        let residual = FloatOps::natural_log(safe_sum) - log_estimate;
        for &sigma in &regions[..len] {
            let n_sigma = FloatOps::exp(phis[sigma]);
            for &tau in &regions[..len] {
                let n_tau = FloatOps::exp(phis[tau]);
                let mut entry =
                    -weight * (1.0 - residual) * (n_sigma * n_tau) / (safe_sum * safe_sum);
                if sigma == tau {
                    entry -= weight * residual * n_sigma / safe_sum;
                }
                hessian[sigma * k + tau] += entry;
            }
        }
    }
}
