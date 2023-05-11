#![feature(test)]
extern crate test;

use hyperloglog_rs::HyperLogLog;

use test::{black_box, Bencher};

const BITS: usize = 6;
const NUMBER_OF_ELEMENTS: usize = 100_000;

#[bench]
fn bench_count_16(b: &mut Bencher) {
    // Optionally include some setup
    const PRECISION: usize = 4;

    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_count_32(b: &mut Bencher) {
    // Optionally include some setup
    const PRECISION: usize = 5;

    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_count_64(b: &mut Bencher) {
    // Optionally include some setup
    const PRECISION: usize = 6;

    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_count_128(b: &mut Bencher) {
    // Optionally include some setup
    const PRECISION: usize = 7;

    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}
