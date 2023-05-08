#![feature(test)]
extern crate test;
use hyperloglog::prelude::*;

use test::{black_box, Bencher};

#[bench]
fn bench_add(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 200_000;
    const PRECISION: usize = 8;
    let mut hll: HyperLogLog<PRECISION> = HyperLogLog::new();

    b.iter(|| {
        // Inner closure, the actual test
        for i in 0..NUMBER_OF_ELEMENTS {
            black_box(hll += i);
        }
    });
}
