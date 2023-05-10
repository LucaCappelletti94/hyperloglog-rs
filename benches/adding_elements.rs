#![feature(test)]
extern crate test;
use hyperloglog_rs::prelude::*;

use test::{black_box, Bencher};

#[bench]
fn bench_add(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    const PRECISION: usize = 4;
    const BITS: usize = 6;
    let mut hll: HyperLogLog<PRECISION, BITS> = HyperLogLog::new();

    b.iter(|| {
        // Inner closure, the actual test
        for i in 0..NUMBER_OF_ELEMENTS {
            black_box(hll.insert(i));
        }
    });
}
