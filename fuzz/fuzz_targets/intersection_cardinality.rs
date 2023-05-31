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

const BITS: usize = 6;

fuzz_target!(|data: FuzzCase| {
    let mut left_unique = data.left.clone();
    left_unique.sort_unstable();
    left_unique.dedup();

    let mut right_unique = data.right.clone();
    right_unique.sort_unstable();
    right_unique.dedup();

    let mut left: HyperLogLog<Precision10, BITS> = HyperLogLog::new();
    let mut right: HyperLogLog<Precision10, BITS> = HyperLogLog::new();
    for item in data.left.iter() {
        // If the item is causing a collision, we stop the test early
        // as it is a known limitation of the data structure and 
        // it is to be expected.
        if left.may_contain(item) {
            return;
        }
        left.insert(item);
    }
    for item in data.right.iter() {
        // If the item is causing a collision, we stop the test early
        // as it is a known limitation of the data structure and 
        // it is to be expected.
        if right.may_contain(item) {
            return;
        }
        right.insert(item);
    }
    // We also check whether there are collisions on the data
    // that is NOT shared between the two sets.
    for item in data.right.iter().filter(|item| !data.left.contains(item)) {
        // If the item is causing a collision, we stop the test early
        // as it is a known limitation of the data structure and 
        // it is to be expected.
        if left.may_contain(item) {
            return;
        }
    }
    for item in data.left.iter().filter(|item| !data.right.contains(item)) {
        // If the item is causing a collision, we stop the test early
        // as it is a known limitation of the data structure and 
        // it is to be expected.
        if right.may_contain(item) {
            return;
        }
    }
    let error: f32 = 1.0;

    // First, we check whether the cardinalities of the HLL counters make sense:

    let left_cardinality = left.estimate_cardinality();
    let right_cardinality = right.estimate_cardinality();

    assert!(
        left_cardinality >= left_unique.len() as f32 * 0.9 - error,
        "Estimated left cardinality was too small: {} vs {}",
        left_cardinality,
        left_unique.len()
    );

    assert!(
        left_cardinality <= left_unique.len() as f32 * 1.1 + error,
        "Estimated left cardinality was too large: {} vs {}",
        left_cardinality,
        left_unique.len()
    );

    assert!(
        right_cardinality >= right_unique.len() as f32 * 0.9 - error,
        "Estimated right cardinality was too small: {} vs {}",
        right_cardinality,
        right_unique.len()
    );

    assert!(
        right_cardinality <= right_unique.len() as f32 * 1.1 + error,
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
        estimated_intersection >= exact_intersection_size as f32 * 0.9 - error,
        "Estimated intersection size was too small: {} vs {} - {:?}",
        estimated_intersection,
        exact_intersection_size,
        left.estimate_union_and_sets_cardinality(&right)
    );

    assert!(
        estimated_intersection <= exact_intersection_size as f32 * 1.1 + error,
        "Estimated intersection size was too large: {} vs {} - {:?}",
        estimated_intersection,
        exact_intersection_size,
        left.estimate_union_and_sets_cardinality(&right)
    );
});