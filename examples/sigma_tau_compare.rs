//! Sigma/tau accuracy and speed on dense HyperLogLog counters, plus the union merge throughput on
//! the HyperBall-shaped hot path. Sigma/tau is now the crate's default register-regime estimator
//! (replaces the shipped polynomial and the HLL++ anchor table), so this example doubles as a
//! smoke test for that transition and as the reference number source for
//! `docs/sigma_tau_measurements.txt`.
//!
//! Run with: `cargo run --release --example sigma_tau_compare`

use hyperloglog_rs::prelude::*;
use std::ops::BitOrAssign;
use std::time::Instant;
use twox_hash::XxHash64;

type Counter<P, B> = HyperLogLog<P, B, <P as PackedRegister<B>>::Array, XxHash64>;

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Builds one dense counter of `card` distinct values under a seed.
fn build<P, B>(card: u64, seed: u64) -> Counter<P, B>
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let mut hll = Counter::<P, B>::default();
    for i in 0..card {
        hll.insert(&splitmix64(seed.wrapping_add(i)));
    }
    hll.into_hll()
}

/// Mean absolute bias across a cardinality sweep, per estimator: `.estimate_cardinality()` is now
/// sigma/tau on dense counters; `.uncorrected_estimate_cardinality()` is the raw `alpha * m^2 / H`
/// with no correction. Averaging across seeds first cancels the `~1/sqrt(m)` sampling variance so
/// what's left is bias.
fn sweep<P, B>(cards: &[u64], seeds: u64)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let (mut e_unc, mut e_st, mut n) = (0.0, 0.0, 0.0);
    for &card in cards {
        let (mut s_raw, mut s_st) = (0.0, 0.0);
        for seed in 0..seeds {
            let hll: Counter<P, B> = build(card, seed.wrapping_mul(0x9E37_79B9));
            s_raw += hll.uncorrected_estimate_cardinality();
            s_st += hll.estimate_cardinality();
        }
        let k = seeds as f64;
        let truth = card as f64;
        e_unc += (s_raw / k - truth).abs() / truth;
        e_st += (s_st / k - truth).abs() / truth;
        n += 1.0;
    }
    println!(
        "P{:<2} B{}  mean |bias| over {:.0} cardinalities: uncorrected {:>7.4}%  sigma/tau {:>7.4}%",
        P::EXPONENT,
        B::NUMBER_OF_BITS,
        n,
        100.0 * e_unc / n,
        100.0 * e_st / n,
    );
}

/// Times single-counter estimators on a fixed dense counter (per-call nanoseconds).
fn timing<P, B>()
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    // Card = 8 * m sits every P above its linear-count crossover, so the counter is in dense
    // harmonic mode and every estimator reads (H, zeros) in O(1). At lower cardinalities the
    // sigma/tau O(1) path falls back to an O(m) scan for H (documented in `sigma_tau.rs`).
    let hll: Counter<P, B> = build(u64::from(1u32 << P::EXPONENT) * 8, 0xC0FFEE);
    let reps = 200_000u32;
    let bench = |f: &dyn Fn() -> f64| -> f64 {
        let start = Instant::now();
        let mut acc = 0.0;
        for _ in 0..reps {
            acc += std::hint::black_box(f());
        }
        std::hint::black_box(acc);
        start.elapsed().as_secs_f64() * 1e9 / f64::from(reps)
    };

    // The bare two-series cost, feeding the moments the counter already carries in O(1). `H` comes
    // from the public raw estimate (`P::ALPHA * m^2 / raw` inverts back to `dense_harmonic_sum`),
    // zeros from the register scan (public API).
    let m = f64::from(1u32 << P::EXPONENT);
    let raw = hll.uncorrected_estimate_cardinality();
    let harmonic_sum = P::ALPHA * m * m / raw;
    let zeros = hll.number_of_zero_registers().unwrap() as f64;

    let t_unc = bench(&|| hll.uncorrected_estimate_cardinality());
    let t_default = bench(&|| hll.estimate_cardinality());
    let t_st_packed = bench(&|| hll.sigma_tau().estimate_cardinality());
    let t_st_hist = bench(&|| hll.sigma_tau_cardinality_from_histogram());
    let t_st_bare = bench(&|| {
        hyperloglog_rs::sigma_tau::ertl_cardinality_from_moments::<P, B>(harmonic_sum, zeros, 0.0)
    });
    println!(
        "P{:<2} B{}  ns/call: uncorrected {:>6.1}  default (sigma/tau) {:>6.1}  sigma/tau O(1) {:>6.1}  sigma/tau O(m) {:>8.1}  bare series {:>6.1}",
        P::EXPONENT,
        B::NUMBER_OF_BITS,
        t_unc,
        t_default,
        t_st_packed,
        t_st_hist,
        t_st_bare,
    );
}

/// Times the union hot path (`a | &b` -> `a.merge(&b)`) on two dense counters of comparable size.
/// HyperBall spends most of its time here, so the packing-per-insert overhead added in step 4
/// MUST NOT dominate the O(m) element-wise-max scan.
fn union_timing<P, B>()
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let m = 1u64 << P::EXPONENT;
    let a_template: Counter<P, B> = build(3 * m, 0x000A_11CE);
    let b_template: Counter<P, B> = build(3 * m, 0x0000_0B0B);
    let reps = 2_000u32;
    let start = Instant::now();
    let mut acc = 0.0;
    for _ in 0..reps {
        // Clone the destination each time: `merge` is destructive, and we want the per-merge
        // cost, not the amortized cost of merging into an ever-larger register set.
        let mut dest = std::hint::black_box(a_template.clone());
        dest.bitor_assign(std::hint::black_box(&b_template));
        acc += dest.estimate_cardinality();
    }
    std::hint::black_box(acc);
    let elapsed = start.elapsed().as_secs_f64();
    let per_merge_us = elapsed * 1e6 / f64::from(reps);
    let per_register_ns = per_merge_us * 1e3 / (m as f64);
    println!(
        "P{:<2} B{}  union merge: {:>7.2} us/merge  {:>6.2} ns/register  (m = {})",
        P::EXPONENT,
        B::NUMBER_OF_BITS,
        per_merge_us,
        per_register_ns,
        m,
    );
}

fn main() {
    println!("Accuracy (mean relative error over a cardinality sweep in the corrected regime):\n");
    let cards_p10: Vec<u64> = (10u64..=29).map(|k| k * 256).collect(); // 2560 .. 7424
    sweep::<Precision10, Bits6>(&cards_p10, 256);
    let cards_p8: Vec<u64> = (21u64..=29).map(|k| k * 64).collect(); // 1344 .. 1856 (threshold 1272)
    sweep::<Precision8, Bits6>(&cards_p8, 512);
    let cards_p12: Vec<u64> = (3u64..=29).map(|k| k * 1024).collect(); // 3072 .. 29696
    sweep::<Precision12, Bits6>(&cards_p12, 128);

    println!("\nSingle-counter speed:\n");
    timing::<Precision8, Bits6>();
    timing::<Precision10, Bits6>();
    timing::<Precision12, Bits6>();
    timing::<Precision14, Bits6>();
    timing::<Precision16, Bits6>();

    println!("\nUnion (merge) speed on HyperBall-shaped counters (both 3*m):\n");
    union_timing::<Precision8, Bits6>();
    union_timing::<Precision10, Bits6>();
    union_timing::<Precision12, Bits6>();
    union_timing::<Precision14, Bits6>();
    union_timing::<Precision16, Bits6>();
}
