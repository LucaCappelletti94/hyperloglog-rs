//! Bench to compare and optimize time performance of inserting a prefix-free encoded list of hashes.
use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use hyperloglog_rs::prelude::*;
use twox_hash::XxHash64;

type HLLW =
    HyperLogLog<Precision9, Bits6, <Precision9 as PackedRegister<Bits6>>::Array, wyhash::WyHash>;

type HLLX = HyperLogLog<Precision9, Bits6, <Precision9 as PackedRegister<Bits6>>::Array, XxHash64>;

type HLLA =
    HyperLogLog<Precision9, Bits6, <Precision9 as PackedRegister<Bits6>>::Array, ahash::AHasher>;

fn bench_hyperloglog_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("hyperloglog_insert");

    let mut hllx = HLLX::default();
    let mut hllw = HLLW::default();
    let mut hlla = HLLA::default();

    // Since we are only interested in the time performance of when the data structure has
    // already switched from the HashList to the HyperLogLog, we will only saturate the
    // data structure with random values until it switches.
    for random_value in iter_random_values::<u64>(50_000, None, None) {
        if hllx.is_hash_list() {
            hllx.insert(&random_value);
        }
        if hllw.is_hash_list() {
            hllw.insert(&random_value);
        }
        if hlla.is_hash_list() {
            hlla.insert(&random_value);
        }
        if !hllx.is_hash_list() && !hllw.is_hash_list() && !hlla.is_hash_list() {
            break;
        }
    }

    group.bench_function("insert_wyhash", |b| {
        b.iter(|| {
            let mut result = false;
            let mut hllw = hllw.clone();
            for random_value in iter_random_values::<u64>(100_000, None, None) {
                result ^= hllw.insert(black_box(&random_value));
            }
            result
        });
    });

    group.bench_function("insert_xxhash", |b| {
        b.iter(|| {
            let mut result = false;
            let mut hllx = hllx.clone();
            for random_value in iter_random_values::<u64>(100_000, None, None) {
                result ^= hllx.insert(black_box(&random_value));
            }
            result
        });
    });

    group.bench_function("insert_ahash", |b| {
        b.iter(|| {
            let mut result = false;
            let mut hlla = hlla.clone();
            for random_value in iter_random_values::<u64>(100_000, None, None) {
                result ^= hlla.insert(black_box(&random_value));
            }
            result
        });
    });

    group.finish();
}

type HLL14 =
    HyperLogLog<Precision14, Bits6, <Precision14 as PackedRegister<Bits6>>::Array, XxHash64>;

/// Builds a counter holding `count` distinct elements, asserting it is still in hash-list
/// mode (`count` must stay below the conversion threshold, ~34k at Precision14/Bits6).
fn hash_list_of(count: u64, seed: u64) -> HLL14 {
    let mut hll = HLL14::default();
    for value in iter_random_values::<u64>(count, None, Some(seed)) {
        hll.insert(&value);
    }
    assert!(
        hll.is_hash_list(),
        "a counter of {count} elements must still be a hash list"
    );
    hll
}

/// Compares the per-insert cost of the hash-list mode (sorted, gap-encoded, O(n) per insert)
/// against the fully-fledged HyperLogLog mode (O(1) register update), inserting the same batch
/// of new elements into bases of increasing size. The HyperLogLog base is obtained by flipping
/// a hash-list counter with `convert_hash_list_to_hyperloglog`, so both modes hold the same
/// elements. The clone in the setup closure is not timed (`iter_batched`).
fn bench_insert_modes(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_mode_p14_bits6");

    // 256 fresh elements, disjoint from the bases, inserted on each measured iteration.
    let batch: Vec<u64> = iter_random_values::<u64>(256, None, Some(0x00BA_7C00)).collect();

    for &base_size in &[256_u64, 4_096, 16_384] {
        let hash_list_base = hash_list_of(base_size, 0x00BA_5E00);
        let mut hll_base = hash_list_base.clone();
        hll_base.convert_hash_list_to_hyperloglog().unwrap();
        assert!(!hll_base.is_hash_list());

        group.bench_with_input(
            BenchmarkId::new("hash_list", base_size),
            &base_size,
            |b, _| {
                b.iter_batched(
                    || hash_list_base.clone(),
                    |mut hll| {
                        for value in &batch {
                            hll.insert(black_box(value));
                        }
                        black_box(hll.is_hash_list())
                    },
                    BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(BenchmarkId::new("hll", base_size), &base_size, |b, _| {
            b.iter_batched(
                || hll_base.clone(),
                |mut hll| {
                    for value in &batch {
                        hll.insert(black_box(value));
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Compares the cost of the cardinality estimators on a fully-fledged HyperLogLog counter: the
/// default HyperLogLog++ corrected estimate, the raw uncorrected estimate, and (under the `mle`
/// feature) the secant-method Maximum Likelihood Estimation.
fn bench_cardinality_estimate(c: &mut Criterion) {
    let mut group = c.benchmark_group("cardinality_estimate_p14_bits6");

    let hll = build_hyperloglog(200_000, 0x0000_CA4D);

    group.bench_function("hyperloglog_pp", |b| {
        b.iter(|| black_box(black_box(&hll).estimate_cardinality()));
    });

    group.bench_function("uncorrected", |b| {
        b.iter(|| black_box(black_box(&hll).uncorrected_estimate_cardinality()));
    });

    #[cfg(feature = "mle")]
    group.bench_function("mle", |b| {
        b.iter(|| black_box(black_box(&hll).estimate_cardinality_mle()));
    });

    group.finish();
}

/// Builds a fully-fledged HyperLogLog counter holding `count` distinct elements.
fn build_hyperloglog(count: u64, seed: u64) -> HLL14 {
    let mut hll = HLL14::default();
    for value in iter_random_values::<u64>(count, None, Some(seed)) {
        hll.insert(&value);
    }
    assert!(!hll.is_hash_list());
    hll
}

criterion_group!(
    benches,
    bench_hyperloglog_insert,
    bench_insert_modes,
    bench_cardinality_estimate
);

criterion_main!(benches);
