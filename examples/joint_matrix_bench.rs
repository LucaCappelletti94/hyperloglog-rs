//! Full-matrix accuracy and speed of the generalized joint MLE vs the pairwise hypersphere
//! sketch, across precisions and (M, N) sizes, on partitions with known exact cells.
//!
//! Run with: `cargo run --release --features mle --example joint_matrix_bench`
#![allow(clippy::needless_range_loop)]
use hyperloglog_rs::prelude::*;
use std::time::Instant;
use twox_hash::XxHash64;

type Hll<P, B> = HyperLogLog<P, B, <P as PackedRegister<B>>::Array, XxHash64>;

/// Builds the exact partition for an `M x N` problem (cells scaled by `unit`), constructs the
/// nested counters, and returns them with the exact cell cardinalities and the total union.
#[allow(clippy::type_complexity)]
fn build_problem<P, B, const M: usize, const N: usize>(
    unit: u64,
) -> (
    [Hll<P, B>; M],
    [Hll<P, B>; N],
    [[u64; N]; M],
    [u64; M],
    [u64; N],
    f64,
)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    // Exact cell cardinalities, varied across the grid.
    let mut o = [[0u64; N]; M];
    for i in 0..M {
        for j in 0..N {
            o[i][j] = unit * (2 + ((i * 7 + j * 3) % 5) as u64);
        }
    }
    let mut da = [0u64; M];
    for i in 0..M {
        da[i] = unit * (1 + (i % 3) as u64);
    }
    let mut db = [0u64; N];
    for j in 0..N {
        db[j] = unit * (1 + (j % 4) as u64);
    }

    // Lay every region out on its own contiguous integer range.
    let mut cursor = 0u64;
    let mut ro = [[(0u64, 0u64); N]; M];
    for i in 0..M {
        for j in 0..N {
            ro[i][j] = (cursor, o[i][j]);
            cursor += o[i][j];
        }
    }
    let mut rda = [(0u64, 0u64); M];
    for i in 0..M {
        rda[i] = (cursor, da[i]);
        cursor += da[i];
    }
    let mut rdb = [(0u64, 0u64); N];
    for j in 0..N {
        rdb[j] = (cursor, db[j]);
        cursor += db[j];
    }

    let build = |ranges: &[(u64, u64)]| -> Hll<P, B> {
        let mut hll = Hll::<P, B>::default();
        for &(start, count) in ranges {
            for v in start..start + count {
                hll.insert(&v);
            }
        }
        hll
    };

    // A_i contains the overlap cells O_{i' j} and left margins D^A_{i'} for i' <= i.
    let lefts: [Hll<P, B>; M] = core::array::from_fn(|i| {
        let mut ranges = Vec::new();
        for i2 in 0..=i {
            for j in 0..N {
                ranges.push(ro[i2][j]);
            }
            ranges.push(rda[i2]);
        }
        build(&ranges)
    });
    // B_j contains the overlap cells O_{i j'} and right margins D^B_{j'} for j' <= j.
    let rights: [Hll<P, B>; N] = core::array::from_fn(|j| {
        let mut ranges = Vec::new();
        for j2 in 0..=j {
            for i in 0..M {
                ranges.push(ro[i][j2]);
            }
            ranges.push(rdb[j2]);
        }
        build(&ranges)
    });

    let mut total = 0u64;
    for i in 0..M {
        for j in 0..N {
            total += o[i][j];
        }
        total += da[i];
    }
    for j in 0..N {
        total += db[j];
    }

    (lefts, rights, o, da, db, total as f64)
}

fn measure<P, B, const M: usize, const N: usize>(unit: u64)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let (lefts, rights, o, da, db, total) = build_problem::<P, B, M, N>(unit);

    if lefts
        .iter()
        .chain(rights.iter())
        .any(Hll::<P, B>::is_hash_list)
    {
        println!(
            "M={M} N={N} P{:<2}: hash-list regime (skipped)",
            P::EXPONENT
        );
        return;
    }

    // Pairwise hypersphere sketch (the current production estimator).
    let t = Instant::now();
    let (pov, pl, pr) = JointSketch::estimate(&lefts, &rights).into_parts();
    let pair_ms = t.elapsed().as_secs_f64() * 1e3;

    // Generalized joint MLE (the .mle() views select the joint optimization).
    let lefts_mle: [_; M] = core::array::from_fn(|i| lefts[i].mle());
    let rights_mle: [_; N] = core::array::from_fn(|j| rights[j].mle());
    let t = Instant::now();
    let (mov, ml, mr) = JointSketch::estimate(&lefts_mle, &rights_mle).into_parts();
    let mle_ms = t.elapsed().as_secs_f64() * 1e3;

    let mut pair_err = 0.0;
    let mut mle_err = 0.0;
    let mut cells = 0;
    let mut acc = |p: f64, m: f64, exact: u64| {
        pair_err += (p - exact as f64).abs() / total;
        mle_err += (m - exact as f64).abs() / total;
        cells += 1;
    };
    for i in 0..M {
        for j in 0..N {
            acc(pov[i][j], mov[i][j], o[i][j]);
        }
    }
    for i in 0..M {
        acc(pl[i], ml[i], da[i]);
    }
    for j in 0..N {
        acc(pr[j], mr[j], db[j]);
    }
    let cf = cells as f64;
    println!(
        "M={M} N={N} P{:<2} load~{:>4.0} cells={:<3} ie_avoided={:<6} | pair={:.4} mle={:.4} ({:+.0}%) | pair={:>7.2}ms mle={:>9.2}ms",
        P::EXPONENT,
        total / (1u64 << P::EXPONENT) as f64,
        cells,
        1u64 << (M + N),
        pair_err / cf,
        mle_err / cf,
        (mle_err / pair_err - 1.0) * 100.0,
        pair_ms,
        mle_ms,
    );
}

fn main() {
    println!("Per-cell error (normalized by union) and wall-clock, Bits6.");
    println!(
        "ie_avoided = 2^(M+N), the exponential term count the polynomial path no longer pays.\n"
    );
    // Square ladder at P8, now reaching sizes the exponential path could not.
    measure::<Precision8, Bits6, 1, 1>(1 << 8);
    measure::<Precision8, Bits6, 2, 2>(1 << 8);
    measure::<Precision8, Bits6, 3, 3>(1 << 8);
    measure::<Precision8, Bits6, 4, 4>(1 << 8);
    measure::<Precision8, Bits6, 5, 5>(1 << 8);
    measure::<Precision8, Bits6, 6, 6>(1 << 8);
    measure::<Precision8, Bits6, 7, 7>(1 << 8);
    measure::<Precision8, Bits6, 8, 8>(1 << 8);
    println!();
    // Rectangular, including large.
    measure::<Precision8, Bits6, 2, 4>(1 << 8);
    measure::<Precision8, Bits6, 3, 5>(1 << 8);
    measure::<Precision8, Bits6, 4, 6>(1 << 8);
    measure::<Precision8, Bits6, 6, 8>(1 << 8);
    println!();
    // Precision ladder at M=N=3 (unit scaled with the register count to hold load constant).
    measure::<Precision6, Bits6, 3, 3>(1 << 6);
    measure::<Precision8, Bits6, 3, 3>(1 << 8);
    measure::<Precision10, Bits6, 3, 3>(1 << 10);
    measure::<Precision12, Bits6, 3, 3>(1 << 12);
}
