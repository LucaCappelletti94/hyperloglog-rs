#![feature(test)]
extern crate test;

use hyperloglog_rs::prelude::*;

use test::{black_box, Bencher};

fn populate_vectors<const N: usize>() -> (
    [HyperLogLog<Precision4, 4>; N],
    [HyperLogLog<Precision4, 4>; N],
) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 100;

    // We create the counters, populate them with data and
    // then we create the arrays to use to estimate the
    // estimated overlap cardinality matrices:

    // Create the counters
    let mut left: [HyperLogLog<Precision4, 4>; N] = [HyperLogLog::default(); N];
    let mut right: [HyperLogLog<Precision4, 4>; N] = [HyperLogLog::default(); N];

    left[0].insert(&56);
    right[0].insert(&32);

    // Populate the counters
    for i in 1..N {
        // We make sure that all values in the leftmost
        // counters are contained in the rightmost counters
        let tmp = left[i] | &left[i - 1];
        left[i] = tmp;
        let tmp = right[i] | &right[i - 1];
        right[i] = tmp;
        for j in 1..NUMBER_OF_ELEMENTS {
            // We populate the countes
            left[i].insert(&((j * i * 3) % 20));
            right[i].insert(&((j * i * 7) % 20));
        }
    }

    (left, right)
}

#[bench]
fn bench_overlap_and_differences_cardinality_matrices_2(b: &mut Bencher) {
    let (left, right) = populate_vectors::<2>();

    b.iter(|| {
        // Inner closure, the actual test
        black_box(for _ in 0..10_000 {
            let _: ([[f32; 2]; 2], [f32; 2], [f32; 2]) =
                HyperLogLog::overlap_and_differences_cardinality_matrices::<2, 2>(&left, &right);
        });
    });
}

#[bench]
fn bench_overlap_and_differences_cardinality_matrices_3(b: &mut Bencher) {
    let (left, right) = populate_vectors::<3>();

    b.iter(|| {
        // Inner closure, the actual test
        black_box(for _ in 0..10_000 {
            let _: ([[f32; 3]; 3], [f32; 3], [f32; 3]) =
                HyperLogLog::overlap_and_differences_cardinality_matrices::<3, 3>(&left, &right);
        });
    });
}

#[bench]
fn bench_overlap_and_differences_cardinality_matrices_4(b: &mut Bencher) {
    let (left, right) = populate_vectors::<4>();

    b.iter(|| {
        // Inner closure, the actual test
        black_box(for _ in 0..10_000 {
            let _: ([[f32; 4]; 4], [f32; 4], [f32; 4]) =
                HyperLogLog::overlap_and_differences_cardinality_matrices::<4, 4>(&left, &right);
        });
    });
}

#[bench]
fn bench_overlap_and_differences_cardinality_matrices_5(b: &mut Bencher) {
    let (left, right) = populate_vectors::<5>();

    b.iter(|| {
        // Inner closure, the actual test
        black_box(for _ in 0..10_000 {
            let _: ([[f32; 5]; 5], [f32; 5], [f32; 5]) =
                HyperLogLog::overlap_and_differences_cardinality_matrices::<5, 5>(&left, &right);
        });
    });
}
