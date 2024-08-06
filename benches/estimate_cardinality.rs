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

const NUMBER_OF_ELEMENTS: usize = 10_000;

/// Macro to generate a criterion benchmark with the provided precision exponent and bits
macro_rules! bench_cardinality {
    ($precision:ty, $bits:ty) => {
        paste::item! {
            fn [<bench_cardinality_ $precision:lower _ $bits:lower>] (b: &mut Criterion) {
                b.bench_function(
                    format!("cardinality_precision_{}_bits_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS).as_str(),
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

            fn [<bench_multiplicities_cardinality_ $precision:lower _ $bits:lower>] (b: &mut Criterion) {
                b.bench_function(
                    format!("multiplicities_cardinality_precision_{}_bits_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS).as_str(),
                    |b| {
                        b.iter(||{
                            let mut hll: HLLMultiplicities<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, <$precision as ArrayMultiplicities<$bits>>::ArrayMultiplicities> = HLLMultiplicities::default();
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
                            let mut hll: MLE<3, HLLMultiplicities<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, <$precision as ArrayMultiplicities<$bits>>::ArrayMultiplicities>> = MLE::<3, _>::default();
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
    ($($precision:ty),*) => {
        $(
            bench_cardinality!($precision, Bits5);
            bench_cardinality!($precision, Bits6);

            paste::item! {
                fn [<bench_tabac_pf_cardinality_ $precision:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("tabac_pf_cardinality_precision_{}_bits_6", $precision::EXPONENT).as_str(),
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

                fn [<bench_tabac_plusplus_cardinality_ $precision:lower>] (b: &mut Criterion) {
                    b.bench_function(
                        format!("tabac_plusplus_cardinality_precision_{}_bits_6", $precision::EXPONENT).as_str(),
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
                    name=[<cardinality_tabac_pf_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_tabac_pf_cardinality_ $precision:lower>]
                }
                criterion_group! {
                    name=[<cardinality_tabac_plusplus_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_tabac_plusplus_cardinality_ $precision:lower>]
                }
                criterion_group! {
                    name=[<cardinality_sa_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_sa_cardinality_ $precision:lower>]
                }
                criterion_group! {
                    name=[<cardinality_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_cardinality_ $precision:lower _bits5>], [<bench_cardinality_ $precision:lower _bits6>]
                }
                criterion_group! {
                    name=[<multiplicities_cardinality_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_multiplicities_cardinality_ $precision:lower _bits5>], [<bench_multiplicities_cardinality_ $precision:lower _bits6>]
                }
                criterion_group! {
                    name=[<mle_cardinality_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_mle_cardinality_ $precision:lower _bits5>], [<bench_mle_cardinality_ $precision:lower _bits6>]
                }
            }
        )*
    };
}

bench_cardinalitys!(
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
    cardinality_precision4,
    cardinality_precision5,
    cardinality_precision6,
    cardinality_precision7,
    cardinality_precision8,
    cardinality_precision9,
    cardinality_precision10,
    cardinality_precision11,
    cardinality_precision12,
    cardinality_precision13,
    cardinality_precision14,
    cardinality_precision15,
    cardinality_precision16,
    multiplicities_cardinality_precision4,
    multiplicities_cardinality_precision5,
    multiplicities_cardinality_precision6,
    multiplicities_cardinality_precision7,
    multiplicities_cardinality_precision8,
    multiplicities_cardinality_precision9,
    multiplicities_cardinality_precision10,
    multiplicities_cardinality_precision11,
    multiplicities_cardinality_precision12,
    multiplicities_cardinality_precision13,
    multiplicities_cardinality_precision14,
    multiplicities_cardinality_precision15,
    multiplicities_cardinality_precision16,
    cardinality_tabac_pf_precision4,
    cardinality_tabac_pf_precision5,
    cardinality_tabac_pf_precision6,
    cardinality_tabac_pf_precision7,
    cardinality_tabac_pf_precision8,
    cardinality_tabac_pf_precision9,
    cardinality_tabac_pf_precision10,
    cardinality_tabac_pf_precision11,
    cardinality_tabac_pf_precision12,
    cardinality_tabac_pf_precision13,
    cardinality_tabac_pf_precision14,
    cardinality_tabac_pf_precision15,
    cardinality_tabac_pf_precision16,
    cardinality_tabac_plusplus_precision4,
    cardinality_tabac_plusplus_precision5,
    cardinality_tabac_plusplus_precision6,
    cardinality_tabac_plusplus_precision7,
    cardinality_tabac_plusplus_precision8,
    cardinality_tabac_plusplus_precision9,
    cardinality_tabac_plusplus_precision10,
    cardinality_tabac_plusplus_precision11,
    cardinality_tabac_plusplus_precision12,
    cardinality_tabac_plusplus_precision13,
    cardinality_tabac_plusplus_precision14,
    cardinality_tabac_plusplus_precision15,
    cardinality_tabac_plusplus_precision16,
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
