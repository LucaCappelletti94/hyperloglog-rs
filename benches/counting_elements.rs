#![feature(test)]
extern crate test;

use hyperloglogplus::HyperLogLog as AlternativeHyperLogLog;
use hyperloglogplus::HyperLogLogPF;
use std::collections::hash_map::RandomState;

use hyperloglog_rs::prelude::*;

use test::{black_box, Bencher};

#[bench]
fn bench_count_16(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 4;
    const BITS: usize = 6;
    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    for i in 0..NUMBER_OF_ELEMENTS {
        hll.insert(i);
    }

    b.iter(|| {
        black_box(hll.estimate_cardinality());
    });
}

#[bench]
fn bench_count_16_tabac(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 4;

    let mut alternative: HyperLogLogPF<usize, _> =
        HyperLogLogPF::new(PRECISION as u8, RandomState::new()).unwrap();

    for i in 0..NUMBER_OF_ELEMENTS {
        alternative.insert(&i);
    }

    b.iter(|| {
        black_box(alternative.count());
    });
}

#[bench]
fn bench_count_64(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 6;
    const BITS: usize = 6;
    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    for i in 0..NUMBER_OF_ELEMENTS {
        hll.insert(i);
    }

    b.iter(|| {
        black_box(hll.estimate_cardinality());
    });
}

#[bench]
fn bench_count_64_tabac(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 6;

    let mut alternative: HyperLogLogPF<usize, _> =
        HyperLogLogPF::new(PRECISION as u8, RandomState::new()).unwrap();

    for i in 0..NUMBER_OF_ELEMENTS {
        alternative.insert(&i);
    }

    b.iter(|| {
        black_box(alternative.count());
    });
}

#[bench]
fn bench_count_128(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 7;
    const BITS: usize = 6;
    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    for i in 0..NUMBER_OF_ELEMENTS {
        hll.insert(i);
    }

    b.iter(|| {
        black_box(hll.estimate_cardinality());
    });
}

#[bench]
fn bench_count_128_tabac(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 7;

    let mut alternative: HyperLogLogPF<usize, _> =
        HyperLogLogPF::new(PRECISION as u8, RandomState::new()).unwrap();

    for i in 0..NUMBER_OF_ELEMENTS {
        alternative.insert(&i);
    }

    b.iter(|| {
        black_box(alternative.count());
    });
}

#[bench]
fn bench_count_256(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 8;
    const BITS: usize = 6;
    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    for i in 0..NUMBER_OF_ELEMENTS {
        hll.insert(i);
    }

    b.iter(|| {
        black_box(hll.estimate_cardinality());
    });
}

#[bench]
fn bench_count_256_tabac(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 8;

    let mut alternative: HyperLogLogPF<usize, _> =
        HyperLogLogPF::new(PRECISION as u8, RandomState::new()).unwrap();

    for i in 0..NUMBER_OF_ELEMENTS {
        alternative.insert(&i);
    }

    b.iter(|| {
        black_box(alternative.count());
    });
}

#[bench]
fn bench_count_512(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 9;
    const BITS: usize = 6;
    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    for i in 0..NUMBER_OF_ELEMENTS {
        hll.insert(i);
    }

    b.iter(|| {
        black_box(hll.estimate_cardinality());
    });
}

#[bench]
fn bench_count_512_tabac(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 9;

    let mut alternative: HyperLogLogPF<usize, _> =
        HyperLogLogPF::new(PRECISION as u8, RandomState::new()).unwrap();

    for i in 0..NUMBER_OF_ELEMENTS {
        alternative.insert(&i);
    }

    b.iter(|| {
        black_box(alternative.count());
    });
}

#[bench]
fn bench_count_1024(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 10;
    const BITS: usize = 6;
    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    for i in 0..NUMBER_OF_ELEMENTS {
        hll.insert(i);
    }

    b.iter(|| {
        black_box(hll.estimate_cardinality());
    });
}

#[bench]
fn bench_count_1024_tabac(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 10;

    let mut alternative: HyperLogLogPF<usize, _> =
        HyperLogLogPF::new(PRECISION as u8, RandomState::new()).unwrap();

    for i in 0..NUMBER_OF_ELEMENTS {
        alternative.insert(&i);
    }

    b.iter(|| {
        black_box(alternative.count());
    });
}

#[bench]
fn bench_count_2048(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 11;
    const BITS: usize = 6;
    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    for i in 0..NUMBER_OF_ELEMENTS {
        hll.insert(i);
    }

    b.iter(|| {
        black_box(hll.estimate_cardinality());
    });
}

#[bench]
fn bench_count_2048_tabac(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 11;

    let mut alternative: HyperLogLogPF<usize, _> =
        HyperLogLogPF::new(PRECISION as u8, RandomState::new()).unwrap();

    for i in 0..NUMBER_OF_ELEMENTS {
        alternative.insert(&i);
    }

    b.iter(|| {
        black_box(alternative.count());
    });
}

#[bench]
fn bench_count_4096(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 12;
    const BITS: usize = 6;
    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    for i in 0..NUMBER_OF_ELEMENTS {
        hll.insert(i);
    }

    b.iter(|| {
        black_box(hll.estimate_cardinality());
    });
}

#[bench]
fn bench_count_4096_tabac(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 12;

    let mut alternative: HyperLogLogPF<usize, _> =
        HyperLogLogPF::new(PRECISION as u8, RandomState::new()).unwrap();

    for i in 0..NUMBER_OF_ELEMENTS {
        alternative.insert(&i);
    }

    b.iter(|| {
        black_box(alternative.count());
    });
}
