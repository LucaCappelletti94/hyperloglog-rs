#![feature(test)]
extern crate test;

use cardinality_estimator::CardinalityEstimator;
use criterion::{criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;
use hyperloglogplus::HyperLogLog as TabacHyperLogLog;
use hyperloglogplus::HyperLogLogPlus as TabacHyperLogLogPlus;
use rust_hyperloglog::HyperLogLog as RustHyperLogLog;
use std::hash::RandomState;
use std::hint::black_box;
use streaming_algorithms::HyperLogLog as SAHyperLogLog;
use wyhash::WyHash;

const RANDOM_STATE: u64 = 87561346897134_u64;
const NUMBER_OF_ELEMENTS: usize = 10_000;

/// Macro to generate a criterion benchmark with the provided precision exponent and bits
macro_rules! bench_insert {
    ($precision:ty, $bits:ty, $($hasher:ty),*) => {
        $(
            paste::item! {
                fn [<bench_hll_insert_ $precision:lower _ $bits:lower _ $hasher:lower >] (b: &mut Criterion) {
                    b.bench_function(
                        format!("hll_insert_precision_{}_bits_{}_hasher_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS, stringify!($hasher)).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: PlusPlus<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher> = PlusPlus::default();
                                for i in iter_random_values(NUMBER_OF_ELEMENTS, None, RANDOM_STATE) {
                                    hll.insert(black_box(i));
                                }
                            })
                    });
                }
            }
        )*
    };
}

macro_rules! bench_ce_insert {
    ($precision:ty, $bits:ty, $($hasher:ty),*) => {
        $(
            paste::item! {
                fn [<bench_ce_insert_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("ce_insert_precision_{}_bits_{}_hasher_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS, stringify!($hasher)).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: CardinalityEstimator<u64, $hasher, {$precision::EXPONENT}, {$bits::NUMBER_OF_BITS}> = CardinalityEstimator::default();
                                for i in iter_random_values(NUMBER_OF_ELEMENTS, None, RANDOM_STATE) {
                                    hll.insert(black_box(&i));
                                }
                            })
                    });
                }

            }
        )*
    };
}

/// Macro to generate a criterion benchmark with the provided precision exponent and bits
macro_rules! bench_insert_bits {
    ($precision:ty, $($bits:ty),*) => {
        $(
            bench_ce_insert!($precision, $bits, WyHash);
            bench_insert!($precision, $bits, XxHash64, WyHash);
        )*
    };
}

type XxHash64 = twox_hash::XxHash64;

/// Macro to generate a criterion benchmark with the provided precision exponents
macro_rules! bench_inserts {
    ($($precision:ty),*) => {
        $(
            bench_insert_bits!($precision, Bits6);

            paste::item! {
                fn [<bench_tabacplusplus_insert_ $precision:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("tabacplusplus_insert_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: TabacHyperLogLogPlus<u64, RandomState> = TabacHyperLogLogPlus::new($precision::EXPONENT as u8, RandomState::new()).unwrap();
                                for i in iter_random_values(NUMBER_OF_ELEMENTS, None, RANDOM_STATE) {
                                    TabacHyperLogLog::insert(&mut hll, black_box(&i));
                                }
                            })
                    });
                }

                fn [<bench_rhll_insert_ $precision:lower _bits6>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("rhll_insert_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: RustHyperLogLog = RustHyperLogLog::new_deterministic($precision::error_rate(), 6785467548654986_128);
                                for i in iter_random_values(NUMBER_OF_ELEMENTS, None, RANDOM_STATE) {
                                    hll.insert(&black_box(i));
                                }
                            })
                    });
                }

                fn [<bench_sa_insert_ $precision:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("sa_insert_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: SAHyperLogLog<u64> = SAHyperLogLog::new($precision::error_rate());
                                for i in iter_random_values(NUMBER_OF_ELEMENTS, None, RANDOM_STATE) {
                                    hll.push(&black_box(i));
                                }
                            })
                    });
                }

                criterion_group! {
                    name=[<insert_tabacplusplus_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_tabacplusplus_insert_ $precision:lower>]
                }
                criterion_group! {
                    name=[<insert_sa_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_sa_insert_ $precision:lower>]
                }
                criterion_group! {
                    name=[<insert_hll_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_hll_insert_ $precision:lower _bits6_xxhash64>], [<bench_hll_insert_ $precision:lower _bits6_wyhash>]
                }
                criterion_group! {
                    name=[<insert_rhll_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_rhll_insert_ $precision:lower _bits6>]
                }
                criterion_group! {
                    name=[<insert_ce_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_ce_insert_ $precision:lower _bits6_wyhash>]
                }

            }
        )*
    };
}

#[cfg(feature = "low_precisions")]
bench_inserts!(
    Precision4,
    Precision5,
    Precision6,
    Precision7,
    Precision8,
    Precision9,
    Precision10
);

#[cfg(feature = "medium_precisions")]
bench_inserts!(
    Precision11,
    Precision12,
    Precision13,
    Precision14,
    Precision15,
    Precision16
);

#[cfg(feature = "high_precisions")]
bench_inserts!(Precision17, Precision18);

criterion_main!(
    insert_hll_precision4,
    insert_hll_precision5,
    insert_hll_precision6,
    insert_hll_precision7,
    insert_hll_precision8,
    insert_hll_precision9,
    insert_hll_precision10,
    insert_hll_precision11,
    insert_hll_precision12,
    insert_hll_precision13,
    insert_hll_precision14,
    insert_hll_precision15,
    insert_hll_precision16,
    // insert_hll_precision17,
    // insert_hll_precision18,
    insert_tabacplusplus_precision4,
    insert_tabacplusplus_precision5,
    insert_tabacplusplus_precision6,
    insert_tabacplusplus_precision7,
    insert_tabacplusplus_precision8,
    insert_tabacplusplus_precision9,
    insert_tabacplusplus_precision10,
    insert_tabacplusplus_precision11,
    insert_tabacplusplus_precision12,
    insert_tabacplusplus_precision13,
    insert_tabacplusplus_precision14,
    insert_tabacplusplus_precision15,
    insert_tabacplusplus_precision16,
    // insert_tabacplusplus_precision17,
    // insert_tabacplusplus_precision18,
    insert_sa_precision4,
    insert_sa_precision5,
    insert_sa_precision6,
    insert_sa_precision7,
    insert_sa_precision8,
    insert_sa_precision9,
    insert_sa_precision10,
    insert_sa_precision11,
    insert_sa_precision12,
    insert_sa_precision13,
    insert_sa_precision14,
    insert_sa_precision15,
    insert_sa_precision16,
    // insert_sa_precision17,
    // insert_sa_precision18,
    insert_ce_precision4,
    insert_ce_precision5,
    insert_ce_precision6,
    insert_ce_precision7,
    insert_ce_precision8,
    insert_ce_precision9,
    insert_ce_precision10,
    insert_ce_precision11,
    insert_ce_precision12,
    insert_ce_precision13,
    insert_ce_precision14,
    insert_ce_precision15,
    insert_ce_precision16,
    // insert_ce_precision17,
    // insert_ce_precision18,
    insert_rhll_precision4,
    insert_rhll_precision5,
    insert_rhll_precision6,
    insert_rhll_precision7,
    insert_rhll_precision8,
    insert_rhll_precision9,
    insert_rhll_precision10,
    insert_rhll_precision11,
    insert_rhll_precision12,
    insert_rhll_precision13,
    insert_rhll_precision14,
    insert_rhll_precision15,
    insert_rhll_precision16,
    // insert_rhll_precision17,
    // insert_rhll_precision18,
);
