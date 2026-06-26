//! Map the generalized joint MLE against the repeated 2-set MLE across the full M x N grid.
//!
//! This sweeps every shape `M` in `1..=8` by `N` in `1..=8` (all 64 combinations, the headline
//! corner being `M=N=8`) at precisions P4, P6, P8, P10 with Bits6. For each shape it builds nested
//! power-law deep-cell counters (a few large shallow cells and many tiny deep cells, the regime where
//! pairwise inclusion-exclusion cancels deep overlaps into noise), exactly as the test helper
//! `build_power_law_cells` does, with the base scaled per precision so the counters stay dense and the
//! deepest cell stays non-trivial. For each seed it computes the mean per-cell relative error (each
//! cell normalized by its own true cardinality) of three estimators: the joint MLE via `.jmle()`, the
//! repeated 2-set MLE via `.mle()`, and the default bare-counter pairwise sketch. It also times each
//! of the three solves (one wall-clock per method). The per-shape rows are averaged over the seeds,
//! and the headline 8x8 jMLE solve cost per precision is echoed to stderr.
//!
//! The work is parallelized over the flat list of (precision, M, N, seed) units with rayon, so it
//! saturates all cores on a many-core server. The per-unit seeds are derived deterministically from a
//! splitmix of (precision tag, M, N, seed index), so a given seed count reproduces the same numbers.
//!
//! Output is CSV on stdout (header plus one row per precision, M, N). Progress goes to stderr only, so
//! stdout stays clean CSV. The optional first CLI argument is the seed count (default 32).
//!
//! Run command (NUMBER_OF_SEEDS optional, default 32):
//!   cargo run --release --example jmle_grid_sweep -- NUMBER_OF_SEEDS > grid.csv
//! For example, 32 seeds across all cores:
//!   cargo run --release --example jmle_grid_sweep -- 32 > grid.csv

use hyperloglog_rs::prelude::*;
use rayon::prelude::*;
use std::time::Instant;
use twox_hash::XxHash64;

type Counter<P, B> = HyperLogLog<P, B, <P as PackedRegister<B>>::Array, XxHash64>;

/// Local copy of the splitmix64 mixing step, so per-unit seeds are derived deterministically from
/// (precision tag, M, N, seed index) without depending on a crate-private helper.
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Builds one dense counter from disjoint integer ranges (each `(start, count)`).
fn build<P, B>(ranges: &[(u64, u64)]) -> Counter<P, B>
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let mut hll = Counter::<P, B>::default();
    for &(start, count) in ranges {
        for v in start..start + count {
            hll.insert(&v);
        }
    }
    hll
}

/// Builds nested left and right power-law deep-cell counters and the exact cell cardinalities, the
/// same layout as the test helper `build_power_law_cells`. The overlap cell at depth `i + j` gets a
/// smaller exponent on average, so deeper cells are tiny. Returns `(lefts, rights, exact)` where
/// `exact` is indexed overlap `i*N + j`, left margin `n_overlap + i`, right margin `n_overlap + M + j`.
#[allow(clippy::type_complexity)]
fn build_power_law_cells<P, B, const M: usize, const N: usize>(
    state: &mut u64,
    base: u64,
) -> ([Counter<P, B>; M], [Counter<P, B>; N], Vec<f64>)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let n_overlap = M * N;
    let k = n_overlap + M + N;
    let mut exact = vec![0.0_f64; k];
    let mut cursor = 0u64;
    let mut ranges_o = [[(0u64, 0u64); N]; M];
    for i in 0..M {
        for j in 0..N {
            *state = splitmix64(*state);
            let depth = (i + j) as u32;
            let e = (*state % 4) as u32;
            let exponent = e.saturating_sub(depth.min(3));
            let count = base * (1 << exponent);
            ranges_o[i][j] = (cursor, count);
            exact[i * N + j] = count as f64;
            cursor += count;
        }
    }
    let mut ranges_da = [(0u64, 0u64); M];
    for i in 0..M {
        *state = splitmix64(*state);
        let count = base * (1 + (*state % 4));
        ranges_da[i] = (cursor, count);
        exact[n_overlap + i] = count as f64;
        cursor += count;
    }
    let mut ranges_db = [(0u64, 0u64); N];
    for j in 0..N {
        *state = splitmix64(*state);
        let count = base * (1 + (*state % 4));
        ranges_db[j] = (cursor, count);
        exact[n_overlap + M + j] = count as f64;
        cursor += count;
    }
    let lefts: [Counter<P, B>; M] = core::array::from_fn(|i| {
        let mut ranges = Vec::new();
        for ii in 0..=i {
            for j in 0..N {
                ranges.push(ranges_o[ii][j]);
            }
            ranges.push(ranges_da[ii]);
        }
        build::<P, B>(&ranges)
    });
    let rights: [Counter<P, B>; N] = core::array::from_fn(|j| {
        let mut ranges = Vec::new();
        for jj in 0..=j {
            for i in 0..M {
                ranges.push(ranges_o[i][jj]);
            }
            ranges.push(ranges_db[jj]);
        }
        build::<P, B>(&ranges)
    });
    (lefts, rights, exact)
}

/// Mean per-cell relative error of a decomposition against the exact cells (each cell normalized by
/// its own true value, so tiny deep cells carry full weight).
fn mean_relative_cell_error<const M: usize, const N: usize>(
    sketch: &JointSketch<M, N>,
    exact: &[f64],
) -> f64 {
    let n_overlap = M * N;
    let mut total = 0.0;
    let mut cells = 0.0;
    let mut add = |got: f64, truth: f64| {
        total += (got - truth).abs() / truth.max(1.0);
        cells += 1.0;
    };
    for i in 0..M {
        for j in 0..N {
            add(sketch.overlap[i][j], exact[i * N + j]);
        }
    }
    for i in 0..M {
        add(sketch.left_diff[i], exact[n_overlap + i]);
    }
    for j in 0..N {
        add(sketch.right_diff[j], exact[n_overlap + M + j]);
    }
    total / cells
}

/// The per-seed measurement for one (precision, M, N) shape: the three mean per-cell errors and the
/// three solve wall-clocks (milliseconds), one per estimator.
#[derive(Clone, Copy)]
struct SeedResult {
    jmle_err: f64,
    two_set_err: f64,
    default_err: f64,
    jmle_solve_ms: f64,
    two_set_solve_ms: f64,
    default_solve_ms: f64,
}

/// Runs one seed of one shape: builds the counters, then scores jMLE, the repeated 2-set MLE, and the
/// default bare-counter sketch. The per-seed state is derived from `unit_seed` so the run is
/// reproducible. `base` keeps the cells dense at this precision.
fn run_one_seed<P, B, const M: usize, const N: usize>(unit_seed: u64, base: u64) -> SeedResult
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let mut state = unit_seed;
    let (lefts, rights, exact) = build_power_law_cells::<P, B, M, N>(&mut state, base);

    // Each estimator is built and timed separately so the table carries a wall-clock per method, not
    // just the jMLE. The `.jmle()`/`.mle()` views are cheap borrows, so the timed work is the joint
    // optimization (jMLE), the repeated 2-set union MLE (`.mle()`), and the inclusion-exclusion over
    // the bare counters (default HyperLogLog++).
    let start = Instant::now();
    let jmle_l: [JointMle<&Counter<P, B>>; M] = core::array::from_fn(|i| lefts[i].jmle());
    let jmle_r: [JointMle<&Counter<P, B>>; N] = core::array::from_fn(|j| rights[j].jmle());
    let jmle = JointSketch::estimate(&jmle_l, &jmle_r);
    let jmle_solve_ms = start.elapsed().as_secs_f64() * 1e3;

    let start = Instant::now();
    let mle_l: [Mle<&Counter<P, B>>; M] = core::array::from_fn(|i| lefts[i].mle());
    let mle_r: [Mle<&Counter<P, B>>; N] = core::array::from_fn(|j| rights[j].mle());
    let mle = JointSketch::estimate(&mle_l, &mle_r);
    let two_set_solve_ms = start.elapsed().as_secs_f64() * 1e3;

    let start = Instant::now();
    let default = JointSketch::estimate(&lefts, &rights);
    let default_solve_ms = start.elapsed().as_secs_f64() * 1e3;

    SeedResult {
        jmle_err: mean_relative_cell_error(&jmle, &exact),
        two_set_err: mean_relative_cell_error(&mle, &exact),
        default_err: mean_relative_cell_error(&default, &exact),
        jmle_solve_ms,
        two_set_solve_ms,
        default_solve_ms,
    }
}

/// A unit of parallel work: one (precision, M, N, seed index) cell. The runner is a monomorphized
/// function pointer selected at build time for the precision and the shape.
struct WorkUnit {
    precision: u8,
    m: usize,
    n: usize,
    base: u64,
    seed_index: usize,
    run: fn(u64, u64) -> SeedResult,
}

/// A deterministic per-unit seed from the shape coordinates and the seed index.
fn unit_seed(precision: u8, m: usize, n: usize, seed_index: usize) -> u64 {
    let mut s = 0x5EED_0000_0000_0000_u64 ^ u64::from(precision);
    s = splitmix64(s ^ ((m as u64) << 32));
    s = splitmix64(s ^ ((n as u64) << 16));
    splitmix64(s ^ seed_index as u64)
}

/// Per-precision dense base, roughly `2^(P-2)`, so the cells stay dense and the deepest cell stays
/// non-trivial across the grid (mirroring the precision-scaling diagnostic).
fn base_for_precision(precision: u8) -> u64 {
    1u64 << (precision.saturating_sub(2))
}

fn main() {
    let seeds: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(32);

    // Build the flat list of work units. The nested macros enumerate the compile-time generics: an
    // outer macro over the precision types, an inner macro over the (M, N) shape pairs.
    let mut units: Vec<WorkUnit> = Vec::new();

    macro_rules! push_shape {
        ($precision_type:ty, $precision_tag:expr, $m:expr, $n:expr) => {{
            let base = base_for_precision($precision_tag);
            for seed_index in 0..seeds {
                units.push(WorkUnit {
                    precision: $precision_tag,
                    m: $m,
                    n: $n,
                    base,
                    seed_index,
                    run: run_one_seed::<$precision_type, Bits6, $m, $n>,
                });
            }
        }};
    }

    // The 64 shapes for one precision. Listed explicitly because M and N are const generics.
    macro_rules! push_all_shapes {
        ($precision_type:ty, $precision_tag:expr) => {{
            push_shape!($precision_type, $precision_tag, 1, 1);
            push_shape!($precision_type, $precision_tag, 1, 2);
            push_shape!($precision_type, $precision_tag, 1, 3);
            push_shape!($precision_type, $precision_tag, 1, 4);
            push_shape!($precision_type, $precision_tag, 1, 5);
            push_shape!($precision_type, $precision_tag, 1, 6);
            push_shape!($precision_type, $precision_tag, 1, 7);
            push_shape!($precision_type, $precision_tag, 1, 8);
            push_shape!($precision_type, $precision_tag, 2, 1);
            push_shape!($precision_type, $precision_tag, 2, 2);
            push_shape!($precision_type, $precision_tag, 2, 3);
            push_shape!($precision_type, $precision_tag, 2, 4);
            push_shape!($precision_type, $precision_tag, 2, 5);
            push_shape!($precision_type, $precision_tag, 2, 6);
            push_shape!($precision_type, $precision_tag, 2, 7);
            push_shape!($precision_type, $precision_tag, 2, 8);
            push_shape!($precision_type, $precision_tag, 3, 1);
            push_shape!($precision_type, $precision_tag, 3, 2);
            push_shape!($precision_type, $precision_tag, 3, 3);
            push_shape!($precision_type, $precision_tag, 3, 4);
            push_shape!($precision_type, $precision_tag, 3, 5);
            push_shape!($precision_type, $precision_tag, 3, 6);
            push_shape!($precision_type, $precision_tag, 3, 7);
            push_shape!($precision_type, $precision_tag, 3, 8);
            push_shape!($precision_type, $precision_tag, 4, 1);
            push_shape!($precision_type, $precision_tag, 4, 2);
            push_shape!($precision_type, $precision_tag, 4, 3);
            push_shape!($precision_type, $precision_tag, 4, 4);
            push_shape!($precision_type, $precision_tag, 4, 5);
            push_shape!($precision_type, $precision_tag, 4, 6);
            push_shape!($precision_type, $precision_tag, 4, 7);
            push_shape!($precision_type, $precision_tag, 4, 8);
            push_shape!($precision_type, $precision_tag, 5, 1);
            push_shape!($precision_type, $precision_tag, 5, 2);
            push_shape!($precision_type, $precision_tag, 5, 3);
            push_shape!($precision_type, $precision_tag, 5, 4);
            push_shape!($precision_type, $precision_tag, 5, 5);
            push_shape!($precision_type, $precision_tag, 5, 6);
            push_shape!($precision_type, $precision_tag, 5, 7);
            push_shape!($precision_type, $precision_tag, 5, 8);
            push_shape!($precision_type, $precision_tag, 6, 1);
            push_shape!($precision_type, $precision_tag, 6, 2);
            push_shape!($precision_type, $precision_tag, 6, 3);
            push_shape!($precision_type, $precision_tag, 6, 4);
            push_shape!($precision_type, $precision_tag, 6, 5);
            push_shape!($precision_type, $precision_tag, 6, 6);
            push_shape!($precision_type, $precision_tag, 6, 7);
            push_shape!($precision_type, $precision_tag, 6, 8);
            push_shape!($precision_type, $precision_tag, 7, 1);
            push_shape!($precision_type, $precision_tag, 7, 2);
            push_shape!($precision_type, $precision_tag, 7, 3);
            push_shape!($precision_type, $precision_tag, 7, 4);
            push_shape!($precision_type, $precision_tag, 7, 5);
            push_shape!($precision_type, $precision_tag, 7, 6);
            push_shape!($precision_type, $precision_tag, 7, 7);
            push_shape!($precision_type, $precision_tag, 7, 8);
            push_shape!($precision_type, $precision_tag, 8, 1);
            push_shape!($precision_type, $precision_tag, 8, 2);
            push_shape!($precision_type, $precision_tag, 8, 3);
            push_shape!($precision_type, $precision_tag, 8, 4);
            push_shape!($precision_type, $precision_tag, 8, 5);
            push_shape!($precision_type, $precision_tag, 8, 6);
            push_shape!($precision_type, $precision_tag, 8, 7);
            push_shape!($precision_type, $precision_tag, 8, 8);
        }};
    }

    push_all_shapes!(Precision4, 4);
    push_all_shapes!(Precision6, 6);
    push_all_shapes!(Precision8, 8);
    push_all_shapes!(Precision10, 10);

    eprintln!(
        "jmle_grid_sweep: {} work units ({} shapes x 4 precisions x {} seeds)",
        units.len(),
        units.len() / (4 * seeds.max(1)),
        seeds
    );

    // Run every unit in parallel. Each unit returns its (precision, M, N, seed) result, and we reduce
    // them per shape afterwards (a clean map then group, so there is no shared mutable accumulation).
    let results: Vec<(u8, usize, usize, SeedResult)> = units
        .par_iter()
        .map(|unit| {
            let seed = unit_seed(unit.precision, unit.m, unit.n, unit.seed_index);
            let result = (unit.run)(seed, unit.base);
            (unit.precision, unit.m, unit.n, result)
        })
        .collect();

    eprintln!(
        "jmle_grid_sweep: all {} units done, aggregating",
        results.len()
    );

    // Aggregate per (precision, M, N).
    use std::collections::BTreeMap;
    let mut grouped: BTreeMap<(u8, usize, usize), Vec<SeedResult>> = BTreeMap::new();
    for (precision, m, n, result) in results {
        grouped.entry((precision, m, n)).or_default().push(result);
    }

    // Paired-difference statistics over the seeds for one shape: the per-seed difference
    // `two_set_err - jmle_err` (positive means jMLE is the more accurate of the two on that seed),
    // returned as (mean, standard error, t = mean / sem, fraction of seeds jMLE wins). The pairing
    // cancels the seed-to-seed variance common to both estimators (same data), which is the only
    // correct way to compare two estimators measured on identical inputs.
    fn paired_2set_minus_jmle(seed_results: &[SeedResult]) -> (f64, f64, f64, f64) {
        let n = seed_results.len() as f64;
        let diffs: Vec<f64> = seed_results
            .iter()
            .map(|r| r.two_set_err - r.jmle_err)
            .collect();
        let mean = diffs.iter().sum::<f64>() / n;
        let var = if n > 1.0 {
            diffs.iter().map(|d| (d - mean) * (d - mean)).sum::<f64>() / (n - 1.0)
        } else {
            0.0
        };
        let sem = (var / n).sqrt();
        let t = if sem > 0.0 { mean / sem } else { 0.0 };
        let wins = diffs.iter().filter(|d| **d > 0.0).count() as f64 / n;
        (mean, sem, t, wins)
    }

    println!("precision,M,N,seeds,jmle_err,two_set_mle_err,default_err,jmle_over_2set,jmle_over_default,paired_2set_minus_jmle,paired_t,jmle_win_frac,jmle_solve_ms,two_set_solve_ms,default_solve_ms");
    for ((precision, m, n), seed_results) in &grouped {
        let count = seed_results.len() as f64;
        let mean =
            |f: fn(&SeedResult) -> f64| -> f64 { seed_results.iter().map(f).sum::<f64>() / count };
        let jmle_err = mean(|r| r.jmle_err);
        let two_set_err = mean(|r| r.two_set_err);
        let default_err = mean(|r| r.default_err);
        let jmle_solve_ms = mean(|r| r.jmle_solve_ms);
        let two_set_solve_ms = mean(|r| r.two_set_solve_ms);
        let default_solve_ms = mean(|r| r.default_solve_ms);
        let (paired_mean, _paired_sem, paired_t, win_frac) = paired_2set_minus_jmle(seed_results);
        let ratio = |a: f64, b: f64| if b > 0.0 { a / b } else { f64::NAN };
        println!(
            "{},{},{},{},{:.6},{:.6},{:.6},{:.4},{:.4},{:.6},{:.3},{:.3},{:.6},{:.6},{:.6}",
            precision,
            m,
            n,
            seed_results.len(),
            jmle_err,
            two_set_err,
            default_err,
            ratio(jmle_err, two_set_err),
            ratio(jmle_err, default_err),
            paired_mean,
            paired_t,
            win_frac,
            jmle_solve_ms,
            two_set_solve_ms,
            default_solve_ms
        );
    }

    // Per-precision significance summary of the paired (2set - jmle) difference. For each precision we
    // count how many of the 64 shapes show jMLE significantly better (t > 2), significantly worse
    // (t < -2), or no significant difference, and we pool every per-seed paired difference across all
    // 64 shapes into one high-powered test (positive pooled mean and t mean jMLE is the better one).
    eprintln!("jmle_grid_sweep: paired (2set_err - jmle_err) significance per precision (positive favors jMLE)");
    for precision in [4u8, 6, 8, 10] {
        let shapes: Vec<&Vec<SeedResult>> = grouped
            .iter()
            .filter(|((p, _, _), _)| *p == precision)
            .map(|(_, v)| v)
            .collect();
        let (mut sig_better, mut sig_worse, mut not_sig) = (0u32, 0u32, 0u32);
        let mut pooled: Vec<f64> = Vec::new();
        for sr in &shapes {
            let (_, _, t, _) = paired_2set_minus_jmle(sr);
            if t > 2.0 {
                sig_better += 1;
            } else if t < -2.0 {
                sig_worse += 1;
            } else {
                not_sig += 1;
            }
            pooled.extend(sr.iter().map(|r| r.two_set_err - r.jmle_err));
        }
        let n = pooled.len() as f64;
        let pmean = pooled.iter().sum::<f64>() / n;
        let pvar = pooled
            .iter()
            .map(|d| (d - pmean) * (d - pmean))
            .sum::<f64>()
            / (n - 1.0);
        let psem = (pvar / n).sqrt();
        let pt = if psem > 0.0 { pmean / psem } else { 0.0 };
        eprintln!(
            "  P{precision}: shapes jMLE sig-better {sig_better}/64, sig-worse {sig_worse}/64, ns {not_sig}/64 | pooled mean {pmean:.4} (rel.err pts) t {pt:.1} over {} paired samples",
            pooled.len()
        );
    }

    // Report the 8x8 jMLE solve cost per precision (mean over seeds), the headline cost figure.
    eprintln!("jmle_grid_sweep: 8x8 jMLE solve cost (mean ms over seeds)");
    for precision in [4u8, 6, 8, 10] {
        if let Some(seed_results) = grouped.get(&(precision, 8, 8)) {
            let count = seed_results.len() as f64;
            let ms = seed_results.iter().map(|r| r.jmle_solve_ms).sum::<f64>() / count;
            eprintln!("  P{precision} 8x8: {ms:.2} ms");
        }
    }
}
