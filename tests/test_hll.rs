use hyperloglog_derive::test_estimator;
use hyperloglog_rs::prelude::*;
use twox_hash::XxHash;

/// Test the HyperLogLog implementation with the provided precision and bits
pub fn test_approximated_counter_at_precision_and_bits<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
>() {
    let number_of_elements = 200_000;
    let mut total_cardinality_error_rate = 0.0;
    let mut total_union_error_rate = 0.0;
    let mut total_cardinality_samples = 0;
    let mut total_union_samples = 0;
    let number_of_iterations = 50;
    let starting_cardinality_sampling_rate = 10;
    let starting_union_sampling_rate = 10;
    let maximal_cardinality_sampling_rate = 5_000;
    let maximal_union_sampling_rate = 5_000;

    let mut left_random_state = splitmix64(splitmix64(99534543539_u64));
    let mut right_random_state = splitmix64(splitmix64(22986224539_u64));
    let mut exact_left = std::collections::HashSet::new();
    let mut exact_right = std::collections::HashSet::new();

    for _ in 0..number_of_iterations {
        let mut left: HyperLogLog<P, B, R, H> = Default::default();
        let mut right: HyperLogLog<P, B, R, H> = Default::default();
        exact_left.clear();
        exact_right.clear();
        left_random_state = splitmix64(left_random_state);
        right_random_state = splitmix64(right_random_state);

        let mut cardinality_sampling_rate = starting_cardinality_sampling_rate;
        let mut union_sampling_rate = starting_union_sampling_rate;

        for (i, element) in iter_var_len_random_values::<u64>(
            0,
            number_of_elements,
            Some(1_000_000),
            Some(left_random_state),
        )
        .enumerate()
        {
            if i % 2 == 0 {
                left.insert(&element);
                exact_left.insert(element);
            } else {
                right.insert(&element);
                exact_right.insert(element);
            }

            if i % cardinality_sampling_rate == 0 {
                if cardinality_sampling_rate < maximal_cardinality_sampling_rate {
                    left_random_state = splitmix64(left_random_state);
                    cardinality_sampling_rate +=
                        left_random_state as usize % cardinality_sampling_rate;
                }
                let estimated_cardinality = left.estimate_cardinality();
                let exact_cardinality = exact_left.len() as f64;

                total_cardinality_samples += 1;
                total_cardinality_error_rate +=
                    (estimated_cardinality - exact_cardinality).abs() / exact_cardinality;
            }

            if i % union_sampling_rate == 0 {
                if union_sampling_rate < maximal_union_sampling_rate {
                    right_random_state = splitmix64(right_random_state);
                    union_sampling_rate += right_random_state as usize % union_sampling_rate;
                }
                // We also check at each iteration of the right set that the union of the two sets
                // is correctly estimated.
                let union = exact_left.union(&exact_right).count() as f64;
                let estimated_union = left.estimate_union_cardinality(&right);

                total_union_error_rate += (estimated_union as f64 - union).abs() / union;

                total_union_samples += 1;
            }
        }
    }

    let mean_error_rate = total_cardinality_error_rate / total_cardinality_samples as f64;

    assert!(
        mean_error_rate <= P::error_rate(),
        concat!(
            "Cardinality error rate ({}) over {} samples is higher than expected ({}) for a precision of {}.",
        ),
        mean_error_rate,
        total_cardinality_samples,
        P::error_rate(),
        P::EXPONENT,
    );

    let mean_union_error_rate = total_union_error_rate / total_union_samples as f64;

    assert!(
        mean_union_error_rate <= P::error_rate(),
        concat!(
            "Union error rate ({}) (cardinalty was: {}) over {} samples is higher than the expected error rate ({}) for a precision of {}.",
        ),
        mean_union_error_rate,
        mean_error_rate,
        total_union_samples,
        P::error_rate(),
        P::EXPONENT,
    );
}

// #[test_estimator]
// #[cfg(feature = "mle")]
// fn test_hybrid_mle_plusplus<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>() {
//     test_approximated_counter_at_precision_and_bits::<P, Hybrid<MLE<PlusPlus<P, B, R, H>>>>();
// }

#[test_estimator]
fn test_hyperloglog<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>() {
    test_approximated_counter_at_precision_and_bits::<P, B, R, H>();
}

/// Tests the `BitOr` merger: building a merged counter via `&a | &b` must yield a
/// counter whose cardinality estimate matches the exact union cardinality, across
/// both the hash-list mode (small cardinalities) and the HyperLogLog mode (large
/// cardinalities). This is stronger than `estimate_union_cardinality`, which never
/// materializes a merged counter.
pub fn test_union_merge_at_precision_and_bits<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
>() {
    let mut left: HyperLogLog<P, B, R, H> = Default::default();
    let mut right: HyperLogLog<P, B, R, H> = Default::default();
    let mut exact_left = std::collections::HashSet::new();
    let mut exact_right = std::collections::HashSet::new();

    let mut left_random_state = splitmix64(splitmix64(0xC0FF_EE00_u64));
    let mut right_random_state = splitmix64(splitmix64(0xDEAD_BEEF_u64));

    let number_of_elements = 100_000;
    let mut sampling_rate = 1;
    let mut total_error_rate = 0.0;
    let mut total_reference_error_rate = 0.0;
    let mut samples = 0;

    for i in 0..number_of_elements {
        left_random_state = splitmix64(left_random_state);
        right_random_state = splitmix64(right_random_state);
        left.insert(&left_random_state);
        exact_left.insert(left_random_state);
        right.insert(&right_random_state);
        exact_right.insert(right_random_state);

        if i % sampling_rate == 0 {
            // Ramp up the sampling rate so we densely cover small cardinalities (where
            // the hash-list mode matters) and sparsely cover the large ones.
            sampling_rate += 1 + i / 16;

            let merged = &left | &right;

            // Every inserted element must be reported as possibly contained.
            assert!(
                merged.may_contain(&left_random_state),
                "Merged counter must contain the last left element."
            );
            assert!(
                merged.may_contain(&right_random_state),
                "Merged counter must contain the last right element."
            );

            // The union operator must be (approximately) commutative. It is not bit-exact
            // because the hash-list estimate carries each operand's own duplicate tally, and
            // the merge inherits the base operand's, but the cardinality estimates must agree
            // to within the precision's error rate.
            let merged_estimate = merged.estimate_cardinality();
            let merged_swapped = &right | &left;
            let merged_swapped_estimate = merged_swapped.estimate_cardinality();
            assert!(
                (merged_estimate - merged_swapped_estimate).abs()
                    <= merged_estimate.max(merged_swapped_estimate) * P::error_rate() + 1.0,
                "The union operator must be approximately commutative ({merged_estimate} vs {merged_swapped_estimate})."
            );

            // Merging a counter with itself must be idempotent.
            let self_union = &left | &left;
            let left_estimate = left.estimate_cardinality();
            assert!(
                (self_union.estimate_cardinality() - left_estimate).abs()
                    <= left_estimate * P::error_rate() + 1.0,
                "Merging a counter with itself must be idempotent (got {} vs {left_estimate}).",
                self_union.estimate_cardinality(),
            );

            let exact_union = exact_left.union(&exact_right).count() as f64;
            let estimated_union = merged.estimate_cardinality();
            total_error_rate += (estimated_union - exact_union).abs() / exact_union;

            // Reference: the established `estimate_union_cardinality` path, which never
            // materializes a merged counter. The merger must be no worse than this.
            let reference_union = left.estimate_union_cardinality(&right);
            total_reference_error_rate += (reference_union - exact_union).abs() / exact_union;

            samples += 1;
        }
    }

    let mean_error_rate = total_error_rate / samples as f64;
    let mean_reference_error_rate = total_reference_error_rate / samples as f64;

    // The merged counter must estimate the union with an error comparable to the precision's
    // nominal error rate and to the reference `estimate_union_cardinality` path. Materializing
    // a counter (rather than estimating directly via inclusion-exclusion) carries the base
    // operand's hash-list duplicate tally, which inflates the variance at the smallest
    // precisions; the 1.5 factor absorbs that. The bound tracks the precision (it shrinks with
    // it), so it still catches regressions at higher precisions.
    let tolerance = P::error_rate().max(mean_reference_error_rate) * 1.5;

    assert!(
        mean_error_rate <= tolerance,
        "Union-merge error rate ({mean_error_rate}) over {samples} samples exceeds the tolerance ({tolerance}; reference {mean_reference_error_rate}, nominal {}) for precision {}.",
        P::error_rate(),
        P::EXPONENT,
    );
}

#[test_estimator]
fn test_union_merge<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>() {
    test_union_merge_at_precision_and_bits::<P, B, R, H>();
}

/// Deterministic regression test pinning the core requirement: a union of two small
/// sets (both still in hash-list mode) must produce an accurate cardinality estimate.
/// A naive register-wise-max union forces the result out of hash-list mode and wildly
/// over-estimates here.
#[test]
fn test_union_small_cardinality_stays_accurate() {
    type Counter =
        HyperLogLog<Precision8, Bits6, <Precision8 as PackedRegister<Bits6>>::Array, XxHash>;

    let mut a: Counter = Default::default();
    let mut b: Counter = Default::default();

    for element in 0..5_u64 {
        a.insert(&element);
    }
    for element in 3..8_u64 {
        b.insert(&element);
    }

    assert!(
        a.is_hash_list(),
        "Small counter `a` must be in hash-list mode."
    );
    assert!(
        b.is_hash_list(),
        "Small counter `b` must be in hash-list mode."
    );

    let union = &a | &b;

    let estimate = union.estimate_cardinality();
    assert!(
        (estimate - 8.0).abs() <= 1.0,
        "Union of {{0..5}} and {{3..8}} should estimate ~8, got {estimate}.",
    );

    for element in 0..8_u64 {
        assert!(
            union.may_contain(&element),
            "Merged counter must contain element {element}.",
        );
    }
}

/// The Maximum Likelihood Estimation of the union cardinality (Ertl's joint estimator) must
/// estimate the union of two fully-fledged HyperLogLog counters within the precision's error
/// rate. Two partially overlapping sets are used: [0, 50_000) and [25_000, 75_000), whose true
/// union is 75_000 distinct elements.
#[cfg(feature = "mle")]
#[test]
fn test_mle_union_matches_exact() {
    type Counter =
        HyperLogLog<Precision10, Bits6, <Precision10 as PackedRegister<Bits6>>::Array, XxHash>;

    let mut left: Counter = Default::default();
    let mut right: Counter = Default::default();

    for element in 0..50_000_u64 {
        left.insert(&element);
    }
    for element in 25_000..75_000_u64 {
        right.insert(&element);
    }

    assert!(!left.is_hash_list() && !right.is_hash_list());

    let exact_union = 75_000.0_f64;
    let mle_union = left.estimate_union_cardinality_mle(&right);

    let error = (mle_union - exact_union).abs() / exact_union;
    assert!(
        error <= Precision10::error_rate(),
        "MLE union estimate {mle_union} differs from exact {exact_union} by {error}, exceeding the error rate {}.",
        Precision10::error_rate(),
    );
}

/// The single-counter Maximum Likelihood Estimation of the cardinality (Ertl's secant-method
/// estimator) must estimate a fully-fledged HyperLogLog counter within the precision's error
/// rate. (It is known to be less accurate and much slower than the default HyperLogLog++
/// corrected estimate, and its advantage over the uncorrected estimate is an average-over-
/// cardinalities property rather than a per-point one; the benchmark quantifies both.)
#[cfg(feature = "mle")]
#[test]
fn test_mle_cardinality_reasonable() {
    type Counter =
        HyperLogLog<Precision12, Bits6, <Precision12 as PackedRegister<Bits6>>::Array, XxHash>;

    let mut hll: Counter = Default::default();
    for element in 0..100_000_u64 {
        hll.insert(&element);
    }
    assert!(!hll.is_hash_list());

    let exact = 100_000.0_f64;
    let mle = hll.estimate_cardinality_mle();
    let mle_error = (mle - exact).abs() / exact;

    assert!(
        mle_error <= Precision12::error_rate(),
        "MLE cardinality estimate {mle} differs from exact {exact} by {mle_error}, exceeding the error rate {}.",
        Precision12::error_rate(),
    );
}

/// `HyperLogLog` must implement `HyperSpheresSketch`, so the overlap and difference cardinality
/// matrices can be computed. For the simplest 1x1 case the overlap is the intersection
/// cardinality and the difference vectors are the set differences. With A = [0, 10_000) and
/// B = [5_000, 15_000): overlap ~ 5_000, left difference ~ 5_000, right difference ~ 5_000.
#[test]
fn test_hyper_spheres_sketch_overlap_and_differences() {
    type Counter =
        HyperLogLog<Precision12, Bits6, <Precision12 as PackedRegister<Bits6>>::Array, XxHash>;

    let mut a: Counter = Default::default();
    let mut b: Counter = Default::default();
    for element in 0..10_000_u64 {
        a.insert(&element);
    }
    for element in 5_000..15_000_u64 {
        b.insert(&element);
    }

    let (overlaps, left_differences, right_differences) =
        <Counter as HyperSpheresSketch<f64>>::overlap_and_differences_cardinality_matrices(
            &[a],
            &[b],
        );

    let close = |got: f64, want: f64| (got - want).abs() <= want * 0.15;
    assert!(
        close(overlaps[0][0], 5_000.0),
        "overlap (intersection) {} should be ~5000",
        overlaps[0][0]
    );
    assert!(
        close(left_differences[0], 5_000.0),
        "left difference {} should be ~5000",
        left_differences[0]
    );
    assert!(
        close(right_differences[0], 5_000.0),
        "right difference {} should be ~5000",
        right_differences[0]
    );
}

/// Inserts every integer in the half-open range `[start, start + count)` into `hll`.
#[cfg(feature = "mle")]
fn insert_range<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>(
    hll: &mut HyperLogLog<P, B, R, H>,
    start: u64,
    count: u64,
) {
    for element in start..start + count {
        hll.insert(&element);
    }
}

/// At `M = N = 1` the generalized joint sketch MLE must reduce to the 2-set joint union MLE:
/// the sum of its three disjoint regions (overlap, left difference, right difference) is the
/// union, and must match both the exact union and `estimate_union_cardinality_mle` within the
/// precision's error rate. A = [0, 50_000), B = [25_000, 75_000), true union 75_000.
#[cfg(feature = "mle")]
#[test]
fn test_joint_sketch_mle_reduces_to_union_mle() {
    type Counter =
        HyperLogLog<Precision10, Bits6, <Precision10 as PackedRegister<Bits6>>::Array, XxHash>;

    let mut left: Counter = Default::default();
    let mut right: Counter = Default::default();
    insert_range(&mut left, 0, 50_000);
    insert_range(&mut right, 25_000, 50_000);
    assert!(!left.is_hash_list() && !right.is_hash_list());

    let (overlap, left_diff, right_diff) =
        Counter::joint_sketch_mle(&[left.clone()], &[right.clone()]);

    let joint_union = overlap[0][0] + left_diff[0] + right_diff[0];
    let exact_union = 75_000.0_f64;
    let union_mle = left.estimate_union_cardinality_mle(&right);

    let error_rate = Precision10::error_rate();
    let joint_err = (joint_union - exact_union).abs() / exact_union;
    assert!(
        joint_err <= error_rate,
        "joint union {joint_union} differs from exact {exact_union} by {joint_err}, exceeding {error_rate}."
    );

    // The two estimators fit the same 3-region likelihood, so their union estimates must agree
    // closely.
    let agreement = (joint_union - union_mle).abs() / union_mle;
    assert!(
        agreement <= 0.05,
        "joint union {joint_union} disagrees with 2-set union MLE {union_mle} by {agreement}."
    );
}

/// For small `M, N` (here `M = N = 2`) built from disjoint integer ranges with known cell
/// cardinalities, every estimated disjoint cell (the `M*N` overlaps and the `M + N` margins)
/// must match its exact cardinality within the precision's error rate, measured relative to the
/// total union. The exact partition is constructed by assigning each of the 8 disjoint regions
/// its own range, then composing the nested counters A_0 subset A_1 and B_0 subset B_1.
#[cfg(feature = "mle")]
#[test]
fn test_joint_sketch_mle_matches_exact_cells() {
    type Counter =
        HyperLogLog<Precision12, Bits6, <Precision12 as PackedRegister<Bits6>>::Array, XxHash>;

    // Exact disjoint-region cardinalities.
    // Overlap grid O_ij = L_i intersect R_j.
    let o = [[40_000_u64, 25_000], [18_000, 30_000]];
    // Left margins D^A_i and right margins D^B_j.
    let da = [22_000_u64, 15_000];
    let db = [20_000_u64, 28_000];

    // Lay every region out on its own contiguous integer range.
    let mut cursor = 0_u64;
    let mut alloc = |count: u64| {
        let start = cursor;
        cursor += count;
        (start, count)
    };
    let ro = [
        [alloc(o[0][0]), alloc(o[0][1])],
        [alloc(o[1][0]), alloc(o[1][1])],
    ];
    let rda = [alloc(da[0]), alloc(da[1])];
    let rdb = [alloc(db[0]), alloc(db[1])];

    // Left shell L_i = union over j of O_ij, plus the left margin D^A_i.
    // A_0 = L_0, A_1 = A_0 union L_1.
    let mut a0: Counter = Default::default();
    insert_range(&mut a0, ro[0][0].0, ro[0][0].1);
    insert_range(&mut a0, ro[0][1].0, ro[0][1].1);
    insert_range(&mut a0, rda[0].0, rda[0].1);
    let mut a1 = a0.clone();
    insert_range(&mut a1, ro[1][0].0, ro[1][0].1);
    insert_range(&mut a1, ro[1][1].0, ro[1][1].1);
    insert_range(&mut a1, rda[1].0, rda[1].1);

    // Right shell R_j = union over i of O_ij, plus the right margin D^B_j.
    // B_0 = R_0, B_1 = B_0 union R_1.
    let mut b0: Counter = Default::default();
    insert_range(&mut b0, ro[0][0].0, ro[0][0].1);
    insert_range(&mut b0, ro[1][0].0, ro[1][0].1);
    insert_range(&mut b0, rdb[0].0, rdb[0].1);
    let mut b1 = b0.clone();
    insert_range(&mut b1, ro[0][1].0, ro[0][1].1);
    insert_range(&mut b1, ro[1][1].0, ro[1][1].1);
    insert_range(&mut b1, rdb[1].0, rdb[1].1);

    let total_union: f64 =
        (o[0][0] + o[0][1] + o[1][0] + o[1][1] + da[0] + da[1] + db[0] + db[1]) as f64;

    let (overlap, left_diff, right_diff) = Counter::joint_sketch_mle(&[a0, a1], &[b0, b1]);

    let error_rate = Precision12::error_rate();
    for i in 0..2 {
        for j in 0..2 {
            let err = (overlap[i][j] - o[i][j] as f64).abs() / total_union;
            assert!(
                err <= error_rate,
                "overlap[{i}][{j}] = {} differs from exact {} by {err} of the union, exceeding {error_rate}.",
                overlap[i][j],
                o[i][j],
            );
        }
    }
    for i in 0..2 {
        let err = (left_diff[i] - da[i] as f64).abs() / total_union;
        assert!(
            err <= error_rate,
            "left_diff[{i}] = {} differs from exact {} by {err} of the union, exceeding {error_rate}.",
            left_diff[i],
            da[i],
        );
    }
    for j in 0..2 {
        let err = (right_diff[j] - db[j] as f64).abs() / total_union;
        assert!(
            err <= error_rate,
            "right_diff[{j}] = {} differs from exact {} by {err} of the union, exceeding {error_rate}.",
            right_diff[j],
            db[j],
        );
    }
}
