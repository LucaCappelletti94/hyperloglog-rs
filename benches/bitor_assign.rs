#![feature(test)]
extern crate test;

use hyperloglog_rs::prelude::*;

use siphasher::sip::SipHasher13;
use test::{black_box, Bencher};

#[bench]
fn bench_bit_or_assign(b: &mut Bencher) {
    // Optionally include some setup
    const NUMBER_OF_ELEMENTS: usize = 100_000;
    const BITS: usize = 6;

    b.iter(|| {
        // Inner closure, the actual test
        black_box({
            let mut left: HyperLogLog<Precision4, BITS, SipHasher13> = HyperLogLog::from(2);
            let mut right: HyperLogLog<Precision4, BITS, SipHasher13> = HyperLogLog::from(5);
            for i in 0..NUMBER_OF_ELEMENTS {
                left.insert(&i);
                right.insert(&(i * 10));
                left |= right.clone();
            }
        });
    });
}
