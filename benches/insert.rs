#![feature(test)]
extern crate test;

use criterion::{criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;
use hyperloglogplus::HyperLogLog as TabacHyperLogLog;
use std::hash::RandomState;
use hyperloglogplus::HyperLogLogPlus as TabacHyperLogLogPlus;
use std::hint::black_box;
use streaming_algorithms::HyperLogLog as SAHyperLogLog;

const NUMBER_OF_ELEMENTS: usize = 50_000;

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
                                let mut hll: HyperLogLog<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher> = HyperLogLog::default();
                                for i in 0..NUMBER_OF_ELEMENTS {
                                    hll.insert(black_box(i));
                                }
                            })
                    });
                }
            }
        )*
    };
}

type XxHash64 = twox_hash::XxHash64;

/// Macro to generate a criterion benchmark with the provided precision exponents
macro_rules! bench_inserts {
    ($($precision:ty),*) => {
        $(
            bench_insert!($precision, Bits4, XxHash64);
            bench_insert!($precision, Bits5, XxHash64);
            bench_insert!($precision, Bits6, XxHash64);

            paste::item! {
                fn [<bench_tabacplusplus_insert_ $precision:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("tabacplusplus_insert_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: TabacHyperLogLogPlus<usize, RandomState> = TabacHyperLogLogPlus::new($precision::EXPONENT as u8, RandomState::new()).unwrap();
                                for i in 0..NUMBER_OF_ELEMENTS {
                                    TabacHyperLogLog::insert(&mut hll, black_box(&i));
                                }
                            })
                    });
                }

                fn [<bench_sa_insert_ $precision:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("sa_insert_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: SAHyperLogLog<usize> = SAHyperLogLog::new($precision::error_rate());
                                for i in 0..NUMBER_OF_ELEMENTS {
                                    hll.push(&black_box(i));
                                }
                            })
                    });
                }

                criterion_group! {
                    name=[<insert_tabacplusplus_ $precision:lower>];
                    config = Criterion::default().sample_size(500);
                    targets=[<bench_tabacplusplus_insert_ $precision:lower>]
                }
                criterion_group! {
                    name=[<insert_sa_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_sa_insert_ $precision:lower>]
                }
                criterion_group! {
                    name=[<insert_hll_ $precision:lower>];
                    config = Criterion::default().sample_size(500);
                    targets=[<bench_hll_insert_ $precision:lower _bits4_xxhash64>], [<bench_hll_insert_ $precision:lower _bits5_xxhash64>], [<bench_hll_insert_ $precision:lower _bits6_xxhash64>]
                }
            }
        )*
    };
}

bench_inserts!(
    Precision4,
    Precision5,
    Precision6,
    Precision7,
    Precision8,
    Precision9,
    Precision10,
    Precision11,
    Precision12,
    Precision13,
    Precision14,
    Precision15,
    Precision16
);

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
    insert_sa_precision16
);
