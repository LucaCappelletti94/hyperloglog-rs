#![cfg(feature = "std")]
#![feature(test)]
extern crate test;

use core::mem::MaybeUninit;
use hyperloglog_rs::prelude::*;
use std::collections::HashSet;

use test::{black_box, Bencher};

// Method to allocate an array of HashSets using maybe uninitialized memory,
// so to circumvent the fact that HashSet does not implement Copy.
fn allocate_array_of_hashsets<const N: usize, T>() -> [HashSet<T>; N] {
    unsafe {
        let mut array: [HashSet<T>; N] = MaybeUninit::uninit().assume_init();
        for i in 0..N {
            // We replace the previosly initialized value with an hashset
            // and we forget the previous value.
            std::mem::forget(std::mem::replace(&mut array[i], HashSet::new()));
        }
        array
    }
}

fn populate_vectors<const N: usize>() -> (
    [HyperLogLog<Precision12, Bits6>; N],
    [HyperLogLog<Precision12, Bits6>; N],
) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 100;

    // We create the counters, populate them with data and
    // then we create the arrays to use to estimate the
    // estimated overlap cardinality matrices:

    // Create the counters
    let mut left: [HyperLogLog<Precision12, Bits6>; N] = [HyperLogLog::default(); N];
    let mut right: [HyperLogLog<Precision12, Bits6>; N] = [HyperLogLog::default(); N];

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

fn populate_hashsets<const N: usize>() -> ([HashSet<usize>; N], [HashSet<usize>; N]) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 100;

    // We create the counters, populate them with data and
    // then we create the arrays to use to estimate the
    // estimated overlap cardinality matrices:

    // Create the counters
    let mut left: [HashSet<usize>; N] = allocate_array_of_hashsets();
    let mut right: [HashSet<usize>; N] = allocate_array_of_hashsets();

    left[0].insert(56);
    right[0].insert(32);

    // Populate the counters
    for i in 1..N {
        // We make sure that all values in the leftmost
        // counters are contained in the rightmost counters
        let tmp = left[i].union(&left[i - 1]).copied().collect();
        left[i] = tmp;
        let tmp = right[i].union(&right[i - 1]).copied().collect();
        right[i] = tmp;
        for j in 1..NUMBER_OF_ELEMENTS {
            // We populate the countes
            left[i].insert((j * i * 3) % 20);
            right[i].insert((j * i * 7) % 20);
        }
    }

    (left, right)
}

#[bench]
fn bench_overlap_and_differences_cardinality_matrices_2(b: &mut Bencher) {
    let (left, right) = populate_vectors::<2>();

    b.iter(|| {
        // Inner closure, the actual test
        black_box(for _ in 0..100 {
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
        black_box(for _ in 0..100 {
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
        black_box(for _ in 0..100 {
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
        black_box(for _ in 0..100 {
            let _: ([[f32; 5]; 5], [f32; 5], [f32; 5]) =
                HyperLogLog::overlap_and_differences_cardinality_matrices::<5, Bits5>(
                    &left, &right,
                );
        });
    });
}

#[bench]
fn bench_vec_overlap_and_differences_cardinality_matrices_2(b: &mut Bencher) {
    let (left, right) = populate_vectors::<2>();

    b.iter(|| {
        // Inner closure, the actual test
        black_box(for _ in 0..100 {
            let _: (Vec<Vec<f32>>, Vec<f32>, Vec<f32>) =
                HyperLogLog::overlap_and_differences_cardinality_matrices_vec(&left, &right);
        });
    });
}

#[bench]
fn bench_vec_overlap_and_differences_cardinality_matrices_3(b: &mut Bencher) {
    let (left, right) = populate_vectors::<3>();

    b.iter(|| {
        // Inner closure, the actual test
        black_box(for _ in 0..100 {
            let _: (Vec<Vec<f32>>, Vec<f32>, Vec<f32>) =
                HyperLogLog::overlap_and_differences_cardinality_matrices_vec(&left, &right);
        });
    });
}

#[bench]
fn bench_vec_overlap_and_differences_cardinality_matrices_4(b: &mut Bencher) {
    let (left, right) = populate_vectors::<4>();

    b.iter(|| {
        // Inner closure, the actual test
        black_box(for _ in 0..100 {
            let _: (Vec<Vec<f32>>, Vec<f32>, Vec<f32>) =
                HyperLogLog::overlap_and_differences_cardinality_matrices_vec(&left, &right);
        });
    });
}

#[bench]
fn bench_vec_overlap_and_differences_cardinality_matrices_5(b: &mut Bencher) {
    let (left, right) = populate_vectors::<5>();

    b.iter(|| {
        // Inner closure, the actual test
        black_box(for _ in 0..100 {
            let _: (Vec<Vec<f32>>, Vec<f32>, Vec<f32>) =
                HyperLogLog::overlap_and_differences_cardinality_matrices_vec(&left, &right);
        });
    });
}

#[bench]
fn bench_overlap_and_differences_cardinality_matrices_2_exact(b: &mut Bencher) {
    let (left, right) = populate_hashsets::<2>();

    b.iter(|| {
        // Inner closure, the actual test
        black_box(for _ in 0..100 {
            let _: ([[f32; 2]; 2], [f32; 2], [f32; 2]) =
                HashSet::overlap_and_differences_cardinality_matrices::<2, 2>(&left, &right);
        });
    });
}

#[bench]
fn bench_overlap_and_differences_cardinality_matrices_3_exact(b: &mut Bencher) {
    let (left, right) = populate_hashsets::<3>();

    b.iter(|| {
        // Inner closure, the actual test
        black_box(for _ in 0..100 {
            let _: ([[f32; 3]; 3], [f32; 3], [f32; 3]) =
                HashSet::overlap_and_differences_cardinality_matrices::<3, 3>(&left, &right);
        });
    });
}

#[bench]
fn bench_overlap_and_differences_cardinality_matrices_4_exact(b: &mut Bencher) {
    let (left, right) = populate_hashsets::<4>();

    b.iter(|| {
        // Inner closure, the actual test
        black_box(for _ in 0..100 {
            let _: ([[f32; 4]; 4], [f32; 4], [f32; 4]) =
                HashSet::overlap_and_differences_cardinality_matrices::<4, 4>(&left, &right);
        });
    });
}

#[bench]
fn bench_overlap_and_differences_cardinality_matrices_5_exact(b: &mut Bencher) {
    let (left, right) = populate_hashsets::<5>();

    b.iter(|| {
        // Inner closure, the actual test
        black_box(for _ in 0..100 {
            let _: ([[f32; 5]; 5], [f32; 5], [f32; 5]) =
                HashSet::overlap_and_differences_cardinality_matrices::<5, Bits5>(&left, &right);
        });
    });
}
