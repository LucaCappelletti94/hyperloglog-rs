
use hyperloglog_rs::prelude::*;

#[test]
pub fn test_hyper_log_log_at_precision_12() {
    const PRECISION: usize = 12;
    
    for elements in [5, 10, 15, 100, 200, 1000, 10_000, 100_000, 1_000_000]{
        let mut hll: HyperLogLog<PRECISION> = HyperLogLog::new();

        for i in 0..elements {
            hll.insert(i);
        }

        assert!(
            hll.count_dispatched() >= elements as f32 * 7.0_f32 / 10.0_f32,
            "Obtained: {}, Expected around: {}",
            hll.count_dispatched(), elements
        );

        assert!(
            hll.count() >= elements as f32 * 7.0_f32 / 10.0_f32,
            "Obtained: {}, Expected around: {}",
            hll.count(), elements
        );

        assert!(
            hll.count_dispatched() <= elements as f32 * 14.0_f32 / 10.0_f32,
            "Obtained: {}, Expected around: {}",
            hll.count_dispatched(), elements
        );

        assert!(
            hll.count() <= elements as f32 * 14.0_f32 / 10.0_f32,
            "Obtained: {}, Expected around: {}",
            hll.count(), elements
        );
    }
}
