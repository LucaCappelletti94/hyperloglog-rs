#![feature(test)]
extern crate test;

use criterion::{criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;
use std::hint::black_box;
use wyhash::WyHash;

mod utils;

use utils::*;

const RANDOM_STATE: u64 = 87561346897134_u64;
const NUMBER_OF_COUNTERS: usize = 1_000;
const NUMBER_OF_ELEMENTS: usize = 50_000;

fn cardinality_bencher<
    H: Estimator<f64> + ExtendableApproximatedSet<u64>,
>(
    b: &mut Criterion,
) {
    let mut random_state = splitmix64(RANDOM_STATE);
    let counters: Vec<H> = (0..NUMBER_OF_COUNTERS).map(|_|{
        let mut counter = H::default();
        random_state = splitmix64(random_state);
        for value in iter_random_values(NUMBER_OF_ELEMENTS, None, random_state) {
            counter.insert(&value);
        }
        counter
    }).collect();

    b.bench_function(
        format!("Cardinality {}", H::default().name()).as_str(),
        |b| {
            b.iter(||{
                let mut total_cardinality = 0.0_f64;
                for counter in counters.iter() {
                    total_cardinality += black_box(counter).estimate_cardinality();
                }
                total_cardinality
            })
    });
}

macro_rules! bench_cardinality {
    ($precision:ty, $bits:ty, $hasher:ty) => {
        paste::item! {
            fn [<bench_plusplusvec_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                cardinality_bencher::<PlusPlus<$precision, $bits, Vec<u64>, $hasher>>(b);
            }

            fn [<bench_plusplusarray_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                cardinality_bencher::<PlusPlus<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>(b);
            }

            fn [<bench_pluspluspackedarray_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                cardinality_bencher::<PlusPlus<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister, $hasher>>(b);
            }

            fn [<bench_betavec_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                cardinality_bencher::<LogLogBeta<$precision, $bits, Vec<u64>, $hasher>>(b);
            }

            fn [<bench_betaarray_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                cardinality_bencher::<LogLogBeta<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>(b);
            }

            fn [<bench_betapackedarray_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                cardinality_bencher::<LogLogBeta<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister, $hasher>>(b);
            }

            fn [<bench_hybridplusplusvec_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                cardinality_bencher::<Hybrid<PlusPlus<$precision, $bits, Vec<u64>, $hasher>>>(b);
            }

            fn [<bench_hybridplusplusarray_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                cardinality_bencher::<Hybrid<PlusPlus<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>>(b);
            }

            fn [<bench_hybridpluspluspackedarray_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                cardinality_bencher::<Hybrid<PlusPlus<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister, $hasher>>>(b);
            }

            fn [<bench_hybridbetavec_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                cardinality_bencher::<Hybrid<LogLogBeta<$precision, $bits, Vec<u64>, $hasher>>>(b);
            }

            fn [<bench_hybridbetaarray_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                cardinality_bencher::<Hybrid<LogLogBeta<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>>(b);
            }

            fn [<bench_hybridbetapackedarray_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                cardinality_bencher::<Hybrid<LogLogBeta<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister, $hasher>>>(b);
            }
        }
    };
}

macro_rules! bench_cludflare_cardinality {
    ($precision:ty, $bits:ty, $($hasher:ty),*) => {
        $(
            paste::item! {
                fn [<bench_cludflare_cardinality_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                    cardinality_bencher::<CloudFlareHLL<{$precision::EXPONENT}, {$bits::NUMBER_OF_BITS}, $hasher>>(b);
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
            bench_cardinality!($precision, $bits, WyHash);
            bench_cardinality!($precision, $bits, XxHash64);
        )*
    };
}

/// Macro to generate criterion groups.
macro_rules! bench_cardinality_registers {
    ($precision:ty, $sample_size:expr, $($register:expr),*) => {
        $(
            paste::paste! {
                criterion_group! {
                    name=[<cardinality_plusplus $register _ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_plusplus $register _cardinality_ $precision:lower _bits6_xxhash64>], [<bench_plusplus $register _cardinality_ $precision:lower _bits6_wyhash>], [<bench_plusplus $register _cardinality_ $precision:lower _bits8_xxhash64>], [<bench_plusplus $register _cardinality_ $precision:lower _bits8_wyhash>],
                }
                criterion_group! {
                    name=[<cardinality_hybridplusplus $register _ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_hybridplusplus $register _cardinality_ $precision:lower _bits6_xxhash64>], [<bench_hybridplusplus $register _cardinality_ $precision:lower _bits6_wyhash>], [<bench_hybridplusplus $register _cardinality_ $precision:lower _bits8_xxhash64>], [<bench_hybridplusplus $register _cardinality_ $precision:lower _bits8_wyhash>]
                }
                criterion_group! {
                    name=[<cardinality_beta $register _ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_beta $register _cardinality_ $precision:lower _bits6_xxhash64>], [<bench_beta $register _cardinality_ $precision:lower _bits6_wyhash>], [<bench_beta $register _cardinality_ $precision:lower _bits8_xxhash64>], [<bench_beta $register _cardinality_ $precision:lower _bits8_wyhash>]
                }
                criterion_group! {
                    name=[<cardinality_hybridbeta $register _ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_hybridbeta $register _cardinality_ $precision:lower _bits6_xxhash64>], [<bench_hybridbeta $register _cardinality_ $precision:lower _bits6_wyhash>], [<bench_hybridbeta $register _cardinality_ $precision:lower _bits8_xxhash64>], [<bench_hybridbeta $register _cardinality_ $precision:lower _bits8_wyhash>]
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
            bench_cardinality_bits!($precision, Bits6);
            bench_cardinality_bits!($precision, Bits8);

            paste::item! {
                fn [<bench_tabacpf_cardinality_ $precision:lower>] (b: &mut Criterion) {
                    cardinality_bencher::<TabacHLL<$precision>>(b);
                }

                fn [<bench_tabacplusplus_cardinality_ $precision:lower>] (b: &mut Criterion) {
                    cardinality_bencher::<TabacHLLPlusPlus<$precision>>(b);
                }

                fn [<bench_rhll_cardinality_ $precision:lower _bits6>] (b: &mut Criterion) {
                    cardinality_bencher::<RustHLL<$precision>>(b);
                }

                fn [<bench_sa_cardinality_ $precision:lower>] (b: &mut Criterion) {
                    cardinality_bencher::<SAHLL<$precision>>(b);
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
                    name=[<cardinality_cludflare_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_cludflare_cardinality_ $precision:lower _bits6_wyhash>], [<bench_cludflare_cardinality_ $precision:lower _bits6_xxhash64>]
                }
                criterion_group! {
                    name=[<cardinality_rhll_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size / 5);
                    targets=[<bench_rhll_cardinality_ $precision:lower _bits6>]
                }

                bench_cardinality_registers!($precision, $sample_size, "vec", "array", "packedarray");
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
macro_rules! bench_cardinality_main {
    ($($precision:ty),*) => {
        paste::paste!{
            criterion_main!(
                $(
                    [<cardinality_betavec_ $precision:lower>],
                    [<cardinality_betaarray_ $precision:lower>],
                    [<cardinality_betapackedarray_ $precision:lower>],
                    [<cardinality_plusplusvec_ $precision:lower>],
                    [<cardinality_plusplusarray_ $precision:lower>],
                    [<cardinality_pluspluspackedarray_ $precision:lower>],
                    [<cardinality_hybridbetavec_ $precision:lower>],
                    [<cardinality_hybridbetaarray_ $precision:lower>],
                    [<cardinality_hybridbetapackedarray_ $precision:lower>],
                    [<cardinality_hybridplusplusvec_ $precision:lower>],
                    [<cardinality_hybridplusplusarray_ $precision:lower>],
                    [<cardinality_hybridpluspluspackedarray_ $precision:lower>],
                    [<cardinality_tabacpf_ $precision:lower>],
                    [<cardinality_tabacplusplus_ $precision:lower>],
                    [<cardinality_sa_ $precision:lower>],
                    [<cardinality_cludflare_ $precision:lower>],
                    [<cardinality_rhll_ $precision:lower>],
                )*
            );
        }
    };
}

bench_cardinality_main!(
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