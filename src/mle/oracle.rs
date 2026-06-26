//! The exact-but-exponential `2^(M+N)` inclusion-exclusion likelihood, retained as the oracle that
//! the polynomial path is validated against. Test-only.

use super::likelihood::tabulate_joint_value_patterns;
use crate::prelude::*;
use crate::utils::{FloatOps, Zero};
use alloc::vec::Vec;

/// One distinct observed joint register pattern and everything needed to evaluate its
/// per-register log-likelihood contribution under the inclusion-exclusion model.
///
/// See `docs/joint_mle_math.md`. Each pattern contributes `count * ln(P_reg)` to the joint
/// log-likelihood, where `P_reg = sum over terms of sign * exp(-sum over (region, c) of
/// e^{phi_region} * c)` and `c = 2^-(P + level)` is the level constant baked in here.
pub(crate) struct JointPattern {
    /// How many registers exhibit this exact `(left values, right values)` pattern.
    pub(crate) count: f64,
    /// The surviving inclusion-exclusion terms. Each is `(sign, [(region index, 2^-(P+level))])`.
    /// A term with a region at the saturation cap drops that region (survival 1, constant 0), and a
    /// term that would knock a zero-valued counter below zero is dropped entirely.
    pub(crate) terms: Vec<(f64, Vec<(usize, f64)>)>,
}

/// Builds the `2^(M+N)` inclusion-exclusion terms for a single observed register pattern
/// `(a_pat, b_pat)` (sorted left/right values). This is the exact-but-exponential reference
/// evaluation. Each term is `(sign, [(region index, 2^-(P+level))])`. A region at the saturation
/// cap is dropped (survival 1), and a term knocking a zero-valued counter below zero is dropped.
pub(crate) fn build_pattern_terms<const M: usize, const N: usize>(
    a_pat: &[u8; M],
    b_pat: &[u8; N],
    p_exponent: u8,
    q_plus_one: u8,
) -> Vec<(f64, Vec<(usize, f64)>)> {
    let n_overlap = M * N;
    let mut terms = Vec::new();
    // Inclusion-exclusion over per-counter unit knockdowns: u over the M lefts, v over the N
    // rights. Bit set means that counter is pushed one level down.
    for u in 0u32..(1u32 << M) {
        for v in 0u32..(1u32 << N) {
            // Adjusted (knocked-down) observed values, where a knockdown below zero kills the term.
            let mut al = [0i16; M];
            let mut killed = false;
            for i in 0..M {
                let adjusted = i16::from(a_pat[i]) - i16::from((u >> i) & 1 == 1);
                if adjusted < 0 {
                    killed = true;
                }
                al[i] = adjusted;
            }
            let mut ar = [0i16; N];
            for j in 0..N {
                let adjusted = i16::from(b_pat[j]) - i16::from((v >> j) & 1 == 1);
                if adjusted < 0 {
                    killed = true;
                }
                ar[j] = adjusted;
            }
            if killed {
                continue;
            }

            let sign = if (u.count_ones() + v.count_ones()) % 2 == 0 {
                1.0
            } else {
                -1.0
            };

            // A region's level is the minimum adjusted value over ALL counters that contain it.
            // The overlap cell `O_ij` is contained in the left counters `i..M-1` and the right
            // counters `j..N-1`, the left margin `D^A_i` in the left counters `i..M-1`, and the
            // right margin `D^B_j` in the right counters `j..N-1`. After a knockdown the adjusted
            // values are not necessarily monotone, so we take the suffix minima explicitly rather
            // than assuming the lowest index binds.
            let mut suffix_min_al = [0i16; M];
            let mut running = i16::MAX;
            for i in (0..M).rev() {
                running = running.min(al[i]);
                suffix_min_al[i] = running;
            }
            let mut suffix_min_ar = [0i16; N];
            running = i16::MAX;
            for j in (0..N).rev() {
                running = running.min(ar[j]);
                suffix_min_ar[j] = running;
            }

            // A region at the saturation cap contributes survival 1 (skipped).
            let mut regions: Vec<(usize, f64)> = Vec::new();
            let mut push_region = |idx: usize, level: i16| {
                if level < i16::from(q_plus_one) {
                    let c = f64::integer_exp2_minus(p_exponent + level as u8);
                    regions.push((idx, c));
                }
            };
            for i in 0..M {
                for j in 0..N {
                    push_region(i * N + j, suffix_min_al[i].min(suffix_min_ar[j]));
                }
            }
            for i in 0..M {
                push_region(n_overlap + i, suffix_min_al[i]);
            }
            for j in 0..N {
                push_region(n_overlap + M + j, suffix_min_ar[j]);
            }

            terms.push((sign, regions));
        }
    }
    terms
}

/// Tabulates the distinct joint register patterns and precomputes their inclusion-exclusion terms.
/// This is the exact-but-exponential `2^(M+N)` reference path, retained as the oracle that the
/// polynomial path is validated against.
pub(crate) fn tabulate_joint_patterns<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
    const M: usize,
    const N: usize,
>(
    lefts: &[HyperLogLog<P, B, R, H>; M],
    rights: &[HyperLogLog<P, B, R, H>; N],
) -> Vec<JointPattern> {
    // q_plus_one is the saturation value, and levels above q contribute survival 1 (constant 0).
    let q_plus_one: u8 = (1 << B::NUMBER_OF_BITS) - 1;
    tabulate_joint_value_patterns::<P, B, R, H, M, N>(lefts, rights)
        .into_iter()
        .map(|(a_pat, b_pat, count)| JointPattern {
            count,
            terms: build_pattern_terms::<M, N>(&a_pat, &b_pat, P::EXPONENT, q_plus_one),
        })
        .collect()
}

/// Evaluates the joint log-likelihood and its exact gradient at `phis` (log-space region
/// cardinalities) over the tabulated register patterns. `k = M*N + M + N` is the region count.
///
/// Forward-mode differentiation: each per-register inclusion-exclusion term is log-linear in the
/// `phis`, so `d/dphi_rho` of `sign * exp(-sum_x)` is `-x_rho * sign * exp(-sum_x)`. The
/// log-likelihood gradient follows by the quotient `d ln P_reg = dP_reg / P_reg`. See
/// `docs/joint_mle_math.md`, section 5. Retained as the test-only oracle for the polynomial path.
pub(crate) fn joint_ll_and_gradient(
    patterns: &[JointPattern],
    phis: &[f64],
    k: usize,
) -> (f64, Vec<f64>) {
    let ephi: Vec<f64> = phis.iter().map(|phi| FloatOps::exp(*phi)).collect();
    let mut gradient = vec![f64::ZERO; k];
    let mut log_likelihood = f64::ZERO;

    for pattern in patterns {
        let mut p_value = f64::ZERO;
        let mut p_gradient = vec![f64::ZERO; k];
        for (sign, regions) in &pattern.terms {
            let mut sum_x = f64::ZERO;
            for &(rho, c) in regions {
                sum_x += ephi[rho] * c;
            }
            let signed_exp = sign * FloatOps::exp(-sum_x);
            p_value += signed_exp;
            for &(rho, c) in regions {
                p_gradient[rho] -= signed_exp * ephi[rho] * c;
            }
        }
        let p_value = FloatOps::maximum(p_value, f64::EPSILON);
        let inverse = 1.0 / p_value;
        log_likelihood += pattern.count * FloatOps::natural_log(p_value);
        for rho in 0..k {
            gradient[rho] += pattern.count * p_gradient[rho] * inverse;
        }
    }

    (log_likelihood, gradient)
}
