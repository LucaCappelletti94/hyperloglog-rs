//! Insertion-speed comparison for `u64` / `u32` / `u16` values, across the two explicit
//! representations (`value list` and `hash list`) of `HyperLogLog<Precision12, Bits6>`.
//!
//! The value list stores literal values gamma-coded, so its per-element bitstream (and hence the cost
//! of the in-place insertion splice) scales with the value magnitude: smaller integers pack tighter
//! and insert faster, and the list holds far more of them. The hash list hashes every value to a
//! fixed-width composite hash, so it should be nearly independent of the value type (only the hashing
//! cost differs slightly with the byte width).
//!
//! Writes `docs/value_type_insert.json` and prints a Markdown table. Render the figure with
//!   env -u PYTHONPATH uv run --isolated --no-project --python 3.12 --with matplotlib \
//!     python3 docs/make_value_type_plot.py
//! Run with:
//!   cargo run --release --example value_type_insert_bench

#![allow(clippy::clone_on_copy)]

use hyperloglog_rs::prelude::*;
use std::time::Instant;

type Hll = HyperLogLog<Precision12, Bits6>;

/// A low-discrepancy multiplicative permutation: distinct `i` give distinct, well-spread values
/// inside the masked universe (so sorted gaps are large, like genuinely random values, but without
/// the duplicates random sampling would produce in the small `u16` universe).
const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

#[inline]
fn value(i: u64, mask: u64) -> u64 {
    i.wrapping_mul(GOLDEN) & mask
}

#[inline(never)]
fn black_box(h: Hll) -> Hll {
    unsafe { std::ptr::read_volatile(&h) }
}

fn build(n: u64, mask: u64, value_list: bool) -> Hll {
    let mut h = Hll::default();
    for i in 0..n {
        let v = value(i, mask);
        if value_list {
            h.insert_value(v);
        } else {
            h.insert(&v);
        }
    }
    h
}

/// Value-list capacity for the given value width.
fn value_list_capacity(mask: u64) -> u64 {
    let mut h = Hll::default();
    let mut i = 0u64;
    loop {
        h.insert_value(value(i, mask));
        i += 1;
        if !h.is_sorted_value_list() || i > 5_000_000 {
            break;
        }
    }
    i - 1
}

fn mean_std(s: &[f64]) -> (f64, f64) {
    let n = s.len() as f64;
    let mean = s.iter().sum::<f64>() / n;
    let var = s.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    (mean, var.sqrt())
}

/// Amortized insertion cost (ns per element) at size `n`: insert a fresh `batch` into a clone of a
/// size-`n` counter, divided by `batch`; reported as mean and std over `reps` clones. The batch uses
/// a value range disjoint from the base (and across reps), so no insert is a duplicate no-op.
fn bench_insert(n: u64, mask: u64, value_list: bool, batch: u64, reps: usize) -> (f64, f64) {
    let base = build(n, mask, value_list);
    let mut samples = Vec::with_capacity(reps);
    for r in 0..reps {
        let mut h = base.clone();
        let start = n + r as u64 * batch;
        let t0 = Instant::now();
        for j in 0..batch {
            let v = value(start + j, mask);
            if value_list {
                h.insert_value(v);
            } else {
                h.insert(&v);
            }
        }
        let ns = t0.elapsed().as_nanos() as f64 / batch as f64;
        black_box(h);
        samples.push(ns);
    }
    mean_std(&samples)
}

fn main() {
    const REPS: usize = 25;
    let widths: [(&str, u64); 3] = [("u64", u64::MAX), ("u32", 0xFFFF_FFFF), ("u16", 0xFFFF)];
    let caps: Vec<(&str, u64, u64)> = widths
        .iter()
        .map(|&(name, mask)| (name, mask, value_list_capacity(mask)))
        .collect();
    for (name, _, cap) in &caps {
        eprintln!("value-list capacity for {name}: {cap}");
    }

    let cards: [u64; 19] = [
        16, 24, 32, 48, 64, 96, 128, 160, 192, 256, 384, 512, 768, 1024, 1536, 2048, 3072, 4096,
        6144,
    ];

    let mut series: Vec<serde_json::Value> = Vec::new();
    println!("\n| representation | type | cardinality | insert ns/elem | std |");
    println!("|---|---|---:|---:|---:|");

    for value_list in [true, false] {
        let repr = if value_list {
            "value_list"
        } else {
            "hash_list"
        };
        for &(name, mask, cap) in &caps {
            let mut rows: Vec<serde_json::Value> = Vec::new();
            for &card in &cards {
                // Validity: the value list must stay below its capacity (with headroom for the
                // batch); the hash list must stay below its saturation (~8000).
                let batch: u64 = if value_list {
                    if card * 100 > cap * 85 {
                        continue;
                    }
                    (cap.saturating_sub(card) / 2).clamp(2, 32)
                } else {
                    if card > 7000 {
                        continue;
                    }
                    200
                };
                let (mean, std) = bench_insert(card, mask, value_list, batch, REPS);
                println!("| {repr} | {name} | {card} | {mean:.0} | {std:.0} |");
                rows.push(serde_json::json!({ "cardinality": card, "mean": mean, "std": std }));
            }
            series.push(serde_json::json!({
                "representation": repr,
                "value_type": name,
                "capacity": cap,
                "rows": rows,
            }));
        }
    }

    let payload = serde_json::json!({
        "precision": 12,
        "bits": 6,
        "reps": REPS,
        "series": series,
    });
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/value_type_insert.json");
    std::fs::write(&path, serde_json::to_string_pretty(&payload).unwrap()).unwrap();
    eprintln!("JSON written to {}", path.display());
}
