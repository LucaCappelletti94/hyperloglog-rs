#![allow(clippy::needless_range_loop)]
#![allow(clippy::upper_case_acronyms)]
//! Bench to measure the performance of the union (`BitOr`) of two HyperLogLog counters.
//!
//! The union takes different code paths depending on the mode of each operand, so we cover
//! all three regimes: both operands still hash lists (the accuracy-preserving merge), both
//! operands saturated into fully-fledged HyperLogLogs (the register-wise maximum), and the
//! mixed case.
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use hyperloglog_rs::prelude::*;

type HLL = HyperLogLog<Precision9, Bits6>;

/// Cardinalities swept across the hash-list regime to expose the (former) super-linear union/sketch
/// time hump and confirm the two-pointer merge is flat. The largest entries sit just under the
/// hash-list-to-registers transition; any that no longer fit a given precision are skipped.
const HASH_LIST_SWEEP: &[u64] = &[100, 450, 1000, 2000, 3000, 4000, 4730];

/// Builds a counter from `count` distinct values drawn from `seed`. Same seed and a larger `count`
/// yields a superset (the random stream is a deterministic prefix), so these nest for the sketch.
fn build_hash_list<P: Precision + PackedRegister<B>, B: Bits>(
    count: u64,
    seed: u64,
) -> HyperLogLog<P, B> {
    let mut hll = HyperLogLog::<P, B>::default();
    for value in iter_random_values::<u64>(count, None, Some(seed)) {
        hll.insert(&value);
    }
    hll
}

/// Both-hash-list union (`a | b`) swept across the hash-list regime at one precision: each operand
/// holds `card` distinct values and stays a hash list. O(n+m) merge, formerly O(n*m).
fn union_hash_list_sweep<P: Precision + PackedRegister<B>, B: Bits>(c: &mut Criterion, name: &str) {
    let mut group = c.benchmark_group(name);
    for &card in HASH_LIST_SWEEP {
        let left = build_hash_list::<P, B>(card, 0x00A1_1CE0);
        let right = build_hash_list::<P, B>(card, 0x0000_B0B0);
        if !(left.is_sorted_hash_list() && right.is_sorted_hash_list()) {
            continue;
        }
        group.bench_with_input(BenchmarkId::from_parameter(card), &card, |b, _| {
            b.iter(|| black_box(black_box(&left) | black_box(&right)));
        });
    }
    group.finish();
}

/// Default (non-MLE) `M=N=2` joint sketch over four nested hash lists, swept across the hash-list
/// regime: the sketch runs `M*N = 4` both-hash-list unions, so it tracks the union hump.
fn sketch_hash_list_sweep<P: Precision + PackedRegister<B>, B: Bits>(
    c: &mut Criterion,
    name: &str,
) {
    let mut group = c.benchmark_group(name);
    for &card in HASH_LIST_SWEEP {
        let inner = card / 2;
        let left = [
            build_hash_list::<P, B>(inner, 0x00A1_1CE0),
            build_hash_list::<P, B>(card, 0x00A1_1CE0),
        ];
        let right = [
            build_hash_list::<P, B>(inner, 0x0000_B0B0),
            build_hash_list::<P, B>(card, 0x0000_B0B0),
        ];
        if !left
            .iter()
            .chain(right.iter())
            .all(HyperLogLog::is_sorted_hash_list)
        {
            continue;
        }
        group.bench_with_input(BenchmarkId::from_parameter(card), &card, |b, _| {
            b.iter(|| black_box(JointSketch::estimate(black_box(&left), black_box(&right))));
        });
    }
    group.finish();
}

/// The cardinality sweeps at the two precisions the handoff calls out.
fn bench_hash_list_sweeps(c: &mut Criterion) {
    union_hash_list_sweep::<Precision12, Bits6>(c, "union_hash_list_sweep_p12");
    union_hash_list_sweep::<Precision14, Bits6>(c, "union_hash_list_sweep_p14");
    sketch_hash_list_sweep::<Precision12, Bits6>(c, "sketch_hash_list_sweep_p12");
    sketch_hash_list_sweep::<Precision14, Bits6>(c, "sketch_hash_list_sweep_p14");
}

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
        if !hll.is_sorted_hash_list() {
            break;
        }
    }
    assert!(!hll.is_sorted_hash_list());
    hll
}

fn bench_hyperloglog_union(c: &mut Criterion) {
    let mut group = c.benchmark_group("hyperloglog_union");

    // Hash-list regime: both operands are still hash lists.
    let left_hash_list = build(100, 0x00A1_1CE0);
    let right_hash_list = build(100, 0x0000_B0B0);
    assert!(left_hash_list.is_sorted_hash_list() && right_hash_list.is_sorted_hash_list());

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

    // The `HyperBall` scenario: `Precision10 Bits5` (1024 5-bit registers, 640 B per counter),
    // both operands forced dense via `into_hll()`. This is the exact shape the `hll_only` cell
    // hits inside HyperBall's inner loop, one merge per graph arc.
    type HllP10B5 = HyperLogLog<Precision10, Bits5>;
    fn build_saturated_p10b5(seed: u64) -> HllP10B5 {
        let mut hll = HllP10B5::default();
        for value in iter_random_values::<u64>(1_000_000, None, Some(seed)) {
            hll.insert(&value);
            if !hll.is_sorted_hash_list() {
                break;
            }
        }
        hll.into_hll()
    }
    let left_p10b5 = build_saturated_p10b5(0x00A1_1CE0);
    let right_p10b5 = build_saturated_p10b5(0x0000_B0B0);
    group.bench_function("union_hyperloglog_p10b5", |b| {
        b.iter(|| black_box(black_box(&left_p10b5) | black_box(&right_p10b5)));
    });

    // Merge in place (no clone) at the same shape, since HyperBall's inner call is
    // `merge_with_helper(&mut dst, &src)`, not a `BitOr` that clones the left operand. The
    // `BitOr` bench above bundles a clone into every sample, which is where a chunk of the
    // `union_hyperloglog_p10b5` time actually goes. This one isolates the merge itself.
    group.bench_function("merge_hyperloglog_p10b5_in_place", |b| {
        b.iter_with_setup(
            || left_p10b5,
            |mut dst| {
                black_box(&mut dst).bitor_assign(black_box(&right_p10b5));
                dst
            },
        );
    });

    group.finish();
}

type HLL14 = HyperLogLog<Precision14, Bits6>;

/// Builds the largest counter that is still in hash-list mode (stops just before the
/// insertion that would convert it to a fully-fledged HyperLogLog).
fn build_full_hash_list(seed: u64) -> HLL14 {
    let mut hll = HLL14::default();
    for value in iter_random_values::<u64>(1_000_000, None, Some(seed)) {
        let mut candidate = hll;
        candidate.insert(&value);
        if !candidate.is_sorted_hash_list() {
            break;
        }
        hll = candidate;
    }
    assert!(hll.is_sorted_hash_list());
    hll
}

/// Compares the two ways of estimating a union cardinality when both operands are large
/// hash lists (the regime where inclusion-exclusion is biased): the current
/// inclusion-exclusion path versus the merger-based estimate.
fn bench_union_estimate(c: &mut Criterion) {
    let mut group = c.benchmark_group("union_estimate_hash_list_p14");

    let left = build_full_hash_list(0x00A1_1CE0);
    let right = build_full_hash_list(0x0000_B0B0);
    assert!(left.is_sorted_hash_list() && right.is_sorted_hash_list());

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
    assert!(!hll.is_sorted_hash_list());
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
    bench_union_estimate_hyperloglog,
    bench_hash_list_sweeps
);

criterion_main!(benches);
