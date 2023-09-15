
use hyperloglog_rs::prelude::*;
use siphasher::sip::SipHasher13;

#[test]
pub fn test_hyper_log_log_at_precision_16_and_bits_4() {
    type PRECISION = Precision16;
    const BITS: usize = 4;
    
    for number_of_elements in [
        5, 10, 15, 100, 200, 1000, 10_000, 100_000, 1_000_000
    ]{
        if BITS <= 4 && 16 <= 5 && number_of_elements > 10_000{
            continue;
        }
    
        let mut hll: HyperLogLog<PRECISION, BITS, SipHasher13> = HyperLogLog::new();
        let hll_default: HyperLogLog<PRECISION, BITS, SipHasher13> = HyperLogLog::default();
        
        assert_eq!(hll, hll_default);
        
        
        assert!(hll.is_empty());

        for i in 0..number_of_elements {
            hll.insert(i);
            assert!(hll.may_contain(&i));
        }
        
        assert!(!hll.is_empty());

        assert!(
            hll.estimate_cardinality() >= number_of_elements as f32 * 7.0_f32 / 10.0_f32,
            concat!(
                "Obtained: {}, Expected around: {}. ",
            ),
            hll.estimate_cardinality(), number_of_elements,
        );

        assert!(
            hll.estimate_cardinality() <= number_of_elements as f32 * 14.0_f32 / 10.0_f32,
            concat!(
                "Obtained: {}, Expected around: {}. ",
            ),
            hll.estimate_cardinality(), number_of_elements,
        );
    }
}



#[test]
pub fn test_hyper_log_log_at_precision_16_and_bits_5() {
    type PRECISION = Precision16;
    const BITS: usize = 5;
    
    for number_of_elements in [
        5, 10, 15, 100, 200, 1000, 10_000, 100_000, 1_000_000
    ]{
        if BITS <= 4 && 16 <= 5 && number_of_elements > 10_000{
            continue;
        }
    
        let mut hll: HyperLogLog<PRECISION, BITS, SipHasher13> = HyperLogLog::new();
        let hll_default: HyperLogLog<PRECISION, BITS, SipHasher13> = HyperLogLog::default();
        
        assert_eq!(hll, hll_default);
        
        
        assert!(hll.is_empty());

        for i in 0..number_of_elements {
            hll.insert(i);
            assert!(hll.may_contain(&i));
        }
        
        assert!(!hll.is_empty());

        assert!(
            hll.estimate_cardinality() >= number_of_elements as f32 * 7.0_f32 / 10.0_f32,
            concat!(
                "Obtained: {}, Expected around: {}. ",
            ),
            hll.estimate_cardinality(), number_of_elements,
        );

        assert!(
            hll.estimate_cardinality() <= number_of_elements as f32 * 14.0_f32 / 10.0_f32,
            concat!(
                "Obtained: {}, Expected around: {}. ",
            ),
            hll.estimate_cardinality(), number_of_elements,
        );
    }
}



#[test]
pub fn test_hyper_log_log_at_precision_16_and_bits_6() {
    type PRECISION = Precision16;
    const BITS: usize = 6;
    
    for number_of_elements in [
        5, 10, 15, 100, 200, 1000, 10_000, 100_000, 1_000_000
    ]{
        if BITS <= 4 && 16 <= 5 && number_of_elements > 10_000{
            continue;
        }
    
        let mut hll: HyperLogLog<PRECISION, BITS, SipHasher13> = HyperLogLog::new();
        let hll_default: HyperLogLog<PRECISION, BITS, SipHasher13> = HyperLogLog::default();
        
        assert_eq!(hll, hll_default);
        
        
        assert!(hll.is_empty());

        for i in 0..number_of_elements {
            hll.insert(i);
            assert!(hll.may_contain(&i));
        }
        
        assert!(!hll.is_empty());

        assert!(
            hll.estimate_cardinality() >= number_of_elements as f32 * 7.0_f32 / 10.0_f32,
            concat!(
                "Obtained: {}, Expected around: {}. ",
            ),
            hll.estimate_cardinality(), number_of_elements,
        );

        assert!(
            hll.estimate_cardinality() <= number_of_elements as f32 * 14.0_f32 / 10.0_f32,
            concat!(
                "Obtained: {}, Expected around: {}. ",
            ),
            hll.estimate_cardinality(), number_of_elements,
        );
    }
}
