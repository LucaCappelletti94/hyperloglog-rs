//! Diagnostic for the low-load (linear-counting) register regime.
//!
//! For force-dense counters (`into_hll`) across a cardinality sweep, it reports, against the exact
//! cardinality: the regime the crate selects, the crate's `estimate_cardinality`, the pure linear
//! counting estimate `m * ln(m / zeros)`, and the raw `alpha * m^2 / sum`. For each it prints the
//! signed bias and the absolute relative error (mean over trials), so we can see whether linear
//! counting is being applied where it should and whether it is itself the accuracy limit.
//!
//! Run with: cargo run --release --example linear_counting_diag

use hyperloglog_rs::prelude::*;

type Hll = HyperLogLog<Precision12, Bits6>;

fn main() {
    let m = (1u64 << 12) as f64;
    let trials = 200u64;

    // Log-spaced cardinalities from very sparse to past the raw threshold.
    let mut cards: Vec<u64> = Vec::new();
    let mut c = 4u64;
    while c <= 40_000 {
        cards.push(c);
        c = (c as f64 * 1.4) as u64 + 1;
    }

    println!(
        "{:>7} | {:>20} | {:>16} | {:>16} | {:>16}",
        "card", "regime (most common)", "cardinality", "union (50% ovlp)", "linear counting"
    );
    println!(
        "{:>7} | {:>20} | {:>7} {:>8} | {:>7} {:>8} | {:>7} {:>8}",
        "", "", "bias%", "abs%", "bias%", "abs%", "bias%", "abs%"
    );

    for &card in &cards {
        let mut est_bias = 0.0;
        let mut est_abs = 0.0;
        let mut lc_bias = 0.0;
        let mut lc_abs = 0.0;
        let mut uni_bias = 0.0;
        let mut uni_abs = 0.0;
        let mut regime_counts = std::collections::HashMap::<String, u32>::new();

        for t in 0..trials {
            let base = splitmix64(0x51E_u64.wrapping_mul(t + 1));
            let dense = |range: std::ops::Range<u64>| -> Hll {
                let mut h = Hll::default();
                for i in range {
                    h.insert(&splitmix64(base.wrapping_add(i)));
                }
                h.into_hll()
            };
            // Single counter of `card`, and a 50%-overlap pair (true union 1.5x).
            let h = dense(0..card);
            let a = dense(0..card);
            let b = dense(card / 2..card / 2 + card);
            // A = [0, card), B = [card/2, card/2 + card): overlap = card - card/2, union = card + card/2.
            let true_union = (card + card / 2) as f64;

            let truth = card as f64;
            let estimate = h.estimate_cardinality();
            let zeros = h.number_of_zero_registers().unwrap() as f64;
            let lc = if zeros > 0.0 {
                m * (m / zeros).ln()
            } else {
                0.0
            };
            let union = a.estimate_union_cardinality(&b);

            *regime_counts
                .entry(format!("{:?}", h.estimation_regime()))
                .or_insert(0) += 1;

            est_bias += (estimate - truth) / truth;
            est_abs += (estimate - truth).abs() / truth;
            lc_bias += (lc - truth) / truth;
            lc_abs += (lc - truth).abs() / truth;
            uni_bias += (union - true_union) / true_union;
            uni_abs += (union - true_union).abs() / true_union;
        }

        let n = trials as f64;
        let regime = regime_counts
            .iter()
            .max_by_key(|(_, &v)| v)
            .map(|(k, _)| k.replace("HyperLogLog", ""))
            .unwrap_or_default();
        println!(
            "{:>7} | {:>20} | {:>7.2} {:>8.2} | {:>7.2} {:>8.2} | {:>7.2} {:>8.2}",
            card,
            regime,
            est_bias / n * 100.0,
            est_abs / n * 100.0,
            uni_bias / n * 100.0,
            uni_abs / n * 100.0,
            lc_bias / n * 100.0,
            lc_abs / n * 100.0,
        );
    }
}
