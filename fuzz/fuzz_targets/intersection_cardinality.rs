//! Fuzzing harness to test whether the intersection estimation works as expected.
#![no_main]
#![feature(generic_const_exprs)]

use arbitrary::Arbitrary;
use hyperloglog_rs::prelude::*;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct FuzzCase {
    left: Vec<u32>,
    right: Vec<u32>,
}

const BITS: usize = 4;
const PRECISION: usize = 10;

fuzz_target!(|data: FuzzCase| {
    let mut left_unique = data.left.clone();
    left_unique.sort_unstable();
    left_unique.dedup();

    let mut right_unique = data.right.clone();
    right_unique.sort_unstable();
    right_unique.dedup();

    let mut left: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();
    let mut right: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();
    for item in data.left.iter() {
        left.insert(item);
    }
    for item in data.right.iter() {
        right.insert(item);
    }
    let error: f32 = 1.0;

    // First, we check whether the cardinalities of the HLL counters make sense:

    let left_cardinality = left.estimate_cardinality();
    let right_cardinality = right.estimate_cardinality();

    assert!(
        left_cardinality >= left_unique.len() as f32 * 0.5 - error,
        "Estimated left cardinality was too small: {} vs {}",
        left_cardinality,
        left_unique.len()
    );

    assert!(
        left_cardinality <= left_unique.len() as f32 * 2.0 + error,
        "Estimated left cardinality was too large: {} vs {}",
        left_cardinality,
        left_unique.len()
    );

    assert!(
        right_cardinality >= right_unique.len() as f32 * 0.5 - error,
        "Estimated right cardinality was too small: {} vs {}",
        right_cardinality,
        right_unique.len()
    );

    assert!(
        right_cardinality <= right_unique.len() as f32 * 2.0 + error,
        "Estimated right cardinality was too large: {} vs {}",
        right_cardinality,
        right_unique.len()
    );

    // And then we check whether the intersection makes sense.

    let exact_intersection_size = left_unique
        .iter()
        .filter(|item| right_unique.contains(item))
        .count();

    let estimated_intersection = left.estimate_intersection_cardinality(&right);

    assert!(
        estimated_intersection >= exact_intersection_size as f32 * 0.5 - 2.0 * error,
        "Estimated intersection size was too small: {} vs {}",
        estimated_intersection,
        exact_intersection_size
    );

    assert!(
        estimated_intersection <= exact_intersection_size as f32 * 2.0 + 2.0 * error,
        "Estimated intersection size was too large: {} vs {}",
        estimated_intersection,
        exact_intersection_size
    );
});