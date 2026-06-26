//! Emits CSV `(n, hybrid_ins, dense_ins, hybrid_est, dense_est)`: per-insert and per-estimate time
//! (nanoseconds) of the hybrid counter versus a pure HyperLogLog++ register counter, across the
//! lifecycle at P10 B6. The hybrid is accurate in the hash list (see the accuracy figure) but pays a
//! higher per-insert cost there (sorted gap-coded splicing) and a higher per-estimate cost (the
//! occupancy Newton inversion) than the dense registers. Run:
//!   cargo run --release --example hash_list_timing > /tmp/hll_timing.csv
use hyperloglog_rs::prelude::*;
use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

const REPS: u64 = 128;
const CHUNK: u64 = 128; // inserts timed per checkpoint
const ESTS: u64 = 512; // estimate calls timed per checkpoint
const MAX_N: u64 = 1 << 17;

fn checkpoints() -> Vec<u64> {
    let mut cps = Vec::new();
    let mut c = 8.0_f64;
    while (c as u64) <= MAX_N {
        let v = c as u64;
        if cps.last() != Some(&v) {
            cps.push(v);
        }
        c *= 1.4;
    }
    cps
}

#[derive(Default, Clone, Copy)]
struct Acc {
    ins: f64,
    est: f64,
    k: u64,
}

fn measure(dense: bool, cps: &[u64]) -> BTreeMap<u64, Acc> {
    type Hll = HyperLogLog<Precision10, Bits6>;
    let mut acc: BTreeMap<u64, Acc> = BTreeMap::new();
    for s in 0..REPS {
        let mut h = Hll::default();
        if dense {
            h.to_hll();
        }
        let mut state = splitmix64(0x0511_2358 ^ s.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let mut n = 0u64;
        for &cp in cps {
            if cp < CHUNK {
                continue;
            }
            while n < cp - CHUNK {
                n += 1;
                state = splitmix64(splitmix64(state));
                if dense {
                    black_box(h.insert(&state));
                } else {
                    black_box(h.insert_value(state));
                }
            }
            // Per-insert time over the chunk that reaches the checkpoint.
            let t0 = Instant::now();
            for _ in 0..CHUNK {
                n += 1;
                state = splitmix64(splitmix64(state));
                if dense {
                    black_box(h.insert(&state));
                } else {
                    black_box(h.insert_value(state));
                }
            }
            let ins = t0.elapsed().as_nanos() as f64 / CHUNK as f64;
            // Per-estimate time at the checkpoint. black_box the receiver each call so the pure call
            // cannot be hoisted out of the loop.
            let t1 = Instant::now();
            for _ in 0..ESTS {
                black_box(black_box(&h).estimate_cardinality());
            }
            let est = t1.elapsed().as_nanos() as f64 / ESTS as f64;
            let e = acc.entry(cp).or_default();
            e.ins += ins;
            e.est += est;
            e.k += 1;
        }
    }
    acc
}

fn main() {
    let cps = checkpoints();
    let hybrid = measure(false, &cps);
    let dense = measure(true, &cps);
    println!("n,hybrid_ins,dense_ins,hybrid_est,dense_est");
    for &cp in &cps {
        if let (Some(h), Some(d)) = (hybrid.get(&cp), dense.get(&cp)) {
            let hk = h.k as f64;
            let dk = d.k as f64;
            println!(
                "{cp},{:.2},{:.2},{:.2},{:.2}",
                h.ins / hk,
                d.ins / dk,
                h.est / hk,
                d.est / dk
            );
        }
    }
}
