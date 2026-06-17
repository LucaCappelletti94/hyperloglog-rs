//! Randomized accuracy sweep: 2-set joint union MLE vs the default union estimator.
//!
//! Reproduces and stresses the premise behind the generalized joint MLE work: across
//! precisions 4-12 (Bits6), many seeds, cardinalities and overlap fractions, does the joint
//! MLE union estimator beat the default register-based union estimator, and by how much.
//!
//! Run with: `cargo run --release --features mle --example mle_union_sweep`
use hyperloglog_rs::prelude::*;
use twox_hash::XxHash64;

/// One measured case: relative errors of the two estimators and the operand mode.
struct CaseResult {
    default_rel_err: f64,
    mle_rel_err: f64,
    /// True when both operands are fully-fledged HyperLogLogs (register mode).
    register_mode: bool,
}

/// Builds a counter over the half-open integer range `[start, start + count)`.
fn build<P, B>(
    start: u64,
    count: u64,
) -> HyperLogLog<P, B, <P as PackedRegister<B>>::Array, XxHash64>
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let mut hll = HyperLogLog::<P, B, <P as PackedRegister<B>>::Array, XxHash64>::default();
    for value in start..start + count {
        hll.insert(&value);
    }
    hll
}

/// Runs a single (cardinality, overlap, seed) case and returns the two relative errors.
fn run_case<P, B>(per_side: u64, overlap: f64, seed: u64) -> CaseResult
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    // Disjoint integer ranges make the exact intersection and union known by construction.
    let shared = (overlap * per_side as f64).round() as u64;
    let left_only = per_side - shared;
    let right_only = per_side - shared;

    // A seed-derived base shifts the integer ranges so different seeds hash to different
    // register layouts while keeping all elements distinct.
    let base = splitmix64(seed).wrapping_mul(4) & 0x0000_FFFF_FFFF_0000;

    let left = build::<P, B>(base, shared + left_only);
    // Right shares the first `shared` integers, then takes its own disjoint tail.
    let mut right = build::<P, B>(base, shared);
    for value in (base + shared + left_only)..(base + shared + left_only + right_only) {
        right.insert(&value);
    }

    let exact_union = (shared + left_only + right_only) as f64;

    let default_est = left.estimate_union_cardinality(&right);
    let mle_est = left.mle().estimate_union_cardinality(&right.mle());

    CaseResult {
        default_rel_err: (default_est - exact_union).abs() / exact_union,
        mle_rel_err: (mle_est - exact_union).abs() / exact_union,
        register_mode: !left.is_sorted_hash_list() && !right.is_sorted_hash_list(),
    }
}

/// Aggregated sweep statistics for one precision.
struct PrecisionSummary {
    exponent: u8,
    // All cases.
    default_all: f64,
    mle_all: f64,
    n_all: usize,
    // Register-mode-only cases (the regime the joint MLE targets).
    default_reg: f64,
    mle_reg: f64,
    n_reg: usize,
}

/// Sweeps one precision over the cardinality x overlap x seed grid.
fn sweep<P, B>() -> PrecisionSummary
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let m = 1u64 << P::EXPONENT;
    // Per-side cardinalities scaled to the register count so each precision faces a
    // proportionally comparable load and the counters reach register mode.
    let cardinalities = [m, 4 * m, 16 * m, 64 * m];
    let overlaps = [0.1_f64, 0.25, 0.5, 0.75, 0.9];
    let n_seeds = 16u64;

    let mut default_all = 0.0;
    let mut mle_all = 0.0;
    let mut n_all = 0usize;
    let mut default_reg = 0.0;
    let mut mle_reg = 0.0;
    let mut n_reg = 0usize;

    for &card in &cardinalities {
        for &overlap in &overlaps {
            for seed in 0..n_seeds {
                let r = run_case::<P, B>(card, overlap, splitmix64(seed ^ (card << 3)));
                default_all += r.default_rel_err;
                mle_all += r.mle_rel_err;
                n_all += 1;
                if r.register_mode {
                    default_reg += r.default_rel_err;
                    mle_reg += r.mle_rel_err;
                    n_reg += 1;
                }
            }
        }
    }

    PrecisionSummary {
        exponent: P::EXPONENT,
        default_all: default_all / n_all as f64,
        mle_all: mle_all / n_all as f64,
        n_all,
        default_reg: if n_reg > 0 {
            default_reg / n_reg as f64
        } else {
            f64::NAN
        },
        mle_reg: if n_reg > 0 {
            mle_reg / n_reg as f64
        } else {
            f64::NAN
        },
        n_reg,
    }
}

fn report(rows: &[PrecisionSummary]) {
    println!("\n=== All cases (any operand mode) ===");
    println!(
        "{:>4} | {:>9} | {:>9} | {:>10} | {:>5}",
        "P", "default", "MLE", "MLE/def", "n"
    );
    for r in rows {
        let ratio = r.mle_all / r.default_all;
        println!(
            "{:>4} | {:>9.5} | {:>9.5} | {:>9.1}% | {:>5}",
            r.exponent,
            r.default_all,
            r.mle_all,
            (ratio - 1.0) * 100.0,
            r.n_all
        );
    }

    println!("\n=== Register-mode-only cases (both operands fully-fledged HLLs) ===");
    println!(
        "{:>4} | {:>9} | {:>9} | {:>10} | {:>5}",
        "P", "default", "MLE", "MLE/def", "n"
    );
    for r in rows {
        let ratio = r.mle_reg / r.default_reg;
        println!(
            "{:>4} | {:>9.5} | {:>9.5} | {:>9.1}% | {:>5}",
            r.exponent,
            r.default_reg,
            r.mle_reg,
            (ratio - 1.0) * 100.0,
            r.n_reg
        );
    }
    println!("\n(MLE/def < 0% means the MLE estimator is better.)");
}

/// Prints a per-(cardinality x overlap) breakdown of the MLE/default ratio (register mode),
/// to localize where an inversion concentrates.
fn breakdown<P, B>()
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let m = 1u64 << P::EXPONENT;
    let cardinalities = [m, 4 * m, 16 * m, 64 * m];
    let overlaps = [0.1_f64, 0.25, 0.5, 0.75, 0.9];
    let n_seeds = 32u64;

    println!(
        "\n--- P{} register-mode MLE/default ratio (%, negative = MLE better) ---",
        P::EXPONENT
    );
    print!("{:>10} |", "per-side");
    for o in overlaps {
        print!(" ov={o:>4} |");
    }
    println!();
    for &card in &cardinalities {
        print!("{card:>10} |");
        for &overlap in &overlaps {
            let mut d = 0.0;
            let mut e = 0.0;
            let mut n = 0usize;
            for seed in 0..n_seeds {
                let r = run_case::<P, B>(card, overlap, splitmix64(seed ^ (card << 3) ^ 0x5151));
                if r.register_mode {
                    d += r.default_rel_err;
                    e += r.mle_rel_err;
                    n += 1;
                }
            }
            if n > 0 {
                print!(" {:>+6.1} |", (e / d - 1.0) * 100.0);
            } else {
                print!(" {:>6} |", "hash");
            }
        }
        println!();
    }
}

fn main() {
    let rows = vec![
        sweep::<Precision4, Bits6>(),
        sweep::<Precision5, Bits6>(),
        sweep::<Precision6, Bits6>(),
        sweep::<Precision7, Bits6>(),
        sweep::<Precision8, Bits6>(),
        sweep::<Precision9, Bits6>(),
        sweep::<Precision10, Bits6>(),
        sweep::<Precision11, Bits6>(),
        sweep::<Precision12, Bits6>(),
    ];
    report(&rows);

    println!("\n=== Breakdowns (localizing the small-precision behaviour) ===");
    breakdown::<Precision4, Bits6>();
    breakdown::<Precision5, Bits6>();
    breakdown::<Precision6, Bits6>();
    breakdown::<Precision8, Bits6>();
}
