#![feature(test)]
extern crate test;

use hyperloglog_rs::prelude::*;
use test::{black_box, Bencher};

const NUMBER_OF_ELEMENTS: usize = 10_000;

/// Macro to generate a criterion benchmark with the provided precision exponent and bits
macro_rules! bench_estimation {
    ($exponent:expr, $bits:expr) => {
        paste::item! {
            #[bench]
            fn [<bench_estimation_ $exponent>] (b: &mut Bencher) {
                let mut hll: HyperLogLog<[<Precision $exponent>], $bits> = HyperLogLog::default();

                b.iter(|| {
                    black_box(for i in 0..NUMBER_OF_ELEMENTS {
                        hll.insert(i);
                        hll.estimate_cardinality();
                    });
                });
            }
        }
    };
}

/// Macro to generate a criterion benchmark with the provided precision exponents
macro_rules! bench_estimations {
    ($($exponent:expr),*) => {
        $(
            bench_estimation!($exponent, 4);
            bench_estimation!($exponent, 5);
            bench_estimation!($exponent, 6);
        )*
    };
}

bench_estimations!(4, 5, 6);