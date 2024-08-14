#![feature(test)]
extern crate test;

use cardinality_estimator::CardinalityEstimator;
use criterion::{criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;
use hyperloglogplus::HyperLogLog as TabacHyperLogLog;
use hyperloglogplus::HyperLogLogPF as TabacHyperLogLogPF;
use hyperloglogplus::HyperLogLogPlus as TabacHyperLogLogPlus;
use rust_hyperloglog::HyperLogLog as RustHyperLogLog;
use std::hash::RandomState;
use std::hint::black_box;
use streaming_algorithms::HyperLogLog as SAHyperLogLog;
use wyhash::WyHash;

const RANDOM_STATE: u64 = 87561346897134_u64;
const NUMBER_OF_ELEMENTS: usize = 10_000;

macro_rules! bench_cardinality {
    ($precision:ty, $bits:ty, $($hasher:ty),*) => {
        $(
            paste::item! {
                fn [<bench_plusplus_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("plusplus_cardinality_precision_{}_bits_{}_hasher_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS, stringify!($hasher)).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: PlusPlus<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher> = PlusPlus::default();
                                let mut total_cardinality = 0.0_f64;
                                for i in iter_random_values(NUMBER_OF_ELEMENTS, None, RANDOM_STATE) {
                                    hll.insert(black_box(&i));
                                    let cardinality: f64 = hll.estimate_cardinality();
                                    total_cardinality += cardinality;
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
                                let mut hll: LogLogBeta<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher> = LogLogBeta::default();
                                let mut total_cardinality = 0.0_f64;
                                for i in iter_random_values(NUMBER_OF_ELEMENTS, None, RANDOM_STATE) {
                                    hll.insert(black_box(&i));
                                    let cardinality: f64 = hll.estimate_cardinality();
                                    total_cardinality += cardinality;
                                }
                                total_cardinality
                            })
                    });
                }

                fn [<bench_hybridplusplus_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("hybridplusplus_cardinality_precision_{}_bits_{}_hasher_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS, stringify!($hasher)).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: Hybrid<PlusPlus<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>> = Default::default();
                                let mut total_cardinality = 0.0_f64;
                                for i in iter_random_values(NUMBER_OF_ELEMENTS, None, RANDOM_STATE) {
                                    hll.insert(black_box(&i));
                                    let cardinality: f64 = hll.estimate_cardinality();
                                    total_cardinality += cardinality;
                                }
                                total_cardinality
                            })
                    });
                }

                fn [<bench_hybridbeta_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("hybridbeta_cardinality_precision_{}_bits_{}_hasher_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS, stringify!($hasher)).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: Hybrid<LogLogBeta<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>> = Default::default();
                                let mut total_cardinality = 0.0_f64;
                                for i in iter_random_values(NUMBER_OF_ELEMENTS, None, RANDOM_STATE) {
                                    hll.insert(black_box(&i));
                                    let cardinality: f64 = hll.estimate_cardinality();
                                    total_cardinality += cardinality;
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
                                for i in iter_random_values(NUMBER_OF_ELEMENTS, None, RANDOM_STATE) {
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
            bench_cardinality!($precision, $bits, WyHash, XxHash64);
        )*
    };
}

/// Macro to generate a criterion benchmark with the provided precision exponents
macro_rules! bench_cardinalities {
    ($(($precision:ty, $sample_size:expr)),*) => {
        $(
            bench_ce_cardinality!($precision, Bits6, WyHash);
            bench_cardinality_bits!($precision, Bits6);
            bench_cardinality_bits!($precision, Bits8);

            paste::item! {
                fn [<bench_tabacpf_cardinality_ $precision:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("tabacpf_cardinality_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: TabacHyperLogLogPF<u64, RandomState> = TabacHyperLogLogPF::new($precision::EXPONENT as u8, RandomState::new()).unwrap();
                                let mut total_cardinality = 0.0;
                                for i in iter_random_values(NUMBER_OF_ELEMENTS, None, RANDOM_STATE) {
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
                                for i in iter_random_values(NUMBER_OF_ELEMENTS, None, RANDOM_STATE) {
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
                                for i in iter_random_values(NUMBER_OF_ELEMENTS, None, RANDOM_STATE) {
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
                                for i in iter_random_values(NUMBER_OF_ELEMENTS, None, RANDOM_STATE) {
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
                    name=[<cardinality_plusplus_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_plusplus_cardinality_ $precision:lower _bits6_xxhash64>], [<bench_plusplus_cardinality_ $precision:lower _bits6_wyhash>], [<bench_plusplus_cardinality_ $precision:lower _bits8_xxhash64>], [<bench_plusplus_cardinality_ $precision:lower _bits8_wyhash>],
                }
                criterion_group! {
                    name=[<cardinality_hybridplusplus_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_hybridplusplus_cardinality_ $precision:lower _bits6_xxhash64>], [<bench_hybridplusplus_cardinality_ $precision:lower _bits6_wyhash>], [<bench_hybridplusplus_cardinality_ $precision:lower _bits8_xxhash64>], [<bench_hybridplusplus_cardinality_ $precision:lower _bits8_wyhash>]
                }
                criterion_group! {
                    name=[<cardinality_beta_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_beta_cardinality_ $precision:lower _bits6_xxhash64>], [<bench_beta_cardinality_ $precision:lower _bits6_wyhash>], [<bench_beta_cardinality_ $precision:lower _bits8_xxhash64>], [<bench_beta_cardinality_ $precision:lower _bits8_wyhash>]
                }
                criterion_group! {
                    name=[<cardinality_hybridbeta_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_hybridbeta_cardinality_ $precision:lower _bits6_xxhash64>], [<bench_hybridbeta_cardinality_ $precision:lower _bits6_wyhash>], [<bench_hybridbeta_cardinality_ $precision:lower _bits8_xxhash64>], [<bench_hybridbeta_cardinality_ $precision:lower _bits8_wyhash>]
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

#[cfg(feature = "low_precisions")]
bench_cardinalities!(
    (Precision4, 100),
    (Precision5, 100),
    (Precision6, 100),
    (Precision7, 100),
    (Precision8, 100),
    (Precision9, 100),
    (Precision10, 100)
);

#[cfg(feature = "medium_precisions")]
bench_cardinalities!(
    (Precision11, 100),
    (Precision12, 100),
    (Precision13, 50),
    (Precision14, 50),
    (Precision15, 50),
    (Precision16, 50)
);

#[cfg(feature = "high_precisions")]
bench_cardinalities!((Precision17, 50), (Precision18, 50));

criterion_main!(
    // cardinality_plusplus_precision4,
    // cardinality_plusplus_precision5,
    // cardinality_plusplus_precision6,
    // cardinality_plusplus_precision7,
    // cardinality_plusplus_precision8,
    // cardinality_plusplus_precision9,
    // cardinality_plusplus_precision10,
    // cardinality_plusplus_precision11,
    // cardinality_plusplus_precision12,
    // cardinality_plusplus_precision13,
    // cardinality_plusplus_precision14,
    // cardinality_plusplus_precision15,
    // cardinality_plusplus_precision16,
    // cardinality_plusplus_precision17,
    // cardinality_plusplus_precision18,
    // cardinality_beta_precision4,
    // cardinality_beta_precision5,
    // cardinality_beta_precision6,
    // cardinality_beta_precision7,
    // cardinality_beta_precision8,
    // cardinality_beta_precision9,
    // cardinality_beta_precision10,
    // cardinality_beta_precision11,
    // cardinality_beta_precision12,
    // cardinality_beta_precision13,
    // cardinality_beta_precision14,
    // cardinality_beta_precision15,
    // cardinality_beta_precision16,
    // cardinality_beta_precision17,
    // cardinality_beta_precision18,
    cardinality_hybridplusplus_precision4,
    cardinality_hybridplusplus_precision5,
    cardinality_hybridplusplus_precision6,
    cardinality_hybridplusplus_precision7,
    cardinality_hybridplusplus_precision8,
    cardinality_hybridplusplus_precision9,
    cardinality_hybridplusplus_precision10,
    cardinality_hybridplusplus_precision11,
    cardinality_hybridplusplus_precision12,
    cardinality_hybridplusplus_precision13,
    cardinality_hybridplusplus_precision14,
    cardinality_hybridplusplus_precision15,
    cardinality_hybridplusplus_precision16,
    cardinality_hybridplusplus_precision17,
    cardinality_hybridplusplus_precision18,
    cardinality_hybridbeta_precision4,
    cardinality_hybridbeta_precision5,
    cardinality_hybridbeta_precision6,
    cardinality_hybridbeta_precision7,
    cardinality_hybridbeta_precision8,
    cardinality_hybridbeta_precision9,
    cardinality_hybridbeta_precision10,
    cardinality_hybridbeta_precision11,
    cardinality_hybridbeta_precision12,
    cardinality_hybridbeta_precision13,
    cardinality_hybridbeta_precision14,
    cardinality_hybridbeta_precision15,
    cardinality_hybridbeta_precision16,
    cardinality_hybridbeta_precision17,
    cardinality_hybridbeta_precision18,
    // cardinality_tabacpf_precision4,
    // cardinality_tabacpf_precision5,
    // cardinality_tabacpf_precision6,
    // cardinality_tabacpf_precision7,
    // cardinality_tabacpf_precision8,
    // cardinality_tabacpf_precision9,
    // cardinality_tabacpf_precision10,
    // cardinality_tabacpf_precision11,
    // cardinality_tabacpf_precision12,
    // cardinality_tabacpf_precision13,
    // cardinality_tabacpf_precision14,
    // cardinality_tabacpf_precision15,
    // cardinality_tabacpf_precision16,
    // cardinality_tabacpf_precision17,
    // cardinality_tabacpf_precision18,
    // cardinality_tabacplusplus_precision4,
    // cardinality_tabacplusplus_precision5,
    // cardinality_tabacplusplus_precision6,
    // cardinality_tabacplusplus_precision7,
    // cardinality_tabacplusplus_precision8,
    // cardinality_tabacplusplus_precision9,
    // cardinality_tabacplusplus_precision10,
    // cardinality_tabacplusplus_precision11,
    // cardinality_tabacplusplus_precision12,
    // cardinality_tabacplusplus_precision13,
    // cardinality_tabacplusplus_precision14,
    // cardinality_tabacplusplus_precision15,
    // cardinality_tabacplusplus_precision16,
    // cardinality_tabacplusplus_precision17,
    // cardinality_tabacplusplus_precision18,
    // cardinality_sa_precision4,
    // cardinality_sa_precision5,
    // cardinality_sa_precision6,
    // cardinality_sa_precision7,
    // cardinality_sa_precision8,
    // cardinality_sa_precision9,
    // cardinality_sa_precision10,
    // cardinality_sa_precision11,
    // cardinality_sa_precision12,
    // cardinality_sa_precision13,
    // cardinality_sa_precision14,
    // cardinality_sa_precision15,
    // cardinality_sa_precision16,
    // cardinality_sa_precision17,
    // cardinality_sa_precision18,
    // cardinality_ce_precision4,
    // cardinality_ce_precision5,
    // cardinality_ce_precision6,
    // cardinality_ce_precision7,
    // cardinality_ce_precision8,
    // cardinality_ce_precision9,
    // cardinality_ce_precision10,
    // cardinality_ce_precision11,
    // cardinality_ce_precision12,
    // cardinality_ce_precision13,
    // cardinality_ce_precision14,
    // cardinality_ce_precision15,
    // cardinality_ce_precision16,
    // cardinality_ce_precision17,
    // cardinality_ce_precision18,
    // cardinality_rhll_precision4,
    // cardinality_rhll_precision5,
    // cardinality_rhll_precision6,
    // cardinality_rhll_precision7,
    // cardinality_rhll_precision8,
    // cardinality_rhll_precision9,
    // cardinality_rhll_precision10,
    // cardinality_rhll_precision11,
    // cardinality_rhll_precision12,
    // cardinality_rhll_precision13,
    // cardinality_rhll_precision14,
    // cardinality_rhll_precision15,
    // cardinality_rhll_precision16,
    // cardinality_rhll_precision17,
    // cardinality_rhll_precision18,
);
