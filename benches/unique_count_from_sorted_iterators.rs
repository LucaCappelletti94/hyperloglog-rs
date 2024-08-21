//! Benchmarks to evaluate improvements on the unique_count_from_sorted_iterators function.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hyperloglog_rs::hybrid::unique_count_from_sorted_iterators;
use hyperloglog_rs::prelude::{iter_var_len_random_values, splitmix64};

fn bench_unique_count_from_sorted_iterators(b: &mut Criterion) {
    let mut group = b.benchmark_group("unique_count_from_sorted_iterators");
    let mut random_state = 76568342984735313_u64;
    // We consider the case of hash of 32 bits in a precision of 18, using 6 bits per register.
    let maximal_possible_size = ((1 << 18) * 6) / 32;
    let entries: Vec<(Vec<u32>, Vec<u32>)> = (0..200)
        .map(|_| {
            random_state = splitmix64(random_state);
            let mut a = iter_var_len_random_values::<u32>(
                0,
                maximal_possible_size,
                None,
                Some(random_state),
            )
            .collect::<Vec<u32>>();
            random_state = splitmix64(random_state);
            let mut b = iter_var_len_random_values::<u32>(
                0,
                maximal_possible_size,
                None,
                Some(random_state),
            )
            .collect::<Vec<u32>>();
            a.sort();
            b.sort();

            (a, b)
        })
        .collect();

    group.bench_function("unique_count_from_sorted_iterators", |b| {
        b.iter(|| {
            for (a, b) in &entries {
                unique_count_from_sorted_iterators(black_box(a.iter()), black_box(b.iter()));
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_unique_count_from_sorted_iterators);

criterion_main!(benches);
