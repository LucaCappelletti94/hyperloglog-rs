//! Compare per-cell accuracy of the pairwise hypersphere sketch (warm start) vs the joint MLE,
//! across precisions, on an M=2,N=2 partition with known exact cells.
use hyperloglog_rs::prelude::*;
use twox_hash::XxHash64;

fn build<P, B>(
    ranges: &[(u64, u64)],
) -> HyperLogLog<P, B, <P as PackedRegister<B>>::Array, XxHash64>
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    let mut hll = HyperLogLog::<P, B, <P as PackedRegister<B>>::Array, XxHash64>::default();
    for &(start, count) in ranges {
        for v in start..start + count {
            hll.insert(&v);
        }
    }
    hll
}

fn sweep<P, B>(unit: u64)
where
    P: Precision + PackedRegister<B>,
    B: Bits,
{
    // Exact cells, scaled by `unit` so load is comparable across precisions.
    let o = [[3 * unit, 2 * unit], [unit + unit / 2, 2 * unit]];
    let da = [2 * unit, unit + unit / 5];
    let db = [unit + unit * 6 / 10, 2 * unit + unit / 5];

    let mut cursor = 0u64;
    let mut alloc = |c: u64| {
        let s = cursor;
        cursor += c;
        (s, c)
    };
    let ro = [
        [alloc(o[0][0]), alloc(o[0][1])],
        [alloc(o[1][0]), alloc(o[1][1])],
    ];
    let rda = [alloc(da[0]), alloc(da[1])];
    let rdb = [alloc(db[0]), alloc(db[1])];

    let a0 = build::<P, B>(&[ro[0][0], ro[0][1], rda[0]]);
    let a1 = build::<P, B>(&[ro[0][0], ro[0][1], rda[0], ro[1][0], ro[1][1], rda[1]]);
    let b0 = build::<P, B>(&[ro[0][0], ro[1][0], rdb[0]]);
    let b1 = build::<P, B>(&[ro[0][0], ro[1][0], rdb[0], ro[0][1], ro[1][1], rdb[1]]);

    if a1.is_hash_list() || b1.is_hash_list() {
        println!("P{:<2}: hash-list regime (skipped)", P::EXPONENT);
        return;
    }

    let total: f64 = (o[0][0] + o[0][1] + o[1][0] + o[1][1] + da[0] + da[1] + db[0] + db[1]) as f64;

    let (wov, wl, wr) =
        <HyperLogLog<P, B, <P as PackedRegister<B>>::Array, XxHash64> as HyperSpheresSketch>::overlap_and_differences_cardinality_matrices(
            &[a0.clone(), a1.clone()],
            &[b0.clone(), b1.clone()],
        );
    let (mov, ml, mr) =
        HyperLogLog::<P, B, <P as PackedRegister<B>>::Array, XxHash64>::joint_sketch_mle(
            &[a0, a1],
            &[b0, b1],
        );

    // Mean absolute cell error, normalized by the total union.
    let mut warm_err = 0.0;
    let mut mle_err = 0.0;
    let mut n = 0;
    let mut acc = |west: f64, mest: f64, exact: f64| {
        warm_err += (west - exact).abs() / total;
        mle_err += (mest - exact).abs() / total;
        n += 1;
    };
    for i in 0..2 {
        for j in 0..2 {
            acc(wov[i][j], mov[i][j], o[i][j] as f64);
        }
    }
    for i in 0..2 {
        acc(wl[i], ml[i], da[i] as f64);
    }
    for j in 0..2 {
        acc(wr[j], mr[j], db[j] as f64);
    }
    let nf = n as f64;
    println!(
        "P{:<2} load~{:>4.0}: warm={:.4} mle={:.4}  mle/warm={:+.0}%  err_rate={:.4}",
        P::EXPONENT,
        total / (1u64 << P::EXPONENT) as f64,
        warm_err / nf,
        mle_err / nf,
        (mle_err / warm_err - 1.0) * 100.0,
        P::error_rate(),
    );
}

fn main() {
    println!("Mean per-cell error (normalized by union), M=2 N=2, Bits6:");
    sweep::<Precision4, Bits6>(1 << 4);
    sweep::<Precision5, Bits6>(1 << 5);
    sweep::<Precision6, Bits6>(1 << 6);
    sweep::<Precision7, Bits6>(1 << 7);
    sweep::<Precision8, Bits6>(1 << 8);
    sweep::<Precision9, Bits6>(1 << 9);
    sweep::<Precision10, Bits6>(1 << 10);
    sweep::<Precision11, Bits6>(1 << 11);
    sweep::<Precision12, Bits6>(1 << 12);
}
