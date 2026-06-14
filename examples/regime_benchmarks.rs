//! Regime-sweep benchmark and quality measurement for HyperLogLog<Precision12, Bits6>.
//!
//! Sweeps a range of cardinalities that land in each of the three internal representations
//! (exact, hash-list, dense), and for each measures operation speed and quality (mean relative
//! error vs a ground-truth HashSet), for both the default and MLE paths.
//!
//! Methodology notes:
//! - Every speed measurement is the median of several calibrated runs, to reject outliers.
//! - `insert` is measured amortized (a batch of fresh unique values into one counter, divided by
//!   the batch size), so it excludes the cost of cloning the counter.
//! - Every two-operand measurement (union, merge, the sketch) uses two counters EACH of the row's
//!   cardinality, built at 50 percent overlap, so the true union is about 1.5x that cardinality.
//! - The joint sketch is measured both ways: `sketch` is the default pairwise inclusion-exclusion
//!   (`JointSketch::estimate` over plain counters) and `skMLE` is the MLE-mode sketch
//!   (`JointSketch::estimate` over `.mle()` views).
//! - MLE is a register-multiplicity estimator, so it only runs when operands are dense. In the
//!   `exact` and `hash_list` regimes the `.mle()` scalar calls and the MLE-mode sketch dispatch to
//!   the exact (or near-exact hash-list) set algebra instead, so those columns coincide with the
//!   exact result there and are not really MLE. The genuine default-vs-MLE comparison is the dense
//!   regime.
//! - In the `exact` regime the two-operand sample sizes are kept small enough that the union stays
//!   within the exact capacity, so the reported `merge` cost is a like-for-like exact merge.
//!
//! Results are printed as a Markdown table and written to docs/regime_benchmarks.json. Run with:
//!   cargo run --release --example regime_benchmarks --features "std mle exact"

// The array-backed `HyperLogLog<Precision12, Bits6>` measured here is `Copy`, but the benchmark
// clones counters and merges via `&a | &b` to mirror the general usage (the same code is correct for
// non-`Copy` register backings such as `VecHll`) and to keep each measurement on a fresh counter.
#![allow(clippy::clone_on_copy, clippy::op_ref)]

use hyperloglog_rs::prelude::*;
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Precision and bits used throughout the sweep.
type P = Precision12;
type B = Bits6;
type Hll = HyperLogLog<P, B>;

/// A no-op use of a value to prevent the compiler from optimizing away calls.
#[inline(never)]
fn black_box_f64(x: f64) -> f64 {
    unsafe { std::ptr::read_volatile(&x) }
}

#[inline(never)]
fn black_box_hll(x: Hll) -> Hll {
    unsafe { std::ptr::read_volatile(&x) }
}

/// Build a counter from the given values using `insert_value`.
fn build_hll(values: &[u64]) -> Hll {
    let mut hll = Hll::default();
    for &v in values {
        hll.insert_value(v);
    }
    hll
}

/// Detect the regime of a counter.
fn regime(hll: &Hll) -> &'static str {
    if hll.is_exact() {
        "exact"
    } else if hll.is_hash_list() {
        "hash_list"
    } else {
        "dense"
    }
}

/// The median of a slice of timings.
fn median(mut samples: Vec<f64>) -> f64 {
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    samples[samples.len() / 2]
}

/// Benchmark a closure as the median, over `reps` runs, of its mean nanoseconds per call. Each run
/// auto-calibrates its iteration count to last at least `target`.
fn autobench<F: FnMut()>(target: Duration, reps: usize, mut f: F) -> f64 {
    f();
    f();
    let mut samples = Vec::with_capacity(reps);
    for _ in 0..reps {
        let mut iters = 1usize;
        let ns_per_call = loop {
            let start = Instant::now();
            for _ in 0..iters {
                f();
            }
            let elapsed = start.elapsed();
            if elapsed >= target || iters >= 4_000_000 {
                break elapsed.as_nanos() as f64 / iters as f64;
            }
            iters = (iters as f64 * target.as_nanos() as f64 / elapsed.as_nanos().max(1) as f64)
                as usize
                + 1;
            iters = iters.min(4_000_000);
        };
        samples.push(ns_per_call);
    }
    median(samples)
}

/// Measures amortized insertion cost (nanoseconds per `insert_value`) near size `base`, excluding
/// the cost of cloning the counter. Inserts `batch` fresh unique values into a clone of `base` and
/// divides by `batch`; reports the median over `reps` clones. `batch` should be small enough that
/// the counter does not change regime during the run.
fn bench_insert(base: &Hll, start_value: u64, batch: u64, reps: usize) -> f64 {
    let mut samples = Vec::with_capacity(reps);
    let mut v = start_value;
    for _ in 0..reps {
        let mut h = base.clone();
        let start = Instant::now();
        for j in 0..batch {
            h.insert_value(v.wrapping_add(j));
        }
        let elapsed = start.elapsed();
        black_box_hll(h);
        samples.push(elapsed.as_nanos() as f64 / batch as f64);
        v = v.wrapping_add(batch);
    }
    median(samples)
}

/// Build representative value lists for a target cardinality `card`.
/// `vals_a` covers `card` distinct values, `vals_b` shares the first half then uses a disjoint range.
fn build_vals(card: u64, seed: u64) -> (Vec<u64>, Vec<u64>) {
    let offset = seed.wrapping_mul(0x517C_C1B7_2722_0A95);
    let shared = card / 2;
    let only_b = card - shared;
    let vals_a: Vec<u64> = (0..card).map(|i| offset.wrapping_add(i)).collect();
    let vals_b: Vec<u64> = (0..shared)
        .map(|i| offset.wrapping_add(i))
        .chain((0..only_b).map(|i| offset.wrapping_add(card + i)))
        .collect();
    (vals_a, vals_b)
}

/// Mean relative error of each estimator over `n_trials` independent runs. The MLE fields are
/// `None` outside the dense regime, where no MLE actually runs (the `.mle()` paths dispatch to the
/// exact or near-exact set algebra), so they are not measured or reported there.
struct Quality {
    /// Default `estimate_cardinality` MRE.
    def_card: f64,
    /// Default `estimate_union_cardinality` MRE.
    def_union: f64,
    /// Default pairwise sketch intersection MRE (`JointSketch::estimate` over plain counters).
    def_inter: f64,
    /// MLE `estimate_union_cardinality` MRE (dense only).
    mle_union: Option<f64>,
    /// MLE joint sketch intersection MRE (`JointSketch::estimate` over `.mle()` views, dense only).
    mle_inter: Option<f64>,
}

fn measure_quality(target_card: u64, n_trials: usize, measure_mle: bool) -> Quality {
    let mut def_card = 0.0f64;
    let mut def_union = 0.0f64;
    let mut def_inter = 0.0f64;
    let mut mle_union = 0.0f64;
    let mut mle_inter = 0.0f64;
    let mut inter_count = 0usize;

    for trial in 0..n_trials {
        let (vals_a, vals_b) = build_vals(target_card, trial as u64);

        let set_a: HashSet<u64> = vals_a.iter().copied().collect();
        let set_b: HashSet<u64> = vals_b.iter().copied().collect();
        let true_card_a = set_a.len() as f64;
        let true_union = set_a.union(&set_b).count() as f64;
        let true_inter = set_a.intersection(&set_b).count() as f64;

        let hll_a = build_hll(&vals_a);
        let hll_b = build_hll(&vals_b);

        let est_card = hll_a.estimate_cardinality();
        let est_union = hll_a.estimate_union_cardinality(&hll_b);
        // The default (non-MLE) joint sketch's intersection cell.
        let sketch_def_inter = JointSketch::estimate(&[hll_a], &[hll_b]).overlap[0][0];

        if true_card_a > 0.0 {
            def_card += (est_card - true_card_a).abs() / true_card_a;
        }
        if true_union > 0.0 {
            def_union += (est_union - true_union).abs() / true_union;
        }
        if true_inter > 0.0 {
            def_inter += (sketch_def_inter - true_inter).abs() / true_inter;
            inter_count += 1;
        }

        // MLE only runs on dense operands; skip it entirely elsewhere.
        if measure_mle {
            let mle_u = hll_a.mle().estimate_union_cardinality(&hll_b.mle());
            let sketch_mle_inter =
                JointSketch::estimate(&[hll_a.mle()], &[hll_b.mle()]).overlap[0][0];
            if true_union > 0.0 {
                mle_union += (mle_u - true_union).abs() / true_union;
            }
            if true_inter > 0.0 {
                mle_inter += (sketch_mle_inter - true_inter).abs() / true_inter;
            }
        }
    }

    let n = n_trials as f64;
    let inter_n = if inter_count > 0 {
        inter_count as f64
    } else {
        1.0
    };
    Quality {
        def_card: def_card / n,
        def_union: def_union / n,
        def_inter: def_inter / inter_n,
        mle_union: measure_mle.then_some(mle_union / n),
        mle_inter: measure_mle.then_some(mle_inter / inter_n),
    }
}

fn main() {
    // Determine the actual switch points empirically by inserting one element at a time. Use the
    // same value sequence the sweep's left operand uses (the seed-42 offset), so the detected
    // thresholds match where the benchmarked counters actually transition and the chosen hash-list
    // sample cardinalities land reliably in the hash-list regime.
    println!("=== Empirical switch-point detection (Precision12, Bits6) ===");
    let probe_offset = 42u64.wrapping_mul(0x517C_C1B7_2722_0A95);
    let mut probe = Hll::default();
    let mut exact_to_hash_list: Option<u64> = None;
    let mut hash_list_to_dense: Option<u64> = None;
    {
        let mut i: u64 = 0;
        loop {
            let was_exact = probe.is_exact();
            let was_hash_list = probe.is_hash_list();
            probe.insert_value(probe_offset.wrapping_add(i));
            if was_exact && probe.is_hash_list() && exact_to_hash_list.is_none() {
                exact_to_hash_list = Some(i);
                println!("  exact -> hash_list transition at insert #{i}");
            }
            if (was_hash_list || was_exact) && probe.is_dense() && hash_list_to_dense.is_none() {
                hash_list_to_dense = Some(i);
                println!("  hash_list -> dense transition at insert #{i}");
                break;
            }
            i += 1;
            if i > 500_000 {
                println!("  ERROR: dense not reached after 500k inserts");
                break;
            }
        }
    }
    let exact_end = exact_to_hash_list.unwrap_or(0);
    let dense_start = hash_list_to_dense.unwrap_or(100_000);

    println!(
        "  exact mode: 0..{exact_end} elements; hash-list: {exact_end}..{dense_start}; dense: {dense_start}.."
    );

    let m: u64 = 1 << 12; // 4096 registers.

    // Exact-mode samples. Keep the largest small enough that a 50%-overlap union (1.5x the
    // cardinality) still fits the exact capacity, so the `merge` numbers are a like-for-like exact
    // merge rather than one that overflows into the hash list partway through.
    let exact_union_safe = (exact_end * 2) / 3; // 1.5x of this stays under exact_end.
    let e_small: u64 = 1;
    let e_mid: u64 = exact_union_safe / 2;
    let e_near: u64 = exact_union_safe.saturating_sub(50);

    // Hash-list samples. The band is narrow when the exact feature is on, so stay a little inside
    // both boundaries (right at the exact->hash-list edge a mixed-mode union path skews the timing).
    let h_band = dense_start - exact_end;
    let h_low: u64 = exact_end + h_band / 4;
    let h_high: u64 = dense_start - h_band / 4;

    // Dense samples: just past the boundary, then 4x, 16x, 64x the register count.
    let d_just: u64 = dense_start + 200;
    let d_4x: u64 = 4 * m;
    let d_16x: u64 = 16 * m;
    let d_64x: u64 = 64 * m;

    let sweep_cards: &[u64] = &[
        e_small, e_mid, e_near, h_low, h_high, d_just, d_4x, d_16x, d_64x,
    ];

    let target_fast = Duration::from_millis(50);
    let target_slow = Duration::from_millis(100);
    const REPS: usize = 5;
    const QUALITY_TRIALS: usize = 100;

    let mut json_rows: Vec<serde_json::Value> = Vec::new();

    println!("\n=== Benchmarks (Precision12 = 4096 registers, Bits6) ===");
    println!("(MRE = mean relative error, ns = nanoseconds per call, median of {REPS} runs)\n");
    println!("(each operand has the row's cardinality, built at 50 percent overlap)");
    println!("(MLE only runs on dense operands; it never runs in exact/hash-list, so the MLE columns are n/a there)\n");
    println!(
        "| {:>10} | {:>9} | {:>9} | {:>9} | {:>9} | {:>9} | {:>10} | {:>11} | {:>11} | {:>11} | {:>11} | {:>14} | {:>14} |",
        "card",
        "regime",
        "ins(ns)",
        "card(ns)",
        "union(ns)",
        "merge(ns)",
        "sketch(ns)",
        "mleUnion(ns)",
        "skMLE(ns)",
        "defCardMRE",
        "defUnionMRE",
        "sketchInterMRE",
        "skMLEinterMRE"
    );
    println!(
        "|{:-<12}|{:-<11}|{:-<11}|{:-<11}|{:-<11}|{:-<11}|{:-<12}|{:-<13}|{:-<13}|{:-<13}|{:-<13}|{:-<16}|{:-<16}|",
        "", "", "", "", "", "", "", "", "", "", "", "", ""
    );

    for &card in sweep_cards {
        if card == 0 {
            continue;
        }

        let (vals_a, vals_b) = build_vals(card, 42);
        let hll_a = build_hll(&vals_a);
        let hll_b = build_hll(&vals_b);
        let reg = regime(&hll_a);

        // Amortized insert (clone excluded). Use a small batch in exact mode so the regime holds.
        let insert_batch = if reg == "exact" { 200 } else { 4000 };
        let insert_ns = bench_insert(
            &hll_a,
            card.wrapping_mul(7).wrapping_add(1),
            insert_batch,
            REPS,
        );

        let est_card_ns = {
            let h = hll_a.clone();
            autobench(target_fast, REPS, || {
                black_box_f64(h.estimate_cardinality());
            })
        };

        let est_union_ns = {
            let ha = hll_a.clone();
            let hb = hll_b.clone();
            autobench(target_fast, REPS, || {
                black_box_f64(ha.estimate_union_cardinality(&hb));
            })
        };

        let merge_ns = {
            let ha = hll_a.clone();
            let hb = hll_b.clone();
            autobench(target_fast, REPS, || {
                black_box_hll(&ha | &hb);
            })
        };

        // The non-MLE joint sketch: pairwise inclusion-exclusion over plain counters.
        let sketch_def_ns = {
            let ha = hll_a.clone();
            let hb = hll_b.clone();
            autobench(target_fast, REPS, || {
                black_box_f64(JointSketch::estimate(&[ha], &[hb]).union());
            })
        };

        // MLE only runs on dense operands, so the MLE speeds are measured only there (in the exact
        // and hash-list regimes the `.mle()` paths dispatch to the exact set algebra, not MLE).
        let is_dense = reg == "dense";
        let mle_union_ns = is_dense.then(|| {
            let ha = hll_a.clone();
            let hb = hll_b.clone();
            autobench(target_slow, REPS, || {
                black_box_f64(ha.mle().estimate_union_cardinality(&hb.mle()));
            })
        });
        let sketch_mle_ns = is_dense.then(|| {
            let ha = hll_a.clone();
            let hb = hll_b.clone();
            autobench(target_slow, REPS, || {
                black_box_f64(JointSketch::estimate(&[ha.mle()], &[hb.mle()]).union());
            })
        });

        let q = measure_quality(card, QUALITY_TRIALS, is_dense);

        let opt_ns = |v: Option<f64>| v.map_or_else(|| "n/a".to_string(), |x| format!("{x:.1}"));
        let opt_pct =
            |v: Option<f64>| v.map_or_else(|| "n/a".to_string(), |x| format!("{:.3}%", x * 100.0));
        println!(
            "| {:>10} | {:>9} | {:>9.1} | {:>9.1} | {:>9.1} | {:>9.1} | {:>10.1} | {:>11} | {:>11} | {:>10.3}% | {:>10.3}% | {:>13.3}% | {:>14} |",
            card,
            reg,
            insert_ns,
            est_card_ns,
            est_union_ns,
            merge_ns,
            sketch_def_ns,
            opt_ns(mle_union_ns),
            opt_ns(sketch_mle_ns),
            q.def_card * 100.0,
            q.def_union * 100.0,
            q.def_inter * 100.0,
            opt_pct(q.mle_inter)
        );

        json_rows.push(serde_json::json!({
            "cardinality": card,
            "regime": reg,
            "insert_ns": insert_ns,
            "est_card_ns": est_card_ns,
            "est_union_ns": est_union_ns,
            "merge_ns": merge_ns,
            "sketch_def_ns": sketch_def_ns,
            "mle_union_ns": mle_union_ns,
            "sketch_mle_ns": sketch_mle_ns,
            "default_card_mre": q.def_card,
            "default_union_mre": q.def_union,
            "default_inter_mre": q.def_inter,
            "mle_union_mre": q.mle_union,
            "sketch_mle_inter_mre": q.mle_inter,
        }));
    }

    println!("\n### Switch points\n");
    println!("- exact -> hash_list: cardinality at which exact mode ends = **{exact_end}**");
    println!("- hash_list -> dense: cardinality at which dense mode starts = **{dense_start}**");

    let json_payload = serde_json::json!({
        "precision": 12,
        "bits": 6,
        "num_registers": 1u64 << 12,
        "switch_exact_to_hash_list": exact_end,
        "switch_hash_list_to_dense": dense_start,
        "rows": json_rows,
    });
    let docs_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/regime_benchmarks.json");
    std::fs::write(
        &docs_path,
        serde_json::to_string_pretty(&json_payload).unwrap(),
    )
    .expect("failed to write docs/regime_benchmarks.json");
    println!("\nJSON written to {}", docs_path.display());
}
