#![feature(test)]
extern crate test;

use criterion::{criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;
use hyperloglogplus::HyperLogLog as TabacHyperLogLog;
use hyperloglogplus::HyperLogLogPF as TabacHyperLogLogPF;
use std::hash::RandomState;
use hyperloglogplus::HyperLogLogPlus as TabacHyperLogLogPlus;
use std::hint::black_box;
use streaming_algorithms::HyperLogLog as SAHyperLogLog;

const NUMBER_OF_ELEMENTS: usize = 50_000;

/// Macro to generate a criterion benchmark with the provided precision exponent and bits
macro_rules! bench_cardinality {
    ($precision:ty, $bits:ty) => {
        paste::item! {
            fn [<bench_hll_cardinality_ $precision:lower _ $bits:lower>] (b: &mut Criterion) {
                b.bench_function(
                    format!("hll_cardinality_precision_{}_bits_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS).as_str(),
                    |b| {
                        b.iter(||{
                            let mut hll: HyperLogLog<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister> = HyperLogLog::default();
                            black_box(for i in 0..NUMBER_OF_ELEMENTS {
                                hll.insert(i);
                                hll.estimate_cardinality::<f64>();
                            })
                        })
                });
            }

            fn [<bench_mle_cardinality_ $precision:lower _ $bits:lower>] (b: &mut Criterion) {
                b.bench_function(
                    format!("mle_cardinality_precision_{}_bits_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS).as_str(),
                    |b| {
                        b.iter(||{
                            let mut hll: MLE<3, HyperLogLog<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister>> = MLE::<3, _>::default();
                            black_box(for i in 0..NUMBER_OF_ELEMENTS {
                                hll.insert(i);
                                hll.estimate_cardinality::<f64>();
                            })
                        })
                });
            }
        }
    };
}

/// Macro to generate a criterion benchmark with the provided precision exponents
macro_rules! bench_cardinalitys {
    ($(($precision:ty, $sample_size:expr)),*) => {
        $(
            bench_cardinality!($precision, Bits5);
            bench_cardinality!($precision, Bits6);

            paste::item! {
                fn [<bench_tabacpf_cardinality_ $precision:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("tabacpf_cardinality_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: TabacHyperLogLogPF<usize, RandomState> = TabacHyperLogLogPF::new($precision::EXPONENT as u8, RandomState::new()).unwrap();
                                black_box(for i in 0..NUMBER_OF_ELEMENTS {
                                    TabacHyperLogLog::insert(&mut hll, &i);
                                    hll.count();
                                })
                            })
                    });
                }

                fn [<bench_tabacplusplus_cardinality_ $precision:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("tabacplusplus_cardinality_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: TabacHyperLogLogPlus<usize, RandomState> = TabacHyperLogLogPlus::new($precision::EXPONENT as u8, RandomState::new()).unwrap();
                                black_box(for i in 0..NUMBER_OF_ELEMENTS {
                                    TabacHyperLogLog::insert(&mut hll, &i);
                                    hll.count();
                                })
                            })
                    });
                }

                fn [<bench_sa_cardinality_ $precision:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("sa_cardinality_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut hll: SAHyperLogLog<usize> = SAHyperLogLog::new($precision::error_rate());
                                black_box(for i in 0..NUMBER_OF_ELEMENTS {
                                    hll.push(&i);
                                    hll.len();
                                })
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
                    config = Criterion::default().sample_size($sample_size);
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
                    targets=[<bench_hll_cardinality_ $precision:lower _bits5>], [<bench_hll_cardinality_ $precision:lower _bits6>]
                }
                criterion_group! {
                    name=[<mle_cardinality_ $precision:lower>];
                    config = Criterion::default().sample_size(10);
                    targets=[<bench_mle_cardinality_ $precision:lower _bits5>], [<bench_mle_cardinality_ $precision:lower _bits6>]
                }
            }
        )*
    };
}

bench_cardinalitys!(
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
    mle_cardinality_precision4,
    mle_cardinality_precision5,
    mle_cardinality_precision6,
    mle_cardinality_precision7,
    mle_cardinality_precision8,
    mle_cardinality_precision9,
    mle_cardinality_precision10,
    mle_cardinality_precision11,
    mle_cardinality_precision12,
    mle_cardinality_precision13,
    mle_cardinality_precision14,
    mle_cardinality_precision15,
    mle_cardinality_precision16,
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
    cardinality_sa_precision16
);
