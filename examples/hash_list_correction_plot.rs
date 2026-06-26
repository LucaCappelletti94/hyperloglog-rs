//! Two outputs for the correction writeup, both over many seeds at P10 B6.
//!
//! stdout: CSV `(n, uncorrected, hllpp, corrected)` of the mean absolute relative error (percent) per
//! cardinality, for the accuracy figure. stderr: the aggregate MAE (mean absolute error, elements),
//! MRE (mean relative error, signed, percent), and MSE (mean squared error, elements^2) over the
//! hash-list operating range for each estimator, plus the representation boundaries.
//!
//! Per seed the main counter uses `insert_value` (value list, then hash list, then registers); a
//! parallel counter forced into registers (`to_hll`) is fed the same elements to give the standard
//! HyperLogLog++ estimate. `uncorrected` is the raw distinct-composite count `D`, `corrected` is the
//! occupancy inverse, `hllpp` is the dense register estimate. Run:
//!   cargo run --release --example hash_list_correction_plot > /tmp/hll_plot_data.csv 2> /tmp/hll_plot_bounds.txt
use hyperloglog_rs::prelude::*;
use std::collections::BTreeMap;

#[derive(Default, Clone, Copy)]
struct Stat {
    abs: f64,    // sum |est - n|
    signed: f64, // sum (est - n) / n
    sq: f64,     // sum (est - n)^2
    rel: f64,    // sum |est - n| / n
    k: u64,
}
impl Stat {
    fn add(&mut self, est: f64, truth: f64) {
        self.abs += (est - truth).abs();
        self.signed += (est - truth) / truth;
        self.sq += (est - truth) * (est - truth);
        self.rel += (est - truth).abs() / truth;
        self.k += 1;
    }
}

fn main() {
    type Hll = HyperLogLog<Precision10, Bits6>;
    const SEEDS: u64 = 256;

    // Per-cardinality accumulator for the plot: (sum unc rel err, sum hllpp rel err, sum corr rel
    // err, sample count, sum hash_bits, hash-list sample count).
    let mut per_n: BTreeMap<u64, (f64, f64, f64, u64, f64, u64)> = BTreeMap::new();
    // Aggregate metrics over the hash-list region.
    let (mut m_unc, mut m_hp, mut m_cor) = (Stat::default(), Stat::default(), Stat::default());
    let (mut value_list_end, mut hll_start) = (0u64, 0u64);

    for s in 0..SEEDS {
        let mut h = Hll::default();
        let mut dense = Hll::default();
        dense.to_hll();
        let mut state = splitmix64(0x00AB_CDEF ^ s.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let mut n = 0u64;
        let mut threshold = 1.0_f64;
        loop {
            n += 1;
            state = splitmix64(splitmix64(state));
            h.insert_value(state);
            dense.insert(&state);
            if h.is_hyperloglog() {
                if s == 0 {
                    hll_start = n;
                }
                break;
            }
            if s == 0 && h.is_sorted_value_list() {
                value_list_end = n;
            }
            let truth = n as f64;
            let in_hash_list = h.is_sorted_hash_list();
            let unc = if in_hash_list {
                f64::from(h.get_number_of_hashes().unwrap())
            } else {
                truth
            };
            let corr = h.estimate_cardinality();
            let hllpp = dense.estimate_cardinality();

            // Aggregate metrics only over the hash-list operating range (where the correction acts).
            if in_hash_list {
                m_unc.add(unc, truth);
                m_hp.add(hllpp, truth);
                m_cor.add(corr, truth);
            }

            let log_point = n as f64 >= threshold;
            if n <= 30 || log_point {
                if log_point {
                    threshold = n as f64 * 1.05;
                }
                let e = per_n.entry(n).or_insert((0.0, 0.0, 0.0, 0, 0.0, 0));
                e.0 += (unc - truth).abs() / truth;
                e.1 += (hllpp - truth).abs() / truth;
                e.2 += (corr - truth).abs() / truth;
                e.3 += 1;
                if in_hash_list {
                    e.4 += f64::from(h.get_hash_bits().unwrap());
                    e.5 += 1;
                }
            }
        }
    }

    // Floor at 0.001 percent so the log axis can render the essentially exact low-cardinality region.
    let pct = |x: f64, k: f64| (100.0 * x / k).max(0.001);
    println!("n,uncorrected,hllpp,corrected,bits");
    for (n, (u, hp, c, count, bits_sum, bits_k)) in per_n {
        let k = count as f64;
        let bits = if bits_k > 0 {
            bits_sum / bits_k as f64
        } else {
            0.0
        };
        println!(
            "{},{:.4},{:.4},{:.4},{:.3}",
            n,
            pct(u, k),
            pct(hp, k),
            pct(c, k),
            bits
        );
    }

    let report = |name: &str, s: &Stat| {
        let k = s.k as f64;
        eprintln!(
            "{name:>12}: MAE={:.2} elements, MRE={:+.2}%, MSE={:.1} elements^2, MARE={:.2}%",
            s.abs / k,
            100.0 * s.signed / k,
            s.sq / k,
            100.0 * s.rel / k
        );
    };
    eprintln!("Metrics over the hash-list region, {SEEDS} seeds, P10 B6:");
    report("uncorrected", &m_unc);
    report("HLL++", &m_hp);
    report("corrected", &m_cor);
    eprintln!("value_list_end={value_list_end} hll_start={hll_start}");
}
