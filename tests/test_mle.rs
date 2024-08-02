#[cfg(feature = "std")]
use hyperloglog_rs::prelude::*;

#[cfg(feature = "std")]
pub fn test_mle<P: Precision, const BITS: usize>()
where
    P: WordType<BITS>,
{
    for number_of_elements in [5, 10, 15, 100, 200, 1000, 10_000, 100_000, 1_000_000] {
        if (BITS <= 4 || P::EXPONENT <= 6) && number_of_elements > 10_000 {
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

        let mle: MLE<2, _> = hll.clone().into();

        assert!(
            mle.estimate_cardinality() >= number_of_elements as f32 * 0.8,
            concat!("Obtained: {}, Expected around: {}. ",),
            mle.estimate_cardinality(),
            number_of_elements,
        );

        assert!(
            mle.estimate_cardinality() <= number_of_elements as f32 * 1.3,
            concat!("Obtained: {}, Expected around: {}. ",),
            mle.estimate_cardinality(),
            number_of_elements,
        );

        let mle: MLE<3, _> = hll.into();

        assert!(
            mle.estimate_cardinality() >= number_of_elements as f32 * 0.8,
            concat!("Obtained: {}, Expected around: {}. ",),
            mle.estimate_cardinality(),
            number_of_elements,
        );

        assert!(
            mle.estimate_cardinality() <= number_of_elements as f32 * 1.3,
            concat!("Obtained: {}, Expected around: {}. ",),
            mle.estimate_cardinality(),
            number_of_elements,
        );

        let mle: MLE<4, _> = hll.into();

        assert!(
            mle.estimate_cardinality() >= number_of_elements as f32 * 0.8,
            concat!("Obtained: {}, Expected around: {}. ",),
            mle.estimate_cardinality(),
            number_of_elements,
        );

        assert!(
            mle.estimate_cardinality() <= number_of_elements as f32 * 1.3,
            concat!("Obtained: {}, Expected around: {}. ",),
            mle.estimate_cardinality(),
            number_of_elements,
        );
    }
}

#[cfg(feature = "std")]
/// Macro to generate a range of tests with the provided lists of precisions and bits
macro_rules! test_mle {
    ($precision:ty, $($bits:expr),*) => {
        $(
            paste::item! {
                #[test]
                pub fn [< test_hyper_log_log_at_ $precision:lower _and_ $bits _bits >]() {
                    test_mle::<$precision, $bits>();
                }
            }
        )*
    };
}

#[cfg(feature = "std")]
/// Macro to generate a range of tests with the provided lists of precisions
macro_rules! test_hyper_log_log_at_precisions {
    ($($precision:ty),*) => {
        $(
            test_mle!($precision, 4, 5, 6);
        )*
    };
}

#[cfg(feature = "std")]
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
