//! Fuzzing harness to test whether the cardinality estimation works as expected.
#![no_main]
#![feature(generic_const_exprs)]

use arbitrary::Arbitrary;
use hyperloglog_rs::prelude::*;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct FuzzCase {
    value: Vec<u32>,
}

const BITS: usize = 6;
const PRECISION: usize = 13;

fuzz_target!(|data: FuzzCase| {
    let mut unique = data.value.clone();
    unique.sort_unstable();
    unique.dedup();

    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    for item in data.value.iter() {
        hll.insert(item);
    }

    let error: f32 = 0.5;

    // First, we check whether the cardinalities of the HLL counters make sense:

    let cardinality = hll.estimate_cardinality();

    // We make sure that the number of zero registers is actually correct

    assert_eq!(
        hll.get_number_of_zero_registers(),
        hll.get_registers()
            .iter()
            .filter(|register| **register == 0)
            .count()
    );

    assert!(
        cardinality >= unique.len() as f32 * 0.9 - error,
        concat!(
            "Estimated cardinality was too small: {} vs {}. ",
            "The counter has {} zero registers."
        ),
        cardinality,
        unique.len(),
        hll.get_number_of_zero_registers()
    );

    assert!(
        cardinality <= unique.len() as f32 * 1.1 + error,
        "Estimated cardinality was too large: {} vs {}",
        cardinality,
        unique.len()
    );
});