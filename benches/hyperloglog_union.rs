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

criterion_group!(benches, bench_hyperloglog_union);

criterion_main!(benches);
