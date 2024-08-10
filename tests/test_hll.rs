use hyperloglog_rs::prelude::*;
mod utils;
use twox_hash::XxHash;

pub fn test_hll_at_precision_and_bits<
    F: FloatNumber,
    P: Precision + PrecisionConstants<F>,
    B: Bits,
    H: HyperLogLogTrait<P, B, Hasher>,
    Hasher: core::hash::Hasher + Default,
>() {
    let number_of_elements = 1_000_000;
    let mut total_error_rate = 0.0;
    let number_of_iterations = 100;

    let mut random_state = splitmix64(534543539_u64);

    for _ in 0..number_of_iterations {
        let mut hll: H = H::default();

        assert!(hll.is_empty());

        let mut exact_set = std::collections::HashSet::new();

        for _ in 0..number_of_elements {
            random_state = splitmix64(xorshift64(random_state));
            hll.insert(&random_state);
            exact_set.insert(random_state);
            assert!(hll.may_contain(&random_state));

            // The result of the harmonic sum method should always be equal, within
            // an epsilon, to the actual harmonic sum of the registers.
            let harmonic_sum = hll.harmonic_sum();
            let actual_harmonic_sum = hll.registers().iter_registers().map(|register| register as i32).map(F::inverse_register).sum::<F>();
            assert!(
                (harmonic_sum - actual_harmonic_sum).abs() < F::EPSILON,
                "The harmonic sum ({}) is different from the actual harmonic sum ({})",
                harmonic_sum,
                actual_harmonic_sum
            );
        }

        let estimated_cardinality = hll.estimate_cardinality();
        let exact_cardinality = exact_set.len() as f64;

        total_error_rate +=
            (estimated_cardinality.to_usize() as f64 - exact_cardinality).abs() / exact_cardinality;

        assert!(!hll.is_empty());
    }

    let mean_error_rate = total_error_rate / number_of_iterations as f64;

    assert!(
        mean_error_rate <= P::error_rate(),
        concat!(
            "The estimated error rate ({}) is higher than the expected error rate ({}) for a precision of {} and bits {}.",
        ),
        mean_error_rate,
        P::error_rate(),
        P::EXPONENT,
        B::NUMBER_OF_BITS
    );
}

/// Macro to generate a range of tests with the provided lists of precisions and bits
macro_rules! test_hll_at_precision_and_bits {
    ($precision:ty, $hasher:ty, $($bits:ty),*) => {
        $(
            paste::item! {
                #[test]
                pub fn [< test_hll_at_ $precision:lower _and_ $bits:lower _bits >]() {
                    test_hll_at_precision_and_bits::<f64, $precision, $bits, HyperLogLog<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>, $hasher>();
                }

                #[test]
                pub fn [< test_mle2_at_ $precision:lower _and_ $bits:lower _bits >]() {
                    test_hll_at_precision_and_bits::<f64, $precision, $bits, MLE<HyperLogLog<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>, $hasher>();
                }
            }
        )*
    };
}

/// Macro to generate a range of tests with the provided lists of precisions
macro_rules! test_hll_at_precision_and_hashers {
    ($precision:ty, $($hasher:ty),*) => {
        $(
            test_hll_at_precision_and_bits!($precision, $hasher, Bits5, Bits6);
        )*
    };
}


/// Macro to generate a range of tests with the provided lists of precisions
macro_rules! test_hll_at_precisions {
    ($($precision:ty),*) => {
        $(
            test_hll_at_precision_and_hashers!($precision, XxHash);
        )*
    };
}

test_hll_at_precisions!(
    Precision4,
    Precision5,
    Precision6,
    Precision7,
    Precision8,
    Precision9,
    Precision10,
    Precision11,
    Precision12,
    Precision13,
    Precision14,
    Precision15,
    Precision16
);
