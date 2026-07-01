//! Validate the occupancy model against Monte Carlo simulation.
//!
//! For a grid of (n, w) pairs, computes:
//! - Predicted E[D] from the analytical occupancy model
//! - Simulated E[D] from Monte Carlo birthday paradox with composite encoding
//!
//! Outputs CSV to stdout: n,hash_bits,predicted_ed,simulated_ed,relative_error
//!
//! Run:
//!   cargo run --release --example occupancy_validation > /tmp/occupancy_validation.csv

use rayon::prelude::*;
use std::collections::HashSet;
use std::hash::{Hash, Hasher as _};

const P: u8 = 10;
const B: u8 = 6;
const M: f64 = 1024.0;

/// Compute predicted E[D] from the analytical occupancy model.
fn predicted_ed(n: f64, hash_bits: u8) -> f64 {
    let t = hash_bits - P;
    let r_max = (1u8 << B) - 1;

    if t == B {
        let mut expected = 0.0;
        for r in 1..r_max {
            let p = 2.0_f64.powi(-((P + r) as i32));
            expected += M * (1.0 - (1.0 - p).powf(n));
        }
        let p_sat = 2.0_f64.powi(-((P + r_max - 1) as i32));
        expected += M * (1.0 - (1.0 - p_sat).powf(n));
        expected
    } else {
        let mut expected = 0.0;

        for l in 0..=(B.min(t - 2)) {
            let cells = 2.0_f64.powi((t - 2 - l) as i32);
            let p = 2.0_f64.powi(-((hash_bits - 1) as i32));
            expected += M * cells * (1.0 - (1.0 - p).powf(n));
        }

        for r in (B + 2)..r_max {
            let cells = 2.0_f64.powi((t - 1 - B) as i32);
            let p = 2.0_f64.powi(-((P + r + t - 1 - B) as i32));
            expected += M * cells * (1.0 - (1.0 - p).powf(n));
        }
        let cells = 2.0_f64.powi((t - 1 - B) as i32);
        let p_sat = 2.0_f64.powi(-((P + r_max - 1 + t - 1 - B) as i32));
        expected += M * cells * (1.0 - (1.0 - p_sat).powf(n));

        expected
    }
}

/// Encode a composite hash at the given width.
fn encode_composite(h: u64, hash_bits: u8) -> u32 {
    let bucket = (h >> (64 - P)) as u32;
    let tail = (h << P) >> P;
    let t = hash_bits - P;

    if t <= B {
        let rank = tail.leading_zeros() as u8 + 1;
        let rank_capped = rank.min((1u8 << B) - 1);
        (bucket << B) | rank_capped as u32
    } else {
        let rank = tail.leading_zeros() as u8 + 1;
        let tail_bits = (tail << (64 - t)) as u32;
        if rank <= B + 1 {
            let leading = tail_bits >> (t - 2);
            (bucket << (t - 1)) | leading
        } else {
            let rank_capped = rank.min((1u8 << B) - 1);
            let residual = tail_bits >> (t - 1 - B);
            (bucket << t) | (1u32 << (t - 1)) | ((rank_capped as u32) << (t - 1 - B)) | residual
        }
    }
}

/// Simulate distinct composite count D for n elements at width w.
fn simulate_ed(n: u64, hash_bits: u8, num_trials: u64) -> f64 {
    let total: f64 = (0..num_trials)
        .into_par_iter()
        .map(|trial| {
            let mut hasher = twox_hash::XxHash64::with_seed(trial);
            let mut seen = HashSet::new();

            for i in 0..n {
                i.hash(&mut hasher);
                let h = hasher.finish();
                let composite = encode_composite(h, hash_bits);
                seen.insert(composite);
            }

            seen.len() as f64
        })
        .sum();

    total / num_trials as f64
}

fn main() {
    const TRIALS: u64 = 10_000;

    let n_values: Vec<u64> = (50..=2000).step_by(100).collect();
    let w_values: Vec<u8> = (16..=24).collect();

    println!("n,hash_bits,predicted_ed,simulated_ed,relative_error");

    for &n in &n_values {
        for &w in &w_values {
            let predicted = predicted_ed(n as f64, w);
            let simulated = simulate_ed(n, w, TRIALS);

            let rel_error = if simulated > 0.0 {
                (predicted - simulated).abs() / simulated
            } else {
                0.0
            };

            println!(
                "{},{},{:.2},{:.2},{:.6}",
                n, w, predicted, simulated, rel_error
            );
        }
    }
}
