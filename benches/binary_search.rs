//! Compares different binary search strategies.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::prelude::*;

const NUMBER_OF_ARRAYS: usize = 10_000;
const ARRAY_SIZE: usize = 200;

fn random_u32_array<const N: usize>() -> [u32; N] {
    let mut rng = thread_rng();
    let mut arr = [u32::default(); N];
    for i in 0..N {
        arr[i] = rng.gen();
    }
    arr.sort_unstable();
    arr
}

fn random_f32_array<const N: usize>() -> [f32; N] {
    let mut rng = thread_rng();
    let mut arr = [f32::default(); N];
    for i in 0..N {
        arr[i] = rng.gen();
    }
    arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    arr
}


fn random_f64_array<const N: usize>() -> [f64; N] {
    let mut rng = thread_rng();
    let mut arr = [f64::default(); N];
    for i in 0..N {
        arr[i] = rng.gen();
    }
    arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    arr
}

fn binary_search_naive<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    arr.iter().position(|x| x == target)
}

fn binary_search_std<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    arr.binary_search(target).ok()
}

fn binary_search_std_by<T: PartialOrd>(arr: &[T], target: &T) -> Option<usize> {
    arr.binary_search_by(|x| x.partial_cmp(target).unwrap())
        .ok()
}

fn partition_point_std<T: PartialOrd>(arr: &[T], target: &T) -> usize {
    arr.partition_point(|x| x < target)
}

fn bench_search_native_u32(b: &mut Criterion) {
    let arrays = (0..NUMBER_OF_ARRAYS)
        .map(|_| (random_u32_array::<ARRAY_SIZE>(), thread_rng().gen()))
        .collect::<Vec<([u32; ARRAY_SIZE], u32)>>();
    b.bench_function("search_native_u32", |b| {
        b.iter(|| {
            let mut position = 0;
            for (arr, target) in &arrays {
                position ^= binary_search_naive(black_box(arr), black_box(&target)).unwrap_or(5);
            }
            position
        })
    });
}

fn bench_search_std_u32(b: &mut Criterion) {
    let arrays = (0..NUMBER_OF_ARRAYS)
        .map(|_| (random_u32_array::<ARRAY_SIZE>(), thread_rng().gen()))
        .collect::<Vec<_>>();
    b.bench_function("binary_search_std_u32", |b| {
        b.iter(|| {
            let mut position = 0;
            for (arr, target) in &arrays {
                position ^= binary_search_std(black_box(arr), black_box(&target)).unwrap_or(5);
            }
            position
        })
    });
}

fn bench_partition_point_std_u32(b: &mut Criterion) {
    let arrays = (0..NUMBER_OF_ARRAYS)
        .map(|_| (random_u32_array::<ARRAY_SIZE>(), thread_rng().gen()))
        .collect::<Vec<_>>();
    b.bench_function("partition_point_std_u32", |b| {
        b.iter(|| {
            let mut position = 0;
            for (arr, target) in &arrays {
                position ^= partition_point_std(black_box(arr), black_box(&target));
            }
            position
        })
    });
}

fn bench_binary_search_f32(b: &mut Criterion) {
    let arrays = (0..NUMBER_OF_ARRAYS)
        .map(|_| (random_f32_array::<ARRAY_SIZE>(), thread_rng().gen()))
        .collect::<Vec<_>>();
    b.bench_function("binary_search_f32", |b| {
        b.iter(|| {
            let mut position = 0;
            for (arr, target) in &arrays {
                position ^= binary_search_std_by(black_box(arr), black_box(&target)).unwrap_or(5);
            }
            position
        })
    });
}

fn bench_partition_point_std_f32(b: &mut Criterion) {
    let arrays = (0..NUMBER_OF_ARRAYS)
        .map(|_| (random_f32_array::<ARRAY_SIZE>(), thread_rng().gen()))
        .collect::<Vec<_>>();
    b.bench_function("partition_point_std_f32", |b| {
        b.iter(|| {
            let mut position = 0;
            for (arr, target) in &arrays {
                position ^= partition_point_std(black_box(arr), black_box(&target));
            }
            position
        })
    });
}


fn bench_binary_search_f64(b: &mut Criterion) {
    let arrays = (0..NUMBER_OF_ARRAYS)
        .map(|_| (random_f64_array::<ARRAY_SIZE>(), thread_rng().gen()))
        .collect::<Vec<_>>();
    b.bench_function("binary_search_f64", |b| {
        b.iter(|| {
            let mut position = 0;
            for (arr, target) in &arrays {
                position ^= binary_search_std_by(black_box(arr), black_box(&target)).unwrap_or(5);
            }
            position
        })
    });
}

fn bench_partition_point_std_f64(b: &mut Criterion) {
    let arrays = (0..NUMBER_OF_ARRAYS)
        .map(|_| (random_f64_array::<ARRAY_SIZE>(), thread_rng().gen()))
        .collect::<Vec<_>>();
    b.bench_function("partition_point_std_f64", |b| {
        b.iter(|| {
            let mut position = 0;
            for (arr, target) in &arrays {
                position ^= partition_point_std(black_box(arr), black_box(&target));
            }
            position
        })
    });
}


criterion_group! {
    name=search_u32;
    config = Criterion::default();
    targets=bench_search_native_u32, bench_search_std_u32, bench_partition_point_std_u32
}

criterion_group! {
    name=search_f32;
    config = Criterion::default();
    targets=bench_binary_search_f32, bench_partition_point_std_f32
}

criterion_group! {
    name=search_f64;
    config = Criterion::default();
    targets=bench_binary_search_f64, bench_partition_point_std_f64
}

criterion_main!(search_u32, search_f32, search_f64);
