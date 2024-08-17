#![feature(test)]
extern crate test;

use criterion::{criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;
use std::{collections::HashSet, hint::black_box};
use wyhash::WyHash;

mod utils;

use utils::*;

const RANDOM_STATE: u64 = 87561346897134_u64;
const NUMBER_OF_ELEMENTS: usize = 10_000;

fn insert_bencher<
    H: Estimator<f64> + hyperloglog_rs::prelude::Named + ExtendableApproximatedSet<u64>,
>(
    b: &mut Criterion,
) {
    b.bench_function(format!("Insert {}", H::default().name()).as_str(), |b| {
        b.iter(|| {
            let mut hll: H = Default::default();
            for i in iter_random_values(NUMBER_OF_ELEMENTS, None, RANDOM_STATE) {
                hll.insert(black_box(&i));
            }
        })
    });
}

macro_rules! bench_cardinality {
    ($precision:ty, $bits:ty, $hasher:ty) => {
        paste::item! {
            fn [<bench_plusplus_insert_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                insert_bencher::<PlusPlus<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>(b);
                insert_bencher::<PlusPlus<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister, $hasher>>(b);
                insert_bencher::<Hybrid<PlusPlus<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>>(b);
                insert_bencher::<Hybrid<PlusPlus<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister, $hasher>>>(b);
            }
        }
    };
}

macro_rules! bench_cludflare_cardinality {
    ($precision:ty, $bits:ty, $($hasher:ty),*) => {
        $(
            paste::item! {
                fn [<bench_cludflare_insert_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                    insert_bencher::<CloudFlareHLL<{$precision::EXPONENT as usize}, {$bits::NUMBER_OF_BITS as usize}, $hasher>>(b);
                }
            }
        )*
    };
}

type XxHash64 = twox_hash::XxHash64;

/// Macro to generate a criterion benchmark with the provided precision exponent and bits
macro_rules! bench_insert_bits {
    ($precision:ty, $($bits:ty),*) => {
        $(
            bench_cardinality!($precision, $bits, WyHash);
            bench_cardinality!($precision, $bits, XxHash64);
        )*
    };
}

/// Macro to generate a criterion benchmark with the provided precision exponents
macro_rules! bench_cardinalities {
    ($(($precision:ty, $sample_size:expr)),*) => {
        $(
            bench_cludflare_cardinality!($precision, Bits6, WyHash, XxHash64);
            bench_insert_bits!($precision, Bits6);
            bench_insert_bits!($precision, Bits8);

            paste::item! {
                fn [<bench_tabacpf_insert_ $precision:lower>] (b: &mut Criterion) {
                    insert_bencher::<TabacHLL<$precision>>(b);
                }

                fn [<bench_tabacplusplus_insert_ $precision:lower>] (b: &mut Criterion) {
                    insert_bencher::<TabacHLLPlusPlus<$precision>>(b);
                }

                fn [<bench_rhll_insert_ $precision:lower _bits6>] (b: &mut Criterion) {
                    insert_bencher::<RustHLL<$precision>>(b);
                }

                fn [<bench_shll_insert_ $precision:lower _bits6>] (b: &mut Criterion) {
                    insert_bencher::<SimpleHLL<{$precision::EXPONENT as usize}>>(b);
                }

                fn [<bench_sm_insert_ $precision:lower _bits6>] (b: &mut Criterion) {
                    insert_bencher::<SourMash<$precision>>(b);
                }

                fn [<bench_sa_insert_ $precision:lower>] (b: &mut Criterion) {
                    insert_bencher::<SAHLL<$precision>>(b);
                }

                criterion_group! {
                    name=[<insert_tabacpf_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size / 5).warm_up_time(std::time::Duration::from_secs(1)).measurement_time(std::time::Duration::from_secs(3));
                    targets=[<bench_tabacpf_insert_ $precision:lower>]
                }
                criterion_group! {
                    name=[<insert_tabacplusplus_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size / 5).warm_up_time(std::time::Duration::from_secs(1)).measurement_time(std::time::Duration::from_secs(3));
                    targets=[<bench_tabacplusplus_insert_ $precision:lower>]
                }
                criterion_group! {
                    name=[<insert_sa_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size).warm_up_time(std::time::Duration::from_secs(1)).measurement_time(std::time::Duration::from_secs(3));
                    targets=[<bench_sa_insert_ $precision:lower>]
                }
                criterion_group! {
                    name=[<insert_cludflare_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size).warm_up_time(std::time::Duration::from_secs(1)).measurement_time(std::time::Duration::from_secs(3));
                    targets=[<bench_cludflare_insert_ $precision:lower _bits6_wyhash>], [<bench_cludflare_insert_ $precision:lower _bits6_xxhash64>]
                }
                criterion_group! {
                    name=[<insert_rhll_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size / 5).warm_up_time(std::time::Duration::from_secs(1)).measurement_time(std::time::Duration::from_secs(3));
                    targets=[<bench_rhll_insert_ $precision:lower _bits6>]
                }
                criterion_group! {
                    name=[<insert_shll_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size / 5).warm_up_time(std::time::Duration::from_secs(1)).measurement_time(std::time::Duration::from_secs(3));
                    targets=[<bench_shll_insert_ $precision:lower _bits6>]
                }
                criterion_group! {
                    name=[<insert_sm_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size / 5).warm_up_time(std::time::Duration::from_secs(1)).measurement_time(std::time::Duration::from_secs(3));
                    targets=[<bench_sm_insert_ $precision:lower _bits6>]
                }

                criterion_group! {
                    name=[<insert_plusplus _ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size).warm_up_time(std::time::Duration::from_secs(1)).measurement_time(std::time::Duration::from_secs(3));
                    targets=[<bench_plusplus _insert_ $precision:lower _bits6_xxhash64>], [<bench_plusplus _insert_ $precision:lower _bits6_wyhash>], [<bench_plusplus _insert_ $precision:lower _bits8_xxhash64>], [<bench_plusplus _insert_ $precision:lower _bits8_wyhash>],
                }
            }
        )*
    };
}

macro_rules! bench_hyper_two_bits {
    ($($sketch:ty),*) => {
        $(
            paste::paste!{
                fn [<bench_hypertwobits_ $sketch:lower  _insert>](b: &mut Criterion) {
                    insert_bencher::<HyperTwoBits<$sketch>>(b);
                }
            }
        )*
    };
}

macro_rules! bench_hyper_three_bits {
    ($($sketch:ty),*) => {
        $(
            paste::paste!{
                fn [<bench_hyperthreebits_ $sketch:lower _insert>](b: &mut Criterion) {
                    insert_bencher::<HyperThreeBits<$sketch>>(b);
                }
            }
        )*
    };
}

use hypertwobits::h2b::{
    M1024 as M1024H2B, M128 as M128H2B, M2048 as M2048H2B, M256 as M256H2B, M4096 as M4096H2B,
    M512 as M512H2B, M64 as M64H2B,
};
bench_hyper_two_bits!(M64H2B, M128H2B, M256H2B, M512H2B, M1024H2B, M2048H2B, M4096H2B);

use hypertwobits::h3b::{
    M1024 as M1024H3B, M128 as M128H3B, M2048 as M2048H3B, M256 as M256H3B, M4096 as M4096H3B,
    M512 as M512H3B, M64 as M64H3B,
};
bench_hyper_three_bits!(M64H3B, M128H3B, M256H3B, M512H3B, M1024H3B, M2048H3B, M4096H3B);

fn bench_hashset_insert(b: &mut Criterion) {
    insert_bencher::<HashSet<u64>>(b);
}

criterion_group!(
    name = insert_hyper_two_bits;
    config = Criterion::default().sample_size(100).warm_up_time(std::time::Duration::from_secs(1)).measurement_time(std::time::Duration::from_secs(3));
    targets = bench_hypertwobits_m64h2b_insert, bench_hypertwobits_m128h2b_insert, bench_hypertwobits_m256h2b_insert, bench_hypertwobits_m512h2b_insert, bench_hypertwobits_m1024h2b_insert, bench_hypertwobits_m2048h2b_insert, bench_hypertwobits_m4096h2b_insert
);

criterion_group!(
    name = insert_hyper_three_bits;
    config = Criterion::default().sample_size(100).warm_up_time(std::time::Duration::from_secs(1)).measurement_time(std::time::Duration::from_secs(3));
    targets = bench_hyperthreebits_m64h3b_insert, bench_hyperthreebits_m128h3b_insert, bench_hyperthreebits_m256h3b_insert, bench_hyperthreebits_m512h3b_insert, bench_hyperthreebits_m1024h3b_insert, bench_hyperthreebits_m2048h3b_insert, bench_hyperthreebits_m4096h3b_insert
);

criterion_group!(
    name = insert_hashset;
    config = Criterion::default().sample_size(100).warm_up_time(std::time::Duration::from_secs(1)).measurement_time(std::time::Duration::from_secs(3));
    targets = bench_hashset_insert
);

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

/// Macro to generate the criterion main for all precisions
macro_rules! bench_insert_main {
    ($($precision:ty),*) => {
        paste::paste!{
            criterion_main!(
                $(
                    [<insert_plusplus_ $precision:lower>],
                    [<insert_tabacpf_ $precision:lower>],
                    [<insert_tabacplusplus_ $precision:lower>],
                    [<insert_sa_ $precision:lower>],
                    [<insert_cludflare_ $precision:lower>],
                    [<insert_rhll_ $precision:lower>],
                    [<insert_sm_ $precision:lower>],
                    [<insert_shll_ $precision:lower>],
                )*
                insert_hyper_two_bits,
                insert_hyper_three_bits,
                insert_hashset
            );
        }
    };
}

bench_insert_main!(
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
    Precision16,
    Precision17,
    Precision18
);
