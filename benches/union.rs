#![feature(test)]
extern crate test;

use criterion::{criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;
use hyperloglogplus::HyperLogLog as TabacHyperLogLog;
use hyperloglogplus::HyperLogLogPF as TabacHyperLogLogPF;
use hyperloglogplus::HyperLogLogPlus as TabacHyperLogLogPlus;
use std::hash::RandomState;
use std::hint::black_box;
use streaming_algorithms::HyperLogLog as SAHyperLogLog;

const MAXIMAL_ARRAY_SIZE: usize = 10_000;
const NUMBER_OF_ELEMENTS: usize = 20;

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

fn iter_xorshift64(x: &mut u64) -> impl Iterator<Item = u64> + '_{
    *x = splitmix64(*x);
    let this_array_size = splitmix64(*x) as usize % MAXIMAL_ARRAY_SIZE;
    (0..this_array_size).map(move |_| xorshift64(x))
}

fn populate_hll_vector<P: Precision, H: InsertValue>(random_state: &mut u64) -> Vec<H> {
    let mut hll_vector = Vec::with_capacity(NUMBER_OF_ELEMENTS);
    for _ in 0..NUMBER_OF_ELEMENTS {
        let mut hll = H::create(P::EXPONENT);
        for value in iter_xorshift64(random_state) {
            hll.insert(value);
        }
        hll_vector.push(hll);
    }
    hll_vector
}

fn populate_hll_vectors_tuple<P: Precision, H: InsertValue>() -> (Vec<H>, Vec<H>) {
    let mut random_state = 534543539_u64;
    let left = populate_hll_vector::<P, H>(&mut random_state);
    let right = populate_hll_vector::<P, H>(&mut random_state);
    (left, right)
}

trait InsertValue {
    fn insert(&mut self, value: u64);

    fn create(precision: usize) -> Self;
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> InsertValue for HyperLogLog<P, B, R, Hasher>
where
    Self: HyperLogLogTrait<P, B, Hasher>,
{
    fn insert(&mut self, value: u64) {
        <Self as HyperLogLogTrait<P, B, Hasher>>::insert(self, &value);
    }

    fn create(precision: usize) -> Self {
        assert_eq!(precision, P::EXPONENT);
        HyperLogLog::default()
    }
}

impl<const ERROR: i32, P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> InsertValue
    for MLE<HyperLogLog<P, B, R, Hasher>, ERROR>
where
    Self: HyperLogLogTrait<P, B, Hasher>,
{
    fn insert(&mut self, value: u64) {
        <Self as HyperLogLogTrait<P, B, Hasher>>::insert(self, &value);
    }

    fn create(precision: usize) -> Self {
        assert_eq!(precision, P::EXPONENT);
        MLE::default()
    }
}

impl InsertValue for TabacHyperLogLogPF<u64, RandomState> {
    fn insert(&mut self, value: u64) {
        TabacHyperLogLog::insert(self, &value);
    }

    fn create(precision: usize) -> Self {
        TabacHyperLogLogPF::new(precision as u8, RandomState::new()).unwrap()
    }
}

impl InsertValue for TabacHyperLogLogPlus<u64, RandomState> {
    fn insert(&mut self, value: u64) {
        TabacHyperLogLog::insert(self, &value);
    }

    fn create(precision: usize) -> Self {
        TabacHyperLogLogPlus::new(precision as u8, RandomState::new()).unwrap()
    }
}

impl InsertValue for SAHyperLogLog<u64> {
    fn insert(&mut self, value: u64) {
        self.push(&value);
    }

    fn create(precision: usize) -> Self {
        let exponent = (precision as f64) / 2.0;
        let error_rate = 1.04 / 2f64.powf(exponent);
        SAHyperLogLog::new(error_rate)
    }
}

/// Macro to generate a criterion benchmark with the provided precision exponent and bits
macro_rules! bench_union {
    ($precision:ty, $bits:ty) => {
        paste::item! {
            fn [<bench_hll_union_ $precision:lower _ $bits:lower>] (b: &mut Criterion) {
                let (left, right) = populate_hll_vectors_tuple::<$precision, HyperLogLog<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, twox_hash::XxHash64>>();
                b.bench_function(
                    format!("hll_union_precision_{}_bits_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS).as_str(),
                    |b| {
                        b.iter(||{
                            let mut total_cardinality = 0.0;
                            for l in &left {
                                for r in &right {
                                    total_cardinality += black_box(l).estimate_union_cardinality::<f64>(black_box(r));
                                }
                            }
                            total_cardinality
                        })
                });
            }
        }
    };
}

/// Macro to generate a criterion benchmark with the provided precision exponent and bits
macro_rules! bench_union_mle {
    ($precision:ty, $bits:ty) => {
        paste::item! {
            fn [<bench_mle_union_ $precision:lower _ $bits:lower>] (b: &mut Criterion) {
                let (left, right) = populate_hll_vectors_tuple::<$precision, MLE<HyperLogLog<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, twox_hash::XxHash64>>>();
                b.bench_function(
                    format!("mle_union_precision_{}_bits_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS).as_str(),
                    |b| {
                        b.iter(||{
                            let mut total_cardinality = 0.0;
                            for l in &left {
                                for r in &right {
                                    total_cardinality += black_box(l).estimate_union_cardinality::<f64>(black_box(r));
                                }
                            }
                            total_cardinality
                        })
                });
            }
        }
    };
}

/// Macro to generate a criterion benchmark with the provided precision exponents
macro_rules! bench_unions {
    ($($precision:ty),*) => {
        $(
            // bench_union!($precision, Bits4);
            bench_union!($precision, Bits6);
            bench_union_mle!($precision, Bits6);
            // bench_union!($precision, Bits8);
            // bench_union_mle!($precision, Bits8);

            paste::item! {
                fn [<bench_tabacpf_union_ $precision:lower>] (b: &mut Criterion) {
                    let (left, right) = populate_hll_vectors_tuple::<$precision, TabacHyperLogLogPF<u64, RandomState>>();
                    b.bench_function(
                        format!("tabacpf_union_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut total_cardinality = 0.0;
                                for l in &left {
                                    for r in &right {
                                        let mut copy = black_box(l).clone();
                                        copy.merge(black_box(&r)).unwrap();
                                        total_cardinality += copy.count();
                                    }
                                }
                                total_cardinality
                            })
                    });
                }

                fn [<bench_tabacplusplus_union_ $precision:lower>] (b: &mut Criterion) {
                    let (left, right) = populate_hll_vectors_tuple::<$precision, TabacHyperLogLogPlus<u64, RandomState>>();
                    b.bench_function(
                        format!("tabacplusplus_union_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut total_cardinality = 0.0;
                                for l in &left {
                                    for r in &right {
                                        let mut copy = black_box(l).clone();
                                        copy.merge(black_box(&r)).unwrap();
                                        total_cardinality += copy.count();
                                    }
                                }
                             total_cardinality
                            })
                    });
                }

                fn [<bench_sa_union_ $precision:lower>] (b: &mut Criterion) {
                    let (left, right) = populate_hll_vectors_tuple::<$precision, SAHyperLogLog<u64>>();
                    b.bench_function(
                        format!("sa_union_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut total_cardinality = 0.0;
                                for l in &left {
                                    for r in &right {
                                        let mut copy = black_box(l).clone();
                                        copy.union(black_box(&r));
                                        total_cardinality += copy.len();
                                    }
                                }
                                total_cardinality
                            })
                    });
                }

                criterion_group! {
                    name=[<union_tabacpf_ $precision:lower>];
                    config = Criterion::default().sample_size(20);
                    targets=[<bench_tabacpf_union_ $precision:lower>]
                }
                criterion_group! {
                    name=[<union_tabacplusplus_ $precision:lower>];
                    config = Criterion::default().sample_size(20);
                    targets=[<bench_tabacplusplus_union_ $precision:lower>]
                }
                criterion_group! {
                    name=[<union_sa_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_sa_union_ $precision:lower>]
                }
                criterion_group! {
                    name=[<union_hll_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_hll_union_ $precision:lower _bits6>]
                }
                criterion_group! {
                    name=[<mle_union_ $precision:lower>];
                    config = Criterion::default().sample_size(5);
                    targets=[<bench_mle_union_ $precision:lower _bits6>]
                }
            }
        )*
    };
}

bench_unions!(
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
    union_hll_precision4,
    union_hll_precision5,
    union_hll_precision6,
    union_hll_precision7,
    union_hll_precision8,
    union_hll_precision9,
    union_hll_precision10,
    union_hll_precision11,
    union_hll_precision12,
    union_hll_precision13,
    union_hll_precision14,
    union_hll_precision15,
    union_hll_precision16,
    mle_union_precision4,
    mle_union_precision5,
    mle_union_precision6,
    mle_union_precision7,
    mle_union_precision8,
    mle_union_precision9,
    mle_union_precision10,
    mle_union_precision11,
    mle_union_precision12,
    mle_union_precision13,
    mle_union_precision14,
    mle_union_precision15,
    mle_union_precision16,
    union_tabacpf_precision4,
    union_tabacpf_precision5,
    union_tabacpf_precision6,
    union_tabacpf_precision7,
    union_tabacpf_precision8,
    union_tabacpf_precision9,
    union_tabacpf_precision10,
    union_tabacpf_precision11,
    union_tabacpf_precision12,
    union_tabacpf_precision13,
    union_tabacpf_precision14,
    union_tabacpf_precision15,
    union_tabacpf_precision16,
    union_tabacplusplus_precision4,
    union_tabacplusplus_precision5,
    union_tabacplusplus_precision6,
    union_tabacplusplus_precision7,
    union_tabacplusplus_precision8,
    union_tabacplusplus_precision9,
    union_tabacplusplus_precision10,
    union_tabacplusplus_precision11,
    union_tabacplusplus_precision12,
    union_tabacplusplus_precision13,
    union_tabacplusplus_precision14,
    union_tabacplusplus_precision15,
    union_tabacplusplus_precision16,
    union_sa_precision4,
    union_sa_precision5,
    union_sa_precision6,
    union_sa_precision7,
    union_sa_precision8,
    union_sa_precision9,
    union_sa_precision10,
    union_sa_precision11,
    union_sa_precision12,
    union_sa_precision13,
    union_sa_precision14,
    union_sa_precision15,
    union_sa_precision16
);
