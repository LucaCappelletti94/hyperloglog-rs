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

const MAXIMAL_ARRAY_SIZE: usize = 10_000;
const NUMBER_OF_ELEMENTS: usize = 30;
const LEFT_RANDOM_STATE: u64 = 87561346897134_u64;
const RIGHT_RANDOM_STATE: u64 = 2344357875478_u64;

fn populate_hll_vector<P: Precision, H: InsertValue>(mut random_state: u64) -> Vec<H> {
    let mut hll_vector = Vec::with_capacity(NUMBER_OF_ELEMENTS);
    for _ in 0..NUMBER_OF_ELEMENTS {
        let mut hll = H::create(P::EXPONENT);
        random_state = splitmix64(random_state);
        for value in iter_random_values(MAXIMAL_ARRAY_SIZE, None, random_state) {
            hll.insert(value);
        }
        hll_vector.push(hll);
    }
    hll_vector
}

fn populate_hll_vectors_tuple<P: Precision, H: InsertValue>() -> (Vec<H>, Vec<H>) {
    let left = populate_hll_vector::<P, H>(LEFT_RANDOM_STATE);
    let right = populate_hll_vector::<P, H>(RIGHT_RANDOM_STATE);
    (left, right)
}

trait InsertValue {
    fn insert(&mut self, value: u64);

    fn create(precision: usize) -> Self;
}

impl InsertValue for RustHyperLogLog {
    fn insert(&mut self, value: u64) {
        self.insert(&value);
    }

    fn create(precision: usize) -> Self {
        let exponent = (precision as f64) / 2.0;
        let error_rate = 1.04 / 2f64.powf(exponent);
        RustHyperLogLog::new_deterministic(error_rate, 857686755678_u128)
    }
}

impl<const P: usize, const B: usize, Hasher: HasherType> InsertValue
    for CardinalityEstimator<u64, Hasher, P, B>
{
    fn insert(&mut self, value: u64) {
        self.insert(&value);
    }

    fn create(precision: usize) -> Self {
        assert_eq!(precision, P);
        CardinalityEstimator::new()
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> InsertValue
    for PlusPlus<P, B, R, Hasher>
where
    Self: HyperLogLog<P, B, Hasher>,
{
    fn insert(&mut self, value: u64) {
        <Self as HyperLogLog<P, B, Hasher>>::insert(self, &value);
    }

    fn create(precision: usize) -> Self {
        assert_eq!(precision, P::EXPONENT);
        Default::default()
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> InsertValue
    for LogLogBeta<P, B, R, Hasher>
where
    Self: HyperLogLog<P, B, Hasher>,
{
    fn insert(&mut self, value: u64) {
        <Self as HyperLogLog<P, B, Hasher>>::insert(self, &value);
    }

    fn create(precision: usize) -> Self {
        assert_eq!(precision, P::EXPONENT);
        Default::default()
    }
}

impl<const ERROR: i32, P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> InsertValue
    for MLE<PlusPlus<P, B, R, Hasher>, ERROR>
where
    Self: HyperLogLog<P, B, Hasher>,
{
    fn insert(&mut self, value: u64) {
        <Self as HyperLogLog<P, B, Hasher>>::insert(self, &value);
    }

    fn create(precision: usize) -> Self {
        assert_eq!(precision, P::EXPONENT);
        MLE::default()
    }
}

impl<const ERROR: i32, P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> InsertValue
    for MLE<LogLogBeta<P, B, R, Hasher>, ERROR>
where
    Self: HyperLogLog<P, B, Hasher>,
{
    fn insert(&mut self, value: u64) {
        <Self as HyperLogLog<P, B, Hasher>>::insert(self, &value);
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
            fn [<bench_plusplus_union_ $precision:lower _ $bits:lower>] (b: &mut Criterion) {
                let (left, right) = populate_hll_vectors_tuple::<$precision, PlusPlus<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, wyhash::WyHash>>();
                b.bench_function(
                    format!("plusplus_union_precision_{}_bits_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS).as_str(),
                    |b| {
                        b.iter(||{
                            let mut total_cardinality = 0.0;
                            for l in &left {
                                for r in &right {
                                    let union: f64 = black_box(l).estimate_union_cardinality(black_box(r));
                                    total_cardinality += union;
                                }
                            }
                            total_cardinality
                        })
                });
            }
            fn [<bench_beta_union_ $precision:lower _ $bits:lower>] (b: &mut Criterion) {
                let (left, right) = populate_hll_vectors_tuple::<$precision, LogLogBeta<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, wyhash::WyHash>>();
                b.bench_function(
                    format!("beta_union_precision_{}_bits_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS).as_str(),
                    |b| {
                        b.iter(||{
                            let mut total_cardinality = 0.0;
                            for l in &left {
                                for r in &right {
                                    let union: f64 = black_box(l).estimate_union_cardinality(black_box(r));
                                    total_cardinality += union;
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
            fn [<bench_mleplusplus_union_ $precision:lower _ $bits:lower>] (b: &mut Criterion) {
                let (left, right) = populate_hll_vectors_tuple::<$precision, MLE<PlusPlus<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, twox_hash::XxHash64>>>();
                b.bench_function(
                    format!("mlepp_union_precision_{}_bits_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS).as_str(),
                    |b| {
                        b.iter(||{
                            let mut total_cardinality = 0.0;
                            for l in &left {
                                for r in &right {
                                    let union: f64 = black_box(l).estimate_union_cardinality(black_box(r));
                                    total_cardinality += union;
                                }
                            }
                            total_cardinality
                        })
                });
            }

            fn [<bench_mlebeta_union_ $precision:lower _ $bits:lower>] (b: &mut Criterion) {
                let (left, right) = populate_hll_vectors_tuple::<$precision, MLE<LogLogBeta<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, twox_hash::XxHash64>>>();
                b.bench_function(
                    format!("mlebeta_union_precision_{}_bits_{}", $precision::EXPONENT, $bits::NUMBER_OF_BITS).as_str(),
                    |b| {
                        b.iter(||{
                            let mut total_cardinality = 0.0;
                            for l in &left {
                                for r in &right {
                                    let union: f64 = black_box(l).estimate_union_cardinality(black_box(r));
                                    total_cardinality += union;
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
            bench_union!($precision, Bits8);
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

                fn [<bench_ce_union_ $precision:lower _bits6>] (b: &mut Criterion) {
                    let (left, right) = populate_hll_vectors_tuple::<$precision, CardinalityEstimator<u64, twox_hash::XxHash64, {$precision::EXPONENT}, 6>>();
                    b.bench_function(
                        format!("ce_union_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut total_cardinality = 0.0;
                                for l in &left {
                                    for r in &right {
                                        let mut copy = black_box(l).clone();
                                        black_box(&mut copy).merge(black_box(&r));
                                        total_cardinality += black_box(copy).estimate() as f64;
                                    }
                                }
                                total_cardinality
                            })
                    });
                }

                fn [<bench_rhll_union_ $precision:lower>] (b: &mut Criterion) {
                    let (left, right) = populate_hll_vectors_tuple::<$precision, RustHyperLogLog>();
                    b.bench_function(
                        format!("rhll_union_precision_{}_bits_6", $precision::EXPONENT).as_str(),
                        |b| {
                            b.iter(||{
                                let mut total_cardinality = 0.0;
                                for l in &left {
                                    for r in &right {
                                        let mut copy = black_box(l).clone();
                                        black_box(&mut copy).merge(black_box(&r));
                                        total_cardinality += black_box(copy).len() as f64;
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
                    name=[<union_plusplus_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_plusplus_union_ $precision:lower _bits6>], [<bench_plusplus_union_ $precision:lower _bits8>]
                }
                criterion_group! {
                    name=[<union_beta_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_beta_union_ $precision:lower _bits6>], [<bench_beta_union_ $precision:lower _bits8>]
                }
                criterion_group! {
                    name=[<union_mleplusplus_ $precision:lower>];
                    config = Criterion::default().sample_size(10);
                    targets=[<bench_mleplusplus_union_ $precision:lower _bits6>]
                }
                criterion_group! {
                    name=[<union_mlebeta_ $precision:lower>];
                    config = Criterion::default().sample_size(10);
                    targets=[<bench_mlebeta_union_ $precision:lower _bits6>]
                }
                criterion_group! {
                    name=[<union_rhll_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_rhll_union_ $precision:lower>]
                }
                criterion_group! {
                    name=[<union_ce_ $precision:lower>];
                    config = Criterion::default();
                    targets=[<bench_ce_union_ $precision:lower _bits6>]
                }
            }
        )*
    };
}

#[cfg(feature = "low_precisions")]
bench_unions!(
    Precision4,
    Precision5,
    Precision6,
    Precision7,
    Precision8,
    Precision9,
    Precision10
);

#[cfg(feature = "medium_precisions")]
bench_unions!(
    Precision11,
    Precision12,
    Precision13,
    Precision14,
    Precision15,
    Precision16
);

#[cfg(feature = "high_precisions")]
bench_unions!(Precision17, Precision18);

criterion_main!(
    union_plusplus_precision4,
    union_plusplus_precision5,
    union_plusplus_precision6,
    union_plusplus_precision7,
    union_plusplus_precision8,
    union_plusplus_precision9,
    union_plusplus_precision10,
    union_plusplus_precision11,
    union_plusplus_precision12,
    union_plusplus_precision13,
    union_plusplus_precision14,
    union_plusplus_precision15,
    union_plusplus_precision16,
    union_plusplus_precision17,
    union_plusplus_precision18,
    union_beta_precision4,
    union_beta_precision5,
    union_beta_precision6,
    union_beta_precision7,
    union_beta_precision8,
    union_beta_precision9,
    union_beta_precision10,
    union_beta_precision11,
    union_beta_precision12,
    union_beta_precision13,
    union_beta_precision14,
    union_beta_precision15,
    union_beta_precision16,
    union_beta_precision17,
    union_beta_precision18,
    union_mleplusplus_precision4,
    union_mleplusplus_precision5,
    union_mleplusplus_precision6,
    union_mleplusplus_precision7,
    union_mleplusplus_precision8,
    union_mleplusplus_precision9,
    union_mleplusplus_precision10,
    union_mleplusplus_precision11,
    union_mleplusplus_precision12,
    union_mleplusplus_precision13,
    union_mleplusplus_precision14,
    union_mleplusplus_precision15,
    union_mleplusplus_precision16,
    union_mleplusplus_precision17,
    union_mleplusplus_precision18,
    union_beta_precision4,
    union_beta_precision5,
    union_beta_precision6,
    union_beta_precision7,
    union_beta_precision8,
    union_beta_precision9,
    union_beta_precision10,
    union_beta_precision11,
    union_beta_precision12,
    union_beta_precision13,
    union_beta_precision14,
    union_beta_precision15,
    union_beta_precision16,
    union_beta_precision17,
    union_beta_precision18,
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
    union_tabacpf_precision17,
    union_tabacpf_precision18,
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
    union_tabacplusplus_precision17,
    union_tabacplusplus_precision18,
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
    union_sa_precision16,
    union_sa_precision17,
    union_sa_precision18,
    union_ce_precision4,
    union_ce_precision5,
    union_ce_precision6,
    union_ce_precision7,
    union_ce_precision8,
    union_ce_precision9,
    union_ce_precision10,
    union_ce_precision11,
    union_ce_precision12,
    union_ce_precision13,
    union_ce_precision14,
    union_ce_precision15,
    union_ce_precision16,
    union_ce_precision17,
    union_ce_precision18,
    union_rhll_precision4,
    union_rhll_precision5,
    union_rhll_precision6,
    union_rhll_precision7,
    union_rhll_precision8,
    union_rhll_precision9,
    union_rhll_precision10,
    union_rhll_precision11,
    union_rhll_precision12,
    union_rhll_precision13,
    union_rhll_precision14,
    union_rhll_precision15,
    union_rhll_precision16,
    union_rhll_precision17,
    union_rhll_precision18,
);
