#![feature(test)]
extern crate test;

use criterion::{criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;
use std::hint::black_box;

const NUMBER_OF_ELEMENTS: usize = 10_000;

/// Macro to generate a criterion benchmark with the provided precision exponent and bits
macro_rules! bench_estimation {
    ($exponent:expr, $bits:expr) => {
        paste::item! {
            fn [<bench_estimation_precision_ $exponent _bits_ $bits>] (b: &mut Criterion) {
                b.bench_function(
                    format!("estimation_precision_{}_bits_{}", $exponent, $bits).as_str(),
                    |b| {
                        b.iter(||{
                            let mut hll: HyperLogLog<[<Precision $exponent>], $bits> = HyperLogLog::default();
                            black_box(for i in 0..NUMBER_OF_ELEMENTS {
                                hll.insert(i);
                                hll.estimate_cardinality();
                            })
                        })
                });
            }

            fn [<bench_multeplicities_estimation_precision_ $exponent _bits_ $bits>] (b: &mut Criterion) {
                b.bench_function(
                    format!("multeplicities_estimation_precision_{}_bits_{}", $exponent, $bits).as_str(),
                    |b| {
                        b.iter(||{
                            let mut hll: HyperLogLogWithMultiplicities<[<Precision $exponent>], $bits> = HyperLogLogWithMultiplicities::default();
                            black_box(for i in 0..NUMBER_OF_ELEMENTS {
                                hll.insert(i);
                                hll.estimate_cardinality();
                            })
                        })
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

            paste::item! {
                criterion_group! {
                    name=[<estimation_precision_ $exponent>];
                    config = Criterion::default();
                    targets=[<bench_estimation_precision_ $exponent _bits_4>], [<bench_estimation_precision_ $exponent _bits_5>], [<bench_estimation_precision_ $exponent _bits_6>]
                }
                criterion_group! {
                    name=[<multiplicities_estimation_precision_ $exponent>];
                    config = Criterion::default();
                    targets=[<bench_multeplicities_estimation_precision_ $exponent _bits_4>], [<bench_multeplicities_estimation_precision_ $exponent _bits_5>], [<bench_multeplicities_estimation_precision_ $exponent _bits_6>]
                }
            }
        )*
    };
}

bench_estimations!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

criterion_main!(
    estimation_precision_4,
    estimation_precision_5,
    estimation_precision_6,
    estimation_precision_7,
    estimation_precision_8,
    estimation_precision_9,
    estimation_precision_10,
    estimation_precision_11,
    estimation_precision_12,
    estimation_precision_13,
    estimation_precision_14,
    estimation_precision_15,
    estimation_precision_16,
    multiplicities_estimation_precision_4,
    multiplicities_estimation_precision_5,
    multiplicities_estimation_precision_6,
    multiplicities_estimation_precision_7,
    multiplicities_estimation_precision_8,
    multiplicities_estimation_precision_9,
    multiplicities_estimation_precision_10,
    multiplicities_estimation_precision_11,
    multiplicities_estimation_precision_12,
    multiplicities_estimation_precision_13,
    multiplicities_estimation_precision_14,
    multiplicities_estimation_precision_15,
    multiplicities_estimation_precision_16
);
