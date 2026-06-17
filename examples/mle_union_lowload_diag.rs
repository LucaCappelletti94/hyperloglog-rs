//! Diagnostic for the low-load union MLE. For a grid of small cardinalities it builds two operands,
//! forces them into HyperLogLog registers (the artificial dense column the regime benchmark plots),
//! and measures, over many trials, the mean relative error of the union estimated by the default
//! estimator versus the 2-set MLE. The default register union applies linear counting at low load
//! (`corrected_register_cardinality`); if the MLE union is much worse at the smallest cardinalities,
//! its harmonic-sum-to-cardinality seed is skipping that correction.

use hyperloglog_rs::prelude::*;
use std::collections::HashSet;

type Hll = HyperLogLog<Precision12, Bits6>;

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

fn main() {
    const TRIALS: usize = 200;
    println!(
        "{:>6} | {:>14} | {:>14} | {:>10}",
        "card", "default MRE", "mle MRE", "mle/default"
    );

    let mut state = 0x0BAD_C0DE_DEAD_BEEFu64;
    for &card in &[16u64, 26, 42, 68, 109, 195, 450, 1154, 2956] {
        let mut default_err = 0.0;
        let mut mle_err = 0.0;
        for _ in 0..TRIALS {
            // Two overlapping random sets: half shared, a quarter exclusive to each side.
            let mut a_set = HashSet::new();
            let mut b_set = HashSet::new();
            let shared = card / 2;
            for _ in 0..shared {
                let v = splitmix64(&mut state);
                a_set.insert(v);
                b_set.insert(v);
            }
            for _ in 0..(card - shared) {
                a_set.insert(splitmix64(&mut state));
            }
            for _ in 0..(card - shared) {
                b_set.insert(splitmix64(&mut state));
            }

            let mut a: Hll = Default::default();
            for &v in &a_set {
                a.insert(&v);
            }
            let mut b: Hll = Default::default();
            for &v in &b_set {
                b.insert(&v);
            }
            let a = a.into_hll();
            let b = b.into_hll();

            let truth = a_set.union(&b_set).count() as f64;
            let def = a.estimate_union_cardinality(&b);
            let mle = a.mle().estimate_union_cardinality(&b.mle());
            default_err += (def - truth).abs() / truth;
            mle_err += (mle - truth).abs() / truth;
        }
        default_err /= TRIALS as f64;
        mle_err /= TRIALS as f64;
        println!(
            "{:>6} | {:>13.3}% | {:>13.3}% | {:>9.2}x",
            card,
            100.0 * default_err,
            100.0 * mle_err,
            mle_err / default_err
        );
    }
}
