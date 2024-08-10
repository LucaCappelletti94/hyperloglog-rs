//! Bench to compare the performance of different hash functions.
#![feature(test)]
extern crate test;

use criterion::{criterion_group, criterion_main, Criterion};
use std::hash::Hash;
use std::hash::Hasher;

use std::hint::black_box;

const NUMBER_OF_ELEMENTS: usize = 1_000_000;


fn bench_xx_hasher(b: &mut Criterion) {
    b.bench_function("xx_hasher", |b| {
        b.iter(|| {
            let mut xor = 4567890987_u64;
            for element in 0..NUMBER_OF_ELEMENTS {
                let mut hasher = twox_hash::XxHash64::default();
                black_box(element).hash(&mut hasher);
                xor ^= hasher.finish();
            }
            xor
        })
    });
}

fn bench_xx3_hasher(b: &mut Criterion) {
    b.bench_function("xx3_hasher", |b| {
        b.iter(|| {
            let mut xor = 4567890987_u64;
            for element in 0..NUMBER_OF_ELEMENTS {
                let mut hasher = twox_hash::Xxh3Hash64::default();
                black_box(element).hash(&mut hasher);
                xor ^= hasher.finish();
            }
            xor
        })
    });
}

fn bench_wyhash(b: &mut Criterion) {
    b.bench_function("wyhash", |b| {
        b.iter(|| {
            let mut xor = 4567890987_u64;
            for element in 0..NUMBER_OF_ELEMENTS {
                let mut hasher = wyhash::WyHash::default();
                black_box(element).hash(&mut hasher);
                xor ^= hasher.finish();
            }
            xor
        })
    });
}

fn bench_default_hasher(b: &mut Criterion) {
    b.bench_function("default_hasher", |b| {
        b.iter(|| {
            let mut xor = 4567890987_u64;
            for element in 0..NUMBER_OF_ELEMENTS {
                let mut hasher = std::collections::hash_map::DefaultHasher::default();
                black_box(element).hash(&mut hasher);
                xor ^= hasher.finish();
            }
            xor
        })
    });
}

criterion_group! {
    name=xx_hasher;
    config = Criterion::default();
    targets=bench_xx_hasher, bench_xx3_hasher
}

criterion_group! {
    name=wyhash;
    config = Criterion::default();
    targets=bench_wyhash
}

criterion_group! {
    name=default_hasher;
    config = Criterion::default();
    targets=bench_default_hasher
}

criterion_main!(xx_hasher, wyhash, default_hasher);
