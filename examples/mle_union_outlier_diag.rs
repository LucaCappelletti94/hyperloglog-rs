//! Reproduces the union-MLE variance spike seen at cardinality ~12111 in the regime benchmark: a
//! rare trial where the 2-set union MLE diverges and returns a wildly wrong union. It replays the
//! benchmark's exact per-trial seeding (`build_vals(card, t + 1)`), forces the dense representation,
//! and reports the worst trials by relative error of the MLE union versus the default union.

use hyperloglog_rs::prelude::*;
use std::collections::HashSet;

type Hll = HyperLogLog<Precision12, Bits6>;

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

fn build_vals(card: u64, seed: u64) -> (Vec<u64>, Vec<u64>) {
    let base = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let shared = card / 2;
    let only_b = card - shared;
    let vals_a: Vec<u64> = (0..card)
        .map(|i| splitmix64(base.wrapping_add(i)))
        .collect();
    let vals_b: Vec<u64> = (0..shared)
        .map(|i| splitmix64(base.wrapping_add(i)))
        .chain((0..only_b).map(|i| splitmix64(base.wrapping_add(card + i))))
        .collect();
    (vals_a, vals_b)
}

fn main() {
    for &card in &[7569u64, 12111, 19378] {
        let mut worst: Vec<(f64, u64, f64, f64, f64)> = Vec::new();
        for t in 1..=256u64 {
            let (va, vb) = build_vals(card, t);
            let sa: HashSet<u64> = va.iter().copied().collect();
            let sb: HashSet<u64> = vb.iter().copied().collect();
            let truth = sa.union(&sb).count() as f64;

            let mut a = Hll::default();
            for &v in &va {
                a.insert(&v);
            }
            let mut b = Hll::default();
            for &v in &vb {
                b.insert(&v);
            }
            let a = a.into_hll();
            let b = b.into_hll();

            let def = a.estimate_union_cardinality(&b);
            let mle = a.mle().estimate_union_cardinality(&b.mle());
            let mle_err = (mle - truth).abs() / truth;
            worst.push((mle_err, t, truth, def, mle));
        }
        worst.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap());
        println!(
            "=== card {card} (truth ~{:.0}) : worst 5 MLE-union trials ===",
            worst[0].2
        );
        println!(
            "{:>5} | {:>10} | {:>12} | {:>12} | {:>10}",
            "trial", "truth", "default", "mle", "mle err"
        );
        for (err, t, truth, def, mle) in worst.iter().take(5) {
            println!(
                "{t:>5} | {truth:>10.0} | {def:>12.1} | {mle:>12.1} | {:>9.2}%",
                100.0 * err
            );
        }
        println!();
    }
}
