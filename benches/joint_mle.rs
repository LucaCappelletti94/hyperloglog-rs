#![allow(clippy::needless_range_loop)]
#![allow(clippy::upper_case_acronyms)]
//! Criterion benchmark for the generalized joint MLE register solve, focused on the expensive,
//! well-identified end: a fixed 8x8 power-law deep-cell instance at P8, P10, P14 and P16 (Bits6). It
//! times one full damped-Newton solve (the analytic-Hessian second-order optimizer, the production
//! path) on each instance. The nested counters are built once outside the timed loop. The operands
//! going in and the returned sketch coming out are black-boxed so nothing is hoisted or elided.
//!
//! Run with:
//!   cargo bench --bench joint_mle

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;
use twox_hash::XxHash64;

type Counter<P, B> = HyperLogLog<P, B, <P as PackedRegister<B>>::Array, XxHash64>;

/// Local splitmix64 mixing step, so the fixed instance is built deterministically without depending
/// on a crate-private helper.
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Builds one dense counter from disjoint integer ranges (each `(start, count)`).
fn build<P, B>(ranges: &[(u64, u64)]) -> Counter<P, B>
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let mut hll = Counter::<P, B>::default();
    for &(start, count) in ranges {
        for v in start..start + count {
            hll.insert(&v);
        }
    }
    hll
}

/// Builds nested left and right power-law deep-cell counters (the same layout the joint-MLE tests
/// use): a few large shallow cells and many tiny deep cells, from disjoint integer ranges so every
/// counter is dense. Returns `(lefts, rights)`.
#[allow(clippy::type_complexity)]
fn build_power_law_cells<P, B, const M: usize, const N: usize>(
    seed: u64,
    base: u64,
) -> ([Counter<P, B>; M], [Counter<P, B>; N])
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let mut state = seed;
    let mut cursor = 0u64;
    let mut ranges_o = [[(0u64, 0u64); N]; M];
    for i in 0..M {
        for j in 0..N {
            state = splitmix64(state);
            let depth = (i + j) as u32;
            let e = (state % 4) as u32;
            let exponent = e.saturating_sub(depth.min(3));
            let count = base * (1 << exponent);
            ranges_o[i][j] = (cursor, count);
            cursor += count;
        }
    }
    let mut ranges_da = [(0u64, 0u64); M];
    for slot in ranges_da.iter_mut() {
        state = splitmix64(state);
        let count = base * (1 + (state % 4));
        *slot = (cursor, count);
        cursor += count;
    }
    let mut ranges_db = [(0u64, 0u64); N];
    for slot in ranges_db.iter_mut() {
        state = splitmix64(state);
        let count = base * (1 + (state % 4));
        *slot = (cursor, count);
        cursor += count;
    }
    let lefts: [Counter<P, B>; M] = core::array::from_fn(|i| {
        let mut ranges = Vec::new();
        for ii in 0..=i {
            for j in 0..N {
                ranges.push(ranges_o[ii][j]);
            }
            ranges.push(ranges_da[ii]);
        }
        build::<P, B>(&ranges)
    });
    let rights: [Counter<P, B>; N] = core::array::from_fn(|j| {
        let mut ranges = Vec::new();
        for jj in 0..=j {
            for i in 0..M {
                ranges.push(ranges_o[i][jj]);
            }
            ranges.push(ranges_db[jj]);
        }
        build::<P, B>(&ranges)
    });
    (lefts, rights)
}

fn bench_joint_mle(c: &mut Criterion) {
    let mut group = c.benchmark_group("joint_mle_8x8");

    // P8 8x8: build the fixed instance once, outside the timed loop.
    {
        let (lefts, rights) = build_power_law_cells::<Precision8, Bits6, 8, 8>(0xC0FFEE, 300);
        group.bench_function("damped_p8", |b| {
            b.iter(|| {
                let sketch =
                    hyperloglog_rs::mle::bench_joint_sketch_mle::<Precision8, Bits6, _, _, _, 8, 8>(
                        black_box(&lefts),
                        black_box(&rights),
                    );
                black_box(sketch)
            });
        });
    }

    // P10 8x8.
    {
        let (lefts, rights) = build_power_law_cells::<Precision10, Bits6, 8, 8>(0xC0FFEE, 1200);
        group.bench_function("damped_p10", |b| {
            b.iter(|| {
                let sketch =
                    hyperloglog_rs::mle::bench_joint_sketch_mle::<Precision10, Bits6, _, _, _, 8, 8>(
                        black_box(&lefts),
                        black_box(&rights),
                    );
                black_box(sketch)
            });
        });
    }

    // High-precision probe: P14 and P16 8x8. The solver reduces to the distinct register patterns, but
    // on the 8x8 grid almost every register has a unique pattern, so the distinct count stays close to
    // m and these still expose the per-evaluation cost at large m (m = 2^14 and 2^16).
    {
        let (lefts, rights) = build_power_law_cells::<Precision14, Bits6, 8, 8>(0xC0FFEE, 4096);
        group.bench_function("damped_p14", |b| {
            b.iter(|| {
                let sketch =
                    hyperloglog_rs::mle::bench_joint_sketch_mle::<Precision14, Bits6, _, _, _, 8, 8>(
                        black_box(&lefts),
                        black_box(&rights),
                    );
                black_box(sketch)
            });
        });
    }
    {
        let (lefts, rights) = build_power_law_cells::<Precision16, Bits6, 8, 8>(0xC0FFEE, 16384);
        group.bench_function("damped_p16", |b| {
            b.iter(|| {
                let sketch =
                    hyperloglog_rs::mle::bench_joint_sketch_mle::<Precision16, Bits6, _, _, _, 8, 8>(
                        black_box(&lefts),
                        black_box(&rights),
                    );
                black_box(sketch)
            });
        });
    }

    group.finish();
}

/// Builds two dense counters with a controlled overlap from disjoint integer pools (shared,
/// left-only, right-only), returned as register-mode HyperLogLogs.
fn build_two_set_overlap<P, B>(
    inter: u64,
    left_only: u64,
    right_only: u64,
    seed: u64,
) -> (Counter<P, B>, Counter<P, B>)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let shared = |i: u64| splitmix64(seed.wrapping_add(i));
    let lo = |i: u64| splitmix64(seed.wrapping_add(1_000_000_000 + i));
    let ro = |i: u64| splitmix64(seed.wrapping_add(2_000_000_000 + i));
    let mut a = Counter::<P, B>::default();
    let mut b = Counter::<P, B>::default();
    for i in 0..inter {
        let s = shared(i);
        a.insert(&s);
        b.insert(&s);
    }
    for i in 0..left_only {
        a.insert(&lo(i));
    }
    for i in 0..right_only {
        b.insert(&ro(i));
    }
    (a, b)
}

/// Times one 2-set union MLE solve (the workhorse hot path behind the default pairwise sketch,
/// Jaccard, intersection and `.mle()`): the production damped-Newton solver, on fixed instances built
/// once outside the timed loop, at P8 and P12. Inputs and the returned regions are black-boxed.
fn bench_two_set_union(c: &mut Criterion) {
    let mut group = c.benchmark_group("two_set_union");

    let (a8, b8) = build_two_set_overlap::<Precision8, Bits6>(5000, 5000, 5000, 0xC0FFEE);
    group.bench_function("p8_balanced", |bencher| {
        bencher.iter(|| {
            black_box(hyperloglog_rs::mle::bench_union_regions(
                black_box(&a8),
                black_box(&b8),
            ))
        });
    });

    // P12 (the production default precision in many examples), balanced overlap.
    let (a12, b12) = build_two_set_overlap::<Precision12, Bits6>(20000, 20000, 20000, 0xBADF00D);
    group.bench_function("p12_balanced", |bencher| {
        bencher.iter(|| {
            black_box(hyperloglog_rs::mle::bench_union_regions(
                black_box(&a12),
                black_box(&b12),
            ))
        });
    });

    group.finish();
}

criterion_group!(benches, bench_joint_mle, bench_two_set_union);
criterion_main!(benches);
