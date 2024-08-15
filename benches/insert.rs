#![feature(test)]
extern crate test;

use criterion::{criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;
use std::hint::black_box;
use wyhash::WyHash;

mod utils;

use utils::*;

const RANDOM_STATE: u64 = 87561346897134_u64;
const NUMBER_OF_ELEMENTS: usize = 10_000;

fn insert_bencher<
    H: Estimator<f64> + ExtendableApproximatedSet<u64>,
>(
    b: &mut Criterion,
) {
    b.bench_function(
        format!("Insert {}", H::default().name()).as_str(),
        |b| {
            b.iter(||{
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
            fn [<bench_plusplusvec_insert_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                insert_bencher::<PlusPlus<$precision, $bits, Vec<u64>, $hasher>>(b);
            }

            fn [<bench_plusplusarray_insert_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                insert_bencher::<PlusPlus<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>(b);
            }

            fn [<bench_pluspluspackedarray_insert_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                insert_bencher::<PlusPlus<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister, $hasher>>(b);
            }

            fn [<bench_hybridplusplusvec_insert_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                insert_bencher::<Hybrid<PlusPlus<$precision, $bits, Vec<u64>, $hasher>>>(b);
            }

            fn [<bench_hybridplusplusarray_insert_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                insert_bencher::<Hybrid<PlusPlus<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>>(b);
            }

            fn [<bench_hybridpluspluspackedarray_insert_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
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
                    insert_bencher::<CloudFlareHLL<{$precision::EXPONENT}, {$bits::NUMBER_OF_BITS}, $hasher>>(b);
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

/// Macro to generate criterion groups.
macro_rules! bench_insert_registers {
    ($precision:ty, $sample_size:expr, $($register:expr),*) => {
        $(
            paste::paste! {
                criterion_group! {
                    name=[<insert_plusplus $register _ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_plusplus $register _insert_ $precision:lower _bits6_xxhash64>], [<bench_plusplus $register _insert_ $precision:lower _bits6_wyhash>], [<bench_plusplus $register _insert_ $precision:lower _bits8_xxhash64>], [<bench_plusplus $register _insert_ $precision:lower _bits8_wyhash>],
                }
                criterion_group! {
                    name=[<insert_hybridplusplus $register _ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_hybridplusplus $register _insert_ $precision:lower _bits6_xxhash64>], [<bench_hybridplusplus $register _insert_ $precision:lower _bits6_wyhash>], [<bench_hybridplusplus $register _insert_ $precision:lower _bits8_xxhash64>], [<bench_hybridplusplus $register _insert_ $precision:lower _bits8_wyhash>]
                }
            }
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

                fn [<bench_sa_insert_ $precision:lower>] (b: &mut Criterion) {
                    insert_bencher::<SAHLL<$precision>>(b);
                }

                criterion_group! {
                    name=[<insert_tabacpf_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size / 5);
                    targets=[<bench_tabacpf_insert_ $precision:lower>]
                }
                criterion_group! {
                    name=[<insert_tabacplusplus_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size / 5);
                    targets=[<bench_tabacplusplus_insert_ $precision:lower>]
                }
                criterion_group! {
                    name=[<insert_sa_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_sa_insert_ $precision:lower>]
                }
                criterion_group! {
                    name=[<insert_cludflare_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_cludflare_insert_ $precision:lower _bits6_wyhash>], [<bench_cludflare_insert_ $precision:lower _bits6_xxhash64>]
                }
                criterion_group! {
                    name=[<insert_rhll_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size / 5);
                    targets=[<bench_rhll_insert_ $precision:lower _bits6>]
                }

                bench_insert_registers!($precision, $sample_size, "vec", "array", "packedarray");
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

/// Macro to generate the criterion main for all precisions
macro_rules! bench_insert_main {
    ($($precision:ty),*) => {
        paste::paste!{
            criterion_main!(
                $(
                    [<insert_plusplusvec_ $precision:lower>],
                    [<insert_plusplusarray_ $precision:lower>],
                    [<insert_pluspluspackedarray_ $precision:lower>],
                    [<insert_hybridplusplusvec_ $precision:lower>],
                    [<insert_hybridplusplusarray_ $precision:lower>],
                    [<insert_hybridpluspluspackedarray_ $precision:lower>],
                    [<insert_tabacpf_ $precision:lower>],
                    [<insert_tabacplusplus_ $precision:lower>],
                    [<insert_sa_ $precision:lower>],
                    [<insert_cludflare_ $precision:lower>],
                    [<insert_rhll_ $precision:lower>],
                )*
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