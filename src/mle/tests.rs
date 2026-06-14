//! Tests and the optimizer comparison harness for the MLE estimators.

use super::exact::*;
use super::likelihood::*;
use super::optimizers::*;
use super::oracle::*;
use super::sketch::*;
use super::union::*;
use crate::prelude::*;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

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

/// Every `JointOptimizer` must maximize a smooth concave objective (here a negated quadratic)
/// to its optimum, mirroring how the joint MLE refines the warm start.
#[cfg(feature = "mle")]
#[test]
fn test_optimizers_maximize_quadratic() {
    // Maximize -((x0-1)^2 + (x1+2)^2 + (x2-3)^2), with maximum at (1, -2, 3).
    fn check<O: JointOptimizer>(tolerance: f64) {
        let eval = |x: &[f64], grad: &mut [f64]| -> f64 {
            grad[0] = -2.0 * (x[0] - 1.0);
            grad[1] = -2.0 * (x[1] + 2.0);
            grad[2] = -2.0 * (x[2] - 3.0);
            -((x[0] - 1.0).powi(2) + (x[1] + 2.0).powi(2) + (x[2] - 3.0).powi(2))
        };
        let result = O::maximize(vec![0.0, 0.0, 0.0], eval, 1e-12);
        assert!((result[0] - 1.0).abs() < tolerance, "{result:?}");
        assert!((result[1] + 2.0).abs() < tolerance, "{result:?}");
        assert!((result[2] - 3.0).abs() < tolerance, "{result:?}");
    }
    check::<Lbfgs>(1e-5);
    check::<Adam>(1e-2);
    check::<RmsProp>(1e-2);
    check::<Chain<Adam, Lbfgs>>(1e-5);
}

/// The analytic forward-mode gradient of the joint log-likelihood must match a central
/// finite-difference estimate at every coordinate, on real tabulated register patterns.
#[cfg(feature = "mle")]
#[test]
fn test_joint_ll_gradient_matches_finite_differences() {
    type Counter =
            HyperLogLog<
                crate::prelude::Precision8,
                crate::prelude::Bits6,
                <crate::prelude::Precision8 as crate::prelude::PackedRegister<
                    crate::prelude::Bits6,
                >>::Array,
                twox_hash::XxHash64,
            >;

    let insert = |hll: &mut Counter, start: u64, count: u64| {
        for v in start..start + count {
            hll.insert(&v);
        }
    };
    let mut a0 = Counter::default();
    insert(&mut a0, 0, 4_000);
    insert(&mut a0, 4_000, 2_500);
    insert(&mut a0, 10_000, 2_200);
    let mut a1 = a0.clone();
    insert(&mut a1, 12_000, 1_800);
    insert(&mut a1, 14_000, 3_000);
    insert(&mut a1, 20_000, 1_500);
    let mut b0 = Counter::default();
    insert(&mut b0, 0, 4_000);
    insert(&mut b0, 12_000, 1_800);
    insert(&mut b0, 30_000, 2_000);
    let mut b1 = b0.clone();
    insert(&mut b1, 4_000, 2_500);
    insert(&mut b1, 14_000, 3_000);
    insert(&mut b1, 40_000, 2_800);

    let patterns = tabulate_joint_patterns::<_, _, _, _, 2, 2>(&[a0, a1], &[b0, b1]);
    let k = 2 * 2 + 2 + 2;

    // A non-trivial, non-degenerate evaluation point.
    let phis: Vec<f64> = (0..k)
        .map(|i| 8.0 + 0.3 * (i as f64) - 0.05 * (i * i) as f64)
        .collect();

    let (_ll, grad) = joint_ll_and_gradient(&patterns, &phis, k);

    let h = 1e-5;
    for rho in 0..k {
        let mut plus = phis.clone();
        let mut minus = phis.clone();
        plus[rho] += h;
        minus[rho] -= h;
        let (ll_plus, _) = joint_ll_and_gradient(&patterns, &plus, k);
        let (ll_minus, _) = joint_ll_and_gradient(&patterns, &minus, k);
        let fd = (ll_plus - ll_minus) / (2.0 * h);
        let scale = grad[rho].abs().max(fd.abs()).max(1.0);
        assert!(
            (grad[rho] - fd).abs() / scale < 1e-3,
            "gradient[{rho}] = {} but finite difference = {fd}",
            grad[rho],
        );
    }
}

/// Evaluates `ln P_reg` for one pattern via the exponential `2^(M+N)` reference (oracle).
#[cfg(feature = "mle")]
fn oracle_pattern_ln_p_reg<const M: usize, const N: usize>(
    a_pat: &[u8; M],
    b_pat: &[u8; N],
    ephi: &[f64],
    p_exponent: u8,
    q_plus_one: u8,
) -> f64 {
    let terms = build_pattern_terms::<M, N>(a_pat, b_pat, p_exponent, q_plus_one);
    let mut p = 0.0_f64;
    for (sign, regions) in &terms {
        let mut sum_x = 0.0;
        for &(rho, c) in regions {
            sum_x += ephi[rho] * c;
        }
        p += sign * (-sum_x).exp();
    }
    p.max(f64::EPSILON).ln()
}

/// Draws a sorted (monotone) register pattern of length `L` with values in `0..=q_plus_one`.
#[cfg(feature = "mle")]
fn random_monotone_pattern<const L: usize>(state: &mut u64, q_plus_one: u8) -> [u8; L] {
    let mut pattern: [u8; L] = core::array::from_fn(|_| {
        *state = splitmix64(*state);
        (*state % (u64::from(q_plus_one) + 1)) as u8
    });
    pattern.sort_unstable();
    pattern
}

/// Draws region cardinalities `n_rho = exp(phi)` with `phi` uniform in `[ln 2, ln 40]`, the
/// regime where the signed `2^(M+N)` oracle is numerically reliable (matching the Python
/// cross-check). The polynomial form is cancellation-free at any scale.
#[cfg(feature = "mle")]
fn random_ephi(state: &mut u64, k: usize) -> Vec<f64> {
    (0..k)
        .map(|_| {
            *state = splitmix64(*state);
            let u = (*state >> 11) as f64 / (1u64 << 53) as f64;
            (2.0_f64.ln() + (40.0_f64.ln() - 2.0_f64.ln()) * u).exp()
        })
        .collect()
}

/// The polynomial per-pattern log-likelihood must match the exponential `2^(M+N)` oracle to
/// floating-point tolerance across random monotone patterns (covering ties, zeros, saturation)
/// and random region cardinalities, for several `(M, N)` including rectangular shapes.
#[cfg(feature = "mle")]
fn check_poly_likelihood_matches_oracle<const M: usize, const N: usize>(
    seed: u64,
    p_exponent: u8,
    q_plus_one: u8,
) {
    let k = M * N + M + N;
    let mut state = seed;
    let mut tested = 0;
    for _ in 0..1500 {
        let a_pat = random_monotone_pattern::<M>(&mut state, q_plus_one);
        let b_pat = random_monotone_pattern::<N>(&mut state, q_plus_one);
        let ephi = random_ephi(&mut state, k);

        let oracle = oracle_pattern_ln_p_reg::<M, N>(&a_pat, &b_pat, &ephi, p_exponent, q_plus_one);
        // Skip patterns where the signed oracle sum loses precision to catastrophic
        // cancellation (small P_reg from tiny x at high levels); the polynomial form is
        // cancellation-free, so the oracle is the limiting factor here, not the poly.
        if oracle < -18.0 {
            continue;
        }
        let poly = joint_pattern_ll_poly::<M, N>(&a_pat, &b_pat, &ephi, p_exponent, q_plus_one);
        assert!(
            (oracle - poly).abs() < 1e-7,
            "M={M} N={N} a={a_pat:?} b={b_pat:?}: oracle ln={oracle} poly ln={poly}"
        );
        tested += 1;
    }
    assert!(tested > 20, "too few non-degenerate trials: {tested}");
}

#[cfg(feature = "mle")]
#[test]
fn test_poly_pattern_likelihood_matches_oracle() {
    // Small p_exponent keeps x in a range where the signed oracle is numerically reliable; the
    // per-pattern algorithm is independent of p_exponent (it only scales x), so this fully
    // validates correctness. q_plus_one = 7 exercises the saturation boundary.
    for &q_plus_one in &[7u8, 15u8] {
        let p = 4u8;
        check_poly_likelihood_matches_oracle::<1, 1>(0x1111, p, q_plus_one);
        check_poly_likelihood_matches_oracle::<2, 2>(0x2222, p, q_plus_one);
        check_poly_likelihood_matches_oracle::<3, 2>(0x3232, p, q_plus_one);
        check_poly_likelihood_matches_oracle::<2, 3>(0x2323, p, q_plus_one);
        check_poly_likelihood_matches_oracle::<3, 3>(0x3333, p, q_plus_one);
    }

    // Explicit edge cases: all-zero, all-saturated, all-equal mid, a single zero.
    let p = 4u8;
    let q1 = 7u8;
    let k = 2 * 2 + 2 + 2;
    let ephi: Vec<f64> = (0..k).map(|i| (1.0 + 0.5 * i as f64).exp()).collect();
    for (a, b) in [
        ([0u8, 0], [0u8, 0]),
        ([7, 7], [7, 7]),
        ([3, 3], [3, 3]),
        ([0, 3], [0, 5]),
        ([0, 7], [2, 7]),
    ] {
        let oracle = oracle_pattern_ln_p_reg::<2, 2>(&a, &b, &ephi, p, q1);
        let poly = joint_pattern_ll_poly::<2, 2>(&a, &b, &ephi, p, q1);
        assert!(
            (oracle - poly).abs() < 1e-7,
            "edge a={a:?} b={b:?}: oracle={oracle} poly={poly}"
        );
    }
}

/// Gradient of `ln P_reg` for one pattern via the exponential `2^(M+N)` oracle.
#[cfg(feature = "mle")]
fn oracle_pattern_gradient<const M: usize, const N: usize>(
    a_pat: &[u8; M],
    b_pat: &[u8; N],
    phis: &[f64],
    p_exponent: u8,
    q_plus_one: u8,
    k: usize,
) -> Vec<f64> {
    let pattern = JointPattern {
        count: 1.0,
        terms: build_pattern_terms::<M, N>(a_pat, b_pat, p_exponent, q_plus_one),
    };
    joint_ll_and_gradient(&[pattern], phis, k).1
}

/// The polynomial per-pattern gradient must match both a central finite difference of the
/// polynomial likelihood and the exponential oracle gradient, across random patterns.
#[cfg(feature = "mle")]
fn check_poly_gradient_matches_oracle_and_fd<const M: usize, const N: usize>(
    seed: u64,
    p_exponent: u8,
    q_plus_one: u8,
) {
    let k = M * N + M + N;
    let mut state = seed;
    let mut tested = 0;
    for _ in 0..1500 {
        let a_pat = random_monotone_pattern::<M>(&mut state, q_plus_one);
        let b_pat = random_monotone_pattern::<N>(&mut state, q_plus_one);
        let ephi = random_ephi(&mut state, k);
        let phis: Vec<f64> = ephi.iter().map(|e| e.ln()).collect();

        let ll = joint_pattern_ll_poly::<M, N>(&a_pat, &b_pat, &ephi, p_exponent, q_plus_one);
        if ll < -18.0 {
            continue;
        }

        let mut grad = vec![0.0_f64; k];
        let ret = joint_pattern_ll_and_gradient_poly::<M, N>(
            &a_pat, &b_pat, &ephi, p_exponent, q_plus_one, 1.0, &mut grad,
        );
        assert!(
            (ret - ll).abs() < 1e-9,
            "returned ll {ret} != value-fn ll {ll}"
        );

        // Finite-difference cross-check (perturb in phi-space).
        let h = 1e-6_f64;
        for rho in 0..k {
            let mut ep = ephi.clone();
            let mut em = ephi.clone();
            ep[rho] = ephi[rho] * h.exp();
            em[rho] = ephi[rho] * (-h).exp();
            let lp = joint_pattern_ll_poly::<M, N>(&a_pat, &b_pat, &ep, p_exponent, q_plus_one);
            let lm = joint_pattern_ll_poly::<M, N>(&a_pat, &b_pat, &em, p_exponent, q_plus_one);
            let fd = (lp - lm) / (2.0 * h);
            let scale = grad[rho].abs().max(fd.abs()).max(1.0);
            assert!(
                (grad[rho] - fd).abs() / scale < 1e-4,
                "FD mismatch M={M} N={N} a={a_pat:?} b={b_pat:?} rho={rho}: grad={} fd={fd}",
                grad[rho]
            );
        }

        // Oracle gradient cross-check.
        let ograd =
            oracle_pattern_gradient::<M, N>(&a_pat, &b_pat, &phis, p_exponent, q_plus_one, k);
        for rho in 0..k {
            let scale = grad[rho].abs().max(ograd[rho].abs()).max(1.0);
            assert!(
                (grad[rho] - ograd[rho]).abs() / scale < 1e-6,
                "oracle mismatch M={M} N={N} a={a_pat:?} b={b_pat:?} rho={rho}: poly={} oracle={}",
                grad[rho],
                ograd[rho]
            );
        }
        tested += 1;
    }
    assert!(tested > 20, "too few non-degenerate trials: {tested}");
}

#[cfg(feature = "mle")]
#[test]
fn test_poly_pattern_gradient_matches_oracle_and_fd() {
    for &q_plus_one in &[7u8, 15u8] {
        let p = 4u8;
        check_poly_gradient_matches_oracle_and_fd::<1, 1>(0xA1A1, p, q_plus_one);
        check_poly_gradient_matches_oracle_and_fd::<2, 2>(0xB2B2, p, q_plus_one);
        check_poly_gradient_matches_oracle_and_fd::<3, 2>(0xC3C2, p, q_plus_one);
        check_poly_gradient_matches_oracle_and_fd::<2, 3>(0xD2D3, p, q_plus_one);
        check_poly_gradient_matches_oracle_and_fd::<3, 3>(0xE3E3, p, q_plus_one);
    }
}

/// The full estimator driven by the polynomial gradient must converge to the same cell matrices
/// as the same optimization driven by the exponential `2^(M+N)` oracle gradient, confirming the
/// production rewrite is faithful end to end.
#[cfg(feature = "mle")]
fn check_full_estimator_poly_vs_oracle<const M: usize, const N: usize>(unit: u64) {
    type Counter =
        HyperLogLog<
            crate::prelude::Precision10,
            crate::prelude::Bits6,
            <crate::prelude::Precision10 as crate::prelude::PackedRegister<
                crate::prelude::Bits6,
            >>::Array,
            twox_hash::XxHash64,
        >;
    let build = |ranges: &[(u64, u64)]| -> Counter {
        let mut hll = Counter::default();
        for &(start, count) in ranges {
            for v in start..start + count {
                hll.insert(&v);
            }
        }
        hll
    };
    // Disjoint integer ranges, one per region; nested counters built from them.
    let mut cursor = 0u64;
    let mut ranges_o = [[(0u64, 0u64); N]; M];
    for i in 0..M {
        for j in 0..N {
            let count = unit * (2 + ((i * 5 + j * 3) % 4) as u64);
            ranges_o[i][j] = (cursor, count);
            cursor += count;
        }
    }
    let mut ranges_da = [(0u64, 0u64); M];
    for i in 0..M {
        let count = unit * (1 + (i % 2) as u64);
        ranges_da[i] = (cursor, count);
        cursor += count;
    }
    let mut ranges_db = [(0u64, 0u64); N];
    for j in 0..N {
        let count = unit * (1 + (j % 3) as u64);
        ranges_db[j] = (cursor, count);
        cursor += count;
    }
    let lefts: [Counter; M] = core::array::from_fn(|i| {
        let mut ranges = Vec::new();
        for ii in 0..=i {
            for j in 0..N {
                ranges.push(ranges_o[ii][j]);
            }
            ranges.push(ranges_da[ii]);
        }
        build(&ranges)
    });
    let rights: [Counter; N] = core::array::from_fn(|j| {
        let mut ranges = Vec::new();
        for jj in 0..=j {
            for i in 0..M {
                ranges.push(ranges_o[i][jj]);
            }
            ranges.push(ranges_db[jj]);
        }
        build(&ranges)
    });

    let p_exponent = crate::prelude::Precision10::EXPONENT;
    let q_plus_one: u8 = (1 << crate::prelude::Bits6::NUMBER_OF_BITS) - 1;
    let k = M * N + M + N;

    // Production path (polynomial gradient). Both paths use the same optimizer (Lbfgs) so the
    // comparison isolates the gradient source.
    let value_patterns = tabulate_joint_value_patterns::<_, _, _, _, M, N>(&lefts, &rights);
    let (ov_poly, l_poly, r_poly) =
        joint_sketch_mle_core::<_, _, _, _, Lbfgs, M, N>(&lefts, &rights, |phis, gradient| {
            let ephi: Vec<f64> = phis.iter().map(|phi| phi.exp()).collect();
            let mut log_likelihood = 0.0;
            for (a_pat, b_pat, count) in &value_patterns {
                log_likelihood += joint_pattern_ll_and_gradient_poly::<M, N>(
                    a_pat, b_pat, &ephi, p_exponent, q_plus_one, *count, gradient,
                );
            }
            log_likelihood
        })
        .into_parts();

    // Oracle path (exponential gradient).
    let oracle_patterns = tabulate_joint_patterns::<_, _, _, _, M, N>(&lefts, &rights);
    let (ov_oracle, l_oracle, r_oracle) =
        joint_sketch_mle_core::<_, _, _, _, Lbfgs, M, N>(&lefts, &rights, |phis, gradient| {
            let (ll, g) = joint_ll_and_gradient(&oracle_patterns, phis, k);
            for (slot, value) in gradient.iter_mut().zip(g) {
                *slot += value;
            }
            ll
        })
        .into_parts();

    // The per-pattern gradients agree to ~1e-7, but the two paths sum patterns in different
    // (HashMap) orders and the oracle carries ~1e-7 cancellation error, which compound over the
    // optimization along weakly-identified directions. A 0.1% end-to-end agreement still
    // confirms the rewrite is faithful; a real bug would diverge grossly (as the tight
    // per-pattern gradient test would already catch).
    let close = |a: f64, b: f64| (a - b).abs() <= 1e-3 * a.abs().max(b.abs()) + 1.0;
    for i in 0..M {
        for j in 0..N {
            assert!(
                close(ov_poly[i][j], ov_oracle[i][j]),
                "M={M} N={N} overlap[{i}][{j}]: poly={} oracle={}",
                ov_poly[i][j],
                ov_oracle[i][j]
            );
        }
    }
    for i in 0..M {
        assert!(close(l_poly[i], l_oracle[i]), "M={M} N={N} left[{i}]");
    }
    for j in 0..N {
        assert!(close(r_poly[j], r_oracle[j]), "M={M} N={N} right[{j}]");
    }
}

#[cfg(feature = "mle")]
#[test]
fn test_poly_joint_sketch_matches_oracle() {
    check_full_estimator_poly_vs_oracle::<1, 1>(20_000);
    check_full_estimator_poly_vs_oracle::<2, 2>(8_000);
    check_full_estimator_poly_vs_oracle::<3, 2>(5_000);
}

/// Experiment (ignored by default): compares how fast different optimizers drive the per-cell
/// error down from the warm start, to see whether the Adam iteration budget can be cut.
/// Run with: `cargo test --release --features mle --lib experiment_optimizers -- --ignored --nocapture`.
// Uses `println!` and `std::time::Instant`, so it is limited to std builds.
#[cfg(all(feature = "mle", feature = "std"))]
fn experiment_optimizers<const M: usize, const N: usize>(unit: u64) {
    type Counter =
            HyperLogLog<
                crate::prelude::Precision8,
                crate::prelude::Bits6,
                <crate::prelude::Precision8 as crate::prelude::PackedRegister<
                    crate::prelude::Bits6,
                >>::Array,
                twox_hash::XxHash64,
            >;
    let n_overlap = M * N;
    let k = n_overlap + M + N;

    // Varied-cell partition (matching joint_matrix_bench), register mode. `exact[rho]` holds
    // each region's true cardinality for the per-cell error.
    let mut exact = vec![0.0_f64; k];
    let mut cursor = 0u64;
    let mut ro = [[(0u64, 0u64); N]; M];
    for i in 0..M {
        for j in 0..N {
            let count = unit * (2 + ((i * 7 + j * 3) % 5) as u64);
            ro[i][j] = (cursor, count);
            cursor += count;
            exact[i * N + j] = count as f64;
        }
    }
    let mut rda = [(0u64, 0u64); M];
    for i in 0..M {
        let count = unit * (1 + (i % 3) as u64);
        rda[i] = (cursor, count);
        cursor += count;
        exact[n_overlap + i] = count as f64;
    }
    let mut rdb = [(0u64, 0u64); N];
    for j in 0..N {
        let count = unit * (1 + (j % 4) as u64);
        rdb[j] = (cursor, count);
        cursor += count;
        exact[n_overlap + M + j] = count as f64;
    }
    let total_union: f64 = exact.iter().sum();
    let build = |ranges: &[(u64, u64)]| -> Counter {
        let mut hll = Counter::default();
        for &(start, count) in ranges {
            for v in start..start + count {
                hll.insert(&v);
            }
        }
        hll
    };
    let lefts: [Counter; M] = core::array::from_fn(|i| {
        let mut ranges = Vec::new();
        for ii in 0..=i {
            for j in 0..N {
                ranges.push(ro[ii][j]);
            }
            ranges.push(rda[ii]);
        }
        build(&ranges)
    });
    let rights: [Counter; N] = core::array::from_fn(|j| {
        let mut ranges = Vec::new();
        for jj in 0..=j {
            for i in 0..M {
                ranges.push(ro[i][jj]);
            }
            ranges.push(rdb[jj]);
        }
        build(&ranges)
    });

    // Warm start and anchors (mirroring joint_sketch_mle_core).
    let (overlap0, left0, right0) =
        <Counter as HyperSpheresSketch>::overlap_and_differences_cardinality_matrices(
            &lefts, &rights,
        )
        .into_parts();
    let mut init = vec![0.0; k];
    for i in 0..M {
        for j in 0..N {
            init[i * N + j] = overlap0[i][j].max(f64::EPSILON).ln();
        }
    }
    for i in 0..M {
        init[n_overlap + i] = left0[i].max(f64::EPSILON).ln();
    }
    for j in 0..N {
        init[n_overlap + M + j] = right0[j].max(f64::EPSILON).ln();
    }
    let anchor_weight = f64::integer_exp2(crate::prelude::Precision8::EXPONENT) / 1.04_f64.powi(2);
    let mut anchors: Vec<(Vec<usize>, f64, f64)> = Vec::new();
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

    let value_patterns = tabulate_joint_value_patterns::<_, _, _, _, M, N>(&lefts, &rights);
    let p_exponent = crate::prelude::Precision8::EXPONENT;
    let q_plus_one: u8 = (1 << crate::prelude::Bits6::NUMBER_OF_BITS) - 1;

    let cell_err = |phis: &[f64]| -> f64 {
        let mut e = 0.0;
        for rho in 0..k {
            e += (phis[rho].exp() - exact[rho]).abs() / total_union;
        }
        e / k as f64
    };
    // The MAP objective being maximized (log-likelihood plus marginal-anchor log-prior), value
    // only (for reporting).
    let objective = |phis: &[f64]| -> f64 {
        let ephi: Vec<f64> = phis.iter().map(|p| p.exp()).collect();
        let mut scratch = vec![0.0; k];
        let mut value = 0.0;
        for (a_pat, b_pat, count) in &value_patterns {
            value += joint_pattern_ll_and_gradient_poly::<M, N>(
                a_pat,
                b_pat,
                &ephi,
                p_exponent,
                q_plus_one,
                *count,
                &mut scratch,
            );
        }
        for (regions, log_estimate, weight) in &anchors {
            let sum: f64 = regions.iter().map(|&r| ephi[r]).sum();
            let residual = sum.max(f64::EPSILON).ln() - log_estimate;
            value -= 0.5 * weight * residual * residual;
        }
        value
    };
    // The same objective with its ascent gradient (the closure each optimizer drives).
    let mut map_objective = |phis: &[f64], gradient: &mut [f64]| -> f64 {
        let ephi: Vec<f64> = phis.iter().map(|p| p.exp()).collect();
        let mut value = 0.0;
        for (a_pat, b_pat, count) in &value_patterns {
            value += joint_pattern_ll_and_gradient_poly::<M, N>(
                a_pat, b_pat, &ephi, p_exponent, q_plus_one, *count, gradient,
            );
        }
        add_marginal_anchor_gradient(&anchors, phis, gradient);
        for (regions, log_estimate, weight) in &anchors {
            let sum: f64 = regions.iter().map(|&r| ephi[r]).sum();
            let residual = sum.max(f64::EPSILON).ln() - log_estimate;
            value -= 0.5 * weight * residual * residual;
        }
        value
    };

    let step_tolerance = 1e-2 / (1u64 << 8) as f64;
    println!(
            "\n=== M={M} N={N} P8 unit={unit} (higher obj = better fit; lower cell_err = more accurate) ==="
        );
    // Each optimizer is selected at compile time by type and run explicitly (the trait is no
    // longer object-safe).
    macro_rules! run {
        ($name:expr, $optimizer:ty) => {{
            let start = std::time::Instant::now();
            let result = <$optimizer as JointOptimizer>::maximize(
                init.clone(),
                &mut map_objective,
                step_tolerance,
            );
            let elapsed_ms = start.elapsed().as_secs_f64() * 1e3;
            println!(
                "{:<20} cell_err={:>6.3}%  obj={:>10.2}  time={:>8.2}ms",
                $name,
                100.0 * cell_err(&result),
                objective(&result),
                elapsed_ms
            );
        }};
    }
    run!("lbfgs", Lbfgs);
    run!("adam", Adam);
    run!("rmsprop", RmsProp);
    run!("adam+lbfgs", Chain<Adam, Lbfgs>);
    run!("rmsprop+lbfgs", Chain<RmsProp, Lbfgs>);
}

#[cfg(all(feature = "mle", feature = "std"))]
#[test]
#[ignore]
fn experiment_optimizers_run() {
    experiment_optimizers::<4, 4>(256);
    experiment_optimizers::<5, 5>(256);
}

/// Builds nested left/right counters from disjoint integer ranges with known-exact cells, sized so
/// every counter stays in hash-list mode. Returns the counters and the exact cell cardinalities.
#[cfg(feature = "mle")]
#[allow(clippy::type_complexity)]
fn build_nested_hash_lists<P, B, const M: usize, const N: usize>(
    unit: u64,
) -> (
    [HyperLogLog<P, B, <P as PackedRegister<B>>::Array, twox_hash::XxHash64>; M],
    [HyperLogLog<P, B, <P as PackedRegister<B>>::Array, twox_hash::XxHash64>; N],
    [[f64; N]; M],
    [f64; M],
    [f64; N],
)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    type Counter<P, B> = HyperLogLog<P, B, <P as PackedRegister<B>>::Array, twox_hash::XxHash64>;
    let build = |ranges: &[(u64, u64)]| -> Counter<P, B> {
        let mut hll = Counter::<P, B>::default();
        for &(start, count) in ranges {
            for v in start..start + count {
                hll.insert(&v);
            }
        }
        hll
    };
    let mut cursor = 0u64;
    let mut overlap = [[0.0_f64; N]; M];
    let mut left_diff = [0.0_f64; M];
    let mut right_diff = [0.0_f64; N];
    let mut ranges_o = [[(0u64, 0u64); N]; M];
    for i in 0..M {
        for j in 0..N {
            let count = unit * (2 + ((i * 5 + j * 3) % 4) as u64);
            ranges_o[i][j] = (cursor, count);
            overlap[i][j] = count as f64;
            cursor += count;
        }
    }
    let mut ranges_da = [(0u64, 0u64); M];
    for i in 0..M {
        let count = unit * (1 + (i % 2) as u64);
        ranges_da[i] = (cursor, count);
        left_diff[i] = count as f64;
        cursor += count;
    }
    let mut ranges_db = [(0u64, 0u64); N];
    for j in 0..N {
        let count = unit * (1 + (j % 3) as u64);
        ranges_db[j] = (cursor, count);
        right_diff[j] = count as f64;
        cursor += count;
    }
    let lefts: [Counter<P, B>; M] = core::array::from_fn(|i| {
        let mut ranges = Vec::new();
        for ii in 0..=i {
            for j in 0..N {
                ranges.push(ranges_o[ii][j]);
            }
            ranges.push(ranges_da[ii]);
        }
        build(&ranges)
    });
    let rights: [Counter<P, B>; N] = core::array::from_fn(|j| {
        let mut ranges = Vec::new();
        for jj in 0..=j {
            for i in 0..M {
                ranges.push(ranges_o[i][jj]);
            }
            ranges.push(ranges_db[jj]);
        }
        build(&ranges)
    });
    (lefts, rights, overlap, left_diff, right_diff)
}

/// The exact path counts each disjoint integer range exactly (collisions are negligible at the high
/// hash sizes of small counters), so every cell matches the known cardinality within one element.
#[cfg(feature = "mle")]
fn check_exact_cells_hash_list<P, B, const M: usize, const N: usize>(unit: u64)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let (lefts, rights, overlap, left_diff, right_diff) =
        build_nested_hash_lists::<P, B, M, N>(unit);
    assert!(
        lefts.iter().all(HyperLogLog::is_hash_list) && rights.iter().all(HyperLogLog::is_hash_list),
        "test inputs must stay in hash-list mode for M={M} N={N} unit={unit}"
    );

    let (ov, ld, rd) =
        joint_sketch_exact_from_hash_lists::<_, _, _, _, M, N>(&lefts, &rights).into_parts();
    let close = |a: f64, b: f64| (a - b).abs() <= (0.05 * b).max(1.0);
    for i in 0..M {
        for j in 0..N {
            assert!(
                close(ov[i][j], overlap[i][j]),
                "M={M} N={N} overlap[{i}][{j}]: got {} exact {}",
                ov[i][j],
                overlap[i][j]
            );
        }
    }
    for i in 0..M {
        assert!(
            close(ld[i], left_diff[i]),
            "M={M} N={N} left_diff[{i}]: got {} exact {}",
            ld[i],
            left_diff[i]
        );
    }
    for j in 0..N {
        assert!(
            close(rd[j], right_diff[j]),
            "M={M} N={N} right_diff[{j}]: got {} exact {}",
            rd[j],
            right_diff[j]
        );
    }
}

#[cfg(feature = "mle")]
#[test]
fn test_exact_joint_sketch_matches_true_cells_hash_list_regime() {
    check_exact_cells_hash_list::<Precision10, Bits6, 1, 1>(5);
    check_exact_cells_hash_list::<Precision10, Bits6, 2, 2>(5);
    check_exact_cells_hash_list::<Precision12, Bits6, 3, 2>(8);
}

#[cfg(feature = "mle")]
#[test]
fn test_exact_joint_reduces_to_union_at_m_n_1() {
    let (lefts, rights, overlap, left_diff, right_diff) =
        build_nested_hash_lists::<Precision10, Bits6, 1, 1>(7);
    let exact_union = overlap[0][0] + left_diff[0] + right_diff[0];
    let (ov, ld, rd) =
        joint_sketch_exact_from_hash_lists::<_, _, _, _, 1, 1>(&lefts, &rights).into_parts();
    let union = ov[0][0] + ld[0] + rd[0];
    assert!(
        (union - exact_union).abs() <= (0.02 * exact_union).max(1.0),
        "union {union} vs exact {exact_union}"
    );
    assert!(
        (ov[0][0] - overlap[0][0]).abs() <= (0.05 * overlap[0][0]).max(1.0),
        "intersection {} vs exact {}",
        ov[0][0],
        overlap[0][0]
    );
}

#[cfg(feature = "mle")]
#[test]
fn test_dispatch_selects_exact_path() {
    // All-hash-list inputs: the union MLE entry must return the exact hash-list union bit-for-bit,
    // and the joint sketch must be optimizer-independent (proving the exact branch was taken).
    let (lefts, rights, ..) = build_nested_hash_lists::<Precision10, Bits6, 2, 2>(6);
    assert_eq!(
        lefts[1].estimate_union_cardinality_mle(&rights[1]),
        lefts[1].estimate_union_cardinality(&rights[1]),
    );

    let default = HyperLogLog::joint_sketch_mle(&lefts, &rights);
    let lbfgs = HyperLogLog::joint_sketch_mle_with::<Lbfgs, 2, 2>(&lefts, &rights);
    assert_eq!(default, lbfgs);
}

#[cfg(feature = "mle")]
#[test]
fn test_estimate_cardinality_mle_hash_list_equals_estimate_cardinality() {
    let (lefts, ..) = build_nested_hash_lists::<Precision10, Bits6, 1, 1>(9);
    assert!(lefts[0].is_hash_list());
    assert_eq!(
        lefts[0].estimate_cardinality_mle(),
        lefts[0].estimate_cardinality()
    );
}

#[cfg(feature = "mle")]
#[test]
fn test_exact_path_deterministic() {
    let (lefts, rights, ..) = build_nested_hash_lists::<Precision10, Bits6, 2, 3>(5);
    let first = joint_sketch_exact_from_hash_lists::<_, _, _, _, 2, 3>(&lefts, &rights);
    let second = joint_sketch_exact_from_hash_lists::<_, _, _, _, 2, 3>(&lefts, &rights);
    assert_eq!(first, second);
}
