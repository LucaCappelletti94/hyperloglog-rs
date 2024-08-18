#![feature(test)]
extern crate test;

use criterion::{criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;
use std::collections::HashSet;
use std::hint::black_box;
use wyhash::WyHash;

mod utils;

use utils::*;

const RANDOM_STATE: u64 = 87561346897134_u64;
const NUMBER_OF_ELEMENTS: usize = 50_000;
const NUMBER_OF_COUNTERS: usize = 2_000;

fn union_bencher<
    H: Estimator<f64> + hyperloglog_rs::prelude::Named + ExtendableApproximatedSet<u64>,
>(
    b: &mut Criterion,
) {
    let mut random_state = splitmix64(RANDOM_STATE);
    let counters: Vec<(H, H)> = (0..NUMBER_OF_COUNTERS)
        .map(|_| {
            let mut left = H::default();
            let mut right = H::default();
            random_state = splitmix64(random_state);
            for value in iter_random_values(NUMBER_OF_ELEMENTS, None, random_state) {
                if value % 2 == 0 {
                    left.insert(&value);
                }

                right.insert(&value);
            }
            (left, right)
        })
        .collect();

    b.bench_function(format!("Union {}", H::default().name()).as_str(), |b| {
        b.iter(|| {
            let mut total_union = 0.0_f64;
            for (left, right) in counters.iter() {
                total_union += black_box(left).estimate_union_cardinality(black_box(right));
            }
            total_union
        })
    });
}

macro_rules! bench_union {
    ($precision:ty, $bits:ty, $hasher:ty) => {
        paste::item! {

            fn [<bench_plusplusarray_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                union_bencher::<PlusPlus<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>(b);
            }

            fn [<bench_pluspluspackedarray_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                union_bencher::<PlusPlus<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister, $hasher>>(b);
            }


            fn [<bench_betaarray_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                union_bencher::<LogLogBeta<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>(b);
            }

            fn [<bench_betapackedarray_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                union_bencher::<LogLogBeta<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister, $hasher>>(b);
            }


            fn [<bench_hybridplusplusarray_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                union_bencher::<Hybrid<PlusPlus<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>>(b);
            }

            fn [<bench_hybridpluspluspackedarray_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                union_bencher::<Hybrid<PlusPlus<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister, $hasher>>>(b);
            }


            fn [<bench_hybridbetaarray_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                union_bencher::<Hybrid<LogLogBeta<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>>(b);
            }

            fn [<bench_hybridbetapackedarray_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                union_bencher::<Hybrid<LogLogBeta<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister, $hasher>>>(b);
            }


            fn [<bench_mleplusplusarray_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                union_bencher::<MLE<PlusPlus<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>>(b);
            }

            fn [<bench_mlepluspluspackedarray_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                union_bencher::<MLE<PlusPlus<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister, $hasher>>>(b);
            }


            fn [<bench_mlebetaarray_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                union_bencher::<MLE<LogLogBeta<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>>(b);
            }

            fn [<bench_mlebetapackedarray_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                union_bencher::<MLE<LogLogBeta<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister, $hasher>>>(b);
            }


            fn [<bench_mlehybridplusplusarray_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                union_bencher::<Hybrid<MLE<PlusPlus<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>>>(b);
            }

            fn [<bench_mlehybridpluspluspackedarray_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                union_bencher::<Hybrid<MLE<PlusPlus<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister, $hasher>>>>(b);
            }


            fn [<bench_mlehybridbetaarray_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                union_bencher::<Hybrid<MLE<LogLogBeta<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>>>>(b);
            }

            fn [<bench_mlehybridbetapackedarray_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                union_bencher::<Hybrid<MLE<LogLogBeta<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister, $hasher>>>>(b);
            }
        }
    };
}

macro_rules! bench_cludflare_union {
    ($precision:ty, $bits:ty, $($hasher:ty),*) => {
        $(
            paste::item! {
                fn [<bench_cludflare_union_ $precision:lower _ $bits:lower _ $hasher:lower>] (b: &mut Criterion) {
                    union_bencher::<CloudFlareHLL<{$precision::EXPONENT as usize}, {$bits::NUMBER_OF_BITS as usize}, $hasher>>(b);
                }
            }
        )*
    };
}

type XxHash64 = twox_hash::XxHash64;

/// Macro to generate a criterion benchmark with the provided precision exponent and bits
macro_rules! bench_union_bits {
    ($precision:ty, $($bits:ty),*) => {
        $(
            bench_union!($precision, $bits, WyHash);
            bench_union!($precision, $bits, XxHash64);
        )*
    };
}

/// Macro to generate criterion groups.
macro_rules! bench_union_registers {
    ($precision:ty, $sample_size:expr, $($register:expr),*) => {
        $(
            paste::paste! {
                criterion_group! {
                    name=[<union_plusplus $register _ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size).warm_up_time(std::time::Duration::from_secs(1)).warm_up_time(std::time::Duration::from_secs(1));
                    targets=[<bench_plusplus $register _union_ $precision:lower _bits6_xxhash64>], [<bench_plusplus $register _union_ $precision:lower _bits6_wyhash>], [<bench_plusplus $register _union_ $precision:lower _bits8_xxhash64>], [<bench_plusplus $register _union_ $precision:lower _bits8_wyhash>],
                }
                criterion_group! {
                    name=[<union_hybridplusplus $register _ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size).warm_up_time(std::time::Duration::from_secs(1)).warm_up_time(std::time::Duration::from_secs(1));
                    targets=[<bench_hybridplusplus $register _union_ $precision:lower _bits6_xxhash64>], [<bench_hybridplusplus $register _union_ $precision:lower _bits6_wyhash>], [<bench_hybridplusplus $register _union_ $precision:lower _bits8_xxhash64>], [<bench_hybridplusplus $register _union_ $precision:lower _bits8_wyhash>]
                }
                criterion_group! {
                    name=[<union_beta $register _ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size).warm_up_time(std::time::Duration::from_secs(1)).warm_up_time(std::time::Duration::from_secs(1));
                    targets=[<bench_beta $register _union_ $precision:lower _bits6_xxhash64>], [<bench_beta $register _union_ $precision:lower _bits6_wyhash>], [<bench_beta $register _union_ $precision:lower _bits8_xxhash64>], [<bench_beta $register _union_ $precision:lower _bits8_wyhash>]
                }
                criterion_group! {
                    name=[<union_hybridbeta $register _ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size).warm_up_time(std::time::Duration::from_secs(1)).warm_up_time(std::time::Duration::from_secs(1));
                    targets=[<bench_hybridbeta $register _union_ $precision:lower _bits6_xxhash64>], [<bench_hybridbeta $register _union_ $precision:lower _bits6_wyhash>], [<bench_hybridbeta $register _union_ $precision:lower _bits8_xxhash64>], [<bench_hybridbeta $register _union_ $precision:lower _bits8_wyhash>]
                }
                criterion_group! {
                    name=[<union_ mleplusplus $register _ $precision:lower>];
                    config = Criterion::default().sample_size(10).warm_up_time(std::time::Duration::from_secs(1)).warm_up_time(std::time::Duration::from_secs(1));
                    targets=[<bench_mleplusplus $register _union_ $precision:lower _bits6_xxhash64>], [<bench_mleplusplus $register _union_ $precision:lower _bits6_wyhash>], [<bench_mleplusplus $register _union_ $precision:lower _bits8_xxhash64>], [<bench_mleplusplus $register _union_ $precision:lower _bits8_wyhash>],
                }
                criterion_group! {
                    name=[<union_ mlehybridplusplus $register _ $precision:lower>];
                    config = Criterion::default().sample_size(10).warm_up_time(std::time::Duration::from_secs(1)).warm_up_time(std::time::Duration::from_secs(1));
                    targets=[<bench_mlehybridplusplus $register _union_ $precision:lower _bits6_xxhash64>], [<bench_mlehybridplusplus $register _union_ $precision:lower _bits6_wyhash>], [<bench_mlehybridplusplus $register _union_ $precision:lower _bits8_xxhash64>], [<bench_mlehybridplusplus $register _union_ $precision:lower _bits8_wyhash>]
                }
                criterion_group! {
                    name=[<union_ mlebeta $register _ $precision:lower>];
                    config = Criterion::default().sample_size(10).warm_up_time(std::time::Duration::from_secs(1)).warm_up_time(std::time::Duration::from_secs(1));
                    targets=[<bench_mlebeta $register _union_ $precision:lower _bits6_xxhash64>], [<bench_mlebeta $register _union_ $precision:lower _bits6_wyhash>], [<bench_mlebeta $register _union_ $precision:lower _bits8_xxhash64>], [<bench_mlebeta $register _union_ $precision:lower _bits8_wyhash>]
                }
                criterion_group! {
                    name=[<union_ mlehybridbeta $register _ $precision:lower>];
                    config = Criterion::default().sample_size(10).warm_up_time(std::time::Duration::from_secs(1)).warm_up_time(std::time::Duration::from_secs(1));
                    targets=[<bench_mlehybridbeta $register _union_ $precision:lower _bits6_xxhash64>], [<bench_mlehybridbeta $register _union_ $precision:lower _bits6_wyhash>], [<bench_mlehybridbeta $register _union_ $precision:lower _bits8_xxhash64>], [<bench_mlehybridbeta $register _union_ $precision:lower _bits8_wyhash>]
                }
            }
        )*
    };
}

/// Macro to generate a criterion benchmark with the provided precision exponents
macro_rules! bench_cardinalities {
    ($(($precision:ty, $sample_size:expr)),*) => {
        $(
            bench_cludflare_union!($precision, Bits6, WyHash, XxHash64);
            bench_union_bits!($precision, Bits6);
            bench_union_bits!($precision, Bits8);

            paste::item! {
                fn [<bench_tabacpf_union_ $precision:lower>] (b: &mut Criterion) {
                    union_bencher::<TabacHLL<$precision>>(b);
                }

                fn [<bench_tabacplusplus_union_ $precision:lower>] (b: &mut Criterion) {
                    union_bencher::<TabacHLLPlusPlus<$precision>>(b);
                }

                fn [<bench_rhll_union_ $precision:lower _bits6>] (b: &mut Criterion) {
                    union_bencher::<RustHLL<$precision>>(b);
                }

                fn [<bench_shll_union_ $precision:lower _bits6>] (b: &mut Criterion) {
                    union_bencher::<SimpleHLL<{$precision::EXPONENT as usize}>>(b);
                }

                fn [<bench_sm_union_ $precision:lower _bits6>] (b: &mut Criterion) {
                    union_bencher::<SourMash<$precision>>(b);
                }

                fn [<bench_sa_union_ $precision:lower>] (b: &mut Criterion) {
                    union_bencher::<AlecHLL<$precision>>(b);
                }

                criterion_group! {
                    name=[<union_tabacpf_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size / 5);
                    targets=[<bench_tabacpf_union_ $precision:lower>]
                }
                criterion_group! {
                    name=[<union_tabacplusplus_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size / 5);
                    targets=[<bench_tabacplusplus_union_ $precision:lower>]
                }
                criterion_group! {
                    name=[<union_sa_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_sa_union_ $precision:lower>]
                }
                criterion_group! {
                    name=[<union_cludflare_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size);
                    targets=[<bench_cludflare_union_ $precision:lower _bits6_wyhash>], [<bench_cludflare_union_ $precision:lower _bits6_xxhash64>]
                }
                criterion_group! {
                    name=[<union_rhll_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size / 5);
                    targets=[<bench_rhll_union_ $precision:lower _bits6>]
                }
                criterion_group! {
                    name=[<union_shll_ $precision:lower>];
                    config = Criterion::default().sample_size($sample_size / 5);
                    targets=[<bench_shll_union_ $precision:lower _bits6>]
                }
                criterion_group! {
                    name=[<union_sm_ $precision:lower>];
                    config = Criterion::default().sample_size(10).warm_up_time(std::time::Duration::from_secs(1)).warm_up_time(std::time::Duration::from_secs(1));
                    targets=[<bench_sm_union_ $precision:lower _bits6>]
                }

                bench_union_registers!($precision, $sample_size, "array", "packedarray");
            }
        )*
    };
}

macro_rules! bench_hyper_two_bits {
    ($($sketch:ty),*) => {
        $(
            paste::paste!{
                fn [<bench_hypertwobits_ $sketch:lower  _union>](b: &mut Criterion) {
                    union_bencher::<HyperTwoBits<$sketch>>(b);
                }
            }
        )*
    };
}

macro_rules! bench_hyper_three_bits {
    ($($sketch:ty),*) => {
        $(
            paste::paste!{
                fn [<bench_hyperthreebits_ $sketch:lower _union>](b: &mut Criterion) {
                    union_bencher::<HyperThreeBits<$sketch>>(b);
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

fn bench_hashset_union(b: &mut Criterion) {
    union_bencher::<HashSet<u64>>(b);
}

criterion_group!(
    name = union_hyper_two_bits;
    config = Criterion::default().sample_size(100).warm_up_time(std::time::Duration::from_secs(1));
    targets = bench_hypertwobits_m64h2b_union, bench_hypertwobits_m128h2b_union, bench_hypertwobits_m256h2b_union, bench_hypertwobits_m512h2b_union, bench_hypertwobits_m1024h2b_union, bench_hypertwobits_m2048h2b_union, bench_hypertwobits_m4096h2b_union
);

criterion_group!(
    name = union_hyper_three_bits;
    config = Criterion::default().sample_size(100).warm_up_time(std::time::Duration::from_secs(1));
    targets = bench_hyperthreebits_m64h3b_union, bench_hyperthreebits_m128h3b_union, bench_hyperthreebits_m256h3b_union, bench_hyperthreebits_m512h3b_union, bench_hyperthreebits_m1024h3b_union, bench_hyperthreebits_m2048h3b_union, bench_hyperthreebits_m4096h3b_union
);

criterion_group!(
    name = union_hashset;
    config = Criterion::default().sample_size(100).warm_up_time(std::time::Duration::from_secs(1));
    targets = bench_hashset_union
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
macro_rules! bench_union_main {
    ($($precision:ty),*) => {
        paste::paste!{
            criterion_main!(
                $(
                    [<union_betaarray_ $precision:lower>],
                    [<union_betapackedarray_ $precision:lower>],
                    [<union_plusplusarray_ $precision:lower>],
                    [<union_pluspluspackedarray_ $precision:lower>],
                    [<union_hybridbetaarray_ $precision:lower>],
                    [<union_hybridbetapackedarray_ $precision:lower>],
                    [<union_hybridplusplusarray_ $precision:lower>],
                    [<union_hybridpluspluspackedarray_ $precision:lower>],
                    [<union_mlebetaarray_ $precision:lower>],
                    [<union_mlebetapackedarray_ $precision:lower>],
                    [<union_mleplusplusarray_ $precision:lower>],
                    [<union_mlepluspluspackedarray_ $precision:lower>],
                    [<union_mlehybridbetaarray_ $precision:lower>],
                    [<union_mlehybridbetapackedarray_ $precision:lower>],
                    [<union_mlehybridplusplusarray_ $precision:lower>],
                    [<union_mlehybridpluspluspackedarray_ $precision:lower>],
                    [<union_tabacpf_ $precision:lower>],
                    [<union_tabacplusplus_ $precision:lower>],
                    [<union_sa_ $precision:lower>],
                    [<union_cludflare_ $precision:lower>],
                    [<union_rhll_ $precision:lower>],
                    [<union_sm_ $precision:lower>],
                    [<union_shll_ $precision:lower>],
                )*
                union_hyper_two_bits,
                union_hyper_three_bits,
                union_hashset
            );
        }
    };
}

bench_union_main!(
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
