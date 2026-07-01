//! Maximum Likelihood Estimation for `HyperLogLog` cardinalities and set sketches.
//!
//! This module is always available and fully allocation-free: it routes float transcendentals
//! through the crate's own `no_std` [`FloatOps`](crate::utils) (no `num-traits`/`libm`), the scalar and
//! register estimators use stack histograms, and the exact value-list joint decomposition is a sorted
//! multi-way merge. It provides three estimators, all maximizing a register-multiplicity likelihood
//! and reached through the [`Mle`] mode wrapper ([`HyperLogLog::mle`]):
//! - `hll.mle().estimate_union_cardinality(&other.mle())`: Ertl's 2-set joint union MLE.
//! - `hll.mle().estimate_cardinality()`: Ertl's single-counter cardinality MLE (provided for
//!   completeness; it is dominated by the default `HyperLogLog`++ estimate).
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
// The generalized joint MLE is fully allocation-free, like the rest of the MLE: it iterates the
// registers directly (no pattern map) and its optimizer state is fixed-size stack arrays sized by the
// `M, N <= 8` bound (`K = M*N + M + N <= 80`). So it is always available, no feature gate.
mod joint_wrapper;
mod likelihood;
mod optimizers;
#[cfg(test)]
mod oracle;
mod sketch;
#[cfg(test)]
mod tests;
mod union;
mod wrapper;

pub use joint_wrapper::JointMle;
pub use wrapper::Mle;

// The test-only oracle deduplicates joint register patterns into this map (the production path
// deduplicates by a sort plus run-length encode instead, never this map). The keys are all `Ord`, so a
// no_std-friendly `alloc::collections::BTreeMap` serves.
#[cfg(test)]
pub(crate) use alloc::collections::BTreeMap as PatternMap;

/// Upper bound on a register-multiplicity histogram length, which is `1 << B::NUMBER_OF_BITS`. It is
/// `1 << 6` because the widest supported `Bits` is `Bits6`. The cardinality and union MLE histograms
/// are stack arrays of this length, indexed only up to their per-`B` logical length, so they avoid a
/// heap allocation per MLE call. A `debug_assert` at each use site guards the bound should a wider
/// `Bits` ever be added.
pub(crate) const REGISTER_MULTIPLICITIES_CAPACITY: usize = 1 << 6;

/// Benchmark and test hook: run the generalized joint MLE over the disjoint-region model (always the
/// full damped-Newton optimizer, no `M = N = 1` short-circuit), assuming every operand is already in
/// register mode. This forwards to the internal `joint_sketch_mle_from_registers_full` so the criterion
/// harness (a separate crate that only sees `pub` items) can time it at a fixed shape. It is hidden
/// from the docs because the supported surface is `JointMle` (`.jmle()`).
#[doc(hidden)]
#[must_use]
pub fn bench_joint_sketch_mle<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
    const M: usize,
    const N: usize,
>(
    lefts: &[HyperLogLog<P, B, R, H>; M],
    rights: &[HyperLogLog<P, B, R, H>; N],
) -> crate::sketches::JointSketch<M, N> {
    sketch::joint_sketch_mle_from_registers_full::<P, B, R, H, M, N>(lefts, rights)
}

/// Benchmark hook: the 2-set union MLE regions via the production damped-Newton solver. Hidden from the
/// docs (the supported surface is `.mle()`), exposed only so the criterion harness can time the 2-set
/// hot path.
#[doc(hidden)]
#[must_use]
pub fn bench_union_regions<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>(
    left: &HyperLogLog<P, B, R, H>,
    right: &HyperLogLog<P, B, R, H>,
) -> [f64; 3] {
    left.mle_union_regions_from_registers(right)
}

use cardinality::mle_cardinality;
use exact::joint_sketch_exact_from_values;
use union::{mle_union_regions, union_region_relative_covariance};

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
    /// MLE consumes. When both are fully-fledged `HyperLogLogs`, the left difference, right difference
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
            self.dense_harmonic_sum(),
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
            return sketching_core::inclusion_exclusion_joint_sketch(lefts, rights);
        }

        // At least one operand is dense: materialize every operand to registers (a no-op for the dense
        // ones) so all cells share the same footing, then decompose by inclusion-exclusion over the
        // 2-set union MLE views.
        let lefts: [Self; M] = core::array::from_fn(|i| lefts[i].clone().into_hll());
        let rights: [Self; N] = core::array::from_fn(|j| rights[j].clone().into_hll());
        let left_views: [Mle<&Self>; M] = core::array::from_fn(|i| lefts[i].mle());
        let right_views: [Mle<&Self>; N] = core::array::from_fn(|j| rights[j].mle());
        sketching_core::inclusion_exclusion_joint_sketch(&left_views, &right_views)
    }

    /// The three disjoint regions `[left_difference, right_difference, intersection]` of the 2-set
    /// joint MLE, assuming both counters are in `HyperLogLog` (register) mode. This is the fast analytic
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

    /// The 2-set union MLE regions `[left_difference, right_difference, intersection]` together with
    /// their asymptotic relative covariance (log space), assuming both counters are in `HyperLogLog`
    /// (register) mode. The covariance diagonal holds the squared relative standard error of each
    /// region; see [`union_region_relative_covariance`].
    pub(crate) fn mle_union_region_covariance_from_registers(
        &self,
        other: &Self,
    ) -> ([f64; 3], [[f64; 3]; 3]) {
        let regions = self.mle_union_regions_from_registers(other);
        let covariance = union_region_relative_covariance::<P, B, _>(
            self.registers.iter_registers_zipped(&other.registers),
            regions,
        );
        (regions, covariance)
    }

    /// Per-cell theoretical standard error of the joint sketch ([`JointSketch`]), as a parallel grid
    /// of the same shape ([`JointSketchError`]).
    ///
    /// All-value-list operands are exact, so every cell error is zero. Otherwise every operand is
    /// materialized to dense registers (as the dense joint sketch does), and each cumulative pairwise
    /// region (`|A_i \ B_j|`, `|B_j \ A_i|`, `|A_i intersect B_j|`) gets its variance from the
    /// Fisher-information covariance of the 2-set union MLE
    /// ([`mle_union_region_covariance_from_registers`](Self::mle_union_region_covariance_from_registers)).
    /// The differential cells of the sketch are differences of these cumulative regions, so the cell
    /// variances propagate through that differencing. For `M = N = 1` the cells ARE the regions and the
    /// error is exact (validated against the measured per-cell spread). For larger grids the
    /// differencing treats the cumulative regions as independent, which over-estimates because it
    /// ignores their strong positive correlation: the bound stays tight on the large diagonal cells but
    /// is loose (an over-estimate by several times) on near-empty off-diagonal or deep cells, where the
    /// correlated cumulative terms would otherwise cancel. Under the nested-shell correlation structure
    /// of a hypersphere sketch this behaves as a conservative over-estimate (it is not an unconditional
    /// mathematical guarantee for arbitrary correlation).
    ///
    /// Note: for operands still pre-dense (hash or value lists) the joint estimate itself uses
    /// near-exact set algebra, so this dense-register-model error is an upper bound in that case.
    #[must_use]
    pub fn joint_sketch_error<const M: usize, const N: usize>(
        lefts: &[Self; M],
        rights: &[Self; N],
    ) -> JointSketchError<M, N> {
        if lefts.iter().all(Self::is_sorted_value_list)
            && rights.iter().all(Self::is_sorted_value_list)
        {
            return JointSketchError {
                overlap_se: [[0.0; N]; M],
                left_diff_se: [0.0; M],
                right_diff_se: [0.0; N],
            };
        }

        // Absolute variance of each cumulative pairwise region `[|A_i \ B_j|, |B_j \ A_i|,
        // |A_i intersect B_j|]`.
        let mut intersection_variance = [[0.0_f64; N]; M];
        let mut left_difference_variance = [[0.0_f64; N]; M];
        let mut right_difference_variance = [[0.0_f64; N]; M];

        if lefts
            .iter()
            .chain(rights.iter())
            .any(super::hyperloglog::HyperLogLog::is_hyperloglog)
        {
            // Any dense operand: materialize everything to registers (as the dense joint sketch does)
            // and take each region variance from the Fisher-information covariance of the 2-set union
            // MLE (log-space covariance scaled back: Var(region) = region^2 * cov_diagonal).
            let lefts: [Self; M] = core::array::from_fn(|i| lefts[i].clone().into_hll());
            let rights: [Self; N] = core::array::from_fn(|j| rights[j].clone().into_hll());
            for (i, left) in lefts.iter().enumerate() {
                for (j, right) in rights.iter().enumerate() {
                    let (regions, covariance) =
                        left.mle_union_region_covariance_from_registers(right);
                    left_difference_variance[i][j] =
                        regions[0] * regions[0] * covariance[0][0].max(0.0);
                    right_difference_variance[i][j] =
                        regions[1] * regions[1] * covariance[1][1].max(0.0);
                    intersection_variance[i][j] =
                        regions[2] * regions[2] * covariance[2][2].max(0.0);
                }
            }
        } else {
            // No dense operand (only value or hash lists): these representations are exact or
            // near-exact, so do NOT materialize them to dense (that would report the far larger dense
            // register error). Instead propagate the pre-dense per-operand standard errors by the
            // delta method. For a pair, `I = |A| + |B| - |A union B|`, `left_diff = |A union B| - |B|`,
            // `right_diff = |A union B| - |A|`, and treating the three terms as independent gives the
            // region variances below.
            let left_variance: [f64; M] = core::array::from_fn(|i| {
                let sd =
                    lefts[i].estimate_cardinality() * lefts[i].predicted_relative_standard_error();
                sd * sd
            });
            let right_variance: [f64; N] = core::array::from_fn(|j| {
                let sd = rights[j].estimate_cardinality()
                    * rights[j].predicted_relative_standard_error();
                sd * sd
            });
            for (i, var_left) in left_variance.iter().enumerate() {
                for (j, var_right) in right_variance.iter().enumerate() {
                    let union = &lefts[i] | &rights[j];
                    let union_sd =
                        union.estimate_cardinality() * union.predicted_relative_standard_error();
                    let var_union = union_sd * union_sd;
                    intersection_variance[i][j] = var_left + var_right + var_union;
                    left_difference_variance[i][j] = var_union + var_right;
                    right_difference_variance[i][j] = var_union + var_left;
                }
            }
        }

        // Overlap cell = 2D difference of cumulative intersections; propagate the four corners.
        let mut overlap_se = [[0.0_f64; N]; M];
        for i in 0..M {
            for j in 0..N {
                let mut variance = intersection_variance[i][j];
                if i > 0 {
                    variance += intersection_variance[i - 1][j];
                }
                if j > 0 {
                    variance += intersection_variance[i][j - 1];
                }
                if i > 0 && j > 0 {
                    variance += intersection_variance[i - 1][j - 1];
                }
                overlap_se[i][j] = FloatOps::sqrt(variance);
            }
        }

        // Left margin = difference of left-difference regions across successive left shells, taken
        // against the largest right operand (index N-1).
        let mut left_diff_se = [0.0_f64; M];
        for i in 0..M {
            let mut variance = left_difference_variance[i][N - 1];
            if i > 0 {
                variance += left_difference_variance[i - 1][N - 1];
            }
            left_diff_se[i] = FloatOps::sqrt(variance);
        }
        // Right margin symmetrically against the largest left operand (index M-1).
        let mut right_diff_se = [0.0_f64; N];
        for j in 0..N {
            let mut variance = right_difference_variance[M - 1][j];
            if j > 0 {
                variance += right_difference_variance[M - 1][j - 1];
            }
            right_diff_se[j] = FloatOps::sqrt(variance);
        }

        JointSketchError {
            overlap_se,
            left_diff_se,
            right_diff_se,
        }
    }

    /// Joint MLE union estimate assuming both counters are in `HyperLogLog` (register) mode.
    fn mle_union_from_registers(&self, other: &Self) -> f64 {
        let [left_difference, right_difference, intersection] =
            self.mle_union_regions_from_registers(other);
        left_difference + right_difference + intersection
    }
}
