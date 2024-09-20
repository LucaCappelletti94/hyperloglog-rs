//! Bench to compare and optimize time performance of inserting a prefix-free encoded list of hashes.
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;
use twox_hash::XxHash64;

type HLLW = HyperLogLog<
    Precision9,
    Bits6,
    <Precision9 as PackedRegister<Bits6>>::Array,
    wyhash::WyHash,
>;

type HLLX = HyperLogLog<
    Precision9,
    Bits6,
    <Precision9 as PackedRegister<Bits6>>::Array,
    XxHash64
>;

type HLLA = HyperLogLog<
    Precision9,
    Bits6,
    <Precision9 as PackedRegister<Bits6>>::Array,
    ahash::AHasher,
>;

type HLLN = HyperLogLog<
    Precision9,
    Bits6,
    <Precision9 as PackedRegister<Bits6>>::Array,
    NaiveIntegerHash,
>;


fn bench_hyperloglog_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("hyperloglog_insert");

    let mut hllx = HLLX::default();
    let mut hllw = HLLW::default();
    let mut hlla = HLLA::default();
    let mut hlln = HLLN::default();

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
        if hlln.is_hash_list() {
            hlln.insert(&random_value);
        }
        if !hllx.is_hash_list() && !hllw.is_hash_list() && !hlla.is_hash_list() && !hlln.is_hash_list() {
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

    group.bench_function("insert_naive", |b| {
        b.iter(|| {
            let mut result = false;
            let mut hlln = hlln.clone();
            for random_value in iter_random_values::<u64>(100_000, None, None) {
                result ^= hlln.insert(black_box(&random_value));
            }
            result
        });
    });

    group.finish();
}

criterion_group!(benches, bench_hyperloglog_insert);

criterion_main!(benches);
