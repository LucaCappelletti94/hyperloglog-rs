#![feature(test)]
extern crate test;

use hyperloglog_rs::HyperLogLog;

use test::{black_box, Bencher};

const BITS: usize = 5;
const NUMBER_OF_ELEMENTS: usize = 10_000;

#[bench]
fn bench_estimation_16(b: &mut Bencher) {
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
fn bench_estimation_32(b: &mut Bencher) {
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
fn bench_estimation_64(b: &mut Bencher) {
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
fn bench_estimation_128(b: &mut Bencher) {
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

#[bench]
fn bench_estimation_256(b: &mut Bencher) {
    // Optionally include some setup
    const PRECISION: usize = 8;

    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_512(b: &mut Bencher) {
    // Optionally include some setup
    const PRECISION: usize = 9;

    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_1024(b: &mut Bencher) {
    // Optionally include some setup
    const PRECISION: usize = 10;

    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_2048(b: &mut Bencher) {
    // Optionally include some setup
    const PRECISION: usize = 11;

    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_4096(b: &mut Bencher) {
    // Optionally include some setup
    const PRECISION: usize = 12;

    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_8192(b: &mut Bencher) {
    // Optionally include some setup
    const PRECISION: usize = 13;

    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_16389(b: &mut Bencher) {
    // Optionally include some setup
    const PRECISION: usize = 14;

    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_32768(b: &mut Bencher) {
    // Optionally include some setup
    const PRECISION: usize = 15;

    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_65536(b: &mut Bencher) {
    // Optionally include some setup
    const PRECISION: usize = 16;

    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}
