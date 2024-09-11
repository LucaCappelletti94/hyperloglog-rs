//! Benchmark for the methods of the array data structure.
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;

const PRECISION: usize = 15;
const REGISTER_SIZE: usize = 6;
const NUMBER_OF_REGISTERS: usize = 1 << PRECISION;
const PACKED_SIZE: usize = (NUMBER_OF_REGISTERS * REGISTER_SIZE).div_ceil(64);

fn bench_array_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("array_insert");

    group.bench_function("packed4_insert", |b| {
        b.iter(|| {
            let mut left = 0;
            let mut right = 0;
            let mut packed: Array<PACKED_SIZE, Bits4> = Array::default();
            for i in 0..NUMBER_OF_REGISTERS {
                for value in 0..64 {
                    let (l, r) = packed.set_apply(black_box(i), black_box(|x: u8| x.max(value)));
                    left ^= l;
                    right ^= r;
                }
            }
            (left, right)
        });
    });

    group.bench_function("packed5_insert", |b| {
        b.iter(|| {
            let mut left = 0;
            let mut right = 0;
            let mut packed: Array<PACKED_SIZE, Bits5> = Array::default();
            for i in 0..NUMBER_OF_REGISTERS {
                for value in 0..64 {
                    let (l, r) = packed.set_apply(black_box(i), black_box(|x: u8| x.max(value)));
                    left ^= l;
                    right ^= r;
                }
            }
            (left, right)
        });
    });

    group.bench_function("packed6_insert", |b| {
        b.iter(|| {
            let mut left = 0;
            let mut right = 0;
            let mut packed: Array<PACKED_SIZE, Bits6> = Array::default();
            for i in 0..NUMBER_OF_REGISTERS {
                for value in 0..64 {
                    let (l, r) = packed.set_apply(black_box(i), black_box(|x: u8| x.max(value)));
                    left ^= l;
                    right ^= r;
                }
            }
            (left, right)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_array_insert);

criterion_main!(benches);
