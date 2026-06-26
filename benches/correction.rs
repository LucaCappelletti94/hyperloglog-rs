//! Speed of the correction step: the shipped lookup-table path (binary search plus linear
//! interpolation over per-cell breakpoint arrays, the "matrix" approach) versus evaluating a fitted
//! polynomial in normalized load. This isolates exactly the operation that would change if the
//! register bias table were replaced by a polynomial, and sweeps the table size so the binary-search
//! scaling shows against the constant-time polynomial.
//!
//! The table side calls the real `correct_cardinality` from the crate. The polynomial side evaluates a
//! degree-8 polynomial with Horner's method over the domain-mapped variable, exactly what a shipped
//! replacement would do. The polynomial timing is independent of the table size by construction.
//!
//! Run with:
//!   cargo bench --bench correction
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hyperloglog_rs::prelude::*;

/// Number of registers for P12, the normalizer for the polynomial's load variable `t = raw / m`.
const M: f64 = 4096.0;

/// Realistic per-cell table sizes. Hash-list cells average ~72 breakpoints, register cells ~157,
/// and the largest register cells reach ~256, so this span covers both arms.
const TABLE_SIZES: [usize; 5] = [40, 72, 110, 157, 256];

/// How many corrections to evaluate per timed iteration (criterion reports per-element time via the
/// throughput below). A spread of inputs makes the binary search land at varied depths.
const BATCH: usize = 1024;

/// splitmix64 for a deterministic input spread (no external rng needed).
fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A sorted table of `n` breakpoints over the register correction domain `[lo, hi]`, with small
/// monotone biases. The exact values do not affect timing, only the size and the search depth do.
fn build_table(n: usize, lo: f64, hi: f64) -> (Vec<u32>, Vec<f64>) {
    let cards: Vec<u32> = (0..n)
        .map(|i| (lo + (hi - lo) * (i as f64) / (n as f64 - 1.0)) as u32)
        .collect();
    let biases: Vec<f64> = (0..n)
        .map(|i| -1200.0 * (1.0 - (i as f64) / (n as f64 - 1.0)))
        .collect();
    (cards, biases)
}

/// `BATCH` raw estimates spread across `(lo, hi)`, the hot interpolation range.
fn build_inputs(lo: f64, hi: f64) -> Vec<f64> {
    let mut state = 0xC0FF_EE12_3456_789A;
    (0..BATCH)
        .map(|_| lo + (hi - lo) * ((splitmix(&mut state) >> 11) as f64 / (1u64 << 53) as f64))
        .collect()
}

/// Local reimplementation of the former lookup-table correction (binary search plus linear
/// interpolation over breakpoints), kept here as the baseline now that the library ships the
/// polynomial form. Inputs stay within `[cards[0], cards[-1]]`, the hot interpolation path.
fn table_lookup(raw: f64, cards: &[u32], biases: &[f64]) -> f64 {
    let estimate = raw as u64 as u32;
    if estimate <= cards[0] {
        return raw + biases[0] * raw / f64::from(cards[0]).max(1.0);
    }
    if estimate > cards[cards.len() - 1] {
        let last = cards.len() - 1;
        return raw + biases[last] * raw / f64::from(cards[last]);
    }
    let index = cards.partition_point(|&x| x < estimate);
    let (lo_c, hi_c) = (cards[index - 1], cards[index]);
    let (lo_b, hi_b) = (biases[index - 1], biases[index]);
    raw + (raw - f64::from(lo_c)) / f64::from(hi_c - lo_c) * (hi_b - lo_b) + lo_b
}

fn bench_correction(c: &mut Criterion) {
    let (lo, hi) = (8000.0_f64, 30000.0_f64); // register correction domain at P12 (below the 7.5*m cutoff)
    let inputs = build_inputs(lo + 50.0, hi - 50.0);

    // Representative degree-8 coefficients in the load variable. Values are arbitrary but finite;
    // polynomial evaluation cost does not depend on them.
    let coeffs: [f64; 9] = [
        -0.31, 0.42, -0.18, 0.07, -0.025, 0.009, -0.003, 0.0008, -0.0002,
    ];
    let (dom_lo, dom_hi) = (lo / M, hi / M);

    let mut group = c.benchmark_group("correction");
    group.throughput(Throughput::Elements(BATCH as u64));

    for &n in &TABLE_SIZES {
        let (cards, biases) = build_table(n, lo, hi);
        group.bench_with_input(BenchmarkId::new("table_lookup", n), &n, |b, _| {
            b.iter(|| {
                let mut acc = 0.0_f64;
                for &raw in &inputs {
                    acc += table_lookup(black_box(raw), &cards, &biases);
                }
                black_box(acc)
            });
        });
    }

    // The polynomial path (the shipped library evaluation) is table-size independent; bench it once.
    let domain = [dom_lo, dom_hi];
    group.bench_function("poly8_eval", |b| {
        b.iter(|| {
            let mut acc = 0.0_f64;
            for &raw in &inputs {
                acc += correct_cardinality(black_box(raw), M, &coeffs, &domain);
            }
            black_box(acc)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_correction);
criterion_main!(benches);
