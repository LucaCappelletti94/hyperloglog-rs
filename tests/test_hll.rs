use hyperloglog_derive::test_estimator;
use hyperloglog_rs::prelude::*;
use twox_hash::XxHash;

/// Test the HyperLogLog implementation with the provided precision and bits
pub fn test_approximated_counter_at_precision_and_bits<
    P: Precision,
    H: ExtendableApproximatedSet<u64> + Estimator<f64> + Default,
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
        let mut left: H = H::default();
        let mut right: H = H::default();
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

                // The union estimate must be symmetric if the approach is not MLE, which is
                // non-deterministic.
                if !left.is_union_estimate_non_deterministic(&right) {
                    assert_eq!(estimated_union, right.estimate_union_cardinality(&left));
                }

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

#[test_estimator]
fn test_plusplus<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>() {
    test_approximated_counter_at_precision_and_bits::<P, PlusPlus<P, B, R, H>>();
}

#[test_estimator]
fn test_beta<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>() {
    test_approximated_counter_at_precision_and_bits::<P, LogLogBeta<P, B, R, H>>();
}

#[test_estimator]
#[cfg(feature = "mle")]
fn test_mle_plusplus<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>() {
    test_approximated_counter_at_precision_and_bits::<P, MLE<PlusPlus<P, B, R, H>>>();
}

#[test_estimator]
#[cfg(feature = "mle")]
fn test_mle_beta<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>() {
    test_approximated_counter_at_precision_and_bits::<P, MLE<LogLogBeta<P, B, R, H>>>();
}

#[test_estimator]
#[cfg(feature = "mle")]
fn test_hybrid_mle_plusplus<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>() {
    test_approximated_counter_at_precision_and_bits::<P, Hybrid<MLE<PlusPlus<P, B, R, H>>>>();
}

#[test_estimator]
#[cfg(feature = "mle")]
fn test_hybrid_mle_beta<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>() {
    test_approximated_counter_at_precision_and_bits::<P, Hybrid<MLE<LogLogBeta<P, B, R, H>>>>();
}

#[test_estimator]
fn test_hybrid_plusplus<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>() {
    test_approximated_counter_at_precision_and_bits::<P, Hybrid<PlusPlus<P, B, R, H>>>();
}

#[test_estimator]
fn test_hybrid_beta<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>() {
    test_approximated_counter_at_precision_and_bits::<P, Hybrid<LogLogBeta<P, B, R, H>>>();
}
