//! Cardinality-sweep speed and quality benchmark for `HyperLogLog<Precision12, Bits6>`.
//!
//! Across a wide range of cardinalities, this measures these modalities for the estimation tasks:
//!   - `value list`:    the exact sorted-value representation (`insert_value`), where it still fits.
//!   - `hash list`:     the sorted composite-hash representation (hashed `insert`), where it fits.
//!   - `registers`:     the register array (forced via `into_hll`), the standard estimate you get
//!     without `.mle()` (linear counting / bias corrected / raw by load).
//!   - `registers MLE`: the same register array, the maximum-likelihood estimate.
//!   - `minhash`: a memory-matched `MinHash<u32, 768>` baseline (768 * 32 = 24576 bits, the same as
//!     the `Precision12, Bits6` register array), a consistency check measured for the insert, merge,
//!     and jaccard tasks only.
//!
//! The standard register estimate passes through the `linear counting`, `bias corrected`, and `raw`
//! sub-regimes as the cardinality grows (see [`EstimationRegime`]). Those boundaries are reported in
//! the JSON so the plots can shade them.
//!
//! Tasks measured:
//!   - `insert`: insertion speed (ns per element), per representation (including `minhash`).
//!   - `merge`:  the actual object union `&a | &b` (producing the merged counter), per representation
//!     (including `minhash`, whose merge is the element-wise minimum of the sketch words).
//!   - `cardinality`: `estimate_cardinality` (speed + accuracy). No MLE (it is dominated there).
//!   - `union`:  `estimate_union_cardinality`, the union CARDINALITY estimate (speed + accuracy). No
//!     MLE (the union total is essentially the default).
//!   - `jaccard`: `estimate_jaccard_index` (speed + accuracy). This is the scalar task where the MLE
//!     earns its keep, because the union error is amplified into a much larger relative error on the
//!     small overlap and the MLE estimates the union (jointly with the regions) more accurately, so
//!     `registers MLE` separates from `registers` here. It is also the comparison point against the
//!     memory-matched `minhash` baseline.
//!   - `sketch`: the `M = N = 2` joint sketch (`JointSketch::estimate`), in every representation it
//!     fits in (speed + accuracy). Its eight differential cells let the error compound, which a
//!     trivial `M = N = 1` sketch (no more information than `union` plus the marginals) would not
//!     show. On value and hash lists it dispatches to the exact set algebra (no real MLE), so MLE is
//!     a distinct estimator on register operands only. The sketch additionally reports the
//!     overlap-grid-only error (`overlap_mre`), the four intersection cells (the joint-sketch analogue
//!     of a scalar intersection).
//!
//! Every number carries a standard deviation: for speed the spread over the timing runs, for quality
//! the spread over the trials. The scalar two-operand tasks use two counters EACH of the row's
//! cardinality at 50 percent overlap (true union 1.5x, intersection 0.5x). The sketch uses eight
//! disjoint pools of `card` values, so every one of its differential cells is exactly `card`.
//!
//! Results are written to `docs/regime_benchmarks.json`. Render the figure with
//!   env -u PYTHONPATH uv run --isolated --no-project --python 3.12 --with matplotlib \
//!     python3 docs/make_regime_plots.py
//! Run with:
//!   cargo run --release --example regime_benchmarks

// The array-backed `HyperLogLog<Precision12, Bits6>` is `Copy`, but the benchmark clones counters to
// keep each measurement on a fresh one (correct also for non-`Copy` backings such as `VecHll`).
#![allow(clippy::clone_on_copy, clippy::op_ref)]

use hyperloglog_rs::prelude::*;
use minhash_rs::prelude::MinHash;
use std::collections::HashSet;
use std::time::{Duration, Instant};

type P = Precision12;
type B = Bits6;
type Hll = HyperLogLog<P, B>;

/// The scalar estimation tasks (a single number compared against a single truth). `intersection` is
/// the derived inclusion-exclusion estimate (`|A| + |B| - |A union B|`). It is where the MLE earns
/// its keep, because the union error gets amplified into a much larger relative error on the small
/// difference, and the MLE estimates that union (jointly with the regions) more accurately.
const SCALAR_OPS: [&str; 3] = ["cardinality", "union", "jaccard"];

/// The maximum-likelihood estimator is only worth measuring on the tasks where it separates from the
/// default: the small amplified quantities (jaccard here, and the joint sketch). On `cardinality` and
/// `union` the MLE is documented-dominated (slower, no more accurate), so it is skipped there.
fn op_uses_mle(op: &str) -> bool {
    op == "jaccard"
}

#[inline(never)]
fn black_box_f64(x: f64) -> f64 {
    unsafe { std::ptr::read_volatile(&x) }
}

#[inline(never)]
fn black_box_hll(x: Hll) -> Hll {
    unsafe { std::ptr::read_volatile(&x) }
}

/// Mean and population standard deviation of a sample.
fn mean_std(samples: &[f64]) -> (f64, f64) {
    let n = samples.len() as f64;
    if n == 0.0 {
        return (0.0, 0.0);
    }
    let mean = samples.iter().sum::<f64>() / n;
    let var = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    (mean, var.sqrt())
}

/// Auto-calibrating benchmark: returns the per-run nanoseconds-per-call samples (one per rep). Each
/// run grows its iteration count until it lasts at least `target`.
fn autobench<F: FnMut()>(target: Duration, reps: usize, mut f: F) -> Vec<f64> {
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
    samples
}

/// The two value sets for cardinality `card` at 50 percent overlap. The stored values are RANDOM
/// (`splitmix64` is a bijection, so distinct inputs give distinct, well-spread values), which is what
/// the value list actually has to cope with: a near-sequential stream would gap-compress so well that
/// the exact representation would survive to absurd cardinalities, overstating its capacity.
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

/// How a benchmarked counter is built. The first three force a single representation; `Hybrid` is the
/// real auto-switching counter that transitions value list -> hash list -> registers on its own.
#[derive(Clone, Copy, PartialEq)]
enum Repr {
    ValueList,
    HashList,
    Dense,
    Hybrid,
}

/// Builds one counter from `values` in the requested representation. `Hybrid` and `ValueList` both
/// feed `insert_value`; `Hybrid` then lets the counter transition naturally (no forcing), while
/// `ValueList` is only used at cardinalities small enough that it stays a value list.
fn build(values: &[u64], repr: Repr) -> Hll {
    let mut hll = Hll::default();
    match repr {
        Repr::ValueList | Repr::Hybrid => {
            for &v in values {
                hll.insert_value(v);
            }
        }
        Repr::HashList => {
            for &v in values {
                hll.insert(&v);
            }
        }
        Repr::Dense => {
            for &v in values {
                hll.insert(&v);
            }
            hll = hll.into_hll();
        }
    }
    hll
}

/// Mean and std (`{mean, std}` JSON object) of a sample.
fn stat_json(samples: &[f64]) -> serde_json::Value {
    let (mean, std) = mean_std(samples);
    serde_json::json!({ "mean": mean, "std": std })
}

/// The default-estimator value of a scalar op on `(a, b)`.
fn scalar_default(op: &str, a: &Hll, b: &Hll) -> f64 {
    match op {
        "cardinality" => a.estimate_cardinality(),
        "union" => a.estimate_union_cardinality(b),
        "intersection" => a.estimate_intersection_cardinality(b),
        "jaccard" => a.estimate_jaccard_index(b),
        _ => unreachable!(),
    }
}

/// The MLE-estimator value of a scalar op on `(a, b)`.
fn scalar_mle(op: &str, a: &Hll, b: &Hll) -> f64 {
    let (am, bm) = (a.mle(), b.mle());
    match op {
        "cardinality" => am.estimate_cardinality(),
        "union" => am.estimate_union_cardinality(&bm),
        "intersection" => am.estimate_intersection_cardinality(&bm),
        "jaccard" => am.estimate_jaccard_index(&bm),
        _ => unreachable!(),
    }
}

/// The no-linear-counting estimator value of a scalar op on `(a, b)`: the register estimate with the
/// linear-counting branch bypassed (always the bias-corrected raw estimate). Comparing this against
/// `scalar_default` measures how much linear counting contributes at low register load.
fn scalar_no_linear(op: &str, a: &Hll, b: &Hll) -> f64 {
    let (an, bn) = (a.sigma_tau(), b.sigma_tau());
    match op {
        "cardinality" => an.estimate_cardinality(),
        "union" => an.estimate_union_cardinality(&bn),
        "intersection" => an.estimate_intersection_cardinality(&bn),
        "jaccard" => an.estimate_jaccard_index(&bn),
        _ => unreachable!(),
    }
}

/// The exact ground truth of a scalar op from the two value sets.
fn scalar_truth(op: &str, set_a: &HashSet<u64>, set_b: &HashSet<u64>) -> f64 {
    match op {
        "cardinality" => set_a.len() as f64,
        "union" => set_a.union(set_b).count() as f64,
        "intersection" => set_a.intersection(set_b).count() as f64,
        "jaccard" => set_a.intersection(set_b).count() as f64 / set_a.union(set_b).count() as f64,
        _ => unreachable!(),
    }
}

/// The joint sketch is measured at `M = N = 2`, a non-trivial decomposition (`M = N = 1` carries no
/// more information than `union` plus the marginals). Its eight differential cells let the estimation
/// error compound, which is the point of measuring it.
const SK: usize = 2;

/// Builds the `M = N = 2` nested joint-sketch operands in representation `repr` at base shell size
/// `card`. Eight disjoint random pools of `card` values each are composed into the nested left and
/// right counters so that every one of the eight differential cells (four overlap, two left, two
/// right) is exactly `card`.
fn build_sketch_operands(card: u64, seed: u64, repr: Repr) -> ([Hll; SK], [Hll; SK]) {
    let base = seed.wrapping_mul(0xD1B5_4A32_D192_ED03);
    let pools: Vec<Vec<u64>> = (0..8u64)
        .map(|k| {
            (0..card)
                .map(|i| splitmix64(base.wrapping_add(k.wrapping_mul(card).wrapping_add(i))))
                .collect()
        })
        .collect();
    let from = |idx: &[usize]| -> Hll {
        let mut values: Vec<u64> = Vec::new();
        for &k in idx {
            values.extend_from_slice(&pools[k]);
        }
        build(&values, repr)
    };
    // overlap[i][j] = pool i*2+j; left_diff[i] = pool 4+i; right_diff[j] = pool 6+j.
    let l0 = from(&[0, 1, 4]);
    let l1 = from(&[0, 1, 4, 2, 3, 5]);
    let r0 = from(&[0, 2, 6]);
    let r1 = from(&[0, 2, 6, 1, 3, 7]);
    ([l0, l1], [r0, r1])
}

/// Total absolute error across the eight differential cells of the `M = N = 2` sketch (each truly
/// `card`), normalized by the true union (`8 * card`).
fn sketch_error_2x2(s: &JointSketch<SK, SK>, card: f64) -> f64 {
    let mut err = 0.0;
    for i in 0..SK {
        for j in 0..SK {
            err += (s.overlap[i][j] - card).abs();
        }
        err += (s.left_diff[i] - card).abs();
        err += (s.right_diff[i] - card).abs();
    }
    err / (8.0 * card)
}

/// Total absolute error across only the four overlap cells of the `M = N = 2` sketch (each truly
/// `card`), normalized by the true overlap mass (`4 * card`). This is the overlap-grid error, the
/// joint sketch's actual job, where the MLE's advantage over the default HLL++ inclusion-exclusion
/// shows. The whole-decomposition error above is dominated by the easy margin cells and hides it.
fn sketch_overlap_error_2x2(s: &JointSketch<SK, SK>, card: f64) -> f64 {
    let mut err = 0.0;
    for i in 0..SK {
        for j in 0..SK {
            err += (s.overlap[i][j] - card).abs();
        }
    }
    err / (4.0 * card)
}

fn rel_err(estimate: f64, truth: f64) -> f64 {
    if truth == 0.0 {
        estimate.abs()
    } else {
        (estimate - truth).abs() / truth
    }
}

fn main() {
    let m: u64 = 1 << 12;

    // Probe the representation/regime boundaries so the plots can mark them.
    let offset = 42u64.wrapping_mul(0x517C_C1B7_2722_0A95);
    let mut vprobe = Hll::default();
    let mut i = 0u64;
    loop {
        vprobe.insert_value(splitmix64(offset.wrapping_add(i)));
        i += 1;
        if !vprobe.is_sorted_value_list() || i > 5_000_000 {
            break;
        }
    }
    let value_list_capacity = i.saturating_sub(1).max(1);

    let mut dense_start = None;
    let mut linear_to_bias = None;
    let mut bias_to_raw = None;
    {
        let mut probe = Hll::default();
        let mut card = 0u64;
        loop {
            match probe.estimation_regime() {
                EstimationRegime::HyperLogLogLinearCounted if dense_start.is_none() => {
                    dense_start = Some(card)
                }
                EstimationRegime::HyperLogLogBiasCorrected if linear_to_bias.is_none() => {
                    linear_to_bias = Some(card)
                }
                EstimationRegime::HyperLogLogRaw if bias_to_raw.is_none() => {
                    bias_to_raw = Some(card);
                    break;
                }
                _ => {}
            }
            probe.insert(&offset.wrapping_add(card));
            card += 1;
            if card > 5_000_000 {
                break;
            }
        }
    }
    let dense_start = dense_start.unwrap_or(8 * m / 4);
    let raw_start = bias_to_raw.unwrap_or(8 * m);

    // Hash lists become transitional and noisy in the last few percent before saturation, so cap the
    // hash-list curve safely below the dense transition.
    let hash_list_max = dense_start * 95 / 100;

    // Log-spaced cardinality sweep with extra points near the dense transitions.
    let mut cards: Vec<u64> = Vec::new();
    let mut c = 16u64;
    while c <= 64 * m {
        cards.push(c);
        c = (c as f64 * 1.6) as u64 + 1;
    }
    for extra in [
        dense_start,
        linear_to_bias.unwrap_or(dense_start),
        raw_start,
    ] {
        cards.push(extra.saturating_sub(40).max(1));
        cards.push(extra + 40);
    }
    // Points near the top of the value-list range so its curves reach close to the value-list ->
    // hash-list transition rather than stopping well short of it.
    cards.push(value_list_capacity * 3 / 4);
    cards.push(value_list_capacity * 9 / 10);
    cards.sort_unstable();
    cards.dedup();

    let fast = Duration::from_millis(40);
    let slow = Duration::from_millis(80);
    const REPS: usize = 25;
    const TRIALS: usize = 128;
    // The 2x2 sketch builds eight pools of `card` values per operand set, so cap its cardinality to
    // keep the build affordable, and use fewer trials than the cheap scalar tasks.
    let sketch_max = 8 * m;
    const SKETCH_TRIALS: usize = 96;

    eprintln!(
        "Precision12 Bits6: value-list capacity ~{value_list_capacity}, dense at ~{dense_start}, \
         bias at ~{}, raw at ~{raw_start}; {} cardinalities",
        linear_to_bias.unwrap_or(0),
        cards.len()
    );

    let mut records: Vec<serde_json::Value> = Vec::new();

    for &card in &cards {
        let mut reprs: Vec<(&str, Repr, bool)> = Vec::new();
        // Value list applies up to 90% of its capacity (leaving headroom for the insert batch). The
        // two-operand union past 2/3 capacity overflows into a hash list, which the union estimator
        // handles, so the value-list curves can run right up near the value list -> hash list line.
        if card * 10 <= value_list_capacity * 9 {
            reprs.push(("value_list", Repr::ValueList, false));
        }
        if card < hash_list_max {
            reprs.push(("hash_list", Repr::HashList, false));
        }
        reprs.push(("dense", Repr::Dense, true));
        // The real auto-switching counter, across every cardinality, to expose transition glitches.
        // MLE is measured here too, so the figure shows the `.mle()` path you actually get on a real
        // counter as it grows (exact value list, corrected hash list, register MLE once dense).
        reprs.push(("hybrid", Repr::Hybrid, true));

        for (repr_name, repr, with_mle) in reprs {
            eprintln!("  card {card:>8}  {repr_name}");
            // The no-linear-counting variant is measured only on the forced-dense series: that series
            // is in linear counting across the whole sub-threshold range, so the gap against the
            // default is visible there. The hybrid is a hash list below the dense transition, where
            // linear counting never applies, so it would only duplicate the default.
            // The no-linear-counting series was dropped from the lineup: its only purpose was to
            // quantify linear counting's low-load contribution, and post the dense zeros-mode change it
            // forces an O(m) harmonic-sum reconstruction at low load, so it is no longer cheap.
            let with_no_linear = false;
            let (vals_a, vals_b) = build_vals(card, 42);
            let a = build(&vals_a, repr);
            let b = build(&vals_b, repr);

            // Insertion speed: amortized over a batch of fresh values into a clone. While the counter
            // is a value list the batch must stay within the remaining capacity, otherwise it would
            // saturate and convert to a hash list mid-run and contaminate the timing.
            let insert_batch: u64 = if a.is_sorted_value_list() {
                (value_list_capacity.saturating_sub(card) / 2).clamp(4, 64)
            } else {
                2000
            };
            let insert_samples: Vec<f64> = (0..REPS)
                .map(|r| {
                    let mut h = a.clone();
                    let base = card
                        .wrapping_mul(7)
                        .wrapping_add(1 + r as u64 * insert_batch);
                    let start = Instant::now();
                    for j in 0..insert_batch {
                        // Random values: a near-sequential batch would gap-compress in the value list
                        // and understate its insert cost near capacity.
                        let v = splitmix64(base.wrapping_add(j));
                        match repr {
                            Repr::ValueList | Repr::Hybrid => {
                                h.insert_value(v);
                            }
                            _ => {
                                h.insert(&v);
                            }
                        }
                    }
                    let ns = start.elapsed().as_nanos() as f64 / insert_batch as f64;
                    black_box_hll(h);
                    ns
                })
                .collect();

            // Merge speed: the actual object union, producing the merged counter.
            let merge_samples = {
                let (a, b) = (a.clone(), b.clone());
                autobench(fast, REPS, || {
                    black_box_hll(&a | &b);
                })
            };

            // Scalar-op speed (default, MLE for dense/hybrid, no-linear-counting for dense), on the
            // representative pair.
            let mut default_speed: Vec<Vec<f64>> = Vec::with_capacity(SCALAR_OPS.len());
            let mut mle_speed: Vec<Option<Vec<f64>>> = Vec::with_capacity(SCALAR_OPS.len());
            let mut no_linear_speed: Vec<Option<Vec<f64>>> = Vec::with_capacity(SCALAR_OPS.len());
            for op in SCALAR_OPS {
                default_speed.push({
                    let (a, b) = (a.clone(), b.clone());
                    autobench(fast, REPS, || {
                        black_box_f64(scalar_default(op, &a, &b));
                    })
                });
                mle_speed.push((with_mle && op_uses_mle(op)).then(|| {
                    let (a, b) = (a.clone(), b.clone());
                    autobench(slow, REPS, || {
                        black_box_f64(scalar_mle(op, &a, &b));
                    })
                }));
                no_linear_speed.push(with_no_linear.then(|| {
                    let (a, b) = (a.clone(), b.clone());
                    autobench(fast, REPS, || {
                        black_box_f64(scalar_no_linear(op, &a, &b));
                    })
                }));
            }
            // Scalar quality over independent trials: build each pair once, score every scalar task.
            let mut default_err: Vec<Vec<f64>> = vec![Vec::with_capacity(TRIALS); SCALAR_OPS.len()];
            let mut mle_err: Vec<Vec<f64>> = vec![Vec::with_capacity(TRIALS); SCALAR_OPS.len()];
            let mut no_linear_err: Vec<Vec<f64>> =
                vec![Vec::with_capacity(TRIALS); SCALAR_OPS.len()];
            for t in 0..TRIALS {
                let (va, vb) = build_vals(card, t as u64 + 1);
                let sa: HashSet<u64> = va.iter().copied().collect();
                let sb: HashSet<u64> = vb.iter().copied().collect();
                let ta = build(&va, repr);
                let tb = build(&vb, repr);
                for (idx, op) in SCALAR_OPS.iter().enumerate() {
                    let truth = scalar_truth(op, &sa, &sb);
                    default_err[idx].push(rel_err(scalar_default(op, &ta, &tb), truth));
                    if with_mle && op_uses_mle(op) {
                        mle_err[idx].push(rel_err(scalar_mle(op, &ta, &tb), truth));
                    }
                    if with_no_linear {
                        no_linear_err[idx].push(rel_err(scalar_no_linear(op, &ta, &tb), truth));
                    }
                }
            }

            // The 2x2 sketch runs in every representation it fits in: the largest operand holds six
            // pools (6 * card distinct values), so each representation is bounded by where those six
            // pools still fit. The `.mle()` sketch is measured on hash-list operands too (not just
            // registers), so the corrected all-hash-list MLE path is plotted and a re-degradation of
            // it would show in the figure; on value lists it stays the truly-exact decomposition.
            let sketch_ok = match repr {
                Repr::ValueList => 6 * card <= value_list_capacity,
                Repr::HashList => 6 * card <= hash_list_max,
                Repr::Dense | Repr::Hybrid => card <= sketch_max,
            };
            // The scalar MLE columns stay dense-only (on hash lists they reduce to the default), but
            // the joint sketch MLE differs on hash lists, so it is also measured there.
            let sketch_with_mle = with_mle || matches!(repr, Repr::HashList);
            let sketch_json = sketch_ok.then(|| {
                let (la, ra) = build_sketch_operands(card, 42, repr);
                let default_speed = autobench(fast, REPS, || {
                    black_box_f64(JointSketch::estimate(&la, &ra).union());
                });
                let mle_speed = sketch_with_mle.then(|| {
                    autobench(slow, REPS, || {
                        let lm = [la[0].mle(), la[1].mle()];
                        let rm = [ra[0].mle(), ra[1].mle()];
                        black_box_f64(JointSketch::estimate(&lm, &rm).union());
                    })
                });
                let no_linear_speed = with_no_linear.then(|| {
                    autobench(fast, REPS, || {
                        let ln = [la[0].sigma_tau(), la[1].sigma_tau()];
                        let rn = [ra[0].sigma_tau(), ra[1].sigma_tau()];
                        black_box_f64(JointSketch::estimate(&ln, &rn).union());
                    })
                });
                let mut de = Vec::with_capacity(SKETCH_TRIALS);
                let mut me = Vec::with_capacity(SKETCH_TRIALS);
                let mut ne = Vec::with_capacity(SKETCH_TRIALS);
                // Overlap-grid-only errors (the four intersection cells), the metric the
                // joint-sketch figure highlights and where the MLE separates from the default.
                let mut de_overlap = Vec::with_capacity(SKETCH_TRIALS);
                let mut me_overlap = Vec::with_capacity(SKETCH_TRIALS);
                let mut ne_overlap = Vec::with_capacity(SKETCH_TRIALS);
                for t in 0..SKETCH_TRIALS {
                    let (l, r) = build_sketch_operands(card, t as u64 + 1, repr);
                    let default_sketch = JointSketch::estimate(&l, &r);
                    de.push(sketch_error_2x2(&default_sketch, card as f64));
                    de_overlap.push(sketch_overlap_error_2x2(&default_sketch, card as f64));
                    if sketch_with_mle {
                        let lm = [l[0].mle(), l[1].mle()];
                        let rm = [r[0].mle(), r[1].mle()];
                        let mle_sketch = JointSketch::estimate(&lm, &rm);
                        me.push(sketch_error_2x2(&mle_sketch, card as f64));
                        me_overlap.push(sketch_overlap_error_2x2(&mle_sketch, card as f64));
                    }
                    if with_no_linear {
                        let ln = [l[0].sigma_tau(), l[1].sigma_tau()];
                        let rn = [r[0].sigma_tau(), r[1].sigma_tau()];
                        let no_linear_sketch = JointSketch::estimate(&ln, &rn);
                        ne.push(sketch_error_2x2(&no_linear_sketch, card as f64));
                        ne_overlap.push(sketch_overlap_error_2x2(&no_linear_sketch, card as f64));
                    }
                }
                (
                    default_speed,
                    mle_speed,
                    no_linear_speed,
                    de,
                    me,
                    ne,
                    de_overlap,
                    me_overlap,
                    ne_overlap,
                )
            });

            let op_json = |speed: &[f64], err: &[f64]| serde_json::json!({ "speed_ns": stat_json(speed), "mre": stat_json(err) });
            let mut ops_json = serde_json::Map::new();
            for (idx, op) in SCALAR_OPS.iter().enumerate() {
                let mle = mle_speed[idx].as_ref().map(|s| op_json(s, &mle_err[idx]));
                let no_linear = no_linear_speed[idx]
                    .as_ref()
                    .map(|s| op_json(s, &no_linear_err[idx]));
                ops_json.insert(
                    op.to_string(),
                    serde_json::json!({
                        "default": op_json(&default_speed[idx], &default_err[idx]),
                        "mle": mle,
                        "no_linear": no_linear,
                    }),
                );
            }
            if let Some((
                default_speed,
                mle_speed,
                no_linear_speed,
                de,
                me,
                ne,
                de_overlap,
                me_overlap,
                ne_overlap,
            )) = &sketch_json
            {
                // The sketch node carries an extra `overlap_mre` (overlap-grid-only error) beside the
                // whole-decomposition `mre` the generic `op_json` emits.
                let sketch_node = |speed: &[f64], err: &[f64], overlap_err: &[f64]| {
                    serde_json::json!({
                        "speed_ns": stat_json(speed),
                        "mre": stat_json(err),
                        "overlap_mre": stat_json(overlap_err),
                    })
                };
                let mle = mle_speed.as_ref().map(|s| sketch_node(s, me, me_overlap));
                let no_linear = no_linear_speed
                    .as_ref()
                    .map(|s| sketch_node(s, ne, ne_overlap));
                ops_json.insert(
                    "sketch".to_string(),
                    serde_json::json!({
                        "default": sketch_node(default_speed, de, de_overlap),
                        "mle": mle,
                        "no_linear": no_linear,
                    }),
                );
            }

            records.push(serde_json::json!({
                "cardinality": card,
                "representation": repr_name,
                "insert_ns": stat_json(&insert_samples),
                "merge_ns": stat_json(&merge_samples),
                "operations": ops_json,
            }));
        }

        // Memory-matched MinHash consistency check: MinHash<u32, 768> occupies 768 * 32 = 24576 bits,
        // exactly the HyperLogLog<Precision12, Bits6> register array (4096 * 6 bits). It is measured
        // for the insert and Jaccard tasks only. MinHash insert costs O(permutations) per element, so
        // it uses fewer accuracy trials than the cheap HLL ops to keep the build affordable at high
        // cardinality.
        {
            eprintln!("  card {card:>8}  minhash");
            type Mh = MinHash<u32, 768>;
            const MINHASH_TRIALS: usize = 32;
            let build_mh = |vals: &[u64]| -> Mh {
                let mut mh = Mh::new();
                for &v in vals {
                    mh.insert_with_siphashes13(v);
                }
                mh
            };
            let (vals_a, vals_b) = build_vals(card, 42);
            let base_a = build_mh(&vals_a);
            let base_b = build_mh(&vals_b);

            // Insert speed: a fresh batch into a copy (MinHash is Copy), amortized per element.
            let insert_batch = 2000u64;
            let insert_samples: Vec<f64> = (0..REPS)
                .map(|r| {
                    let mut h = base_a;
                    let base = card
                        .wrapping_mul(7)
                        .wrapping_add(1 + r as u64 * insert_batch);
                    let start = Instant::now();
                    for j in 0..insert_batch {
                        h.insert_with_siphashes13(splitmix64(base.wrapping_add(j)));
                    }
                    let ns = start.elapsed().as_nanos() as f64 / insert_batch as f64;
                    core::hint::black_box(h);
                    ns
                })
                .collect();

            // Merge speed: the union of two sketches via `|` (element-wise minimum of the words).
            let merge_samples = autobench(fast, REPS, || {
                core::hint::black_box(base_a | base_b);
            });

            // Jaccard speed on the representative pair.
            let jaccard_speed = autobench(fast, REPS, || {
                black_box_f64(base_a.estimate_jaccard_index(&base_b));
            });

            // Jaccard accuracy over independent trials, scored against the true Jaccard index.
            let mut jaccard_err = Vec::with_capacity(MINHASH_TRIALS);
            for t in 0..MINHASH_TRIALS {
                let (va, vb) = build_vals(card, t as u64 + 1);
                let sa: HashSet<u64> = va.iter().copied().collect();
                let sb: HashSet<u64> = vb.iter().copied().collect();
                let truth = sa.intersection(&sb).count() as f64 / sa.union(&sb).count() as f64;
                let ma = build_mh(&va);
                let mb = build_mh(&vb);
                jaccard_err.push(rel_err(ma.estimate_jaccard_index(&mb), truth));
            }

            let mut ops = serde_json::Map::new();
            ops.insert(
                "jaccard".to_string(),
                serde_json::json!({
                    "default": { "speed_ns": stat_json(&jaccard_speed), "mre": stat_json(&jaccard_err) },
                    "mle": serde_json::Value::Null,
                    "no_linear": serde_json::Value::Null,
                }),
            );
            records.push(serde_json::json!({
                "cardinality": card,
                "representation": "minhash",
                "insert_ns": stat_json(&insert_samples),
                "merge_ns": stat_json(&merge_samples),
                "operations": ops,
            }));
        }
    }

    let payload = serde_json::json!({
        "precision": 12,
        "bits": 6,
        "num_registers": m,
        "value_list_capacity": value_list_capacity,
        "dense_start": dense_start,
        "linear_to_bias": linear_to_bias,
        "bias_to_raw": raw_start,
        "reps": REPS,
        "trials": TRIALS,
        "records": records,
    });
    let docs_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/regime_benchmarks.json");
    std::fs::write(&docs_path, serde_json::to_string_pretty(&payload).unwrap())
        .expect("failed to write docs/regime_benchmarks.json");
    eprintln!("JSON written to {}", docs_path.display());
}
