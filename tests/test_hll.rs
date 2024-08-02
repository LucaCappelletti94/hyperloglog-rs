use hyperloglog_rs::prelude::*;

pub fn test_hyper_log_log_at_precision_and_bits<P: Precision, const BITS: usize>()
where
    P: WordType<BITS>,
{
    for number_of_elements in [5, 10, 15, 100, 200, 1000, 10_000, 100_000, 1_000_000] {
        if BITS == 1 && (number_of_elements >= 100 || P::EXPONENT == 4) {
            continue;
        }
        if BITS == 2 && (number_of_elements > 200 || P::EXPONENT == 4) {
            continue;
        }
        if (BITS <= 4 || P::EXPONENT == 4) && number_of_elements >= 1_000 {
            continue;
        }

        let mut hll: HyperLogLog<P, BITS> = HyperLogLog::default();
        let hll_default: HyperLogLog<P, BITS> = HyperLogLog::default();

        assert_eq!(hll, hll_default);

        assert!(hll.is_empty());

        for i in 0..number_of_elements {
            hll.insert(i);
            assert!(hll.may_contain(&i));
        }

        assert!(!hll.is_empty());

        assert!(
            hll.estimate_cardinality() >= number_of_elements as f32 * 0.7,
            concat!("Obtained: {}, Expected around: {}. ",),
            hll.estimate_cardinality(),
            number_of_elements,
        );

        assert!(
            hll.estimate_cardinality() <= number_of_elements as f32 * 1.3,
            concat!("Obtained: {}, Expected around: {}. ",),
            hll.estimate_cardinality(),
            number_of_elements,
        );

        let mut hll: HyperLogLogWithMultiplicities<P, BITS> = HyperLogLogWithMultiplicities::default();
        let hll_default: HyperLogLogWithMultiplicities<P, BITS> = HyperLogLogWithMultiplicities::default();

        assert_eq!(hll, hll_default);

        assert!(hll.is_empty());

        for i in 0..number_of_elements {
            hll.insert(i);
            assert!(hll.may_contain(&i));
        }

        assert!(!hll.is_empty());

        assert!(
            hll.estimate_cardinality() >= number_of_elements as f32 * 0.7,
            concat!("Obtained: {}, Expected around: {}. ",),
            hll.estimate_cardinality(),
            number_of_elements,
        );

        assert!(
            hll.estimate_cardinality() <= number_of_elements as f32 * 1.3,
            concat!("Obtained: {}, Expected around: {}. ",),
            hll.estimate_cardinality(),
            number_of_elements,
        );
    }
}

/// Macro to generate a range of tests with the provided lists of precisions and bits
macro_rules! test_hyper_log_log_at_precision_and_bits {
    ($precision:ty, $($bits:expr),*) => {
        $(
            paste::item! {
                #[test]
                pub fn [< test_hyper_log_log_at_ $precision:lower _and_ $bits _bits >]() {
                    test_hyper_log_log_at_precision_and_bits::<$precision, $bits>();
                }
            }
        )*
    };
}

/// Macro to generate a range of tests with the provided lists of precisions
macro_rules! test_hyper_log_log_at_precisions {
    ($($precision:ty),*) => {
        $(
            test_hyper_log_log_at_precision_and_bits!($precision, 1, 2, 3, 4, 5, 6);
        )*
    };
}

test_hyper_log_log_at_precisions!(
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
