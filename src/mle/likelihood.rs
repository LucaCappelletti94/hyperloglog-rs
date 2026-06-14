//! The polynomial `O((M+N)^2)` per-register joint log-likelihood (level-factor / block-collapse
//! form) and its exact gradient, plus the distinct-register-pattern tabulation. See
//! `docs/joint_mle_math.md` sections 4-9.

use crate::prelude::*;
use crate::utils::FloatOps;
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use num_traits::Float;

/// Tabulates the distinct joint register value patterns and their multiplicities (the cheap part
/// of pattern accounting, shared by the polynomial and reference paths). Nesting is enforced by a
/// cumulative max along each chain so the observed values are monotone.
///
/// `K = M*N + M + N` is the number of disjoint regions, indexed as: overlap `O_ij` at `i*N + j`,
/// left margin `D^A_i` at `M*N + i`, right margin `D^B_j` at `M*N + M + j`.
pub(crate) fn tabulate_joint_value_patterns<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
    const M: usize,
    const N: usize,
>(
    lefts: &[HyperLogLog<P, B, R, H>; M],
    rights: &[HyperLogLog<P, B, R, H>; N],
) -> Vec<([u8; M], [u8; N], f64)> {
    use super::PatternMap;

    let left_regs: [Vec<u8>; M] =
        core::array::from_fn(|i| lefts[i].registers.iter_registers().collect());
    let right_regs: [Vec<u8>; N] =
        core::array::from_fn(|j| rights[j].registers.iter_registers().collect());

    let m_registers = 1_usize << P::EXPONENT;
    let mut counts: PatternMap<([u8; M], [u8; N]), f64> = PatternMap::new();
    for r in 0..m_registers {
        let mut a_pat = [0u8; M];
        let mut acc = 0u8;
        for i in 0..M {
            acc = acc.max(left_regs[i][r]);
            a_pat[i] = acc;
        }
        let mut b_pat = [0u8; N];
        acc = 0u8;
        for j in 0..N {
            acc = acc.max(right_regs[j][r]);
            b_pat[j] = acc;
        }
        *counts.entry((a_pat, b_pat)).or_insert(0.0) += 1.0;
    }

    counts
        .into_iter()
        .map(|((a_pat, b_pat), count)| (a_pat, b_pat, count))
        .collect()
}

/// Polynomial per-pattern log-likelihood: the level-factor / block-collapse evaluation that
/// reproduces [`build_pattern_terms`] exactly in `O((M+N)^2 + M*N)` instead of `O(2^(M+N))`.
///
/// `ephi[rho] = n_rho = exp(phi_rho)`. The likelihood factorizes as `P_reg = exp(base) * prod_w
/// Q_w`: a CDF base (empty-above-ceiling) times one achievement factor per distinct observed value
/// `w`. Each counter block (the contiguous run of counters sharing value `w`) collapses to its
/// smallest index because nesting makes "contained in `A_l`" monotone. See `docs/joint_mle_math.md`.
///
/// The cancellation-free value-only reference (production optimizes via the gradient form below);
/// used by the likelihood and finite-difference cross-checks.
#[cfg(test)]
pub(crate) fn joint_pattern_ll_poly<const M: usize, const N: usize>(
    a_pat: &[u8; M],
    b_pat: &[u8; N],
    ephi: &[f64],
    p_exponent: u8,
    q_plus_one: u8,
) -> f64 {
    let n_overlap = M * N;
    let q = q_plus_one - 1;

    // x_rho at level k, and y_rho(w) = P(region empty at level w). The saturated top bucket
    // (w = q+1) uses level q, matching the existing register model.
    let x = |rho: usize, level: u8| ephi[rho] * f64::integer_exp2_minus(p_exponent + level);
    let y = |rho: usize, w: u8| (-x(rho, w.min(q))).exp();

    // CDF base: -sum over regions of x_rho(ceil_rho). Saturated ceilings (>= q+1) contribute 0.
    let mut ln_p = 0.0_f64;
    for i in 0..M {
        for j in 0..N {
            let ceiling = a_pat[i].min(b_pat[j]);
            if ceiling < q_plus_one {
                ln_p -= x(i * N + j, ceiling);
            }
        }
    }
    for i in 0..M {
        if a_pat[i] < q_plus_one {
            ln_p -= x(n_overlap + i, a_pat[i]);
        }
    }
    for j in 0..N {
        if b_pat[j] < q_plus_one {
            ln_p -= x(n_overlap + M + j, b_pat[j]);
        }
    }

    // Achievement factor Q_w for every value w that some counter attains.
    for w in 1..=q_plus_one {
        let left_p = (0..M).find(|&i| a_pat[i] == w);
        let right_r = (0..N).find(|&j| b_pat[j] == w);
        if left_p.is_none() && right_r.is_none() {
            continue;
        }

        // PL = P(smallest left counter at value w not hit); PR symmetric; PLR = both missed.
        let product_left = |p: usize| {
            let mut product = y(n_overlap + p, w);
            for j in 0..N {
                if b_pat[j] >= w {
                    product *= y(p * N + j, w);
                }
            }
            product
        };
        let product_right = |r: usize| {
            let mut product = y(n_overlap + M + r, w);
            for i in 0..M {
                if a_pat[i] >= w {
                    product *= y(i * N + r, w);
                }
            }
            product
        };

        let q_w = match (left_p, right_r) {
            (Some(p), Some(r)) => {
                let pl = product_left(p);
                let pr = product_right(r);
                // Union product: rows of p, plus column r excluding the shared cell O_{p,r}.
                let mut plr = y(n_overlap + p, w) * y(n_overlap + M + r, w);
                for j in 0..N {
                    if b_pat[j] >= w {
                        plr *= y(p * N + j, w);
                    }
                }
                for i in 0..M {
                    if a_pat[i] >= w && i != p {
                        plr *= y(i * N + r, w);
                    }
                }
                1.0 - pl - pr + plr
            }
            (Some(p), None) => 1.0 - product_left(p),
            (None, Some(r)) => 1.0 - product_right(r),
            (None, None) => unreachable!(),
        };

        ln_p += q_w.max(f64::MIN_POSITIVE).ln();
    }

    ln_p
}

/// Polynomial per-pattern log-likelihood and its exact gradient. Accumulates `count * d ln P_reg /
/// d phi_rho` into `gradient` and returns `count * ln P_reg`. The gradient is the closed-form
/// derivative of the level-factor form in [`joint_pattern_ll_poly`]: `d base / d phi_rho =
/// -x_rho(ceil_rho)` (one term per region), and `d ln Q_w / d phi_rho = (1/Q_w) * dQ_w` built from
/// `d(prod y)/d phi_rho = prod * (-x_rho(w))` for the regions in that product.
pub(crate) fn joint_pattern_ll_and_gradient_poly<const M: usize, const N: usize>(
    a_pat: &[u8; M],
    b_pat: &[u8; N],
    ephi: &[f64],
    p_exponent: u8,
    q_plus_one: u8,
    count: f64,
    gradient: &mut [f64],
) -> f64 {
    let n_overlap = M * N;
    let q = q_plus_one - 1;

    let x = |rho: usize, level: u8| ephi[rho] * f64::integer_exp2_minus(p_exponent + level);
    let y = |rho: usize, w: u8| (-x(rho, w.min(q))).exp();

    let mut ln_p = 0.0_f64;

    // CDF base and its gradient: each region contributes -x_rho(ceil_rho) to both.
    let mut base_region = |rho: usize, ceiling: u8| {
        if ceiling < q_plus_one {
            let xv = x(rho, ceiling);
            ln_p -= xv;
            gradient[rho] += count * (-xv);
        }
    };
    for i in 0..M {
        for j in 0..N {
            base_region(i * N + j, a_pat[i].min(b_pat[j]));
        }
    }
    for i in 0..M {
        base_region(n_overlap + i, a_pat[i]);
    }
    for j in 0..N {
        base_region(n_overlap + M + j, b_pat[j]);
    }

    // Achievement factors and their gradients.
    for w in 1..=q_plus_one {
        let left_p = (0..M).find(|&i| a_pat[i] == w);
        let right_r = (0..N).find(|&j| b_pat[j] == w);
        if left_p.is_none() && right_r.is_none() {
            continue;
        }

        // Hitter region indices for the smallest left/right counters at value w.
        let left_hitters = |p: usize| -> Vec<usize> {
            let mut hitters = vec![n_overlap + p];
            for j in 0..N {
                if b_pat[j] >= w {
                    hitters.push(p * N + j);
                }
            }
            hitters
        };
        let right_hitters = |r: usize| -> Vec<usize> {
            let mut hitters = vec![n_overlap + M + r];
            for i in 0..M {
                if a_pat[i] >= w {
                    hitters.push(i * N + r);
                }
            }
            hitters
        };

        let l_hit = left_p.map(left_hitters).unwrap_or_default();
        let r_hit = right_r.map(right_hitters).unwrap_or_default();
        let pl: f64 = l_hit.iter().map(|&rho| y(rho, w)).product();
        let pr: f64 = r_hit.iter().map(|&rho| y(rho, w)).product();
        let mut plr = pl;
        for &rho in &r_hit {
            if !l_hit.contains(&rho) {
                plr *= y(rho, w);
            }
        }

        let both = left_p.is_some() && right_r.is_some();
        let q_w = if both {
            1.0 - pl - pr + plr
        } else if left_p.is_some() {
            1.0 - pl
        } else {
            1.0 - pr
        };
        ln_p += q_w.max(f64::MIN_POSITIVE).ln();

        // dQ_w/dphi_rho = [in L]*PL + [in R]*PR - [in L or R]*PLR, times x_rho(w); then /Q_w.
        let inverse = count / q_w.max(f64::MIN_POSITIVE);
        let mut accumulate = |rho: usize| {
            let in_l = l_hit.contains(&rho);
            let in_r = r_hit.contains(&rho);
            let coefficient = if both {
                f64::from(u8::from(in_l)) * pl + f64::from(u8::from(in_r)) * pr - plr
            } else if left_p.is_some() {
                pl
            } else {
                pr
            };
            gradient[rho] += inverse * coefficient * x(rho, w.min(q));
        };
        for &rho in &l_hit {
            accumulate(rho);
        }
        for &rho in &r_hit {
            if !l_hit.contains(&rho) {
                accumulate(rho);
            }
        }
    }

    count * ln_p
}
