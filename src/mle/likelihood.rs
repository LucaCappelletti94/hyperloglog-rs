//! The polynomial `O((M+N)^2)` per-register joint log-likelihood (level-factor / block-collapse
//! form), its exact gradient, and its exact Hessian. See `docs/joint_mle_math.md` sections 4 to 11.
//!
//! These per-register routines take a `count` multiplier. With `alloc` the production joint sketch
//! reduces the registers to their distinct patterns first and calls them once per distinct pattern
//! with its multiplicity, so each evaluation is `O(distinct patterns)`. The no-alloc fallback calls
//! them once per register with `count = 1`. Either way the routines themselves are allocation-free.
//! The `BTreeMap`-backed tabulation below is test-only (it backs the `2^(M+N)` oracle and the oracle
//! cross-checks), where allocation is fine.

#[cfg(test)]
use crate::prelude::*;
use crate::utils::FloatOps;
#[cfg(test)]
use alloc::vec::Vec;

/// Tabulates the distinct joint register value patterns and their multiplicities into a `BTreeMap`.
/// Nesting is enforced by a cumulative max along each chain so the observed values are monotone.
/// Test-only: the production joint sketch tabulates its distinct patterns with a sort plus run-length
/// encode (see `sketch.rs`), and only the oracle cross-checks need this `BTreeMap` form.
///
/// `K = M*N + M + N` is the number of disjoint regions, indexed as: overlap `O_ij` at `i*N + j`,
/// left margin `D^A_i` at `M*N + i`, right margin `D^B_j` at `M*N + M + j`.
#[cfg(test)]
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
/// The cancellation-free value-only reference (production optimizes via the gradient form below).
/// Used by the likelihood and finite-difference cross-checks.
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
    let y = |rho: usize, w: u8| FloatOps::exp(-x(rho, w.min(q)));

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

        // PL = P(smallest left counter at value w not hit). PR symmetric, PLR = both missed.
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

        ln_p += FloatOps::natural_log(FloatOps::maximum(q_w, f64::MIN_POSITIVE));
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
    let y = |rho: usize, w: u8| FloatOps::exp(-x(rho, w.min(q)));

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

    // Achievement factors and their gradients. The hitter sets are walked directly by index (the
    // smallest left counter at value w is `p`, the smallest right counter `r`), with no per-level
    // allocation: a region is in the left block if it is the left margin `n_overlap + p` or an overlap
    // cell `p*N + j` with `b_pat[j] >= w`, and in the right block symmetrically.
    for w in 1..=q_plus_one {
        let left_p = (0..M).find(|&i| a_pat[i] == w);
        let right_r = (0..N).find(|&j| b_pat[j] == w);
        if left_p.is_none() && right_r.is_none() {
            continue;
        }

        // PL = product of y over the left block, PR over the right block, PLR over their union (which
        // shares only the overlap cell O_{p,r}).
        let mut pl = 1.0_f64;
        if let Some(p) = left_p {
            pl *= y(n_overlap + p, w);
            for j in 0..N {
                if b_pat[j] >= w {
                    pl *= y(p * N + j, w);
                }
            }
        }
        let mut pr = 1.0_f64;
        if let Some(r) = right_r {
            pr *= y(n_overlap + M + r, w);
            for i in 0..M {
                if a_pat[i] >= w {
                    pr *= y(i * N + r, w);
                }
            }
        }
        let mut plr = pl;
        if let (Some(p), Some(r)) = (left_p, right_r) {
            // The right block minus the shared cell O_{p,r}: the right margin, plus column r over
            // rows i != p with a_pat[i] >= w.
            plr *= y(n_overlap + M + r, w);
            for i in 0..M {
                if a_pat[i] >= w && i != p {
                    plr *= y(i * N + r, w);
                }
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
        ln_p += FloatOps::natural_log(FloatOps::maximum(q_w, f64::MIN_POSITIVE));

        // dQ_w/dphi_rho = [in L]*PL + [in R]*PR - [in L or R]*PLR, times x_rho(w), then over Q_w. Walk
        // the left block then the right-only cells, so each touched region is visited exactly once.
        let inverse = count / FloatOps::maximum(q_w, f64::MIN_POSITIVE);
        let mut accumulate = |rho: usize, in_l: bool, in_r: bool| {
            let coefficient = if both {
                f64::from(u8::from(in_l)) * pl + f64::from(u8::from(in_r)) * pr - plr
            } else if left_p.is_some() {
                pl
            } else {
                pr
            };
            gradient[rho] += inverse * coefficient * x(rho, w.min(q));
        };
        if let Some(p) = left_p {
            // Left margin and the left row, marking which also fall in the right block (the cell at
            // column r).
            accumulate(n_overlap + p, true, false);
            for j in 0..N {
                if b_pat[j] >= w {
                    let in_r = right_r == Some(j);
                    accumulate(p * N + j, true, in_r);
                }
            }
        }
        if let Some(r) = right_r {
            // Right margin and the right column, excluding cells already counted in the left block
            // (the row p cell and, when there is no left block, nothing to exclude).
            accumulate(n_overlap + M + r, false, true);
            for i in 0..M {
                if a_pat[i] >= w && left_p != Some(i) {
                    accumulate(i * N + r, false, true);
                }
            }
        }
    }

    count * ln_p
}

/// Polynomial per-pattern log-likelihood with its exact gradient AND Hessian. Accumulates
/// `count * d ln P_reg / d phi_rho` into `gradient` and `count * d^2 ln P_reg / d phi_sigma d phi_tau`
/// into `hessian` (row-major `K x K`), and returns `count * ln P_reg`. The Hessian is the closed form
/// of section 11 of `docs/joint_mle_math.md`: the base contributes `-x_rho(ceil_rho)` on the diagonal
/// only, and each `ln Q_w` contributes `(1/Q_w) d^2 Q_w - (1/Q_w^2) (dQ_w)(dQ_w)^T`, assembled from
/// the products `PL`, `PR`, `PLR` over their hitter sets. Shares the `y`, `x`, `Q_w` work with the
/// value and gradient so one pass yields all three.
#[allow(clippy::too_many_lines)]
pub(crate) fn joint_pattern_ll_grad_hess_poly<const M: usize, const N: usize>(
    a_pat: &[u8; M],
    b_pat: &[u8; N],
    ephi: &[f64],
    p_exponent: u8,
    q_plus_one: u8,
    count: f64,
    gradient: &mut [f64],
    hessian: &mut [f64],
) -> f64 {
    let n_overlap = M * N;
    let k = n_overlap + M + N;
    let q = q_plus_one - 1;

    let x = |rho: usize, level: u8| ephi[rho] * f64::integer_exp2_minus(p_exponent + level);

    let mut ln_p = 0.0_f64;

    // CDF base. Gradient and Hessian (diagonal only) each get -x_rho(ceil_rho).
    let mut base_region = |rho: usize, ceiling: u8| {
        if ceiling < q_plus_one {
            let xv = x(rho, ceiling);
            ln_p -= xv;
            gradient[rho] += count * (-xv);
            hessian[rho * k + rho] += count * (-xv);
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

    // Per-level scratch describing each touched region: its global index, its `x_rho(w)`, whether it
    // is in the left block and the right block, and its `dQ_w/dphi_rho`. Built into reused stack
    // arrays (no per-level allocation), capped at the maximum touched-set size `M + N + 2`. The joint
    // MLE is only practical for small `M, N` (the 8x8 corner gives a touched set of at most 18), so a
    // fixed cap is safe and a debug assertion guards it.
    const TOUCHED_CAP: usize = 64;
    debug_assert!(
        M + N + 2 <= TOUCHED_CAP,
        "touched-set cap exceeded for M={M} N={N}"
    );
    let mut idx = [0usize; TOUCHED_CAP];
    let mut xw_arr = [0.0_f64; TOUCHED_CAP];
    let mut in_l = [false; TOUCHED_CAP];
    let mut in_r = [false; TOUCHED_CAP];
    let mut dq_arr = [0.0_f64; TOUCHED_CAP];

    // Achievement factors, their gradients, and their Hessians.
    for w in 1..=q_plus_one {
        let left_p = (0..M).find(|&i| a_pat[i] == w);
        let right_r = (0..N).find(|&j| b_pat[j] == w);
        if left_p.is_none() && right_r.is_none() {
            continue;
        }

        // Enumerate the touched regions once into the scratch arrays: the left block (margin plus the
        // left row), then the right-only cells (margin plus the right column excluding the shared cell).
        let mut t = 0usize;
        let level = w.min(q);
        if let Some(p) = left_p {
            idx[t] = n_overlap + p;
            xw_arr[t] = x(n_overlap + p, level);
            in_l[t] = true;
            in_r[t] = false;
            t += 1;
            for j in 0..N {
                if b_pat[j] >= w {
                    idx[t] = p * N + j;
                    xw_arr[t] = x(p * N + j, level);
                    in_l[t] = true;
                    in_r[t] = right_r == Some(j);
                    t += 1;
                }
            }
        }
        if let Some(r) = right_r {
            idx[t] = n_overlap + M + r;
            xw_arr[t] = x(n_overlap + M + r, level);
            in_l[t] = false;
            in_r[t] = true;
            t += 1;
            for i in 0..M {
                if a_pat[i] >= w && left_p != Some(i) {
                    idx[t] = i * N + r;
                    xw_arr[t] = x(i * N + r, level);
                    in_l[t] = false;
                    in_r[t] = true;
                    t += 1;
                }
            }
        }

        // Block products PL, PR, PLR from the touched scratch (a region is in PLR iff it is touched).
        let mut pl = 1.0_f64;
        let mut pr = 1.0_f64;
        let mut plr = 1.0_f64;
        for s in 0..t {
            let ys = FloatOps::exp(-xw_arr[s]);
            if in_l[s] {
                pl *= ys;
            }
            if in_r[s] {
                pr *= ys;
            }
            plr *= ys;
        }
        let both = left_p.is_some() && right_r.is_some();
        if !both {
            plr = 1.0;
        }

        let q_w = if both {
            1.0 - pl - pr + plr
        } else if left_p.is_some() {
            1.0 - pl
        } else {
            1.0 - pr
        };
        ln_p += FloatOps::natural_log(FloatOps::maximum(q_w, f64::MIN_POSITIVE));

        let q_safe = FloatOps::maximum(q_w, f64::MIN_POSITIVE);
        let inverse = 1.0 / q_safe;

        // dQ_w/dphi_sigma per touched region: -(dPL + dPR - dPLR) with dPS = -x_sigma [in S] PS.
        for s in 0..t {
            let value = if both {
                xw_arr[s]
                    * (f64::from(u8::from(in_l[s])) * pl + f64::from(u8::from(in_r[s])) * pr - plr)
            } else if left_p.is_some() {
                xw_arr[s] * pl
            } else {
                xw_arr[s] * pr
            };
            dq_arr[s] = value;
            gradient[idx[s]] += count * inverse * value;
        }

        // Hessian: d^2 lnQ = d^2 Q / Q - (dQ)(dQ) / Q^2, over the touched set. The d^2 Q term reuses
        // the products: d^2 PS = (x_sigma x_tau [both in S] - x_sigma [sigma==tau in S]) PS. Accumulate
        // the symmetric pair once and mirror, halving the inner work.
        let d2_product = |ps: f64, sa: bool, sb: bool, s: usize, u: usize| -> f64 {
            if !sa || !sb {
                return 0.0;
            }
            if s == u {
                (xw_arr[s] * xw_arr[s] - xw_arr[s]) * ps
            } else {
                xw_arr[s] * xw_arr[u] * ps
            }
        };
        for s in 0..t {
            for u in s..t {
                let d2q = if both {
                    -(d2_product(pl, in_l[s], in_l[u], s, u)
                        + d2_product(pr, in_r[s], in_r[u], s, u)
                        - d2_product(plr, true, true, s, u))
                } else if left_p.is_some() {
                    -d2_product(pl, in_l[s], in_l[u], s, u)
                } else {
                    -d2_product(pr, in_r[s], in_r[u], s, u)
                };
                let entry = count * (inverse * d2q - inverse * inverse * dq_arr[s] * dq_arr[u]);
                hessian[idx[s] * k + idx[u]] += entry;
                if s != u {
                    hessian[idx[u] * k + idx[s]] += entry;
                }
            }
        }
    }

    count * ln_p
}
