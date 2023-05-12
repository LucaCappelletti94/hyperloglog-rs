use hyperloglog_rs::prelude::*;

#[test]
pub fn test_hyper_log_log_at_precision_4_and_bits_5() {
    const PRECISION: usize = 4;
    const BITS: usize = 5;

    for number_of_elements in [5, 10, 15, 100, 200, 1000, 10_000, 100_000, 1_000_000] {
        let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();
        let hll_default: HyperLogLog<PRECISION, BITS> = HyperLogLog::default();

        assert_eq!(hll, hll_default);

        assert_eq!(hll.len(), 1 << PRECISION);
        assert_eq!(hll.get_number_of_bits(), BITS);
        assert!(hll.is_empty());

        for i in 0..number_of_elements {
            hll.insert(i);
            assert!(hll.may_contain(&i));
        }

        assert!(!hll.is_empty());

        assert!(
            hll.estimate_cardinality() >= number_of_elements as f32 * 7.0_f32 / 10.0_f32,
            concat!("Obtained: {}, Expected around: {}. ",),
            hll.estimate_cardinality(),
            number_of_elements,
        );

        assert!(
            hll.estimate_cardinality() <= number_of_elements as f32 * 14.0_f32 / 10.0_f32,
            concat!("Obtained: {}, Expected around: {}. ",),
            hll.estimate_cardinality(),
            number_of_elements,
        );
    }
}

#[test]
pub fn test_hyper_log_log_at_precision_4_and_bits_6() {
    const PRECISION: usize = 4;
    const BITS: usize = 6;

    for number_of_elements in [5, 10, 15, 100, 200, 1000, 10_000, 100_000, 1_000_000] {
        let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();
        let hll_default: HyperLogLog<PRECISION, BITS> = HyperLogLog::default();

        assert_eq!(hll, hll_default);

        assert_eq!(hll.len(), 1 << PRECISION);
        assert_eq!(hll.get_number_of_bits(), BITS);
        assert!(hll.is_empty());

        for i in 0..number_of_elements {
            hll.insert(i);
            assert!(hll.may_contain(&i));
        }

        assert!(!hll.is_empty());

        assert!(
            hll.estimate_cardinality() >= number_of_elements as f32 * 7.0_f32 / 10.0_f32,
            concat!("Obtained: {}, Expected around: {}. ",),
            hll.estimate_cardinality(),
            number_of_elements,
        );

        assert!(
            hll.estimate_cardinality() <= number_of_elements as f32 * 14.0_f32 / 10.0_f32,
            concat!("Obtained: {}, Expected around: {}. ",),
            hll.estimate_cardinality(),
            number_of_elements,
        );
    }
}
