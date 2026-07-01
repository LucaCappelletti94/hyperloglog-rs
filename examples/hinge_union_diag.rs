//! Diagnostic for the hash-list <-> registers union hinge. For a grid of operand sizes around the
//! transition it builds the two operands, forces each into both the hash-list and the registers
//! representation, and prints the union estimate for all four representation combinations alongside
//! the exact truth. If the MIXED combinations (hash x registers, registers x hash) are worse than
//! BOTH same-representation combinations, the mixed union path has a residual hinge bug; if mixed
//! sits between the two same-rep results, it is just blending two accuracy levels.

use hyperloglog_rs::prelude::*;

type C = HyperLogLog<Precision12, Bits6>;

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

fn build(values: &[u64]) -> (C, C) {
    let mut base: C = Default::default();
    for &v in values {
        base.insert_value(v);
    }
    let hash = base.into_sorted_hash_list();
    let registers = base.into_hll();
    (hash, registers)
}

fn main() {
    println!(
        "{:>6} {:>6} {:>6} | {:>8} | {:>9} {:>9} {:>9} {:>9} | {:>6} {:>6}",
        "a", "shared", "b", "true_U", "h x h", "h x R", "R x h", "R x R", "ahash?", "bhash?"
    );

    let mut state = 0x1234_5678_9abc_def0u64;
    let next = splitmix64;

    for &(n_a, n_shared, n_b) in &[
        (400usize, 0usize, 400usize),
        (400, 200, 400),
        (700, 100, 700),
        (800, 300, 800),
        (900, 350, 900),
        (600, 0, 600),
        (500, 500, 500),
        (1000, 0, 1000),
    ] {
        // Three disjoint pools by domain offset.
        let only_a: Vec<u64> = (0..n_a).map(|_| next(&mut state) % 5_000_000).collect();
        let shared: Vec<u64> = (0..n_shared)
            .map(|_| 5_000_000 + next(&mut state) % 5_000_000)
            .collect();
        let only_b: Vec<u64> = (0..n_b)
            .map(|_| 10_000_000 + next(&mut state) % 5_000_000)
            .collect();

        let a: Vec<u64> = only_a.iter().chain(&shared).copied().collect();
        let b: Vec<u64> = shared.iter().chain(&only_b).copied().collect();

        let (a_hash, a_reg) = build(&a);
        let (b_hash, b_reg) = build(&b);

        let true_union = (n_a + n_shared + n_b) as f64;
        let hh = a_hash.estimate_union_cardinality(&b_hash);
        let hr = a_hash.estimate_union_cardinality(&b_reg);
        let rh = a_reg.estimate_union_cardinality(&b_hash);
        let rr = a_reg.estimate_union_cardinality(&b_reg);

        let pct = |x: f64| 100.0 * (x - true_union) / true_union;
        println!(
            "{:>6} {:>6} {:>6} | {:>8.0} | {:>8.1}% {:>8.1}% {:>8.1}% {:>8.1}% | {:>6} {:>6}",
            n_a,
            n_shared,
            n_b,
            true_union,
            pct(hh),
            pct(hr),
            pct(rh),
            pct(rr),
            a_hash.is_sorted_hash_list(),
            b_hash.is_sorted_hash_list(),
        );
    }
}
