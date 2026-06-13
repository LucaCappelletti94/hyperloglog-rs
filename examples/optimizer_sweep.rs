//! Does the best optimizer for the joint MLE depend on the task (M, N, precision)? Drives several
//! optimizers through the public `joint_sketch_mle_with` API across a grid of shapes and
//! precisions on exact-cell partitions, averaging over seeds, and reports mean per-cell error and
//! wall-clock for each. Uses the public optimizer family and `Chain` composition.
//!
//! Run with: `cargo run --release --features mle --example optimizer_sweep`
#![allow(clippy::needless_range_loop)]
use hyperloglog_rs::prelude::*;
use std::time::Instant;
use twox_hash::XxHash64;

type Hll<P, B> = HyperLogLog<P, B, <P as PackedRegister<B>>::Array, XxHash64>;

/// Builds the nested counters and the exact per-region cardinalities (indexed like the estimator:
/// overlap `O_ij` at `i*N+j`, left margin at `M*N+i`, right margin at `M*N+M+j`).
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
    let total: f64 = exact.iter().sum();
    (lefts, rights, exact, total)
}

/// Runs one optimizer over `seeds` instances of the (M, N) problem, returning mean per-cell error
/// (% of union) and mean wall-clock in milliseconds.
fn measure<P, B, O, const M: usize, const N: usize>(unit: u64, seeds: u64) -> (f64, f64)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
    O: JointOptimizer,
{
    let n_overlap = M * N;
    let k = n_overlap + M + N;
    let mut total_err = 0.0;
    let mut total_ms = 0.0;
    for seed in 0..seeds {
        let (lefts, rights, exact, total) = build::<P, B, M, N>(unit, seed);
        let start = Instant::now();
        let (overlap, left_diff, right_diff) =
            Hll::<P, B>::joint_sketch_mle_with::<O, M, N>(&lefts, &rights);
        total_ms += start.elapsed().as_secs_f64() * 1e3;
        let mut err = 0.0;
        for i in 0..M {
            for j in 0..N {
                err += (overlap[i][j] - exact[i * N + j]).abs() / total;
            }
        }
        for i in 0..M {
            err += (left_diff[i] - exact[n_overlap + i]).abs() / total;
        }
        for j in 0..N {
            err += (right_diff[j] - exact[n_overlap + M + j]).abs() / total;
        }
        total_err += err / k as f64;
    }
    (100.0 * total_err / seeds as f64, total_ms / seeds as f64)
}

fn compare<P, B, const M: usize, const N: usize>(unit: u64, seeds: u64)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let results = [
        ("lbfgs", measure::<P, B, Lbfgs, M, N>(unit, seeds)),
        ("adam", measure::<P, B, Adam, M, N>(unit, seeds)),
        (
            "adam+lbfgs",
            measure::<P, B, Chain<Adam, Lbfgs>, M, N>(unit, seeds),
        ),
        (
            "rms+lbfgs",
            measure::<P, B, Chain<RmsProp, Lbfgs>, M, N>(unit, seeds),
        ),
    ];
    let best = results
        .iter()
        .map(|(_, (err, _))| *err)
        .fold(f64::INFINITY, f64::min);

    print!("M={M} N={N} P{:<2} |", P::EXPONENT);
    for (name, (err, ms)) in results {
        let marker = if (err - best).abs() < 1e-9 { "*" } else { " " };
        print!(" {name}={err:>5.3}%{marker}({ms:>6.1}ms)");
    }
    println!();
}

fn main() {
    println!("Mean per-cell error (% union) and time per optimizer. '*' = most accurate for that task.\n");
    println!("--- shape sweep at P8 ---");
    compare::<Precision8, Bits6, 1, 1>(1 << 8, 8);
    compare::<Precision8, Bits6, 2, 2>(1 << 8, 8);
    compare::<Precision8, Bits6, 3, 3>(1 << 8, 6);
    compare::<Precision8, Bits6, 4, 4>(1 << 8, 5);
    compare::<Precision8, Bits6, 5, 5>(1 << 8, 4);
    compare::<Precision8, Bits6, 6, 6>(1 << 8, 3);
    compare::<Precision8, Bits6, 2, 4>(1 << 8, 6);
    println!("\n--- precision sweep at M=N=3 ---");
    compare::<Precision6, Bits6, 3, 3>(1 << 6, 6);
    compare::<Precision8, Bits6, 3, 3>(1 << 8, 6);
    compare::<Precision10, Bits6, 3, 3>(1 << 10, 4);
    compare::<Precision12, Bits6, 3, 3>(1 << 12, 3);
}
