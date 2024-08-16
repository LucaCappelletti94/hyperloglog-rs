use hyperloglog_rs::prelude::*;
use twox_hash::XxHash;
use wyhash::WyHash;

/// Test the HyperLogLog implementation with the provided precision and bits
pub fn test_approximated_counter_at_precision_and_bits<
    P: Precision,
    B: Bits,
    H: ExtendableApproximatedSet<u64> + Estimator<f64>,
    Hasher: HasherType,
>() {
    let number_of_elements = 200_000;
    let mut total_cardinality_error_rate = 0.0;
    let mut total_union_error_rate = 0.0;
    let mut total_cardinality_samples = 0;
    let mut total_union_samples = 0;
    let number_of_iterations = 500;
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

        for (i, element) in
            iter_random_values(number_of_elements, Some(1_000_000), left_random_state).enumerate()
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
                    cardinality_sampling_rate *= 2;
                }
                let estimated_cardinality = left.estimate_cardinality();
                let exact_cardinality = exact_left.len() as f64;

                total_cardinality_samples += 1;
                total_cardinality_error_rate +=
                    (estimated_cardinality - exact_cardinality).abs() / exact_cardinality;
            }

            if i % union_sampling_rate == 0 {
                if union_sampling_rate < maximal_union_sampling_rate {
                    union_sampling_rate *= 2;
                }
                // We also check at each iteration of the right set that the union of the two sets
                // is correctly estimated.
                let union = exact_left.union(&exact_right).count() as f64;
                let estimated_union = left.estimate_union_cardinality(&right);

                // The union estimate must be symmetric if the approach is not MLE, which is
                // non-deterministic.
                if !left.is_union_estimate_non_deterministic(&right) {
                    assert_eq!(
                        estimated_union,
                        right.estimate_union_cardinality(&left)
                    );
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
            "Cardinality error rate ({}) over {} samples is higher than expected ({}) for a precision of {} and bits {}.",
        ),
        mean_error_rate,
        total_cardinality_samples,
        P::error_rate(),
        P::EXPONENT,
        B::NUMBER_OF_BITS
    );

    let mean_union_error_rate = total_union_error_rate / total_union_samples as f64;

    assert!(
        mean_union_error_rate <= P::error_rate(),
        concat!(
            "Union error rate ({}) (cardinalty was: {}) over {} samples is higher than the expected error rate ({}) for a precision of {} and bits {}.",
        ),
        mean_union_error_rate,
        mean_error_rate,
        total_union_samples,
        P::error_rate(),
        P::EXPONENT,
        B::NUMBER_OF_BITS
    );
}

/// Macro to generate a range of tests with the provided lists of precisions, bits and register types
macro_rules! test_hll_at_precision_and_bits_and_register {
    ($precision:ty, $hasher:ty, $bits:ty, $($register:ty),*) => {
        $(
            paste::item! {
                #[test]
                #[cfg(feature = "plusplus")]
                pub fn [< test_plusplus_at_ $precision:lower _and_ $bits:lower _bits_and_ $hasher:lower _and_ $register:lower>]() {
                    test_approximated_counter_at_precision_and_bits::<$precision, $bits, PlusPlus<$precision, $bits, $register, $hasher>, $hasher>();
                }
                #[test]
                #[cfg(feature = "beta")]
                pub fn [< test_beta_at_ $precision:lower _and_ $bits:lower _bits_and_ $hasher:lower _and_ $register:lower>]() {
                    test_approximated_counter_at_precision_and_bits::<$precision, $bits, LogLogBeta<$precision, $bits, $register, $hasher>, $hasher>();
                }
                #[test]
                #[cfg(feature = "plusplus")]
                pub fn [< test_hybrid_plusplus_at_ $precision:lower _and_ $bits:lower _bits_and_ $hasher:lower _and_ $register:lower>]() {
                    test_approximated_counter_at_precision_and_bits::<$precision, $bits, Hybrid<PlusPlus<$precision, $bits, $register, $hasher>>, $hasher>();
                }
                #[test]
                #[cfg(feature = "beta")]
                pub fn [< test_hybrid_beta_at_ $precision:lower _and_ $bits:lower _bits_and_ $hasher:lower _and_ $register:lower>]() {
                    test_approximated_counter_at_precision_and_bits::<$precision, $bits, Hybrid<LogLogBeta<$precision, $bits, $register, $hasher>>, $hasher>();
                }
                #[test]
                #[cfg(all(feature = "mle", feature = "plusplus"))]
                pub fn [< test_mleplusplus_at_ $precision:lower _and_ $bits:lower _bits_and_ $hasher:lower _and_ $register:lower>]() {
                    test_approximated_counter_at_precision_and_bits::<$precision, $bits, MLE<PlusPlus<$precision, $bits, $register, $hasher>>, $hasher>();
                }
                #[test]
                #[cfg(all(feature = "mle", feature = "beta"))]
                pub fn [< test_mlebeta_at_ $precision:lower _and_ $bits:lower _bits_and_ $hasher:lower _and_ $register:lower>]() {
                    test_approximated_counter_at_precision_and_bits::<$precision, $bits, MLE<LogLogBeta<$precision, $bits, $register, $hasher>>, $hasher>();
                }
                #[test]
                #[cfg(all(feature = "mle", feature = "plusplus"))]
                pub fn [< test_hybrid_mleplusplus_at_ $precision:lower _and_ $bits:lower _bits_and_ $hasher:lower _and_ $register:lower>]() {
                    test_approximated_counter_at_precision_and_bits::<$precision, $bits, Hybrid<MLE<PlusPlus<$precision, $bits, $register, $hasher>>>, $hasher>();
                }
                #[test]
                #[cfg(all(feature = "mle", feature = "beta"))]
                pub fn [< test_hybrid_mlebeta_at_ $precision:lower _and_ $bits:lower _bits_and_ $hasher:lower _and_ $register:lower>]() {
                    test_approximated_counter_at_precision_and_bits::<$precision, $bits, Hybrid<MLE<LogLogBeta<$precision, $bits, $register, $hasher>>>, $hasher>();
                }
            }
        )*
    }
}

/// Macro to generate a range of tests with the provided lists of precisions and bits
macro_rules! test_hll_at_precision_and_bits {
    ($precision:ty, $hasher:ty, $($bits:ty),*) => {
        $(
            paste::paste!{
                type [<Array $precision $hasher $bits>] = <$precision as ArrayRegister<$bits>>::ArrayRegister;
                type [<PackedArray $precision $hasher $bits>] = <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister;
                test_hll_at_precision_and_bits_and_register!($precision, $hasher, $bits, [<Array $precision $hasher $bits>], [<PackedArray $precision $hasher $bits>]);
            }
        )*
    };
}

/// Macro to generate a range of tests with the provided lists of precisions
macro_rules! test_hll_at_precision_and_hashers {
    ($precision:ty, $($hasher:ty),*) => {
        $(
            test_hll_at_precision_and_bits!($precision, $hasher, Bits6);
        )*
    };
}

/// Macro to generate a range of tests with the provided lists of precisions
macro_rules! test_hll_at_precisions {
    ($($precision:ty),*) => {
        $(
            test_hll_at_precision_and_hashers!($precision, XxHash);
            test_hll_at_precision_and_hashers!($precision, WyHash);
        )*
    };
}

#[cfg(feature = "precision_4")]
test_hll_at_precisions!(Precision4);
#[cfg(feature = "precision_5")]
test_hll_at_precisions!(Precision5);
#[cfg(feature = "precision_6")]
test_hll_at_precisions!(Precision6);
#[cfg(feature = "precision_7")]
test_hll_at_precisions!(Precision7);
#[cfg(feature = "precision_8")]
test_hll_at_precisions!(Precision8);
#[cfg(feature = "precision_9")]
test_hll_at_precisions!(Precision9);
#[cfg(feature = "precision_10")]
test_hll_at_precisions!(Precision10);
#[cfg(feature = "precision_11")]
test_hll_at_precisions!(Precision11);
#[cfg(feature = "precision_12")]
test_hll_at_precisions!(Precision12);
#[cfg(feature = "precision_13")]
test_hll_at_precisions!(Precision13);
#[cfg(feature = "precision_14")]
test_hll_at_precisions!(Precision14);
#[cfg(feature = "precision_15")]
test_hll_at_precisions!(Precision15);
#[cfg(feature = "precision_16")]
test_hll_at_precisions!(Precision16);
#[cfg(feature = "precision_17")]
test_hll_at_precisions!(Precision17);
#[cfg(feature = "precision_18")]
test_hll_at_precisions!(Precision18);
