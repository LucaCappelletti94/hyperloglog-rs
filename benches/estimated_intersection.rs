#![cfg(feature = "std")]
#![feature(test)]

extern crate test;

use hyperloglog_rs::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use test::{black_box, Bencher};

fn populate_vectors(
    random_state: u64,
) -> (HyperLogLog<Precision10, 6>, HyperLogLog<Precision10, 6>) {
    // Create the counters
    let mut left = HyperLogLog::default();
    let mut right = HyperLogLog::default();

    // We randomize the number of elements to insert in the first
    // counter, to make sure we test different cases.
    let mut rng = rand::rngs::StdRng::seed_from_u64(random_state);
    let left_size = rng.gen_range(0..100_000);

    // We update the random state to make sure the second counter
    // is different from the first one.
    let right_size = rng.gen_range(0..100_000);

    // We also compute the maximal size of the left and right universe,
    // to make sure we test different cases.
    let left_max_size = rng.gen_range(0..100_000);
    let right_max_size = rng.gen_range(0..100_000);

    // We insert the elements in both counters
    for _ in 0..left_size {
        left.insert(&rng.gen::<u64>() % left_max_size);
    }

    for _ in 0..right_size {
        right.insert(&rng.gen::<u64>() % right_max_size);
    }

    (left, right)
}

#[bench]
fn bench_intersection_hll(b: &mut Bencher) {
    let mut cases = Vec::new();

    let number_of_cases = 10;
    for random_state in 0..number_of_cases {
        let (left, right) = populate_vectors(random_state + 56);
        cases.push((left, right));
    }

    b.iter(|| {
        // Inner closure, the actual test
        black_box(for (left, right) in &cases {
            left.estimate_intersection_cardinality::<f32>(&right);
        });
    });
}

#[bench]
fn bench_intersection_mle_2(b: &mut Bencher) {
    let mut cases: Vec<(
        MLE<2, HyperLogLog<Precision10, 6>>,
        MLE<2, HyperLogLog<Precision10, 6>>,
    )> = Vec::new();

    let number_of_cases = 10;

    for random_state in 0..number_of_cases {
        let (left, right) = populate_vectors(random_state + 56);
        cases.push((left.into(), right.into()));
    }

    b.iter(|| {
        // Inner closure, the actual test
        black_box(for (left, right) in &cases {
            left.estimate_intersection_cardinality::<f32>(&right);
        });
    });
}

#[bench]
fn bench_intersection_mle_3(b: &mut Bencher) {
    let mut cases: Vec<(
        MLE<3, HyperLogLog<Precision10, 6>>,
        MLE<3, HyperLogLog<Precision10, 6>>,
    )> = Vec::new();

    let number_of_cases = 10;

    for random_state in 0..number_of_cases {
        let (left, right) = populate_vectors(random_state + 56);
        cases.push((left.into(), right.into()));
    }

    b.iter(|| {
        // Inner closure, the actual test
        black_box(for (left, right) in &cases {
            left.estimate_intersection_cardinality::<f32>(&right);
        });
    });
}
