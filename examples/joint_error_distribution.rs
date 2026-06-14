//! Where does the per-cell error live in the M x N matrix? Uses uniform-size cells so any
//! structure is positional (deep vs shallow, corner vs edge), not a cell-size artifact, and
//! averages over many seeds to separate structure from noise. Reports, per cell, the mean absolute
//! relative error and the mean signed relative error (bias) for both the pairwise hypersphere
//! sketch (HLL++ inclusion-exclusion) and the joint MLE.
//!
//! Run with: `cargo run --release --features mle --example joint_error_distribution`
#![allow(clippy::needless_range_loop)]
use hyperloglog_rs::prelude::*;
use twox_hash::XxHash64;

type Hll<P, B> = HyperLogLog<P, B, <P as PackedRegister<B>>::Array, XxHash64>;

/// Accumulated per-cell error for one estimator.
struct Acc<const M: usize, const N: usize> {
    abs_overlap: [[f64; N]; M],
    signed_overlap: [[f64; N]; M],
    abs_left: [f64; M],
    abs_right: [f64; N],
    samples: f64,
}

impl<const M: usize, const N: usize> Acc<M, N> {
    fn new() -> Self {
        Acc {
            abs_overlap: [[0.0; N]; M],
            signed_overlap: [[0.0; N]; M],
            abs_left: [0.0; M],
            abs_right: [0.0; N],
            samples: 0.0,
        }
    }
}

fn run<P, B, const M: usize, const N: usize>(unit: u64, seeds: u64)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    // Uniform exact cell cardinalities (every region has the same size `unit`).
    let exact = unit as f64;
    let mut pair = Acc::<M, N>::new();
    let mut mle = Acc::<M, N>::new();

    for seed in 0..seeds {
        // A seed-derived base shifts every integer range, changing the hashing.
        let base = splitmix64(seed.wrapping_add(1)).wrapping_mul(4) & 0x0000_FFFF_FFFF_0000;
        let mut cursor = base;
        let mut alloc = |count: u64| {
            let start = cursor;
            cursor += count;
            (start, count)
        };
        let mut ro = [[(0u64, 0u64); N]; M];
        for i in 0..M {
            for j in 0..N {
                ro[i][j] = alloc(unit);
            }
        }
        let mut rda = [(0u64, 0u64); M];
        for i in 0..M {
            rda[i] = alloc(unit);
        }
        let mut rdb = [(0u64, 0u64); N];
        for j in 0..N {
            rdb[j] = alloc(unit);
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
        let lefts: [Hll<P, B>; M] = core::array::from_fn(|i| {
            let mut ranges = Vec::new();
            for ii in 0..=i {
                for j in 0..N {
                    ranges.push(ro[ii][j]);
                }
                ranges.push(rda[ii]);
            }
            build(&ranges)
        });
        let rights: [Hll<P, B>; N] = core::array::from_fn(|j| {
            let mut ranges = Vec::new();
            for jj in 0..=j {
                for i in 0..M {
                    ranges.push(ro[i][jj]);
                }
                ranges.push(rdb[jj]);
            }
            build(&ranges)
        });

        let (pov, pl, pr) =
            <Hll<P, B> as HyperSpheresSketch>::overlap_and_differences_cardinality_matrices(
                &lefts, &rights,
            )
            .into_parts();
        let (mov, ml, mr) = Hll::<P, B>::joint_sketch_mle(&lefts, &rights).into_parts();

        for i in 0..M {
            for j in 0..N {
                pair.abs_overlap[i][j] += (pov[i][j] - exact).abs() / exact;
                pair.signed_overlap[i][j] += (pov[i][j] - exact) / exact;
                mle.abs_overlap[i][j] += (mov[i][j] - exact).abs() / exact;
                mle.signed_overlap[i][j] += (mov[i][j] - exact) / exact;
            }
        }
        for i in 0..M {
            pair.abs_left[i] += (pl[i] - exact).abs() / exact;
            mle.abs_left[i] += (ml[i] - exact).abs() / exact;
        }
        for j in 0..N {
            pair.abs_right[j] += (pr[j] - exact).abs() / exact;
            mle.abs_right[j] += (mr[j] - exact).abs() / exact;
        }
        pair.samples += 1.0;
        mle.samples += 1.0;
    }

    let print_matrix = |title: &str, m: &[[f64; N]; M], samples: f64| {
        println!("  {title}:");
        for i in 0..M {
            print!("    i={i} |");
            for j in 0..N {
                print!(" {:>+6.1}", 100.0 * m[i][j] / samples);
            }
            println!();
        }
    };

    println!(
        "=== M={M} N={N} P{} unit={unit} seeds={seeds} (values are mean per-cell error in %) ===",
        P::EXPONENT
    );
    print_matrix(
        "pairwise |rel err| (overlap O_ij)",
        &pair.abs_overlap,
        pair.samples,
    );
    print_matrix(
        "MLE      |rel err| (overlap O_ij)",
        &mle.abs_overlap,
        mle.samples,
    );
    print_matrix(
        "pairwise signed bias (overlap O_ij)",
        &pair.signed_overlap,
        pair.samples,
    );
    print_matrix(
        "MLE      signed bias (overlap O_ij)",
        &mle.signed_overlap,
        mle.samples,
    );
    print!("  pairwise |rel err| left margins:");
    for i in 0..M {
        print!(" {:>5.1}", 100.0 * pair.abs_left[i] / pair.samples);
    }
    println!();
    print!("  MLE      |rel err| left margins:");
    for i in 0..M {
        print!(" {:>5.1}", 100.0 * mle.abs_left[i] / mle.samples);
    }
    println!();
    print!("  pairwise |rel err| right margins:");
    for j in 0..N {
        print!(" {:>5.1}", 100.0 * pair.abs_right[j] / pair.samples);
    }
    println!();
    print!("  MLE      |rel err| right margins:");
    for j in 0..N {
        print!(" {:>5.1}", 100.0 * mle.abs_right[j] / mle.samples);
    }
    println!("\n");
}

fn main() {
    run::<Precision8, Bits6, 5, 5>(256, 80);
    run::<Precision8, Bits6, 8, 8>(256, 30);
}
