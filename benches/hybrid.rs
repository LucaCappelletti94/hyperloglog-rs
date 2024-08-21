//! Benchmark to try and improve performance of the principal hybrid cases.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;

fn bench_hybrid(c: &mut Criterion) {
    let mut random_state = 76568342984735313_u64;
    // We consider the case of hash of 32 bits in a precision of 18, using 6 bits per register.
    let entries: Vec<(_, _)> = (0..1_000)
        .map(|_| {
            random_state = splitmix64(random_state);

            let mut keep_hybrid: Hybrid<
                PlusPlus<Precision10, Bits6, <Precision10 as ArrayRegister<Bits6>>::Array>,
                u32,
            > = Hybrid::default();
            let mut to_dehybridize: Hybrid<
                PlusPlus<Precision10, Bits6, <Precision10 as ArrayRegister<Bits6>>::Array>,
                u32,
            > = Hybrid::default();

            keep_hybrid.extend(iter_random_values::<u64>(
                keep_hybrid.capacity() as u64,
                None,
                Some(random_state),
            ));
            random_state = splitmix64(random_state);
            to_dehybridize.extend(iter_random_values::<u64>(
                to_dehybridize.capacity() as u64 * 2,
                None,
                Some(random_state),
            ));

            assert!(keep_hybrid.is_hybrid());
            assert!(!to_dehybridize.is_hybrid());

            (keep_hybrid, to_dehybridize)
        })
        .collect();

    let mut group = c.benchmark_group("hybrid");

    group.bench_function("hybrid_mix_union", |b| {
        b.iter(|| {
            let mut total_cardinality = 0.0;
            for (left, right) in &entries {
                total_cardinality += black_box(left).estimate_union_cardinality(right);
            }
            total_cardinality
        })
    });

    group.finish();
}

criterion_group!(benches, bench_hybrid);

criterion_main!(benches);
