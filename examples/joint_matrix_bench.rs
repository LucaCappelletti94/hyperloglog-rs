#![allow(clippy::needless_range_loop)]
#![allow(clippy::upper_case_acronyms)]
//! Joint hypersphere sketch benchmark on random spheres (no graph).
//!
//! Nested spheres are populated by uniformly sampling random `u32` ids over the whole `0..=u32::MAX`
//! range. Each layer adds `LAYER` fresh values on top of the previous layer, so a nested chain is
//! `A_1 subset A_2 subset ... subset A_M` with cardinalities `LAYER, 2*LAYER, ...`. We build two such
//! chains (left and right) from independent streams and decompose their joint structure two ways, the
//! pairwise hypersphere sketch (HLL++ cardinalities combined by inclusion-exclusion) and the joint
//! MLE, comparing both to the exact decomposition from the std `HashSet`s. We sweep M=N and write the
//! per-shape error and timing to `docs/sketch_benchmark.json`.
//!
//! Run with: `cargo run --release --example joint_matrix_bench`

use hyperloglog_rs::estimator::HllCardinalityEstimator;
use hyperloglog_rs::prelude::*;
use rayon::prelude::*;
use sketching_core::CardinalityEstimator;
use std::collections::HashSet;
use std::time::Instant;
use twox_hash::XxHash64;

type Hll<P, B> = HyperLogLog<P, B, <P as PackedRegister<B>>::Array, XxHash64>;

/// Fresh values added per sphere layer.
const LAYER: usize = 500_000;
/// Values are sampled uniformly from `0..RANGE`. A smaller universe makes the two chains overlap.
const RANGE: u32 = 10_000_000;
/// Independent seed pairs per shape; accuracy is averaged over them and an std band is reported.
const SEEDS: usize = 16;

/// splitmix64-backed uniform `u32` sampler over `0..=u32::MAX`.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Uniform sample in `0..n`.
    fn below(&mut self, n: u32) -> u32 {
        self.0 = splitmix64(self.0);
        ((self.0 >> 32) as u32) % n
    }
}

/// Inclusion-exclusion decomposition (the default pairwise hypersphere sketch) over an arbitrary
/// cardinality estimator `E`. Wrapping `Mle` views in this runs the same pairwise pipeline but fed
/// Ertl's 2-set MLE union per pair instead of the HLL++ union: "the union approach applied
/// repeatedly". Comparing it to the bare `Mle` joint sketch isolates the gain of the generalized
/// joint optimizer over repeated pairwise MLE.
#[derive(Clone, Copy)]
struct IePairwise<E>(E);

impl<E: CardinalityEstimator> CardinalityEstimator for IePairwise<E> {
    fn estimate_cardinality(&self) -> f64 {
        self.0.estimate_cardinality()
    }
    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        self.0.estimate_union_cardinality(&other.0)
    }
}

impl<E: HllCardinalityEstimator> HllCardinalityEstimator for IePairwise<E> {
    fn predicted_relative_standard_error(&self) -> f64 {
        self.0.predicted_relative_standard_error()
    }
    fn predicted_bias(&self) -> f64 {
        self.0.predicted_bias()
    }
    fn relative_standard_error_at(&self, cardinality: f64) -> f64 {
        self.0.relative_standard_error_at(cardinality)
    }
    fn bias_at(&self, cardinality: f64) -> f64 {
        self.0.bias_at(cardinality)
    }
}
// Empty body: inherits the default inclusion-exclusion `joint_sketch`, unlike the bare `Mle` view
// which overrides it to run the joint optimizer.
impl<E: CardinalityEstimator> HyperSpheresSketch for IePairwise<E> {}

/// Builds one nested chain of `K` HLL counters: each layer adds `LAYER` fresh uniform `u32` values on
/// top of the previous layer. Returns the counters and the cumulative value set at each layer.
fn build_chain<P, B, const K: usize>(rng: &mut Rng) -> ([Hll<P, B>; K], Vec<HashSet<u32>>)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let mut hll = Hll::<P, B>::default();
    let mut exact: HashSet<u32> = HashSet::new();
    let mut chain: Vec<Hll<P, B>> = Vec::with_capacity(K);
    let mut sets: Vec<HashSet<u32>> = Vec::with_capacity(K);
    for _ in 0..K {
        for _ in 0..LAYER {
            let v = rng.below(RANGE);
            hll.insert(&v);
            exact.insert(v);
        }
        chain.push(hll.clone());
        sets.push(exact.clone());
    }
    let chain: [Hll<P, B>; K] = chain.try_into().expect("K counters");
    (chain, sets)
}

fn intersection_size(a: &HashSet<u32>, b: &HashSet<u32>) -> u64 {
    let (small, big) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    small.iter().filter(|v| big.contains(v)).count() as u64
}

/// Exact joint decomposition from the cumulative value sets, in `JointSketch`'s differential cell
/// convention: partial sums of `overlap` reconstruct `|A_i intersect B_j|`, partial sums of
/// `left_diff` reconstruct `|A_i \ B_last|`, partial sums of `right_diff` reconstruct `|B_j \ A_last|`.
fn exact_decomp<const M: usize, const N: usize>(
    left: &[HashSet<u32>],
    right: &[HashSet<u32>],
) -> ([[u64; N]; M], [u64; M], [u64; N]) {
    let mut inter = [[0u64; N]; M];
    for i in 0..M {
        for j in 0..N {
            inter[i][j] = intersection_size(&left[i], &right[j]);
        }
    }
    let at = |i: isize, j: isize| -> i64 {
        if i < 0 || j < 0 {
            0
        } else {
            inter[i as usize][j as usize] as i64
        }
    };
    let mut overlap = [[0u64; N]; M];
    for i in 0..M {
        for j in 0..N {
            let d = at(i as isize, j as isize)
                - at(i as isize - 1, j as isize)
                - at(i as isize, j as isize - 1)
                + at(i as isize - 1, j as isize - 1);
            overlap[i][j] = d.max(0) as u64;
        }
    }
    let mut left_diff = [0u64; M];
    let mut prev = 0i64;
    for i in 0..M {
        let cum = left[i].len() as i64 - inter[i][N - 1] as i64;
        left_diff[i] = (cum - prev).max(0) as u64;
        prev = cum;
    }
    let mut right_diff = [0u64; N];
    let mut prev = 0i64;
    for j in 0..N {
        let cum = right[j].len() as i64 - inter[M - 1][j] as i64;
        right_diff[j] = (cum - prev).max(0) as u64;
        prev = cum;
    }
    (overlap, left_diff, right_diff)
}

/// Relative error of the whole decomposition (all cells, normalized by the union) and of just the
/// overlap grid (the M*N intersection cells, normalized by the total overlap mass). The overlap grid
/// is the joint sketch's actual job; the whole-decomposition number is dominated by the margins.
fn decomposition_error<const M: usize, const N: usize>(
    overlap: &[[f64; N]; M],
    left: &[f64; M],
    right: &[f64; N],
    eo: &[[u64; N]; M],
    el: &[u64; M],
    er: &[u64; N],
) -> (f64, f64) {
    let mut overlap_err = 0.0;
    let mut overlap_mass = 0.0;
    let mut whole_err = 0.0;
    let mut union = 0.0;
    for i in 0..M {
        for j in 0..N {
            let e = (overlap[i][j] - eo[i][j] as f64).abs();
            overlap_err += e;
            overlap_mass += eo[i][j] as f64;
            whole_err += e;
            union += eo[i][j] as f64;
        }
    }
    for i in 0..M {
        whole_err += (left[i] - el[i] as f64).abs();
        union += el[i] as f64;
    }
    for j in 0..N {
        whole_err += (right[j] - er[j] as f64).abs();
        union += er[j] as f64;
    }
    (
        whole_err / union.max(1.0),
        overlap_err / overlap_mass.max(1.0),
    )
}

/// Two normalized overlap-grid metrics of an estimated decomposition against the exact one, both
/// averaged with equal weight over the `M*N` intersection cells (so deep tiny cells count as much as
/// shallow huge ones):
/// - `(a)` our-analysis: mean per-cell `|est - exact| / exact_shell_maximum`, the error in units of
///   the exact shell maximum. Uses ground-truth denominators, the clean yardstick for our analysis.
/// - `(b)` real-world: mean per-cell `|est.normalize() - exact.normalize()|`, each side
///   self-normalized by its own shell maxima. This is the question you can actually ask in production
///   (no `HashSet`): how close is the estimator's normalized sketch to the true normalized sketch.
fn normalized_overlap_errors<const M: usize, const N: usize>(
    est: &JointSketch<M, N>,
    exact: &JointSketch<M, N>,
    exact_smax: &JointSketch<M, N>,
    exact_norm: &JointSketch<M, N>,
) -> (f64, f64) {
    let est_norm = est.normalize();
    let mut a = 0.0;
    let mut b = 0.0;
    for i in 0..M {
        for j in 0..N {
            a += (est.overlap[i][j] - exact.overlap[i][j]).abs()
                / exact_smax.overlap[i][j].max(f64::EPSILON);
            b += (est_norm.overlap[i][j] - exact_norm.overlap[i][j]).abs();
        }
    }
    let cells = (M * N) as f64;
    (a / cells, b / cells)
}

/// Median wall-clock (ms) of `f`, repeated to fill at least `budget_ms`, capped at `max_iters`.
fn time_ms(budget_ms: u128, max_iters: u32, mut f: impl FnMut()) -> f64 {
    let start = Instant::now();
    let mut iters = 0u32;
    while start.elapsed().as_millis() < budget_ms && iters < max_iters {
        f();
        iters += 1;
    }
    start.elapsed().as_secs_f64() * 1e3 / iters.max(1) as f64
}

/// Per-trial accuracy for one seed pair: overlap-mass error, and the normalized metrics (a) and (b),
/// each as `[pairwise, union2, joint]`.
struct Trial {
    card: usize,
    overlap: [f64; 3],
    norm_a: [f64; 3],
    norm_b: [f64; 3],
}

/// Builds the two chains for one seed pair and returns the accuracy of all three estimators (no
/// timing, so it is cheap enough to repeat over many seeds).
fn trial<P, B, const M: usize, const N: usize>(seed_l: u64, seed_r: u64) -> Trial
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let mut lrng = Rng::new(seed_l);
    let mut rrng = Rng::new(seed_r);
    let (left, left_sets) = build_chain::<P, B, M>(&mut lrng);
    let (right, right_sets) = build_chain::<P, B, N>(&mut rrng);
    let (eo, el, er) = exact_decomp::<M, N>(&left_sets, &right_sets);

    let pair = JointSketch::estimate(&left, &right);
    let ulm: [_; M] = core::array::from_fn(|i| IePairwise(left[i].mle()));
    let urm: [_; N] = core::array::from_fn(|j| IePairwise(right[j].mle()));
    let union = JointSketch::estimate(&ulm, &urm);
    // The generalized joint MLE goes through the `.jmle()` views (the bare `.mle()` joint sketch now
    // runs the same repeated 2-set inclusion-exclusion as `IePairwise` above, so it would not isolate
    // the joint optimizer).
    let lm: [_; M] = core::array::from_fn(|i| left[i].jmle());
    let rm: [_; N] = core::array::from_fn(|j| right[j].jmle());
    let joint = JointSketch::estimate(&lm, &rm);

    let overlap_of = |js: &JointSketch<M, N>| {
        decomposition_error::<M, N>(&js.overlap, &js.left_diff, &js.right_diff, &eo, &el, &er).1
    };
    let exact_js = JointSketch::<M, N> {
        overlap: core::array::from_fn(|i| core::array::from_fn(|j| eo[i][j] as f64)),
        left_diff: core::array::from_fn(|i| el[i] as f64),
        right_diff: core::array::from_fn(|j| er[j] as f64),
    };
    let exact_smax = exact_js.shell_maxima();
    let exact_norm = exact_js.normalize();
    let na =
        |js: &JointSketch<M, N>| normalized_overlap_errors(js, &exact_js, &exact_smax, &exact_norm);
    let (pa, pb) = na(&pair);
    let (ua, ub) = na(&union);
    let (ja, jb) = na(&joint);

    Trial {
        card: left_sets[M - 1].len(),
        overlap: [overlap_of(&pair), overlap_of(&union), overlap_of(&joint)],
        norm_a: [pa, ua, ja],
        norm_b: [pb, ub, jb],
    }
}

/// Median wall-clock of each estimator for one seed pair (timing is stable, so a single seed is
/// enough and we keep it out of the per-trial accuracy loop).
fn timings<P, B, const M: usize, const N: usize>(seed_l: u64, seed_r: u64) -> [f64; 4]
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let mut lrng = Rng::new(seed_l);
    let mut rrng = Rng::new(seed_r);
    let (left, left_sets) = build_chain::<P, B, M>(&mut lrng);
    let (right, right_sets) = build_chain::<P, B, N>(&mut rrng);
    let time_exact = time_ms(50, 1000, || {
        std::hint::black_box(exact_decomp::<M, N>(&left_sets, &right_sets));
    });
    let time_pair = time_ms(50, 1000, || {
        std::hint::black_box(JointSketch::estimate(&left, &right));
    });
    let ulm: [_; M] = core::array::from_fn(|i| IePairwise(left[i].mle()));
    let urm: [_; N] = core::array::from_fn(|j| IePairwise(right[j].mle()));
    let time_union = time_ms(50, 50, || {
        std::hint::black_box(JointSketch::estimate(&ulm, &urm));
    });
    let lm: [_; M] = core::array::from_fn(|i| left[i].jmle());
    let rm: [_; N] = core::array::from_fn(|j| right[j].jmle());
    let time_mle = time_ms(50, 5, || {
        std::hint::black_box(JointSketch::estimate(&lm, &rm));
    });
    [time_exact, time_pair, time_union, time_mle]
}

/// Population mean and standard deviation.
fn mean_std(xs: &[f64]) -> (f64, f64) {
    let n = xs.len() as f64;
    let mean = xs.iter().sum::<f64>() / n;
    let var = xs.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / n;
    (mean, var.sqrt())
}

fn run<P, B, const M: usize, const N: usize>(rows: &mut Vec<serde_json::Value>)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    // The per-seed accuracy trials are independent and each seed is derived deterministically from its
    // index, so running them across cores does not change the results, only the wall-clock. Timing is
    // measured separately (below) on a single seed and stays off the parallel section.
    let trials: Vec<Trial> = (0..SEEDS)
        .into_par_iter()
        .map(|s| {
            let seed_l = 0xDEAD_BEEF_CAFE_F00D ^ (s as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let seed_r = 0x0BAD_C0DE_1234_5678 ^ (s as u64).wrapping_mul(0xD1B5_4A32_D192_ED03);
            trial::<P, B, M, N>(seed_l, seed_r)
        })
        .collect();
    let t = timings::<P, B, M, N>(0xDEAD_BEEF_CAFE_F00D, 0x0BAD_C0DE_1234_5678);

    let column = |sel: &dyn Fn(&Trial) -> f64| -> (f64, f64) {
        mean_std(&trials.iter().map(sel).collect::<Vec<_>>())
    };
    let (op_m, op_s) = column(&|t| t.overlap[0]);
    let (uo_m, uo_s) = column(&|t| t.overlap[1]);
    let (jo_m, jo_s) = column(&|t| t.overlap[2]);
    let (pa_m, pa_s) = column(&|t| t.norm_a[0]);
    let (ua_m, ua_s) = column(&|t| t.norm_a[1]);
    let (ja_m, ja_s) = column(&|t| t.norm_a[2]);
    let (pb_m, pb_s) = column(&|t| t.norm_b[0]);
    let (ub_m, ub_s) = column(&|t| t.norm_b[1]);
    let (jb_m, jb_s) = column(&|t| t.norm_b[2]);
    let card = trials.iter().map(|t| t.card).sum::<usize>() / trials.len();

    println!(
        "M={M} N={N} card~{card:>8} | overlap pair={:>5.2}+-{:>4.2}% union2={:>5.2}+-{:>4.2}% joint={:>5.2}+-{:>4.2}% | time pair={:>7.3}ms union2={:>7.3}ms joint={:>9.2}ms",
        op_m * 100.0, op_s * 100.0, uo_m * 100.0, uo_s * 100.0, jo_m * 100.0, jo_s * 100.0,
        t[1], t[2], t[3],
    );

    // Paired comparison: joint and 2-set are evaluated on the SAME data each seed, so the common
    // seed variance cancels. The per-seed difference (2set - joint, positive = joint better) has a far
    // smaller std than the marginal errors; its mean +- standard-error and t = mean/sem say whether
    // the joint MLE is significantly better. |t| >~ 2 is significant at 95%.
    let paired = |sel: &dyn Fn(&Trial) -> f64| -> (f64, f64, f64, usize) {
        let d: Vec<f64> = trials.iter().map(sel).collect();
        let (m, s) = mean_std(&d);
        let sem = s / (SEEDS as f64).sqrt();
        let wins = d.iter().filter(|x| **x > 0.0).count();
        (m, sem, if sem > 0.0 { m / sem } else { 0.0 }, wins)
    };
    let (uj_m, uj_sem, uj_t, uj_w) = paired(&|t| t.overlap[1] - t.overlap[2]);
    println!(
        "        paired (2set - joint): {:+.3}% +- {:.3}%  t={:>5.1}  joint better in {}/{} seeds",
        uj_m * 100.0,
        uj_sem * 100.0,
        uj_t,
        uj_w,
        SEEDS,
    );

    rows.push(serde_json::json!({
        "M": M,
        "N": N,
        "card": card,
        "seeds": SEEDS,
        "overlap_err_pairwise": op_m,
        "overlap_err_pairwise_std": op_s,
        "overlap_err_union2": uo_m,
        "overlap_err_union2_std": uo_s,
        "overlap_err_mle": jo_m,
        "overlap_err_mle_std": jo_s,
        "paired_2set_minus_joint": uj_m,
        "paired_2set_minus_joint_sem": uj_sem,
        "norm_a_pairwise": pa_m,
        "norm_a_pairwise_std": pa_s,
        "norm_a_union2": ua_m,
        "norm_a_union2_std": ua_s,
        "norm_a_mle": ja_m,
        "norm_a_mle_std": ja_s,
        "norm_b_pairwise": pb_m,
        "norm_b_pairwise_std": pb_s,
        "norm_b_union2": ub_m,
        "norm_b_union2_std": ub_s,
        "norm_b_mle": jb_m,
        "norm_b_mle_std": jb_s,
        "time_exact_ms": t[0],
        "time_pairwise_ms": t[1],
        "time_union2_ms": t[2],
        "time_mle_ms": t[3],
    }));
}

fn main() {
    println!("Random spheres: {LAYER} fresh values per layer sampled uniformly from 0..{RANGE}, independent left/right streams, P12 Bits6.\n");
    let mut rows = Vec::new();
    run::<Precision12, Bits6, 1, 1>(&mut rows);
    run::<Precision12, Bits6, 2, 2>(&mut rows);
    run::<Precision12, Bits6, 3, 3>(&mut rows);
    run::<Precision12, Bits6, 4, 4>(&mut rows);
    run::<Precision12, Bits6, 5, 5>(&mut rows);

    let out = std::path::Path::new("docs/sketch_benchmark.json");
    let payload = serde_json::json!({ "layer": LAYER, "range": RANGE, "rows": rows });
    std::fs::write(out, serde_json::to_string_pretty(&payload).unwrap()).unwrap();
    println!("\nJSON written to {}", out.display());
}
