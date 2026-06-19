//! Maximum Likelihood Estimation for HyperLogLog cardinalities and set sketches.
//!
//! This module is behind the `mle` feature, which works in no_std + alloc: it stores joint patterns
//! in an `alloc::collections::BTreeMap` and routes the float transcendentals to `libm` (via
//! `num-traits`) when `std` is unavailable, and to the standard library otherwise. It provides three
//! estimators, all maximizing a register-multiplicity likelihood and reached through the [`Mle`]
//! mode wrapper ([`HyperLogLog::mle`]):
//! - `hll.mle().estimate_union_cardinality(&other.mle())`: Ertl's 2-set joint union MLE.
//! - `hll.mle().estimate_cardinality()`: Ertl's single-counter cardinality MLE (provided for
//!   completeness; it is dominated by the default HyperLogLog++ estimate).
//! - [`JointSketch::estimate`] over `.mle()` views: the hypersphere-sketch decomposition over `M`
//!   nested left and `N` nested right counters. It is counted exactly from stored values while every
//!   operand is a value list, formed by inclusion-exclusion over the corrected estimates while still
//!   pre-dense, and decomposed by inclusion-exclusion over the 2-set union MLE once any operand is
//!   dense.
//!
//! The submodules hold the implementation: `union` and `cardinality` (the classic Ertl estimators)
//! and `exact` (the exact value-list joint set-algebra decomposition).

use crate::prelude::*;

mod cardinality;
mod exact;
#[cfg(test)]
mod tests;
mod union;
mod wrapper;

pub use wrapper::Mle;

// The associative map used to tabulate joint patterns and classify exact cells. It is the
// no_std-friendly `alloc::collections::BTreeMap` (the keys are all `Ord`); benchmarks showed it
// matches `std::collections::HashMap` end to end (often faster, the maps are small).
pub(crate) use alloc::collections::BTreeMap as PatternMap;

/// Upper bound on a register-multiplicity histogram length, which is `1 << B::NUMBER_OF_BITS`. It is
/// `1 << 6` because the widest supported `Bits` is `Bits6`. The cardinality and union MLE histograms
/// are stack arrays of this length, indexed only up to their per-`B` logical length, so they avoid a
/// heap allocation per MLE call. A `debug_assert` at each use site guards the bound should a wider
/// `Bits` ever be added.
pub(crate) const REGISTER_MULTIPLICITIES_CAPACITY: usize = 1 << 6;

use cardinality::mle_cardinality;
use exact::joint_sketch_exact_from_values;
use union::mle_union_regions;

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperLogLog<P, B, R, H> {
    /// Returns the union cardinality estimated with the joint Maximum Likelihood Estimation.
    ///
    /// # Examples
    /// ```
    /// use hyperloglog_rs::prelude::*;
    /// type Hll = HyperLogLog<Precision12, Bits6>;
    ///
    /// let mut a = Hll::default();
    /// let mut b = Hll::default();
    /// for x in 0u64..30_000 {
    ///     a.insert(&x); // A = [0, 30000)
    /// }
    /// for x in 20_000u64..50_000 {
    ///     b.insert(&x); // B = [20000, 50000), so the true union is 50000
    /// }
    ///
    /// let union = a.mle().estimate_union_cardinality(&b.mle());
    /// assert!((union - 50_000.0).abs() / 50_000.0 < 0.1);
    /// ```
    ///
    /// # Implementative details
    /// The estimator dispatches on the representation of the two operands. When both are still hash
    /// lists, the near-exact sorted hash list union ([`HyperLogLog::estimate_union_cardinality`]) is used
    /// directly, since the stored hashes carry more information than the register multiplicities the
    /// MLE consumes. When both are fully-fledged HyperLogLogs, the left difference, right difference
    /// and intersection likelihood is maximized jointly and the union estimate is their sum. In the
    /// mixed case the sorted hash list operand is materialized into registers first and the MLE is run.
    #[inline]
    pub(crate) fn estimate_union_cardinality_mle(&self, other: &Self) -> f64 {
        // Exact-values operands are resolved first: two exact operands give the exact union, and a
        // mixed pair promotes the exact one to a sorted hash list (a clone) before falling through.
        {
            if self.is_sorted_value_list() && other.is_sorted_value_list() {
                return self.estimate_union_cardinality(other);
            }
            if self.is_sorted_value_list() {
                let mut promoted = self.clone();
                promoted.to_sorted_hash_list();
                return promoted.estimate_union_cardinality_mle(other);
            }
            if other.is_sorted_value_list() {
                let mut promoted = other.clone();
                promoted.to_sorted_hash_list();
                return self.estimate_union_cardinality_mle(&promoted);
            }
        }

        if self.is_sorted_hash_list() && other.is_sorted_hash_list() {
            return self.estimate_union_cardinality(other);
        }

        if self.is_sorted_hash_list() || other.is_sorted_hash_list() {
            let mut left = self.clone();
            let mut right = other.clone();
            if left.is_sorted_hash_list() {
                left.to_hll();
            }
            if right.is_sorted_hash_list() {
                right.to_hll();
            }
            return left.mle_union_from_registers(&right);
        }

        self.mle_union_from_registers(other)
    }

    /// Returns the cardinality estimated with the single-counter Maximum Likelihood Estimation.
    ///
    /// # Examples
    /// ```
    /// use hyperloglog_rs::prelude::*;
    /// type Hll = HyperLogLog<Precision12, Bits6>;
    ///
    /// let mut counter = Hll::default();
    /// for x in 0u64..40_000 {
    ///     counter.insert(&x); // true cardinality is 40000
    /// }
    ///
    /// let estimate = counter.mle().estimate_cardinality();
    /// assert!((estimate - 40_000.0).abs() / 40_000.0 < 0.1);
    /// ```
    ///
    /// # Implementative details
    /// This is Ertl's secant-method maximum-likelihood estimator over the register multiplicities.
    /// A pre-HyperLogLog operand (sorted value list or a sorted hash list) returns the default estimate directly
    /// ([`HyperLogLog::estimate_cardinality`]) rather than being run through the register MLE: those
    /// representations already carry a more accurate direct count, and their backing buffer is not a
    /// register multiset. In register mode the MLE is provided for completeness and comparison: it is
    /// less accurate, and substantially slower, than the default corrected estimate.
    #[inline]
    pub(crate) fn estimate_cardinality_mle(&self) -> f64 {
        // Both pre-HyperLogLog representations (sorted value list and the sorted hash list) carry a more accurate
        // direct count than the register MLE could recover, and their `registers` buffer is not a
        // register multiset, so fall back to the default estimate rather than running the MLE.
        if !self.is_hyperloglog() {
            return self.estimate_cardinality();
        }

        mle_cardinality::<P, B>(
            self.registers.iter_registers(),
            self.harmonic_sum,
            self.is_full(),
            2,
        )
    }

    /// Joint sketch of the disjoint-cell cardinalities of the hypersphere sketch for `M` nested left
    /// counters and `N` nested right counters, using maximum-likelihood union estimates.
    ///
    /// Returns the exclusive overlap grid `overlap[i][j] = |L_i intersect R_j|` (where `L_i = A_i \
    /// A_{i-1}` and `R_j = B_j \ B_{j-1}` are the shells) and the margins `left_diff[i]`,
    /// `right_diff[j]`.
    ///
    /// # Implementative details
    /// The dispatch mirrors the scalar [`mle()`](HyperLogLog::mle) estimators and the default joint
    /// sketch, differing only once an operand is genuinely dense. When every operand is a sorted value
    /// list the disjoint cells are counted EXACTLY from the stored literal values (no hashing, no
    /// collisions). When no operand is dense yet (every operand a value or hash list, in any mix) the
    /// cells are formed by pairwise inclusion-exclusion over the corrected, allocation-free
    /// union/cardinality estimates ([`estimate_union_cardinality`](HyperLogLog::estimate_union_cardinality)
    /// promotes value lists to hash lists as needed). This is far more accurate than materializing the
    /// near-exact pre-dense operands into registers: the raw distinct-hash decomposition drifts 15-30%
    /// once the common hash size narrows, and register-izing a value/hash mix injects register-level
    /// error in the representation-transition band. Only once at least one operand is actually dense
    /// are all operands materialized into registers and decomposed by inclusion-exclusion over the
    /// 2-set union MLE views, keeping every cell on the same (register) footing and avoiding the
    /// mismatch of mixing near-exact hash-list cells with probabilistic register ones.
    #[inline]
    pub(crate) fn joint_sketch_mle<const M: usize, const N: usize>(
        lefts: &[Self; M],
        rights: &[Self; N],
    ) -> JointSketch<M, N> {
        if lefts.iter().all(Self::is_sorted_value_list)
            && rights.iter().all(Self::is_sorted_value_list)
        {
            return joint_sketch_exact_from_values::<P, B, R, H, M, N>(lefts, rights);
        }
        if !lefts.iter().chain(rights.iter()).any(Self::is_hyperloglog) {
            return crate::sketches::inclusion_exclusion_joint_sketch(lefts, rights);
        }

        // At least one operand is dense: materialize every operand to registers (a no-op for the dense
        // ones) so all cells share the same footing, then decompose by inclusion-exclusion over the
        // 2-set union MLE views.
        let lefts: [Self; M] = core::array::from_fn(|i| lefts[i].clone().into_hll());
        let rights: [Self; N] = core::array::from_fn(|j| rights[j].clone().into_hll());
        let left_views: [Mle<&Self>; M] = core::array::from_fn(|i| lefts[i].mle());
        let right_views: [Mle<&Self>; N] = core::array::from_fn(|j| rights[j].mle());
        crate::sketches::inclusion_exclusion_joint_sketch(&left_views, &right_views)
    }

    /// The three disjoint regions `[left_difference, right_difference, intersection]` of the 2-set
    /// joint MLE, assuming both counters are in HyperLogLog (register) mode. This is the fast analytic
    /// estimator the `M = N = 1` joint sketch uses.
    pub(crate) fn mle_union_regions_from_registers(&self, other: &Self) -> [f64; 3] {
        // Maps a union harmonic sum (and zero-register count) to the corrected cardinality, exactly as
        // the default register-based union estimator does. This must apply linear counting at low
        // union load, not just the bias-corrected raw estimate: at small cardinalities almost every
        // union register is zero, where linear counting is far more accurate, and seeding the regions
        // from the badly-biased raw estimate makes the union MLE several times worse than the default
        // (a low-load analogue of the mixed hash-list/registers union bug).
        let estimate = |harmonic_sum: f64, zeros: u32| {
            Self::corrected_register_cardinality(harmonic_sum, zeros as usize)
        };

        mle_union_regions::<P, B, _>(
            self.registers.iter_registers_zipped(&other.registers),
            self.estimate_cardinality(),
            other.estimate_cardinality(),
            estimate,
            2,
        )
    }

    /// Joint MLE union estimate assuming both counters are in HyperLogLog (register) mode.
    fn mle_union_from_registers(&self, other: &Self) -> f64 {
        let [left_difference, right_difference, intersection] =
            self.mle_union_regions_from_registers(other);
        left_difference + right_difference + intersection
    }
}
