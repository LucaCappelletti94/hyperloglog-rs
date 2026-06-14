//! Bench to measure the performance of the union (`BitOr`) of two HyperLogLog counters.
//!
//! The union takes different code paths depending on the mode of each operand, so we cover
//! all three regimes: both operands still hash lists (the accuracy-preserving merge), both
//! operands saturated into fully-fledged HyperLogLogs (the register-wise maximum), and the
//! mixed case.
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;
use twox_hash::XxHash64;

type HLL = HyperLogLog<Precision9, Bits6, <Precision9 as PackedRegister<Bits6>>::Array, XxHash64>;

/// Builds a counter from `count` distinct values drawn from the given seed.
fn build(count: u64, seed: u64) -> HLL {
    let mut hll = HLL::default();
    for value in iter_random_values::<u64>(count, None, Some(seed)) {
        hll.insert(&value);
    }
    hll
}

/// Builds a counter saturated out of the hash-list mode.
fn build_saturated(seed: u64) -> HLL {
    let mut hll = HLL::default();
    for value in iter_random_values::<u64>(1_000_000, None, Some(seed)) {
        hll.insert(&value);
        if !hll.is_hash_list() {
            break;
        }
    }
    assert!(!hll.is_hash_list());
    hll
}

fn bench_hyperloglog_union(c: &mut Criterion) {
    let mut group = c.benchmark_group("hyperloglog_union");

    // Hash-list regime: both operands are still hash lists.
    let left_hash_list = build(100, 0x00A1_1CE0);
    let right_hash_list = build(100, 0x0000_B0B0);
    assert!(left_hash_list.is_hash_list() && right_hash_list.is_hash_list());

    group.bench_function("union_hash_list", |b| {
        b.iter(|| black_box(black_box(&left_hash_list) | black_box(&right_hash_list)));
    });

    // HyperLogLog regime: both operands are saturated.
    let left_hll = build_saturated(0x00A1_1CE0);
    let right_hll = build_saturated(0x0000_B0B0);

    group.bench_function("union_hyperloglog", |b| {
        b.iter(|| black_box(black_box(&left_hll) | black_box(&right_hll)));
    });

    // Mixed regime: one hash list, one saturated.
    group.bench_function("union_mixed", |b| {
        b.iter(|| black_box(black_box(&left_hash_list) | black_box(&right_hll)));
    });

    group.finish();
}

type HLL14 =
    HyperLogLog<Precision14, Bits6, <Precision14 as PackedRegister<Bits6>>::Array, XxHash64>;

/// Builds the largest counter that is still in hash-list mode (stops just before the
/// insertion that would convert it to a fully-fledged HyperLogLog).
fn build_full_hash_list(seed: u64) -> HLL14 {
    let mut hll = HLL14::default();
    for value in iter_random_values::<u64>(1_000_000, None, Some(seed)) {
        let mut candidate = hll.clone();
        candidate.insert(&value);
        if !candidate.is_hash_list() {
            break;
        }
        hll = candidate;
    }
    assert!(hll.is_hash_list());
    hll
}

/// Compares the two ways of estimating a union cardinality when both operands are large
/// hash lists (the regime where inclusion-exclusion is biased): the current
/// inclusion-exclusion path versus the merger-based estimate.
fn bench_union_estimate(c: &mut Criterion) {
    let mut group = c.benchmark_group("union_estimate_hash_list_p14");

    let left = build_full_hash_list(0x00A1_1CE0);
    let right = build_full_hash_list(0x0000_B0B0);
    assert!(left.is_hash_list() && right.is_hash_list());

    // Current: inclusion-exclusion (fast, biased low in this regime).
    group.bench_function("inclusion_exclusion", |b| {
        b.iter(|| black_box(black_box(&left).estimate_union_cardinality(black_box(&right))));
    });

    // Proposed: build the merged counter and estimate its cardinality (accurate).
    group.bench_function("merger", |b| {
        b.iter(|| black_box((black_box(&left) | black_box(&right)).estimate_cardinality()));
    });

    group.finish();
}

/// Builds a fully-fledged HyperLogLog counter (well past the hash-list conversion threshold)
/// holding `count` distinct elements.
fn build_hyperloglog(count: u64, seed: u64) -> HLL14 {
    let mut hll = HLL14::default();
    for value in iter_random_values::<u64>(count, None, Some(seed)) {
        hll.insert(&value);
    }
    assert!(!hll.is_hash_list());
    hll
}

/// Compares the default register-based union estimator against the joint Maximum Likelihood
/// Estimation union estimator (only available under the `mle` feature) on two fully-fledged
/// HyperLogLog counters.
fn bench_union_estimate_hyperloglog(c: &mut Criterion) {
    let mut group = c.benchmark_group("union_estimate_hyperloglog_p14");

    let left = build_hyperloglog(200_000, 0x00A1_1CE0);
    let right = build_hyperloglog(200_000, 0x0000_B0B0);

    group.bench_function("default", |b| {
        b.iter(|| black_box(black_box(&left).estimate_union_cardinality(black_box(&right))));
    });

    #[cfg(feature = "mle")]
    group.bench_function("mle", |b| {
        b.iter(|| {
            black_box(
                black_box(&left)
                    .mle()
                    .estimate_union_cardinality(&black_box(&right).mle()),
            )
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_hyperloglog_union,
    bench_union_estimate,
    bench_union_estimate_hyperloglog
);

criterion_main!(benches);
