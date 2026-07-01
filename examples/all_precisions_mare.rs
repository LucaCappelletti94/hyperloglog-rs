//! Evaluate hash-list MARE for all (P, B) configurations.
//!
//! For each supported precision (P4-P18) and register width (B4, B5, B6),
//! inserts elements until the counter is in hash list mode or reaches
//! the insert limit, then reports MARE over 256 seeds.
//!
//! The approach: for each seed, insert elements until the counter is in
//! hash list mode. If it reaches hash list, record the cardinality and
//! error. If it converts directly to HLL (value list dump overflows
//! hash list), that seed is skipped. This captures the hash-list regime
//! for configurations where it exists.
//!
//! Outputs CSV to stdout:
//!   p,b,hl_n,mare_corrected,mare_linear,samples
//!
//! Run:
//!   cargo run --release --example all_precisions_mare

use hyperloglog_rs::prelude::*;

const SEEDS: u64 = 256;
const MAX_INSERTS: u64 = 10000;

fn eval_config<P, B>() -> (u64, u64, u64, f64, f64, u64)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let p = P::EXPONENT as u64;
    let b = B::NUMBER_OF_BITS as u64;

    let (mut sum_cor, mut sum_lin) = (0.0, 0.0);
    let mut hl_n: u64 = 0;
    let mut samples: u64 = 0;

    for s in 0..SEEDS {
        let mut h: HyperLogLog<P, B> = HyperLogLog::<P, B>::default();
        let mut h_force: HyperLogLog<P, B> = HyperLogLog::<P, B>::default();
        let mut state = s.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mut n: u64 = 0;

        for _i in 0..MAX_INSERTS {
            state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
            h.insert_value(state);
            h_force.insert_value(state);
            n += 1;

            // Check mode: if hash list, record and stop
            if h.is_sorted_hash_list() {
                break;
            }
            // If HLL, this seed skipped hash list entirely
            if h.is_hyperloglog() {
                n = 0;
                break;
            }
            // Still in value list, continue
        }

        if n == 0 {
            continue;
        }

        h_force.to_hll();
        let n_f = n as f64;
        let corr = h.estimate_cardinality();
        let lin = h_force.estimate_cardinality();

        sum_cor += (corr - n_f).abs() / n_f;
        sum_lin += (lin - n_f).abs() / n_f;
        hl_n = n;
        samples += 1;
    }

    let mare_cor = if samples > 0 {
        100.0 * sum_cor / samples as f64
    } else {
        -1.0
    };
    let mare_lin = if samples > 0 {
        100.0 * sum_lin / samples as f64
    } else {
        -1.0
    };

    (p, b, hl_n, mare_cor, mare_lin, samples)
}

fn main() {
    println!("p,b,hl_n,mare_corrected,mare_linear,samples");

    macro_rules! eval_and_print {
        ($P:ty, $B:ty) => {
            let (p, b, hl_n, mare_cor, mare_lin, samples) = eval_config::<$P, $B>();
            println!(
                "{},{},{},{:.4},{:.4},{}",
                p, b, hl_n, mare_cor, mare_lin, samples
            );
        };
    }

    eval_and_print!(Precision4, Bits4);
    eval_and_print!(Precision4, Bits5);
    eval_and_print!(Precision4, Bits6);
    eval_and_print!(Precision5, Bits4);
    eval_and_print!(Precision5, Bits5);
    eval_and_print!(Precision5, Bits6);
    eval_and_print!(Precision6, Bits4);
    eval_and_print!(Precision6, Bits5);
    eval_and_print!(Precision6, Bits6);
    eval_and_print!(Precision7, Bits4);
    eval_and_print!(Precision7, Bits5);
    eval_and_print!(Precision7, Bits6);
    eval_and_print!(Precision8, Bits4);
    eval_and_print!(Precision8, Bits5);
    eval_and_print!(Precision8, Bits6);
    eval_and_print!(Precision9, Bits4);
    eval_and_print!(Precision9, Bits5);
    eval_and_print!(Precision9, Bits6);
    eval_and_print!(Precision10, Bits4);
    eval_and_print!(Precision10, Bits5);
    eval_and_print!(Precision10, Bits6);
    eval_and_print!(Precision11, Bits4);
    eval_and_print!(Precision11, Bits5);
    eval_and_print!(Precision11, Bits6);
    eval_and_print!(Precision12, Bits4);
    eval_and_print!(Precision12, Bits5);
    eval_and_print!(Precision12, Bits6);
    eval_and_print!(Precision13, Bits4);
    eval_and_print!(Precision13, Bits5);
    eval_and_print!(Precision13, Bits6);
    eval_and_print!(Precision14, Bits4);
    eval_and_print!(Precision14, Bits5);
    eval_and_print!(Precision14, Bits6);
    eval_and_print!(Precision15, Bits4);
    eval_and_print!(Precision15, Bits5);
    eval_and_print!(Precision15, Bits6);
    eval_and_print!(Precision16, Bits4);
    eval_and_print!(Precision16, Bits5);
    eval_and_print!(Precision16, Bits6);
    eval_and_print!(Precision17, Bits4);
    eval_and_print!(Precision17, Bits5);
    eval_and_print!(Precision17, Bits6);
    eval_and_print!(Precision18, Bits4);
    eval_and_print!(Precision18, Bits5);
    eval_and_print!(Precision18, Bits6);
}
