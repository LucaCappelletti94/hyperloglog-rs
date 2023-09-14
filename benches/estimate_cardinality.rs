#![feature(test)]
extern crate test;

use hyperloglog_rs::prelude::*;

use test::{black_box, Bencher};

const BITS: usize = 5;
const NUMBER_OF_ELEMENTS: usize = 10_000;

#[bench]
fn bench_estimation_16(b: &mut Bencher) {
    // Optionally include some setup
    let mut hll: HyperLogLog<Precision4, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_32(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision5, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_64(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision6, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_128(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision7, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_256(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision8, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_512(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision9, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_1024(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision10, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_2048(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision11, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_4096(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision12, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_8192(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision13, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_16389(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision14, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_32768(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision15, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_65536(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision16, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_multiplicity_16(b: &mut Bencher) {
    // Optionally include some setup
    let mut hll: HyperLogLog<Precision4, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_multiplicity_32(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision5, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_multiplicity_64(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision6, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_multiplicity_128(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision7, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_multiplicity_256(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision8, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_multiplicity_512(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision9, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_multiplicity_1024(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision10, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_multiplicity_2048(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision11, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_multiplicity_4096(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision12, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_multiplicity_8192(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision13, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_multiplicity_16389(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision14, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_multiplicity_32768(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision15, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_multiplicity_65536(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision16, BITS> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}
