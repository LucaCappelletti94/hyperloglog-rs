//! Studies dense-register saturation in the worst case of tiny registers (`Bits4`, register cap 15).
//! With 4-bit registers the raw `alpha * m^2 / sum` estimate has a hard ceiling near `alpha * m *
//! 2^15`, and above `7.5 * 2^P` the library returns that raw estimate uncorrected (the bias polynomial
//! covers only the lower load band). The full-histogram MLE can still read partial saturation. This
//! emits per-seed `(cell, seed, n, default_est, mle_est)` so aggregation and plotting happen downstream.
//! Run: cargo run --release --example saturation_study > /tmp/sat.csv
use hyperloglog_rs::prelude::*;

#[inline]
fn mix(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^= x >> 31;
    x
}

fn checkpoints(start: f64, cap: u64, factor: f64) -> Vec<u64> {
    let mut cps = Vec::new();
    let mut c = start;
    while (c as u64) <= cap {
        let v = c as u64;
        if cps.last() != Some(&v) {
            cps.push(v);
        }
        c *= factor;
    }
    cps
}

macro_rules! run {
    ($P:ty, $B:ty, $label:expr, $cap:expr, $seeds:expr) => {{
        type Hll = HyperLogLog<$P, $B>;
        let cps = checkpoints(1000.0, $cap, 1.5);
        for seed in 0u64..$seeds {
            let mut h = Hll::default();
            h.to_hll(); // force dense registers from the start
            let base = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
            let mut n = 0u64;
            for &cp in &cps {
                while n < cp {
                    n += 1;
                    h.insert(&mix(base.wrapping_add(n)));
                }
                let default_est = h.estimate_cardinality();
                let mle_est = h.mle().estimate_cardinality();
                println!(
                    "{},{},{},{:.4},{:.4}",
                    $label, seed, cp, default_est, mle_est
                );
            }
        }
    }};
}

fn main() {
    println!("cell,seed,n,default_est,mle_est");
    run!(Precision4, Bits4, "P4B4", 3_000_000, 128);
    run!(Precision6, Bits4, "P6B4", 12_000_000, 64);
    run!(Precision8, Bits4, "P8B4", 24_000_000, 32);
    run!(Precision4, Bits6, "P4B6", 3_000_000, 128); // control: 6-bit registers never saturate in range
}
