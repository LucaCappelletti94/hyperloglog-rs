//! Benchmarks for the Maximum Likelihood Estimation paths, which build register-multiplicity
//! histograms once per call. Used to confirm that moving those histograms from heap `Vec`s to stack
//! arrays does not regress (and ideally improves) the per-call cost. MLE runs only on fully-fledged
//! HyperLogLog (register) operands, so every counter here is built well past the hash-list threshold.
//!
//! Run with:
//!   cargo bench --features mle --bench mle
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use hyperloglog_rs::prelude::*;

type Hll = HyperLogLog<Precision12, Bits6>;

/// Builds a register-mode counter holding `count` distinct elements.
fn build_dense(count: u64, seed: u64) -> Hll {
    let mut hll = Hll::default();
    for value in iter_random_values::<u64>(count, None, Some(seed)) {
        hll.insert(&value);
    }
    assert!(
        hll.is_hyperloglog(),
        "counter must be dense for the MLE path"
    );
    hll
}

/// Single-counter cardinality MLE (one multiplicity histogram per call).
fn bench_cardinality_mle(c: &mut Criterion) {
    let mut group = c.benchmark_group("mle_cardinality");
    for &card in &[20_000u64, 50_000, 100_000] {
        let hll = build_dense(card, 0x00A1_1CE0);
        group.bench_with_input(BenchmarkId::from_parameter(card), &card, |b, _| {
            b.iter(|| black_box(black_box(&hll).mle().estimate_cardinality()));
        });
    }
    group.finish();
}

/// Two-set union MLE (five multiplicity histograms per call).
fn bench_union_mle(c: &mut Criterion) {
    let mut group = c.benchmark_group("mle_union");
    for &card in &[20_000u64, 50_000, 100_000] {
        let left = build_dense(card, 0x00A1_1CE0);
        let right = build_dense(card, 0x0000_B0B0);
        group.bench_with_input(BenchmarkId::from_parameter(card), &card, |b, _| {
            b.iter(|| {
                black_box(
                    black_box(&left)
                        .mle()
                        .estimate_union_cardinality(&black_box(&right).mle()),
                )
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_cardinality_mle, bench_union_mle);
criterion_main!(benches);
