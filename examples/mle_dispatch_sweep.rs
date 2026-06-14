//! Measures the effect of dispatching the MLE estimators by representation: when every operand is
//! still a hash list the estimators now count the disjoint cells exactly from the stored hashes
//! instead of materializing into registers and running MLE. This sweep crosses the hash-list ->
//! dense saturation boundary at several precisions and reports, per regime, accuracy and wall-clock
//! for the union estimator and the joint sketch, plus an aggregate "better on average" comparison.
//!
//! Two families:
//! - Union (M=N=1): dispatching `estimate_union_cardinality_mle` vs the default
//!   `estimate_union_cardinality`.
//! - Joint (M=N=2): dispatching `joint_sketch_mle` (exact in the hash-list regime) vs the register
//!   path forced by pre-materializing the operands (always MLE), isolating the exact-vs-MLE delta.
//!
//! Run with: `cargo run --release --features mle --example mle_dispatch_sweep`
#![allow(clippy::needless_range_loop)]
use hyperloglog_rs::prelude::*;
use std::time::Instant;
use twox_hash::XxHash64;

type Hll<P, B> = HyperLogLog<P, B, <P as PackedRegister<B>>::Array, XxHash64>;

const UNITS: [u64; 11] = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];
const SEEDS: u64 = 8;

/// Builds nested left/right counters from disjoint integer ranges with known-exact cells. Returns
/// the counters, the exact cell vector (overlap `i*N+j`, left margin `M*N+i`, right margin
/// `M*N+M+j`), and the exact union (the sum of all cells).
#[allow(clippy::type_complexity)]
fn build<P, B, const M: usize, const N: usize>(
    unit: u64,
    seed: u64,
) -> ([Hll<P, B>; M], [Hll<P, B>; N], Vec<f64>, f64)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let n_overlap = M * N;
    let k = n_overlap + M + N;
    let mut exact = vec![0.0_f64; k];
    let base = splitmix64(seed.wrapping_add(1)).wrapping_mul(4) & 0x0000_FFFF_FFFF_0000;
    let mut cursor = base;
    let mut ro = [[(0u64, 0u64); N]; M];
    for i in 0..M {
        for j in 0..N {
            let count = unit * (2 + ((i * 7 + j * 3) % 5) as u64);
            ro[i][j] = (cursor, count);
            cursor += count;
            exact[i * N + j] = count as f64;
        }
    }
    let mut rda = [(0u64, 0u64); M];
    for i in 0..M {
        let count = unit * (1 + (i % 3) as u64);
        rda[i] = (cursor, count);
        cursor += count;
        exact[n_overlap + i] = count as f64;
    }
    let mut rdb = [(0u64, 0u64); N];
    for j in 0..N {
        let count = unit * (1 + (j % 4) as u64);
        rdb[j] = (cursor, count);
        cursor += count;
        exact[n_overlap + M + j] = count as f64;
    }
    let make = |ranges: &[(u64, u64)]| -> Hll<P, B> {
        let mut hll = Hll::<P, B>::default();
        for &(start, count) in ranges {
            for v in start..start + count {
                hll.insert(&v);
            }
        }
        hll
    };
    let lefts: [Hll<P, B>; M] = core::array::from_fn(|i| {
        let mut ranges = Vec::new();
        for ii in 0..=i {
            for j in 0..N {
                ranges.push(ro[ii][j]);
            }
            ranges.push(rda[ii]);
        }
        make(&ranges)
    });
    let rights: [Hll<P, B>; N] = core::array::from_fn(|j| {
        let mut ranges = Vec::new();
        for jj in 0..=j {
            for i in 0..M {
                ranges.push(ro[i][jj]);
            }
            ranges.push(rdb[jj]);
        }
        make(&ranges)
    });
    let union: f64 = exact.iter().sum();
    (lefts, rights, exact, union)
}

/// A bucket is one of the three dispatch regimes; runs are accumulated into the matching bucket.
#[derive(Clone, Copy, Default)]
struct Bucket {
    def_err: f64,
    disp_err: f64,
    def_ms: f64,
    disp_ms: f64,
    n: usize,
}

impl Bucket {
    fn add(&mut self, def_err: f64, disp_err: f64, def_ms: f64, disp_ms: f64) {
        self.def_err += def_err;
        self.disp_err += disp_err;
        self.def_ms += def_ms;
        self.disp_ms += disp_ms;
        self.n += 1;
    }
    fn merge(&mut self, other: &Bucket) {
        self.def_err += other.def_err;
        self.disp_err += other.disp_err;
        self.def_ms += other.def_ms;
        self.disp_ms += other.disp_ms;
        self.n += other.n;
    }
}

/// Per-(precision, unit) averaged result for one family, plus the path the dispatcher took.
struct Row {
    exponent: u8,
    unit: u64,
    path: &'static str,
    def_err: f64,
    disp_err: f64,
    def_ms: f64,
    disp_ms: f64,
}

fn path_label(all_hash_list: bool, all_dense: bool) -> &'static str {
    if all_hash_list {
        "hash-exact"
    } else if all_dense {
        "dense-mle"
    } else {
        "mixed"
    }
}

/// Union family (M=N=1): dispatching `estimate_union_cardinality_mle` vs default
/// `estimate_union_cardinality`. Accumulates per-run results into the regime buckets.
fn sweep_union<P, B>(rows: &mut Vec<Row>, hash: &mut Bucket, dense: &mut Bucket)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    for &unit in &UNITS {
        let mut row = Bucket::default();
        let mut path = "mixed";
        for seed in 0..SEEDS {
            let (lefts, rights, _exact, union) = build::<P, B, 1, 1>(unit, seed);
            let left = &lefts[0];
            let right = &rights[0];
            let all_hash_list = left.is_hash_list() && right.is_hash_list();
            let all_dense = !left.is_hash_list() && !right.is_hash_list();
            path = path_label(all_hash_list, all_dense);

            let t = Instant::now();
            let def = left.estimate_union_cardinality(right);
            let def_ms = t.elapsed().as_secs_f64() * 1e3;
            let t = Instant::now();
            let disp = left.mle().estimate_union_cardinality(&right.mle());
            let disp_ms = t.elapsed().as_secs_f64() * 1e3;

            let def_err = (def - union).abs() / union;
            let disp_err = (disp - union).abs() / union;
            row.add(def_err, disp_err, def_ms, disp_ms);
            if all_hash_list {
                hash.add(def_err, disp_err, def_ms, disp_ms);
            } else if all_dense {
                dense.add(def_err, disp_err, def_ms, disp_ms);
            }
        }
        rows.push(Row {
            exponent: P::EXPONENT,
            unit,
            path,
            def_err: row.def_err / row.n as f64,
            disp_err: row.disp_err / row.n as f64,
            def_ms: row.def_ms / row.n as f64,
            disp_ms: row.disp_ms / row.n as f64,
        });
    }
}

/// Mean per-cell absolute error normalized by the exact union.
fn cell_error<const M: usize, const N: usize>(
    overlap: &[[f64; N]; M],
    left_diff: &[f64; M],
    right_diff: &[f64; N],
    exact: &[f64],
    union: f64,
) -> f64 {
    let k = M * N + M + N;
    let mut err = 0.0;
    for i in 0..M {
        for j in 0..N {
            err += (overlap[i][j] - exact[i * N + j]).abs() / union;
        }
    }
    for i in 0..M {
        err += (left_diff[i] - exact[M * N + i]).abs() / union;
    }
    for j in 0..N {
        err += (right_diff[j] - exact[M * N + M + j]).abs() / union;
    }
    err / k as f64
}

/// Joint family (M=N=2): dispatching `joint_sketch_mle` (exact when all hash list) vs the register
/// path forced by pre-materializing every operand (always MLE). The "default" columns here are the
/// forced register MLE, so a negative dispatch-vs-default ratio means the new path wins.
fn sweep_joint<P, B>(rows: &mut Vec<Row>, hash: &mut Bucket, dense: &mut Bucket)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    const M: usize = 2;
    const N: usize = 2;
    for &unit in &UNITS {
        let mut row = Bucket::default();
        let mut path = "mixed";
        for seed in 0..SEEDS {
            let (lefts, rights, exact, union) = build::<P, B, M, N>(unit, seed);
            let all_hash_list =
                lefts.iter().all(Hll::is_hash_list) && rights.iter().all(Hll::is_hash_list);
            let all_dense =
                lefts.iter().all(|c| !c.is_hash_list()) && rights.iter().all(|c| !c.is_hash_list());
            path = path_label(all_hash_list, all_dense);

            // Forced register MLE: materialize every operand, then run the (now all-dense) MLE.
            let mut forced_lefts = lefts.clone();
            let mut forced_rights = rights.clone();
            for c in forced_lefts.iter_mut().chain(forced_rights.iter_mut()) {
                if c.is_hash_list() {
                    c.convert_hash_list_to_hyperloglog().unwrap();
                }
            }
            let t = Instant::now();
            let (fo, fl, fr) =
                Hll::<P, B>::joint_sketch_mle::<M, N>(&forced_lefts, &forced_rights).into_parts();
            let forced_ms = t.elapsed().as_secs_f64() * 1e3;

            let t = Instant::now();
            let (o, l, r) = Hll::<P, B>::joint_sketch_mle::<M, N>(&lefts, &rights).into_parts();
            let disp_ms = t.elapsed().as_secs_f64() * 1e3;

            let forced_err = cell_error::<M, N>(&fo, &fl, &fr, &exact, union);
            let disp_err = cell_error::<M, N>(&o, &l, &r, &exact, union);
            row.add(forced_err, disp_err, forced_ms, disp_ms);
            if all_hash_list {
                hash.add(forced_err, disp_err, forced_ms, disp_ms);
            } else if all_dense {
                dense.add(forced_err, disp_err, forced_ms, disp_ms);
            }
        }
        rows.push(Row {
            exponent: P::EXPONENT,
            unit,
            path,
            def_err: row.def_err / row.n as f64,
            disp_err: row.disp_err / row.n as f64,
            def_ms: row.def_ms / row.n as f64,
            disp_ms: row.disp_ms / row.n as f64,
        });
    }
}

fn print_rows(title: &str, def_label: &str, rows: &[Row]) {
    println!("\n=== {title} ===");
    println!(
        "{:>3} {:>6} {:>11} | {:>10} {:>10} {:>9} | {:>9} {:>9}",
        "P", "unit", "path", def_label, "dispatch", "disp/def", "def_ms", "disp_ms"
    );
    for r in rows {
        let ratio = if r.def_err > 0.0 {
            (r.disp_err / r.def_err - 1.0) * 100.0
        } else if r.disp_err > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };
        println!(
            "{:>3} {:>6} {:>11} | {:>10.5} {:>10.5} {:>+8.1}% | {:>9.4} {:>9.4}",
            r.exponent, r.unit, r.path, r.def_err, r.disp_err, ratio, r.def_ms, r.disp_ms
        );
    }
}

fn print_aggregate(title: &str, def_label: &str, hash: &Bucket, dense: &Bucket) {
    let mut combined = *hash;
    combined.merge(dense);
    println!("\n=== {title}: better on average (negative disp/def = dispatch better) ===");
    println!(
        "{:>12} | {:>10} {:>10} {:>9} | {:>9} {:>9} {:>6}",
        "regime", def_label, "dispatch", "disp/def", "def_ms", "disp_ms", "n"
    );
    for (name, b) in [
        ("hash-list", hash),
        ("dense", dense),
        ("combined", &combined),
    ] {
        if b.n == 0 {
            println!(
                "{name:>12} | {:>10} {:>10} {:>9} | {:>9} {:>9} {:>6}",
                "-", "-", "-", "-", "-", 0
            );
            continue;
        }
        let def_err = b.def_err / b.n as f64;
        let disp_err = b.disp_err / b.n as f64;
        let ratio = if def_err > 0.0 {
            (disp_err / def_err - 1.0) * 100.0
        } else {
            0.0
        };
        println!(
            "{name:>12} | {:>10.5} {:>10.5} {:>+8.1}% | {:>9.4} {:>9.4} {:>6}",
            def_err,
            disp_err,
            ratio,
            b.def_ms / b.n as f64,
            b.disp_ms / b.n as f64,
            b.n
        );
    }
}

fn main() {
    println!(
        "MLE dispatch sweep: accuracy (relative error) and time across the hash-list boundary."
    );

    let mut union_rows = Vec::new();
    let mut union_hash = Bucket::default();
    let mut union_dense = Bucket::default();
    sweep_union::<Precision6, Bits6>(&mut union_rows, &mut union_hash, &mut union_dense);
    sweep_union::<Precision8, Bits6>(&mut union_rows, &mut union_hash, &mut union_dense);
    sweep_union::<Precision10, Bits6>(&mut union_rows, &mut union_hash, &mut union_dense);
    sweep_union::<Precision12, Bits6>(&mut union_rows, &mut union_hash, &mut union_dense);
    print_rows(
        "Union family (M=N=1): default vs dispatching estimate_union_cardinality_mle",
        "default",
        &union_rows,
    );
    print_aggregate("Union family", "default", &union_hash, &union_dense);

    let mut joint_rows = Vec::new();
    let mut joint_hash = Bucket::default();
    let mut joint_dense = Bucket::default();
    sweep_joint::<Precision6, Bits6>(&mut joint_rows, &mut joint_hash, &mut joint_dense);
    sweep_joint::<Precision8, Bits6>(&mut joint_rows, &mut joint_hash, &mut joint_dense);
    sweep_joint::<Precision10, Bits6>(&mut joint_rows, &mut joint_hash, &mut joint_dense);
    sweep_joint::<Precision12, Bits6>(&mut joint_rows, &mut joint_hash, &mut joint_dense);
    print_rows(
        "Joint family (M=N=2): forced register MLE vs dispatching joint_sketch_mle",
        "forced-mle",
        &joint_rows,
    );
    print_aggregate("Joint family", "forced-mle", &joint_hash, &joint_dense);
}
