//! Property-based hunt for "hinge" bugs: places where a binary operation's result depends on the
//! internal REPRESENTATION of its operands rather than on the sets they represent.
//!
//! A `HyperLogLog` counter passes through three representations as it fills: a sorted value list
//! (exact), then a sorted hash list (near-exact), then HyperLogLog registers. The same set can be
//! held in more than one of these (we can force a small counter up the ladder with
//! `into_sorted_hash_list` / `into_hll`). A correct estimator should give essentially the same
//! answer for `op(a, b)` no matter which representation `a` and `b` happen to be in, to within the
//! accuracy each representation can deliver at that cardinality.
//!
//! The bug class this targets (already fixed once on this branch, see `docs/hinge_bug_handoff.md`)
//! is the MIXED-representation pair: `a` a hash list and `b` registers (or value list vs hash list),
//! where a stray code path read the wrong operand's state and produced a wildly wrong answer (a
//! union that was ~50 percent of the truth). These properties oracle every reachable representation
//! combination against the exact set-theoretic truth, so a representation-dependent error shows up
//! as one combination breaking the tolerance while the others hold.
//!
//! Two transitions are probed separately because the representations on each side have different
//! intrinsic accuracy:
//!   - value list <-> hash list (small cardinalities): both near-exact, tight tolerance.
//!   - hash list <-> registers (mid cardinalities): registers carry HyperLogLog's standard error, so
//!     a looser tolerance, but still far tighter than the ~50 percent a hinge bug produces.

use hyperloglog_rs::prelude::*;
use proptest::collection::btree_set;
use proptest::prelude::*;

/// Builds the disjoint element pools (`only_a`, `shared`, `only_b`) into the two operand sets and
/// returns them alongside the exact set-theoretic truths. The three pools are drawn from disjoint
/// integer domains, so the union/intersection/difference cardinalities are exactly the pool sizes.
fn operands_and_truth(
    only_a: &[u64],
    shared: &[u64],
    only_b: &[u64],
) -> (Vec<u64>, Vec<u64>, Truth) {
    let a: Vec<u64> = only_a.iter().chain(shared).copied().collect();
    let b: Vec<u64> = shared.iter().chain(only_b).copied().collect();
    let truth = Truth {
        card_a: (only_a.len() + shared.len()) as f64,
        card_b: (shared.len() + only_b.len()) as f64,
        union: (only_a.len() + shared.len() + only_b.len()) as f64,
        intersection: shared.len() as f64,
        difference_a: only_a.len() as f64,
        jaccard: if only_a.len() + shared.len() + only_b.len() == 0 {
            0.0
        } else {
            shared.len() as f64 / (only_a.len() + shared.len() + only_b.len()) as f64
        },
    };
    (a, b, truth)
}

#[derive(Clone, Copy, Debug)]
struct Truth {
    card_a: f64,
    card_b: f64,
    union: f64,
    intersection: f64,
    difference_a: f64,
    jaccard: f64,
}

/// Tolerances for one transition. `card_rel` bounds the relative error of a marginal/union estimate;
/// `set_op_frac` bounds the absolute error of an intersection/difference estimate as a fraction of
/// the true union (so a near-zero intersection is judged on an absolute scale, not a meaningless
/// relative one); `jaccard_abs` bounds the absolute Jaccard error.
#[derive(Clone, Copy)]
struct Tol {
    card_rel: f64,
    set_op_frac: f64,
    jaccard_abs: f64,
}

/// Generates the per-type hinge suite. `mid_*` bound the element-pool sizes for the hash-list <->
/// registers transition; they are chosen per precision so both operands stay inside the hash list
/// (otherwise the hash-list form is unreachable and the case is rejected).
macro_rules! hinge_suite {
    (
        $modname:ident,
        $counter:ty,
        cases = $cases:expr,
        small_only = $small_only:expr,
        small_shared = $small_shared:expr,
        mid_only = $mid_only:expr,
        mid_shared = $mid_shared:expr
    ) => {
        mod $modname {
            use super::*;

            type C = $counter;

            /// The reachable representations of `values`, labelled. A counter is built with
            /// `insert_value` (so it starts in the lowest representation its size allows), then forced
            /// up the ladder. Forms that are not reachable for this set are omitted.
            fn forms(values: &[u64]) -> Vec<(&'static str, C)> {
                let mut base: C = Default::default();
                for &v in values {
                    base.insert_value(v);
                }
                let mut out: Vec<(&'static str, C)> = Vec::new();
                if base.is_sorted_value_list() {
                    out.push(("value", base.clone()));
                }
                let hash = base.clone().into_sorted_hash_list();
                if hash.is_sorted_hash_list() {
                    out.push(("hash", hash));
                }
                out.push(("registers", base.into_hll()));
                out
            }

            fn rel_err(est: f64, truth: f64) -> f64 {
                if truth == 0.0 {
                    est.abs()
                } else {
                    (est - truth).abs() / truth
                }
            }

            /// Oracles every combination of representations of `a` and `b` against the exact truth.
            /// `keep` selects which representations participate (so each transition compares only the
            /// two representations that straddle it).
            fn check_all_ops(
                a: &[u64],
                b: &[u64],
                truth: Truth,
                keep: &[&'static str],
                tol: Tol,
            ) -> Result<(), TestCaseError> {
                let fa: Vec<_> = forms(a)
                    .into_iter()
                    .filter(|(l, _)| keep.contains(l))
                    .collect();
                let fb: Vec<_> = forms(b)
                    .into_iter()
                    .filter(|(l, _)| keep.contains(l))
                    .collect();

                // Only exercise the case when BOTH straddling representations are reachable for both
                // operands, so the mixed combinations actually occur.
                for rep in keep {
                    prop_assume!(fa.iter().any(|(l, _)| l == rep));
                    prop_assume!(fb.iter().any(|(l, _)| l == rep));
                }

                // Marginal cardinality of `b` must not depend on representation either.
                for (lb, cb) in &fb {
                    prop_assert!(
                        rel_err(cb.estimate_cardinality(), truth.card_b) <= tol.card_rel,
                        "card_b[{lb}] est {} truth {} rel_tol {}",
                        cb.estimate_cardinality(),
                        truth.card_b,
                        tol.card_rel
                    );
                }

                for (la, ca) in &fa {
                    // Marginal cardinality must not depend on representation.
                    prop_assert!(
                        rel_err(ca.estimate_cardinality(), truth.card_a) <= tol.card_rel,
                        "card_a[{la}] est {} truth {} rel_tol {}",
                        ca.estimate_cardinality(),
                        truth.card_a,
                        tol.card_rel
                    );
                    for (lb, cb) in &fb {
                        let union = ca.estimate_union_cardinality(cb);
                        let intersection = ca.estimate_intersection_cardinality(cb);
                        let difference = ca.estimate_difference_cardinality(cb);
                        let jaccard = ca.estimate_jaccard_index(cb);

                        prop_assert!(
                            rel_err(union, truth.union) <= tol.card_rel,
                            "union[{la}x{lb}] est {union} truth {} rel_tol {}",
                            truth.union,
                            tol.card_rel
                        );
                        prop_assert!(
                            (intersection - truth.intersection).abs()
                                <= tol.set_op_frac * truth.union,
                            "intersection[{la}x{lb}] est {intersection} truth {} abs_tol {}",
                            truth.intersection,
                            tol.set_op_frac * truth.union
                        );
                        prop_assert!(
                            (difference - truth.difference_a).abs()
                                <= tol.set_op_frac * truth.union,
                            "difference[{la}x{lb}] est {difference} truth {} abs_tol {}",
                            truth.difference_a,
                            tol.set_op_frac * truth.union
                        );
                        prop_assert!(
                            (jaccard - truth.jaccard).abs() <= tol.jaccard_abs,
                            "jaccard[{la}x{lb}] est {jaccard} truth {} abs_tol {}",
                            truth.jaccard,
                            tol.jaccard_abs
                        );
                    }
                }
                Ok(())
            }

            proptest! {
                #![proptest_config(ProptestConfig {
                    cases: $cases,
                    max_global_rejects: 16384,
                    ..ProptestConfig::default()
                })]

                /// value list <-> hash list transition. Small sets drawn from a packed u16-ish domain
                /// so the value-list form is reachable; both sides are near-exact.
                #[test]
                fn value_vs_hash_list_is_representation_independent(
                    only_a in btree_set(0u64..50_000, 1..=$small_only),
                    shared in btree_set(50_000u64..100_000, 0..=$small_shared),
                    only_b in btree_set(100_000u64..150_000, 1..=$small_only),
                ) {
                    let only_a: Vec<u64> = only_a.into_iter().collect();
                    let shared: Vec<u64> = shared.into_iter().collect();
                    let only_b: Vec<u64> = only_b.into_iter().collect();
                    let (a, b, truth) = operands_and_truth(&only_a, &shared, &only_b);
                    check_all_ops(
                        &a,
                        &b,
                        truth,
                        &["value", "hash"],
                        Tol { card_rel: 0.06, set_op_frac: 0.06, jaccard_abs: 0.06 },
                    )?;
                }

                /// hash list <-> registers transition: the band where Bug 1 (the ~50 percent mixed
                /// union) lived. Mid-size sets that stay inside the hash list while loading the
                /// registers enough to be accurate.
                #[test]
                fn hash_list_vs_registers_is_representation_independent(
                    only_a in btree_set(0u64..5_000_000, $mid_only),
                    shared in btree_set(5_000_000u64..10_000_000, 0..=$mid_shared),
                    only_b in btree_set(10_000_000u64..15_000_000, $mid_only),
                ) {
                    let only_a: Vec<u64> = only_a.into_iter().collect();
                    let shared: Vec<u64> = shared.into_iter().collect();
                    let only_b: Vec<u64> = only_b.into_iter().collect();
                    let (a, b, truth) = operands_and_truth(&only_a, &shared, &only_b);
                    check_all_ops(
                        &a,
                        &b,
                        truth,
                        &["hash", "registers"],
                        Tol { card_rel: 0.12, set_op_frac: 0.12, jaccard_abs: 0.12 },
                    )?;
                }
            }
        }
    };
}

hinge_suite!(
    p10_b6,
    HyperLogLog<Precision10, Bits6>,
    cases = 96,
    small_only = 60,
    small_shared = 40,
    mid_only = 150..=450,
    mid_shared = 200
);

hinge_suite!(
    p12_b6,
    HyperLogLog<Precision12, Bits6>,
    cases = 96,
    small_only = 60,
    small_shared = 40,
    mid_only = 400..=900,
    mid_shared = 350
);

hinge_suite!(
    p14_b6,
    HyperLogLog<Precision14, Bits6>,
    cases = 48,
    small_only = 60,
    small_shared = 40,
    mid_only = 1200..=2800,
    mid_shared = 1200
);
