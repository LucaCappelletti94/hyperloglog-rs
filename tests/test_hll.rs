use hyperloglog::prelude::*;
use hyperloglogplus::HyperLogLog as AlternativeHyperLogLog;
use hyperloglogplus::HyperLogLogPF;
use std::collections::hash_map::RandomState;

#[test]
pub fn test_hll() {
    const NUMBER_OF_ELEMENTS: usize = 20000;
    const PRECISION: usize = 8;

    let mut hll: HyperLogLog<PRECISION> = HyperLogLog::new();
    let mut alternative: HyperLogLogPF<usize, _> = HyperLogLogPF::new(PRECISION as u8, RandomState::new()).unwrap();

    for i in 0..NUMBER_OF_ELEMENTS {
        hll += i;
        alternative.insert(&i);
    }

    assert!(hll.get_registers().len() == hll.len());

    assert!(
         hll.count() >= NUMBER_OF_ELEMENTS as f32,
         "Expected >= {}, got {} ({}). The registers are: {:?}, of which {} out of {} are zeroed and {} are not. We are using {} bits.",
         NUMBER_OF_ELEMENTS,
         hll.count(),
         alternative.count(),
         hll.get_registers(),
         hll.get_number_of_zero_registers(),
         hll.len(),
         hll.get_number_of_non_zero_registers(),
         hll.get_number_of_bits()
     );
}
