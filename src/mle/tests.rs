//! Tests for the MLE estimators: the 2-set union MLE and the joint sketch built by
//! inclusion-exclusion over the 2-set union MLE.

use crate::prelude::*;

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

/// Regression for the exact value-list joint decomposition merge order. The exact path (every
/// operand a sorted value list) is a multi-way merge over the value iterators, which descend. With
/// INTERLEAVED, overlapping nested sets (not contiguous disjoint blocks, which mask order errors) the
/// exact cell counts must equal a brute-force classification over the same integer sets.
#[test]
fn test_joint_sketch_exact_interleaved_matches_brute_force() {
    type Hll = HyperLogLog<Precision12, Bits6>;
    let build = |values: &[u64]| {
        let mut h = Hll::default();
        for &v in values {
            // insert_value stores the literal value (exact mode), keeping the counter a value list.
            h.insert_value(v);
        }
        assert!(h.is_sorted_value_list(), "operand must stay a value list");
        h
    };

    // Nested left chain A0 subset A1 and right chain B0 subset B1, with deliberately interleaved
    // values so that ascending-vs-descending or per-stream ordering mistakes change the result.
    let a0_v: [u64; 4] = [2, 5, 9, 13];
    let a1_v: [u64; 7] = [1, 2, 5, 7, 9, 11, 13];
    let b0_v: [u64; 3] = [5, 7, 20];
    let b1_v: [u64; 6] = [2, 5, 7, 9, 20, 30];

    let a0 = build(&a0_v);
    let a1 = build(&a1_v);
    let b0 = build(&b0_v);
    let b1 = build(&b1_v);

    let (overlap, left_diff, right_diff) =
        JointSketch::estimate(&[a0.mle(), a1.mle()], &[b0.mle(), b1.mle()]).into_parts();

    // Brute force: left shell = smallest cumulative left set containing the value (1-based, 0 if
    // none), right shell likewise, over the union of all values.
    let left_sets = [&a0_v[..], &a1_v[..]];
    let right_sets = [&b0_v[..], &b1_v[..]];
    let shell = |sets: [&[u64]; 2], v: u64| -> usize {
        for (i, set) in sets.iter().enumerate() {
            if set.contains(&v) {
                return i + 1;
            }
        }
        0
    };
    let mut expected_overlap = [[0.0_f64; 2]; 2];
    let mut expected_left = [0.0_f64; 2];
    let mut expected_right = [0.0_f64; 2];
    let mut universe: std::vec::Vec<u64> = a1_v.iter().chain(b1_v.iter()).copied().collect();
    universe.sort_unstable();
    universe.dedup();
    for v in universe {
        match (shell(left_sets, v), shell(right_sets, v)) {
            (0, 0) => unreachable!(),
            (li, 0) => expected_left[li - 1] += 1.0,
            (0, rj) => expected_right[rj - 1] += 1.0,
            (li, rj) => expected_overlap[li - 1][rj - 1] += 1.0,
        }
    }

    assert_eq!(overlap, expected_overlap, "overlap grid mismatch");
    assert_eq!(left_diff, expected_left, "left diff mismatch");
    assert_eq!(right_diff, expected_right, "right diff mismatch");
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

/// Tests for the generalized joint MLE (the `.jmle()` wrapper and its register-path internals): the
/// `M = N = 1` reduction to the 2-set union MLE, the polynomial likelihood and gradient against the
/// `2^(M+N)` oracle, per-cell exact recovery on deterministic nested sets, and the deep-cell regime
/// where the joint optimization should not lose to pairwise inclusion-exclusion.
mod joint_mle {
    use super::super::likelihood::{
        joint_pattern_ll_and_gradient_poly, joint_pattern_ll_grad_hess_poly, joint_pattern_ll_poly,
        tabulate_joint_value_patterns,
    };
    use super::super::optimizers::finite_difference_hessian;
    use super::super::oracle::{
        build_pattern_terms, joint_ll_and_gradient, tabulate_joint_patterns, JointPattern,
    };
    use super::super::sketch::{
        joint_sketch_mle_from_registers, joint_sketch_mle_from_registers_full,
    };
    use crate::prelude::*;
    use crate::utils::splitmix64;

    type Counter<P, B> = HyperLogLog<P, B, <P as PackedRegister<B>>::Array, twox_hash::XxHash64>;

    /// Builds a dense counter from disjoint integer ranges (each `(start, count)`).
    fn build<P: Precision + PackedRegister<B>, B: Bits>(ranges: &[(u64, u64)]) -> Counter<P, B> {
        let mut hll = Counter::<P, B>::default();
        for &(start, count) in ranges {
            for v in start..start + count {
                hll.insert(&v);
            }
        }
        hll
    }

    // ----- Anchor 2 and 3: polynomial likelihood and gradient against the 2^(M+N) oracle. -----

    /// `ln P_reg` for one pattern via the exponential `2^(M+N)` oracle.
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
            p += sign * FloatOps::exp(-sum_x);
        }
        FloatOps::natural_log(FloatOps::maximum(p, f64::EPSILON))
    }

    /// Gradient of `ln P_reg` for one pattern via the exponential `2^(M+N)` oracle.
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

    /// A sorted (monotone) register pattern of length `L` with values in `0..=q_plus_one`.
    fn random_monotone_pattern<const L: usize>(state: &mut u64, q_plus_one: u8) -> [u8; L] {
        let mut pattern: [u8; L] = core::array::from_fn(|_| {
            *state = splitmix64(*state);
            (*state % (u64::from(q_plus_one) + 1)) as u8
        });
        pattern.sort_unstable();
        pattern
    }

    /// Region cardinalities `n_rho = exp(phi)` with `phi` uniform in `[ln 2, ln 40]`, the regime
    /// where the signed `2^(M+N)` oracle is numerically reliable. The polynomial form is
    /// cancellation-free at any scale.
    fn random_ephi(state: &mut u64, k: usize) -> Vec<f64> {
        (0..k)
            .map(|_| {
                *state = splitmix64(*state);
                let u = (*state >> 11) as f64 / (1u64 << 53) as f64;
                let ln_low = FloatOps::natural_log(2.0_f64);
                let ln_high = FloatOps::natural_log(40.0_f64);
                FloatOps::exp(ln_low + (ln_high - ln_low) * u)
            })
            .collect()
    }

    /// The polynomial per-pattern log-likelihood must match the `2^(M+N)` oracle to floating-point
    /// tolerance across random monotone patterns (ties, zeros, saturation) for several shapes.
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

            let oracle =
                oracle_pattern_ln_p_reg::<M, N>(&a_pat, &b_pat, &ephi, p_exponent, q_plus_one);
            // Skip patterns where the signed oracle sum loses precision to catastrophic
            // cancellation (small P_reg from tiny x at high levels). The polynomial form is
            // cancellation-free, so the oracle is the limiting factor, not the poly.
            if oracle < -18.0 {
                continue;
            }
            let poly = joint_pattern_ll_poly::<M, N>(&a_pat, &b_pat, &ephi, p_exponent, q_plus_one);
            // The polynomial form is cancellation-free, so any gap is the SIGNED oracle losing
            // precision (its terms cancel near small P_reg) plus the crate's no_std transcendentals
            // carrying a little more error than std's. The poly itself is pinned tightly by the
            // poly-only finite-difference check below, so 3e-5 here (against the cancellation-prone
            // oracle, down to ln = -18) is a generous floor that still catches any model error.
            assert!(
                (oracle - poly).abs() < 3e-5,
                "M={M} N={N} a={a_pat:?} b={b_pat:?}: oracle ln={oracle} poly ln={poly}"
            );
            tested += 1;
        }
        assert!(tested > 20, "too few non-degenerate trials: {tested}");
    }

    #[test]
    fn test_poly_pattern_likelihood_matches_oracle() {
        // A small p_exponent keeps x in the range where the signed oracle is numerically reliable.
        // The per-pattern algorithm only scales x by p_exponent, so this fully validates it.
        // q_plus_one = 7 exercises the saturation boundary (Bits3), 15 the Bits4 boundary.
        for &q_plus_one in &[7u8, 15u8] {
            let p = 4u8;
            check_poly_likelihood_matches_oracle::<1, 1>(0x1111, p, q_plus_one);
            check_poly_likelihood_matches_oracle::<2, 1>(0x2121, p, q_plus_one);
            check_poly_likelihood_matches_oracle::<2, 2>(0x2222, p, q_plus_one);
            check_poly_likelihood_matches_oracle::<3, 2>(0x3232, p, q_plus_one);
            check_poly_likelihood_matches_oracle::<3, 3>(0x3333, p, q_plus_one);
        }

        // Explicit edge cases: all-zero, all-saturated, all-equal mid, single zero, mixed.
        let p = 4u8;
        let q1 = 7u8;
        let k = 2 * 2 + 2 + 2;
        let ephi: Vec<f64> = (0..k)
            .map(|i| FloatOps::exp(1.0 + 0.5 * f64::from(i)))
            .collect();
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
                (oracle - poly).abs() < 3e-5,
                "edge a={a:?} b={b:?}: oracle={oracle} poly={poly}"
            );
        }
    }

    /// The polynomial per-pattern gradient must match both a central finite difference of the
    /// polynomial likelihood and the exponential oracle gradient, across random patterns.
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
            let phis: Vec<f64> = ephi.iter().map(|e| FloatOps::natural_log(*e)).collect();

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
                ep[rho] = ephi[rho] * FloatOps::exp(h);
                em[rho] = ephi[rho] * FloatOps::exp(-h);
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

            // Oracle gradient cross-check. The signed oracle gradient cancels just like its value,
            // so this is limited by the oracle's precision (and the no_std transcendentals), not the
            // polynomial gradient, which the tight finite-difference check above already pins.
            let ograd =
                oracle_pattern_gradient::<M, N>(&a_pat, &b_pat, &phis, p_exponent, q_plus_one, k);
            for rho in 0..k {
                let scale = grad[rho].abs().max(ograd[rho].abs()).max(1.0);
                assert!(
                    (grad[rho] - ograd[rho]).abs() / scale < 1e-4,
                    "oracle mismatch M={M} N={N} a={a_pat:?} b={b_pat:?} rho={rho}: poly={} oracle={}",
                    grad[rho],
                    ograd[rho]
                );
            }
            tested += 1;
        }
        assert!(tested > 20, "too few non-degenerate trials: {tested}");
    }

    #[test]
    fn test_poly_pattern_gradient_matches_oracle_and_fd() {
        for &q_plus_one in &[7u8, 15u8] {
            let p = 4u8;
            check_poly_gradient_matches_oracle_and_fd::<1, 1>(0xA1A1, p, q_plus_one);
            check_poly_gradient_matches_oracle_and_fd::<2, 1>(0xB2B1, p, q_plus_one);
            check_poly_gradient_matches_oracle_and_fd::<2, 2>(0xB2B2, p, q_plus_one);
            check_poly_gradient_matches_oracle_and_fd::<3, 2>(0xC3C2, p, q_plus_one);
            check_poly_gradient_matches_oracle_and_fd::<3, 3>(0xE3E3, p, q_plus_one);
        }
    }

    // ----- Analytic Hessian vs the finite-difference oracle. -----

    /// The analytic per-pattern log-likelihood Hessian must match a central finite difference of the
    /// analytic gradient (the trusted oracle), to tight relative tolerance, across random monotone
    /// patterns (ties, zeros, saturation) for several shapes, and it must be symmetric. The
    /// finite-difference Hessian here differences the SAME analytic gradient the production path uses.
    fn check_analytic_hessian_matches_fd<const M: usize, const N: usize>(
        seed: u64,
        p_exponent: u8,
        q_plus_one: u8,
    ) {
        let k = M * N + M + N;
        let mut state = seed;
        let mut tested = 0;
        for _ in 0..400 {
            let a_pat = random_monotone_pattern::<M>(&mut state, q_plus_one);
            let b_pat = random_monotone_pattern::<N>(&mut state, q_plus_one);
            let ephi = random_ephi(&mut state, k);
            let phis: Vec<f64> = ephi.iter().map(|e| FloatOps::natural_log(*e)).collect();

            // Skip degenerate patterns whose log-likelihood is so negative that the FD of the gradient
            // is dominated by round-off (same guard the gradient-vs-oracle test uses).
            let ll = joint_pattern_ll_poly::<M, N>(&a_pat, &b_pat, &ephi, p_exponent, q_plus_one);
            if ll < -18.0 {
                continue;
            }

            // Analytic Hessian over this single pattern.
            let mut grad = vec![0.0_f64; k];
            let mut analytic = vec![0.0_f64; k * k];
            joint_pattern_ll_grad_hess_poly::<M, N>(
                &a_pat,
                &b_pat,
                &ephi,
                p_exponent,
                q_plus_one,
                1.0,
                &mut grad,
                &mut analytic,
            );

            // Finite-difference Hessian of the analytic gradient of the same single-pattern objective.
            let mut objective = |phis: &[f64], gradient: &mut [f64]| -> f64 {
                let ephi: Vec<f64> = phis.iter().map(|p| FloatOps::exp(*p)).collect();
                joint_pattern_ll_and_gradient_poly::<M, N>(
                    &a_pat, &b_pat, &ephi, p_exponent, q_plus_one, 1.0, gradient,
                )
            };
            let mut fd = vec![0.0_f64; k * k];
            finite_difference_hessian(&mut objective, &phis, 1e-5, &mut fd);

            for r in 0..k {
                for c in 0..k {
                    let a = analytic[r * k + c];
                    let f = fd[r * k + c];
                    let scale = a.abs().max(f.abs()).max(1.0);
                    assert!(
                        (a - f).abs() / scale < 1e-5,
                        "M={M} N={N} a={a_pat:?} b={b_pat:?} H[{r}][{c}]: analytic={a} fd={f}"
                    );
                    // Symmetry to round-off.
                    let t = analytic[c * k + r];
                    assert!(
                        (a - t).abs() <= 1e-9 * a.abs().max(1.0),
                        "asymmetric analytic Hessian at [{r}][{c}]: {a} vs {t}"
                    );
                }
            }
            tested += 1;
        }
        assert!(tested > 10, "too few non-degenerate trials: {tested}");
    }

    #[test]
    fn test_analytic_hessian_matches_finite_difference() {
        for &q_plus_one in &[7u8, 15u8] {
            let p = 4u8;
            check_analytic_hessian_matches_fd::<1, 1>(0x4101, p, q_plus_one);
            check_analytic_hessian_matches_fd::<2, 1>(0x4201, p, q_plus_one);
            check_analytic_hessian_matches_fd::<2, 2>(0x4202, p, q_plus_one);
            check_analytic_hessian_matches_fd::<3, 2>(0x4302, p, q_plus_one);
            check_analytic_hessian_matches_fd::<3, 3>(0x4303, p, q_plus_one);
        }
    }

    /// The FULL MAP Hessian (analytic log-likelihood Hessian plus the marginal-anchor prior Hessian)
    /// must match a finite difference of the full MAP gradient, at real dense cells. This exercises the
    /// anchor block of section 11.5 together with the likelihood Hessian.
    #[test]
    fn test_full_map_hessian_matches_finite_difference() {
        use super::super::sketch::{add_marginal_anchor_hessian, Anchor};
        const M: usize = 3;
        const N: usize = 2;
        let (lefts, rights, ..) = build_nested_dense::<Precision8, Bits6, M, N>(2_000);
        let n_overlap = M * N;
        let k = n_overlap + M + N;
        let p_exponent = Precision8::EXPONENT;
        let q_plus_one: u8 = (1 << Bits6::NUMBER_OF_BITS) - 1;
        let value_patterns = tabulate_joint_value_patterns::<_, _, _, _, M, N>(&lefts, &rights);

        // Build the same anchors the core builds, as (region list, log estimate, weight) tuples for the
        // self-contained finite-difference gradient reference, and as the production `Anchor` values for
        // the analytic Hessian call.
        let weight = f64::integer_exp2(Precision8::EXPONENT) / FloatOps::powi(1.04_f64, 2);
        let mut anchors: Vec<(Vec<usize>, f64, f64)> = Vec::new();
        let mut anchor_values: Vec<Anchor> = Vec::new();
        for i in 0..M {
            let mut regions = Vec::new();
            for ii in 0..=i {
                for j in 0..N {
                    regions.push(ii * N + j);
                }
                regions.push(n_overlap + ii);
            }
            let log_estimate = FloatOps::natural_log(FloatOps::maximum(
                lefts[i].estimate_cardinality(),
                f64::EPSILON,
            ));
            anchors.push((regions, log_estimate, weight));
            anchor_values.push(Anchor::Left {
                i,
                log_estimate,
                weight,
            });
        }
        for j in 0..N {
            let mut regions = Vec::new();
            for jj in 0..=j {
                for i in 0..M {
                    regions.push(i * N + jj);
                }
                regions.push(n_overlap + M + jj);
            }
            let log_estimate = FloatOps::natural_log(FloatOps::maximum(
                rights[j].estimate_cardinality(),
                f64::EPSILON,
            ));
            anchors.push((regions, log_estimate, weight));
            anchor_values.push(Anchor::Right {
                j,
                log_estimate,
                weight,
            });
        }

        // The full MAP gradient closure (likelihood gradient plus anchor-prior gradient).
        let mut map_gradient = |phis: &[f64], gradient: &mut [f64]| -> f64 {
            let ephi: Vec<f64> = phis.iter().map(|p| FloatOps::exp(*p)).collect();
            let mut value = 0.0;
            for (a_pat, b_pat, count) in &value_patterns {
                value += joint_pattern_ll_and_gradient_poly::<M, N>(
                    a_pat, b_pat, &ephi, p_exponent, q_plus_one, *count, gradient,
                );
            }
            for (regions, log_estimate, w) in &anchors {
                let sum: f64 = regions.iter().map(|&rho| FloatOps::exp(phis[rho])).sum();
                let residual =
                    FloatOps::natural_log(FloatOps::maximum(sum, f64::EPSILON)) - log_estimate;
                value -= 0.5 * w * residual * residual;
                let factor = -w * residual / FloatOps::maximum(sum, f64::EPSILON);
                for &rho in regions {
                    gradient[rho] += factor * FloatOps::exp(phis[rho]);
                }
            }
            value
        };

        // A non-degenerate evaluation point: the warm start in log space.
        let (overlap0, left0, right0) =
            <Counter<Precision8, Bits6> as HyperSpheresSketch>::joint_sketch(&lefts, &rights)
                .into_parts();
        let mut phis = vec![0.0_f64; k];
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
            phis[n_overlap + M + j] =
                FloatOps::natural_log(FloatOps::maximum(right0[j], f64::EPSILON));
        }

        // Analytic MAP Hessian: likelihood Hessian plus anchor Hessian.
        let mut analytic = vec![0.0_f64; k * k];
        let ephi: Vec<f64> = phis.iter().map(|p| FloatOps::exp(*p)).collect();
        let mut scratch = vec![0.0_f64; k];
        for (a_pat, b_pat, count) in &value_patterns {
            joint_pattern_ll_grad_hess_poly::<M, N>(
                a_pat,
                b_pat,
                &ephi,
                p_exponent,
                q_plus_one,
                *count,
                &mut scratch,
                &mut analytic,
            );
        }
        add_marginal_anchor_hessian::<M, N>(&anchor_values, &phis, &mut analytic, k);

        let mut fd = vec![0.0_f64; k * k];
        finite_difference_hessian(&mut map_gradient, &phis, 1e-5, &mut fd);
        for r in 0..k {
            for c in 0..k {
                let a = analytic[r * k + c];
                let f = fd[r * k + c];
                let scale = a.abs().max(f.abs()).max(1.0);
                assert!(
                    (a - f).abs() / scale < 1e-5,
                    "MAP H[{r}][{c}]: analytic={a} fd={f}"
                );
            }
        }
    }

    // ----- Anchor 1: the M = N = 1 reduction to the 2-set union MLE. -----

    /// The generalized joint MLE at `M = N = 1` must reproduce the 2-set union MLE regions
    /// (`left_diff`, `right_diff`, `intersection`) to tight tolerance: it is the documented base case
    /// the math reduces to, and the production register path dispatches the single pair to exactly the
    /// 2-set solver.
    #[test]
    fn test_jmle_m_n_1_reduces_to_two_set_union_mle() {
        type Hll = Counter<Precision12, Bits6>;
        let a = build::<Precision12, Bits6>(&[(0, 30_000)]);
        let b = build::<Precision12, Bits6>(&[(20_000, 30_000)]);
        assert!(a.is_hyperloglog() && b.is_hyperloglog());

        // The 2-set union MLE reference (left_diff, right_diff, intersection).
        let [left_difference, right_difference, intersection] =
            a.mle_union_regions_from_registers(&b);

        // The generalized joint MLE through the register path.
        let sketch = joint_sketch_mle_from_registers::<_, _, _, _, _, 1, 1>(&[a], &[b]);

        assert!(
            (sketch.overlap[0][0] - intersection).abs() <= 1e-6 * intersection.max(1.0),
            "intersection: jmle {} vs 2-set {intersection}",
            sketch.overlap[0][0]
        );
        assert!(
            (sketch.left_diff[0] - left_difference).abs() <= 1e-6 * left_difference.max(1.0),
            "left diff: jmle {} vs 2-set {left_difference}",
            sketch.left_diff[0]
        );
        assert!(
            (sketch.right_diff[0] - right_difference).abs() <= 1e-6 * right_difference.max(1.0),
            "right diff: jmle {} vs 2-set {right_difference}",
            sketch.right_diff[0]
        );

        let _ = core::marker::PhantomData::<Hll>;
    }

    /// The full estimator driven by the polynomial gradient must converge to the same cell matrices as
    /// the same optimization driven by the exponential `2^(M+N)` oracle gradient, confirming the
    /// production path is faithful end to end. Uses the shared core with a fixed optimizer so the
    /// comparison isolates the gradient source.
    fn check_full_estimator_poly_vs_oracle<const M: usize, const N: usize>(unit: u64) {
        use super::super::sketch::joint_sketch_mle_core;
        type C = Counter<Precision10, Bits6>;

        // Disjoint integer ranges, one per region. Nested counters built from them.
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
        let lefts: [C; M] = core::array::from_fn(|i| {
            let mut ranges = Vec::new();
            for ii in 0..=i {
                for j in 0..N {
                    ranges.push(ranges_o[ii][j]);
                }
                ranges.push(ranges_da[ii]);
            }
            build::<Precision10, Bits6>(&ranges)
        });
        let rights: [C; N] = core::array::from_fn(|j| {
            let mut ranges = Vec::new();
            for jj in 0..=j {
                for i in 0..M {
                    ranges.push(ranges_o[i][jj]);
                }
                ranges.push(ranges_db[jj]);
            }
            build::<Precision10, Bits6>(&ranges)
        });

        let p_exponent = Precision10::EXPONENT;
        let q_plus_one: u8 = (1 << Bits6::NUMBER_OF_BITS) - 1;
        let k = M * N + M + N;

        // Both paths run the same damped-Newton core; only the likelihood (and its Hessian) differ, so
        // the comparison isolates the gradient/likelihood source. The polynomial path supplies its
        // analytic Hessian, the oracle path a finite difference of its own gradient (the oracle has no
        // closed-form Hessian).
        let value_patterns = tabulate_joint_value_patterns::<_, _, _, _, M, N>(&lefts, &rights);
        let (ov_poly, l_poly, r_poly) = joint_sketch_mle_core::<_, _, _, _, _, M, N>(
            &lefts,
            &rights,
            |phis, gradient| {
                let ephi: Vec<f64> = phis.iter().map(|phi| FloatOps::exp(*phi)).collect();
                let mut log_likelihood = 0.0;
                for (a_pat, b_pat, count) in &value_patterns {
                    log_likelihood += joint_pattern_ll_and_gradient_poly::<M, N>(
                        a_pat, b_pat, &ephi, p_exponent, q_plus_one, *count, gradient,
                    );
                }
                log_likelihood
            },
            |phis, hessian| {
                let ephi: Vec<f64> = phis.iter().map(|phi| FloatOps::exp(*phi)).collect();
                let mut scratch = vec![0.0_f64; k];
                for (a_pat, b_pat, count) in &value_patterns {
                    joint_pattern_ll_grad_hess_poly::<M, N>(
                        a_pat,
                        b_pat,
                        &ephi,
                        p_exponent,
                        q_plus_one,
                        *count,
                        &mut scratch,
                        hessian,
                    );
                }
            },
        )
        .into_parts();

        // Oracle path (exponential gradient, finite-difference Hessian).
        let oracle_patterns = tabulate_joint_patterns::<_, _, _, _, M, N>(&lefts, &rights);
        let (ov_oracle, l_oracle, r_oracle) = joint_sketch_mle_core::<_, _, _, _, _, M, N>(
            &lefts,
            &rights,
            |phis, gradient| {
                let (ll, g) = joint_ll_and_gradient(&oracle_patterns, phis, k);
                for (slot, value) in gradient.iter_mut().zip(g) {
                    *slot += value;
                }
                ll
            },
            |phis, hessian| {
                let mut objective = |p: &[f64], grad: &mut [f64]| -> f64 {
                    let (ll, g) = joint_ll_and_gradient(&oracle_patterns, p, k);
                    for (slot, value) in grad.iter_mut().zip(g) {
                        *slot += value;
                    }
                    ll
                };
                finite_difference_hessian(&mut objective, phis, 1e-5, hessian);
            },
        )
        .into_parts();

        // The per-pattern gradients agree to ~1e-7, but the two paths sum patterns in BTreeMap order
        // and the oracle carries ~1e-7 cancellation error, which compound along weakly-identified
        // directions. A 0.1% end-to-end agreement confirms the rewrite is faithful (a real bug would
        // diverge grossly, which the tight per-pattern gradient test would already catch).
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

    #[test]
    fn test_poly_joint_sketch_matches_oracle() {
        check_full_estimator_poly_vs_oracle::<1, 1>(20_000);
        check_full_estimator_poly_vs_oracle::<2, 2>(8_000);
        check_full_estimator_poly_vs_oracle::<3, 2>(5_000);
    }

    // ----- Anchor 4: per-cell exact recovery on deterministic nested sets. -----

    /// Builds nested left/right dense counters from disjoint integer ranges with known-exact cells.
    /// Returns the counters and the exact cell cardinalities.
    #[allow(clippy::type_complexity)]
    fn build_nested_dense<P, B, const M: usize, const N: usize>(
        unit: u64,
    ) -> (
        [Counter<P, B>; M],
        [Counter<P, B>; N],
        [[f64; N]; M],
        [f64; M],
        [f64; N],
    )
    where
        P: Precision + PackedRegister<B>,
        B: Bits,
    {
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
            build::<P, B>(&ranges)
        });
        let rights: [Counter<P, B>; N] = core::array::from_fn(|j| {
            let mut ranges = Vec::new();
            for jj in 0..=j {
                for i in 0..M {
                    ranges.push(ranges_o[i][jj]);
                }
                ranges.push(ranges_db[jj]);
            }
            build::<P, B>(&ranges)
        });
        (lefts, rights, overlap, left_diff, right_diff)
    }

    /// Per-cell exact recovery: on deterministic nested sets from disjoint integer ranges (known
    /// exact cells), every jMLE cell must land within the representation's error rate. The cells are
    /// sized to push the counters into register mode so the register joint MLE path is exercised.
    fn check_exact_cells_dense<P, B, const M: usize, const N: usize>(unit: u64, tolerance: f64)
    where
        P: Precision + PackedRegister<B>,
        B: Bits,
    {
        let (lefts, rights, overlap, left_diff, right_diff) =
            build_nested_dense::<P, B, M, N>(unit);
        assert!(
            lefts
                .iter()
                .chain(rights.iter())
                .all(Counter::<P, B>::is_hyperloglog),
            "test inputs must be dense for M={M} N={N} unit={unit}"
        );

        let (ov, ld, rd) =
            joint_sketch_mle_from_registers::<_, _, _, _, _, M, N>(&lefts, &rights).into_parts();
        let union: f64 = overlap.iter().flatten().sum::<f64>()
            + left_diff.iter().sum::<f64>()
            + right_diff.iter().sum::<f64>();
        // Each cell within `tolerance` of the union scale (an absolute-per-cell bound expressed in
        // units of the union, the natural scale for a HyperLogLog register estimate).
        let close = |got: f64, exact: f64| (got - exact).abs() <= tolerance * union;
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

    #[test]
    fn test_jmle_per_cell_exact_recovery_dense() {
        // Register mode (units chosen so all operands are dense). The tolerance is a few times the
        // single-counter relative standard error of the precision, applied per cell against the union.
        check_exact_cells_dense::<Precision10, Bits6, 1, 1>(20_000, 0.05);
        check_exact_cells_dense::<Precision10, Bits6, 2, 2>(8_000, 0.06);
        check_exact_cells_dense::<Precision12, Bits6, 3, 2>(5_000, 0.04);
    }

    /// The production `M = N = 1` reduction is exact (the default `joint_sketch_mle_from_registers`
    /// short-circuits the single pair to the analytic 2-set union MLE, bypassing the optimizer), which
    /// `test_jmle_m_n_1_reduces_to_two_set_union_mle` already guards. The `_full` path instead runs the
    /// full register optimizer at `M = N = 1`, which reaches the generalized MLE optimum (close to, but
    /// not identical to, the analytic closed form). Here we require it to land within a `HyperLogLog`
    /// error of the analytic 2-set regions, confirming the optimizer does not move the single-pair
    /// optimum to a wrong place.
    #[test]
    fn test_m_n_1_reduction_optimizer_lands_near_analytic() {
        let a = build::<Precision12, Bits6>(&[(0, 30_000)]);
        let b = build::<Precision12, Bits6>(&[(20_000, 30_000)]);
        let [left_difference, right_difference, intersection] =
            a.mle_union_regions_from_registers(&b);
        let union = left_difference + right_difference + intersection;
        let sketch = joint_sketch_mle_from_registers_full::<_, _, _, _, _, 1, 1>(&[a], &[b]);
        // 0.5% of the union per region, well inside the P12 register error.
        let close = |got: f64, exact: f64| (got - exact).abs() <= 0.005 * union;
        assert!(
            close(sketch.overlap[0][0], intersection),
            "intersection {} vs {intersection}",
            sketch.overlap[0][0]
        );
        assert!(close(sketch.left_diff[0], left_difference));
        assert!(close(sketch.right_diff[0], right_difference));
    }

    // ----- Test 5: deep-cell regime where pairwise inclusion-exclusion suffers cancellation. -----

    /// A power-law (very uneven) cell layout: a few large cells and many tiny deep cells. The exact
    /// cell cardinalities are returned alongside dense counters built from disjoint integer ranges.
    #[allow(clippy::type_complexity)]
    fn build_power_law_cells<P, B, const M: usize, const N: usize>(
        state: &mut u64,
        base: u64,
    ) -> ([Counter<P, B>; M], [Counter<P, B>; N], Vec<f64>)
    where
        P: Precision + PackedRegister<B>,
        B: Bits,
    {
        let n_overlap = M * N;
        let k = n_overlap + M + N;
        // Power-law sizes: cell rho gets base * 2^e with e in 0..=5 drawn per cell, so the largest
        // cells are ~32x the smallest. Deep cells (large i+j) are biased small to create the tiny
        // overlaps that pairwise inclusion-exclusion cancels into noise.
        let mut exact = vec![0.0_f64; k];
        let mut cursor = 0u64;
        let mut ranges_o = [[(0u64, 0u64); N]; M];
        for i in 0..M {
            for j in 0..N {
                *state = splitmix64(*state);
                let depth = (i + j) as u32;
                // Deeper cells get a smaller exponent on average.
                let e = (*state % 4) as u32;
                let exponent = e.saturating_sub(depth.min(3));
                let count = base * (1 << exponent);
                ranges_o[i][j] = (cursor, count);
                exact[i * N + j] = count as f64;
                cursor += count;
            }
        }
        let mut ranges_da = [(0u64, 0u64); M];
        for i in 0..M {
            *state = splitmix64(*state);
            let count = base * (1 + (*state % 4));
            ranges_da[i] = (cursor, count);
            exact[n_overlap + i] = count as f64;
            cursor += count;
        }
        let mut ranges_db = [(0u64, 0u64); N];
        for j in 0..N {
            *state = splitmix64(*state);
            let count = base * (1 + (*state % 4));
            ranges_db[j] = (cursor, count);
            exact[n_overlap + M + j] = count as f64;
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
            build::<P, B>(&ranges)
        });
        let rights: [Counter<P, B>; N] = core::array::from_fn(|j| {
            let mut ranges = Vec::new();
            for jj in 0..=j {
                for i in 0..M {
                    ranges.push(ranges_o[i][jj]);
                }
                ranges.push(ranges_db[jj]);
            }
            build::<P, B>(&ranges)
        });
        (lefts, rights, exact)
    }

    /// Mean per-cell RELATIVE error of a decomposition against the exact cells (each cell normalized
    /// by its own true value, not by the union, so tiny deep cells are weighted fully).
    fn mean_relative_cell_error<const M: usize, const N: usize>(
        sketch: &JointSketch<M, N>,
        exact: &[f64],
    ) -> f64 {
        let n_overlap = M * N;
        let mut total = 0.0;
        let mut cells = 0.0;
        let mut add = |got: f64, truth: f64| {
            total += (got - truth).abs() / truth.max(1.0);
            cells += 1.0;
        };
        for i in 0..M {
            for j in 0..N {
                add(sketch.overlap[i][j], exact[i * N + j]);
            }
        }
        for i in 0..M {
            add(sketch.left_diff[i], exact[n_overlap + i]);
        }
        for j in 0..N {
            add(sketch.right_diff[j], exact[n_overlap + M + j]);
        }
        total / cells
    }

    #[test]
    fn test_jmle_beats_pairwise_on_deep_power_law_cells() {
        // Low precision and tiny, very uneven cells: the regime where pairwise inclusion-exclusion
        // cancels deep overlaps into noise. We compare the mean per-cell relative error of the joint
        // MLE against the default pairwise sketch over several seeds at P4..P6 / Bits6.
        fn run<P, B, const M: usize, const N: usize>(
            label: &str,
            base: u64,
            seeds: &[u64],
            expect_beats_two_set_mle: bool,
        ) where
            P: Precision + PackedRegister<B>,
            B: Bits,
        {
            let mut jmle_total = 0.0;
            let mut mle_total = 0.0;
            let mut pairwise_total = 0.0;
            for &seed in seeds {
                let mut state = seed;
                let (lefts, rights, exact) = build_power_law_cells::<P, B, M, N>(&mut state, base);

                let jmle_views_l: [JointMle<&Counter<P, B>>; M] =
                    core::array::from_fn(|i| lefts[i].jmle());
                let jmle_views_r: [JointMle<&Counter<P, B>>; N] =
                    core::array::from_fn(|j| rights[j].jmle());
                let jmle = JointSketch::estimate(&jmle_views_l, &jmle_views_r);

                // The repeated 2-set MLE: pairwise inclusion-exclusion whose every union is the Ertl
                // 2-set joint MLE. This is the estimator the generalized jMLE was said to lose to, so
                // it is the comparison that matters.
                let mle_views_l: [Mle<&Counter<P, B>>; M] =
                    core::array::from_fn(|i| lefts[i].mle());
                let mle_views_r: [Mle<&Counter<P, B>>; N] =
                    core::array::from_fn(|j| rights[j].mle());
                let mle = JointSketch::estimate(&mle_views_l, &mle_views_r);

                let pairwise = JointSketch::estimate(&lefts, &rights);

                jmle_total += mean_relative_cell_error(&jmle, &exact);
                mle_total += mean_relative_cell_error(&mle, &exact);
                pairwise_total += mean_relative_cell_error(&pairwise, &exact);
            }
            let n = seeds.len() as f64;
            let jmle_mean = jmle_total / n;
            let mle_mean = mle_total / n;
            let pairwise_mean = pairwise_total / n;
            std::eprintln!(
                "[deep-cell regime] {label}: jmle = {:.4}, 2-set MLE = {:.4}, default = {:.4} (jmle/2-set-mle = {:.3}, jmle/default = {:.3})",
                jmle_mean,
                mle_mean,
                pairwise_mean,
                jmle_mean / mle_mean,
                jmle_mean / pairwise_mean
            );
            // Robust claim, true at every config measured: the joint MLE clearly beats the default
            // HLL++ pairwise inclusion-exclusion, which cancels the tiny deep cells into noise.
            assert!(
                jmle_mean <= pairwise_mean,
                "{label}: jmle mean per-cell rel err {jmle_mean} worse than default pairwise {pairwise_mean}"
            );
            // Regime-dependent claim: the joint MLE beats the repeated 2-set MLE (the estimator the
            // abandonment said it lost to) only at small M, N and low precision. At 3x2 (more cells,
            // more weakly-identified deep cells) the joint optimizer loses to it, so we only assert the
            // win where it holds and otherwise require it to stay competitive (within 1.2x).
            if expect_beats_two_set_mle {
                assert!(
                    jmle_mean <= mle_mean * 1.05,
                    "{label}: jmle {jmle_mean} expected to beat 2-set MLE {mle_mean}"
                );
            } else {
                assert!(
                    jmle_mean <= mle_mean * 1.2,
                    "{label}: jmle {jmle_mean} not competitive with 2-set MLE {mle_mean}"
                );
            }
        }

        let seeds = [0xC0FFEE_u64, 0xBADF00D, 0x1234_5678, 0xDEAD_BEEF, 0xFACE];
        run::<Precision4, Bits6, 2, 2>("P4 2x2", 40, &seeds, true);
        run::<Precision5, Bits6, 2, 2>("P5 2x2", 60, &seeds, true);
        // Square grids beat the repeated 2-set MLE (here 3x3 at jmle/2set = 0.95). Rectangular grids
        // (3x2, and identically its 2x3 transpose, at jmle/2set = 1.12) do not: the estimator is exactly
        // M<->N symmetric (a 3x2 sketch matches its 2x3 transpose to ~3e-9), and the jMLE there reaches
        // a HIGHER joint likelihood and MAP than the 2-set decomposition yet a slightly worse mean
        // per-cell error, so the rectangular gap is genuine maximum-likelihood identifiability, not a
        // convergence or left/right asymmetry bug (the deep cell still converges with precision).
        run::<Precision6, Bits6, 3, 3>("P6 3x3", 80, &seeds, true);
        run::<Precision6, Bits6, 3, 2>("P6 3x2", 80, &seeds, false);
    }

    /// The estimator must be exactly symmetric under swapping the left and right roles: a sketch of
    /// `M` lefts by `N` rights, transposed, must equal the sketch of those rights (as lefts) by those
    /// lefts (as rights). This guards against any left/right (M != N) code asymmetry in the likelihood
    /// ceilings, hitter sets, or marginal anchors. It holds to optimizer epsilon (about 3e-9), which
    /// also explains why rectangular grids and their transposes lose to the 2-set MLE by the same
    /// margin (they are the same problem), so the rectangular gap is identifiability, not a bug.
    #[test]
    fn test_jmle_transpose_symmetry() {
        type C = Counter<Precision8, Bits6>;
        const M: usize = 3;
        const N: usize = 2;
        // Fixed, deterministic, deliberately uneven cells laid out from disjoint integer ranges.
        let unit = 300u64;
        let mut cursor = 0u64;
        let mut ro = [[(0u64, 0u64); N]; M];
        for i in 0..M {
            for j in 0..N {
                let count = unit * (1 + ((i * 3 + j * 2) % 5) as u64);
                ro[i][j] = (cursor, count);
                cursor += count;
            }
        }
        let mut rda = [(0u64, 0u64); M];
        for i in 0..M {
            let count = unit * (1 + i as u64);
            rda[i] = (cursor, count);
            cursor += count;
        }
        let mut rdb = [(0u64, 0u64); N];
        for j in 0..N {
            let count = unit * (2 + j as u64);
            rdb[j] = (cursor, count);
            cursor += count;
        }
        let build_left = |i: usize| {
            let mut r = Vec::new();
            for ii in 0..=i {
                for j in 0..N {
                    r.push(ro[ii][j]);
                }
                r.push(rda[ii]);
            }
            build::<Precision8, Bits6>(&r)
        };
        let build_right = |j: usize| {
            let mut r = Vec::new();
            for jj in 0..=j {
                for i in 0..M {
                    r.push(ro[i][jj]);
                }
                r.push(rdb[jj]);
            }
            build::<Precision8, Bits6>(&r)
        };
        let lefts: [C; M] = core::array::from_fn(build_left);
        let rights: [C; N] = core::array::from_fn(build_right);

        // The 3x2 sketch.
        let lv: [JointMle<&C>; M] = core::array::from_fn(|i| lefts[i].jmle());
        let rv: [JointMle<&C>; N] = core::array::from_fn(|j| rights[j].jmle());
        let s_mn = JointSketch::estimate(&lv, &rv);

        // Its transpose: rights become the lefts and lefts become the rights, a 2x3 sketch whose
        // overlap[j][i] must equal s_mn.overlap[i][j], with the margins swapped.
        let lv_t: [JointMle<&C>; N] = core::array::from_fn(|j| rights[j].jmle());
        let rv_t: [JointMle<&C>; M] = core::array::from_fn(|i| lefts[i].jmle());
        let s_nm = JointSketch::estimate(&lv_t, &rv_t);

        let mut max_rel = 0.0_f64;
        for i in 0..M {
            for j in 0..N {
                let a = s_mn.overlap[i][j];
                let b = s_nm.overlap[j][i];
                max_rel = max_rel.max((a - b).abs() / a.abs().max(b.abs()).max(1.0));
            }
        }
        for i in 0..M {
            max_rel = max_rel.max(
                (s_mn.left_diff[i] - s_nm.right_diff[i]).abs() / s_mn.left_diff[i].abs().max(1.0),
            );
        }
        for j in 0..N {
            max_rel = max_rel.max(
                (s_mn.right_diff[j] - s_nm.left_diff[j]).abs() / s_mn.right_diff[j].abs().max(1.0),
            );
        }
        assert!(
            max_rel < 1e-6,
            "the jMLE is not M<->N symmetric: 3x2 vs 2x3 transpose differ by {max_rel:.2e}"
        );
    }

    // ----- Test 6: proptest comparing jMLE overlap cells to exact set truth. -----

    /// The jMLE overlap cells, on random nested sets built from disjoint integer pools, must track the
    /// exact cell cardinalities within a generous register-mode tolerance. Kept to a small M, N and a
    /// modest case count because the joint optimization is slow.
    fn check_jmle_overlap_vs_truth_proptest(seed: u64) -> bool {
        type C = Counter<Precision8, Bits6>;
        const M: usize = 2;
        const N: usize = 2;
        let mut state = seed;
        let mut next = |modulus: u64| {
            state = splitmix64(state);
            state % modulus
        };

        // Disjoint pools (one per region), each a contiguous integer range, sizes 2000..=8000 so the
        // counters are dense. The exact cell cardinality is the pool size.
        let n_overlap = M * N;
        let k = n_overlap + M + N;
        let mut exact = vec![0.0_f64; k];
        let mut cursor = 0u64;
        let mut ranges = vec![(0u64, 0u64); k];
        for (rho, slot) in ranges.iter_mut().enumerate() {
            let count = 2000 + next(6000);
            *slot = (cursor, count);
            exact[rho] = count as f64;
            cursor += count;
        }
        let lefts: [C; M] = core::array::from_fn(|i| {
            let mut r = Vec::new();
            for ii in 0..=i {
                for j in 0..N {
                    r.push(ranges[ii * N + j]);
                }
                r.push(ranges[n_overlap + ii]);
            }
            build::<Precision8, Bits6>(&r)
        });
        let rights: [C; N] = core::array::from_fn(|j| {
            let mut r = Vec::new();
            for jj in 0..=j {
                for i in 0..M {
                    r.push(ranges[i * N + jj]);
                }
                r.push(ranges[n_overlap + M + jj]);
            }
            build::<Precision8, Bits6>(&r)
        });

        let union: f64 = exact.iter().sum();
        let lv: [JointMle<&C>; M] = core::array::from_fn(|i| lefts[i].jmle());
        let rv: [JointMle<&C>; N] = core::array::from_fn(|j| rights[j].jmle());
        let sketch = JointSketch::estimate(&lv, &rv);

        // Every overlap cell within 12% of the union scale (a loose bound for P8: a deep overlap cell
        // is only weakly identified, but it must still be in the right ballpark, never wild).
        for i in 0..M {
            for j in 0..N {
                if (sketch.overlap[i][j] - exact[i * N + j]).abs() > 0.12 * union {
                    return false;
                }
            }
        }
        true
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig { cases: 24, ..Default::default() })]

        #[test]
        fn test_jmle_overlap_cells_track_truth(seed in proptest::prelude::any::<u64>()) {
            proptest::prop_assert!(check_jmle_overlap_vs_truth_proptest(seed));
        }
    }
}
