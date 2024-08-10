#![feature(test)]
extern crate test;

use criterion::{criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;
use hyperloglogplus::HyperLogLog as TabacHyperLogLog;
use hyperloglogplus::HyperLogLogPF as TabacHyperLogLogPF;
use std::hash::RandomState;
use hyperloglogplus::HyperLogLogPlus as TabacHyperLogLogPlus;
use std::hint::black_box;
use wyhash::WyHash;
use streaming_algorithms::HyperLogLog as SAHyperLogLog;
use rust_hyperloglog::HyperLogLog as RustHyperLogLog;
use cardinality_estimator::CardinalityEstimator;

const NUMBER_OF_ELEMENTS: usize = 7564;

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

fn xorshift64(x: &mut u64) -> u64 {
    *x ^= *x << 13;
    *x ^= *x >> 7;
    *x ^= *x << 17;
    *x
}

fn iter_xorshift64() -> impl Iterator<Item = u64> {
    let mut x = 8675438654567345_128;
    x = splitmix64(x);
    let this_array_size = splitmix64(x) as usize % NUMBER_OF_ELEMENTS;
    (0..this_array_size).map(move |_| xorshift64(&mut x))
}

macro_rules! bench_cardinality {
    ($precision:ty, $bits:ty, $($hasher:ty),*) => {
        $(
            paste::item! {
                fn [<bench_hll_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("hll_cardinality_precision_{}_bits_{}_hasher_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS, stringify!($hasher)).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: HyperLogLog<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher> = HyperLogLog::default();
                                let mut total_cardinality = 0.0;
                                for i in iter_xorshift64() {
                                    hll.insert(black_box(i));
                                    total_cardinality += hll.estimate_cardinality::<f64>();
                                }
                                total_cardinality
                            })
                    });
                }

                fn [<bench_beta_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("beta_cardinality_precision_{}_bits_{}_hasher_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS, stringify!($hasher)).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: HyperLogLog<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher> = HyperLogLog::default();
                                let mut total_cardinality = 0.0;
                                for i in iter_xorshift64() {
                                    hll.insert(black_box(i));
                                    total_cardinality += hll.estimate_cardinality_with_beta::<f64>();
                                }
                                total_cardinality
                            })
                    });
                }
            }
        )*
    };
}

macro_rules! bench_ce_cardinality {
    ($precision:ty, $bits:ty, $($hasher:ty),*) => {
        $(
            paste::item! {
                fn [<bench_ce_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("ce_cardinality_precision_{}_bits_{}_hasher_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS, stringify!($hasher)).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: CardinalityEstimator<u64, $hasher, {$precision::EXPONENT}, {$bits::NUMBER_OF_BITS}> = CardinalityEstimator::default();
                                let mut total_cardinality = 0;
                                for i in iter_xorshift64() {
                                    hll.insert(black_box(&i));
                                    total_cardinality += hll.estimate();
                                }
                                total_cardinality
                            })
                    });
                }
                
            }
        )*
    };
}

type XxHash64 = twox_hash::XxHash64;

/// Macro to generate a criterion benchmark with the provided precision exponent and bits
macro_rules! bench_cardinality_bits {
    ($precision:ty, $($bits:ty),*) => {
        $(
            bench_ce_cardinality!($precision, $bits, WyHash);
            bench_cardinality!($precision, $bits, WyHash, XxHash64);
        )*
    };
}

/// Macro to generate a criterion benchmark with the provided precision exponents
macro_rules! bench_cardinalities {
    ($(($precision:ty, $sample_size:expr)),*) => {
        $(
            bench_cardinality_bits!($precision, Bits6);

            paste::item! {
                fn [<bench_tabacpf_cardinality_ $precision:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("tabacpf_cardinality_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: TabacHyperLogLogPF<u64, RandomState> = TabacHyperLogLogPF::new($precision::EXPONENT as u8, RandomState::new()).unwrap();
                                let mut total_cardinality = 0.0;
                                for i in iter_xorshift64() {
                                    TabacHyperLogLog::insert(&mut hll, black_box(&i));
                                    total_cardinality += hll.count();
                                }
                                total_cardinality
                            })
                    });
                }

                fn [<bench_tabacplusplus_cardinality_ $precision:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("tabacplusplus_cardinality_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: TabacHyperLogLogPlus<u64, RandomState> = TabacHyperLogLogPlus::new($precision::EXPONENT as u8, RandomState::new()).unwrap();
                                let mut total_cardinality = 0.0;
                                for i in iter_xorshift64() {
                                    TabacHyperLogLog::insert(&mut hll, black_box(&i));
                                    total_cardinality += hll.count();
                                }
                                total_cardinality
                            })
                    });
                }

                fn [<bench_rhll_cardinality_ $precision:lower _bits6>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("rhll_cardinality_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: RustHyperLogLog = RustHyperLogLog::new_deterministic($precision::error_rate(), 6785467548654986_128);
                                let mut total_cardinality = 0;
                                for i in iter_xorshift64() {
                                    hll.insert(black_box(&i));
                                    total_cardinality ^= black_box(hll.len()) as usize;
                                }
                                total_cardinality
                            })
                    });
                }

                fn [<bench_sa_cardinality_ $precision:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("sa_cardinality_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: SAHyperLogLog<u64> = SAHyperLogLog::new($precision::error_rate());
                                let mut total_cardinality = 0.0;
                                for i in iter_xorshift64() {
                                    hll.push(black_box(&i));
                                    total_cardinality += hll.len();
                                }
                                total_cardinality
                            })
                    });
                }

                criterion_group! {
                    name=[<cardinality_tabacpf_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size / 5);
                    targets=[<bench_tabacpf_cardinality_ $precision:lower>]
                }
                criterion_group! {
                    name=[<cardinality_tabacplusplus_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size / 5);
                    targets=[<bench_tabacplusplus_cardinality_ $precision:lower>]
                }
                criterion_group! {
                    name=[<cardinality_sa_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_sa_cardinality_ $precision:lower>]
                }
                criterion_group! {
                    name=[<cardinality_hll_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_hll_cardinality_ $precision:lower _bits6_xxhash64>], [<bench_hll_cardinality_ $precision:lower _bits6_wyhash>], [<bench_beta_cardinality_ $precision:lower _bits6_xxhash64>], [<bench_beta_cardinality_ $precision:lower _bits6_wyhash>]
                }
                criterion_group! {
                    name=[<cardinality_rhll_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size / 5);
                    targets=[<bench_rhll_cardinality_ $precision:lower _bits6>]
                }
                criterion_group! {
                    name=[<cardinality_ce_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_ce_cardinality_ $precision:lower _bits6_wyhash>]
                }
            }
        )*
    };
}

bench_cardinalities!(
    (Precision4, 500),
    (Precision5, 500),
    (Precision6, 500),
    (Precision7, 500),
    (Precision8, 500),
    (Precision9, 500),
    (Precision10, 500),
    (Precision11, 500),
    (Precision12, 500),
    (Precision13, 50),
    (Precision14, 50),
    (Precision15, 50),
    (Precision16, 50)
);

criterion_main!(
    cardinality_hll_precision4,
    cardinality_hll_precision5,
    cardinality_hll_precision6,
    cardinality_hll_precision7,
    cardinality_hll_precision8,
    cardinality_hll_precision9,
    cardinality_hll_precision10,
    cardinality_hll_precision11,
    cardinality_hll_precision12,
    cardinality_hll_precision13,
    cardinality_hll_precision14,
    cardinality_hll_precision15,
    cardinality_hll_precision16,
    cardinality_tabacpf_precision4,
    cardinality_tabacpf_precision5,
    cardinality_tabacpf_precision6,
    cardinality_tabacpf_precision7,
    cardinality_tabacpf_precision8,
    cardinality_tabacpf_precision9,
    cardinality_tabacpf_precision10,
    cardinality_tabacpf_precision11,
    cardinality_tabacpf_precision12,
    cardinality_tabacpf_precision13,
    cardinality_tabacpf_precision14,
    cardinality_tabacpf_precision15,
    cardinality_tabacpf_precision16,
    cardinality_tabacplusplus_precision4,
    cardinality_tabacplusplus_precision5,
    cardinality_tabacplusplus_precision6,
    cardinality_tabacplusplus_precision7,
    cardinality_tabacplusplus_precision8,
    cardinality_tabacplusplus_precision9,
    cardinality_tabacplusplus_precision10,
    cardinality_tabacplusplus_precision11,
    cardinality_tabacplusplus_precision12,
    cardinality_tabacplusplus_precision13,
    cardinality_tabacplusplus_precision14,
    cardinality_tabacplusplus_precision15,
    cardinality_tabacplusplus_precision16,
    cardinality_sa_precision4,
    cardinality_sa_precision5,
    cardinality_sa_precision6,
    cardinality_sa_precision7,
    cardinality_sa_precision8,
    cardinality_sa_precision9,
    cardinality_sa_precision10,
    cardinality_sa_precision11,
    cardinality_sa_precision12,
    cardinality_sa_precision13,
    cardinality_sa_precision14,
    cardinality_sa_precision15,
    cardinality_sa_precision16,
    cardinality_ce_precision4,
    cardinality_ce_precision5,
    cardinality_ce_precision6,
    cardinality_ce_precision7,
    cardinality_ce_precision8,
    cardinality_ce_precision9,
    cardinality_ce_precision10,
    cardinality_ce_precision11,
    cardinality_ce_precision12,
    cardinality_ce_precision13,
    cardinality_ce_precision14,
    cardinality_ce_precision15,
    cardinality_ce_precision16,
    cardinality_rhll_precision4,
    cardinality_rhll_precision5,
    cardinality_rhll_precision6,
    cardinality_rhll_precision7,
    cardinality_rhll_precision8,
    cardinality_rhll_precision9,
    cardinality_rhll_precision10,
    cardinality_rhll_precision11,
    cardinality_rhll_precision12,
    cardinality_rhll_precision13,
    cardinality_rhll_precision14,
    cardinality_rhll_precision15,
    cardinality_rhll_precision16    
);
