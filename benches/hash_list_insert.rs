//! Bench to compare and optimize time performance of inserting a prefix-free encoded list of hashes.
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;

type Gap = HyperLogLog<
    Precision14,
    Bits5,
    <Precision14 as ArrayRegister<Bits5>>::Packed,
    twox_hash::XxHash64,
>;

fn bench_hash_list_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_list_insert");

    group.bench_function("prefix_free_insert", |b| {
        b.iter(|| {
            let mut result = false;
            let mut switch: Gap = Gap::default();
            for random_value in iter_random_values::<u64>(12_000, None, None) {
                result ^= switch.insert(black_box(&random_value));
            }
            result
        });
    });

    group.finish();
}

criterion_group!(benches, bench_hash_list_insert);

criterion_main!(benches);
