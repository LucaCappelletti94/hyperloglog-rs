//! Bench to compare and optimize time performance of inserting a prefix-free encoded list of hashes.
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hyperloglog_rs::composite_hash::GapHash;
use hyperloglog_rs::composite_hash::SwitchHash;
use hyperloglog_rs::prelude::*;

type Switch = Hybrid<
    PlusPlus<
        Precision14,
        Bits5,
        <Precision14 as ArrayRegister<Bits5>>::Packed,
        twox_hash::XxHash64,
    >,
    SwitchHash<Precision14, Bits5>,
>;
type Gap = Hybrid<
    PlusPlus<
        Precision14,
        Bits5,
        <Precision14 as ArrayRegister<Bits5>>::Packed,
        twox_hash::XxHash64,
    >,
    GapHash<Precision14, Bits5>,
>;

fn bench_hash_list_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_list_insert");

    group.bench_function("switch_insert", |b| {
        b.iter(|| {
            let mut result = false;
            let mut switch: Switch = Switch::default();
            for random_value in iter_random_values::<u64>(3_000, None, None) {
                result ^= switch.insert(black_box(&random_value));
            }
            let mut switch: Switch = Switch::default();
            for random_value in iter_random_values::<u64>(3_000, None, Some(876865675675)) {
                result ^= switch.insert(black_box(&random_value));
            }
            let mut switch: Switch = Switch::default();
            for random_value in iter_random_values::<u64>(3_000, None, Some(9775546767677)) {
                result ^= switch.insert(black_box(&random_value));
            }
            let mut switch: Switch = Switch::default();
            for random_value in iter_random_values::<u64>(3_000, None, Some(8775746767677)) {
                result ^= switch.insert(black_box(&random_value));
            }
            result
        });
    });

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
