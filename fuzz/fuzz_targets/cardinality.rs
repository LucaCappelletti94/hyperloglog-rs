//! Fuzzing harness to test whether the cardinality estimation works as expected.
#![no_main]

use arbitrary::Arbitrary;
use hyperloglog_rs::prelude::*;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct FuzzCase {
    value: Vec<u32>,
}

const BITS: usize = 6;

fuzz_target!(|data: FuzzCase| {
    let mut unique = data.value.clone();
    unique.sort_unstable();
    unique.dedup();

    let mut hll: HyperLogLog<Precision10, BITS> = HyperLogLog::new();

    for item in data.value.iter() {
        // If the item is causing a collision, we stop the test early
        // as it is a known limitation of the data structure and 
        // it is to be expected.
        if hll.may_contain(item) {
            return;
        }
        hll.insert(item);
    }

    let error: f32 = 4.0;

    // If we are dealing with the small range correction, we just skip it as
    // it is a lookup table and it is not worth fuzzing it.
    if hll.use_small_range_correction() {
        return;
    }

    // First, we check whether the cardinalities of the HLL counters make sense:

    let cardinality = hll.estimate_cardinality();

    assert!(
        (cardinality - unique.len() as f32).abs() < error,
        concat!(
            "Estimated cardinality did not match expectations: {} vs {}. ",
            "The counter has {} zero registers our of {}. ",
        ),
        cardinality,
        unique.len(),
        hll.get_number_of_zero_registers(),
        hll.get_number_of_registers(),
    );
});