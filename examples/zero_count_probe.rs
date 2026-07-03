//! Measures the maximum number of zero registers a counter has while it is in the dense
//! bias-corrected (harmonic) regime, per `(P, B)`, by binary-searching each cell's linear-counting
//! crossover (where that count is maximal) and sampling the zero count just above it over several
//! seeds. Reports the count and the number of bits it needs, so `ZERO_BITS[P-4][B-4]` can be frozen.
//!
//! Run with: `cargo run --release --example zero_count_probe`

use hyperloglog_rs::prelude::*;
use twox_hash::XxHash64;

type Counter<P, B> = HyperLogLog<P, B, <P as PackedRegister<B>>::Array, XxHash64>;

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// A forced-dense counter of `card` distinct values under `seed`.
fn dense<P, B>(card: u64, seed: u64) -> Counter<P, B>
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let mut hll = Counter::<P, B>::default();
    for i in 0..card {
        hll.insert(&splitmix64(seed.wrapping_mul(0x9E37_79B9).wrapping_add(i)));
    }
    hll.into_hll()
}

/// True once the forced-dense counter has left the low-load linear-counting (zeros) regime, i.e. it is
/// bias-corrected or raw. Regime is monotone in cardinality, so this is binary-searchable.
fn past_crossover<P, B>(card: u64, seed: u64) -> bool
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    !matches!(
        dense::<P, B>(card, seed).estimation_regime(),
        EstimationRegime::HyperLogLogLinearCounted
    )
}

fn probe<P, B>() -> u32
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let m = 1u64 << P::EXPONENT;
    // Binary-search the smallest cardinality that is past the linear-counting crossover (seed 0), then
    // sample the zero count in a small window just above it over several seeds and take the maximum.
    let (mut lo, mut hi) = (1u64, 8 * m);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if past_crossover::<P, B>(mid, 0) {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    let crossover = lo;
    let mut max_zeros = 0u32;
    // A small window at and just above the crossover, where the zero count is largest.
    for step in 0..8u64 {
        let card = crossover + step * (m / 64).max(1);
        for seed in 0..8u64 {
            let hll = dense::<P, B>(card, seed);
            if hll.estimation_regime() == EstimationRegime::HyperLogLogBiasCorrected {
                if let Ok(z) = hll.number_of_zero_registers() {
                    max_zeros = max_zeros.max(z as u32);
                }
            }
        }
    }
    let bits = if max_zeros == 0 {
        0
    } else {
        32 - (max_zeros).leading_zeros()
    };
    println!(
        "P{:<2} B{}  m={:>7}  crossover~{:>8}  max zeros in band = {:>6}  needs {:>2} bits",
        P::EXPONENT,
        B::NUMBER_OF_BITS,
        m,
        crossover,
        max_zeros,
        bits
    );
    bits
}

fn main() {
    println!("Max zero-register count and bit width in the dense bias-corrected band:\n");
    let mut max_bits = 0u32;
    macro_rules! run {
        ($p:ty) => {{
            max_bits = max_bits.max(probe::<$p, Bits4>());
            max_bits = max_bits.max(probe::<$p, Bits5>());
            max_bits = max_bits.max(probe::<$p, Bits6>());
        }};
    }
    run!(Precision4);
    run!(Precision5);
    run!(Precision6);
    run!(Precision7);
    run!(Precision8);
    run!(Precision9);
    run!(Precision10);
    run!(Precision11);
    run!(Precision12);
    run!(Precision13);
    run!(Precision14);
    run!(Precision15);
    run!(Precision16);
    run!(Precision17);
    run!(Precision18);
    println!("\nMax bits across P4..P18, B4..B6: {max_bits}");
}
