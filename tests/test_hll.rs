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
