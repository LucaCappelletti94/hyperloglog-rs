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
//! - [`JointSketch::estimate`] / [`JointSketch::estimate_with`] over `.mle()` views: the generalized
//!   hypersphere-sketch MLE over `M` nested left and `N` nested right counters, jointly estimating
//!   all `M*N + M + N` disjoint-cell cardinalities.
//!
//! The submodules hold the implementation: `union` and `cardinality` (the classic Ertl
//! estimators), `likelihood` (the polynomial per-register likelihood and gradient), `sketch` (warm
//! start, marginal anchor, optimization), `optimizers` (the pluggable, compile-time `JointOptimizer`
//! family), and the test-only `oracle` (the exponential reference). The maths is in
//! `docs/joint_mle_math.md`.

use crate::correction_coefficients::{
    HYPERLOGLOG_CORRECTION_BIAS, HYPERLOGLOG_CORRECTION_CARDINALITIES,
};
use crate::hyperloglog::correct_cardinality;
use crate::prelude::*;
use crate::utils::FloatOps;

mod cardinality;
mod exact;
mod likelihood;
mod optimizers;
#[cfg(test)]
mod oracle;
mod sketch;
#[cfg(test)]
mod tests;
mod union;
mod wrapper;

pub use optimizers::{Adam, Chain, JointOptimizer, Lbfgs, RmsProp};
pub use wrapper::Mle;

// The associative map used to tabulate joint patterns and classify exact cells. It is the
// no_std-friendly `alloc::collections::BTreeMap` (the keys are all `Ord`); benchmarks showed it
// matches `std::collections::HashMap` end to end (often faster, the maps are small).
pub(crate) use alloc::collections::BTreeMap as PatternMap;

use cardinality::mle_cardinality;
use exact::joint_sketch_exact_from_hash_lists;
#[cfg(feature = "exact")]
use exact::joint_sketch_exact_from_values;
use sketch::{joint_sketch_mle_from_registers, joint_sketch_mle_from_registers_with};
use union::mle_union_cardinality;

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
    /// lists, the near-exact hash-list union ([`HyperLogLog::estimate_union_cardinality`]) is used
    /// directly, since the stored hashes carry more information than the register multiplicities the
    /// MLE consumes. When both are fully-fledged HyperLogLogs, the left difference, right difference
    /// and intersection likelihood is maximized jointly and the union estimate is their sum. In the
    /// mixed case the hash-list operand is materialized into registers first and the MLE is run.
    #[inline]
    pub(crate) fn estimate_union_cardinality_mle(&self, other: &Self) -> f64 {
        // Exact-values operands are resolved first: two exact operands give the exact union, and a
        // mixed pair promotes the exact one to a hash list (a clone) before falling through.
        #[cfg(feature = "exact")]
        {
            if self.is_exact() && other.is_exact() {
                return self.estimate_union_cardinality(other);
            }
            if self.is_exact() {
                let mut promoted = self.clone();
                promoted.convert_exact_to_hash_list().unwrap();
                return promoted.estimate_union_cardinality_mle(other);
            }
            if other.is_exact() {
                let mut promoted = other.clone();
                promoted.convert_exact_to_hash_list().unwrap();
                return self.estimate_union_cardinality_mle(&promoted);
            }
        }

        if self.is_hash_list() && other.is_hash_list() {
            return self.estimate_union_cardinality(other);
        }

        if self.is_hash_list() || other.is_hash_list() {
            let mut left = self.clone();
            let mut right = other.clone();
            if left.is_hash_list() {
                left.convert_hash_list_to_hyperloglog().unwrap();
            }
            if right.is_hash_list() {
                right.convert_hash_list_to_hyperloglog().unwrap();
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
    /// A pre-dense operand (exact values or a hash list) returns the default estimate directly
    /// ([`HyperLogLog::estimate_cardinality`]) rather than being run through the register MLE: those
    /// representations already carry a more accurate direct count, and their backing buffer is not a
    /// register multiset. In register mode the MLE is provided for completeness and comparison: it is
    /// less accurate, and substantially slower, than the default corrected estimate.
    #[inline]
    pub(crate) fn estimate_cardinality_mle(&self) -> f64 {
        // Both pre-dense representations (exact values and the hash list) carry a more accurate
        // direct count than the register MLE could recover, and their `registers` buffer is not a
        // register multiset, so fall back to the default estimate rather than running the MLE.
        if !self.is_dense() {
            return self.estimate_cardinality();
        }

        mle_cardinality::<P, B>(
            self.registers.iter_registers(),
            self.harmonic_sum,
            self.is_full(),
            2,
        )
    }

    /// Generalized joint Maximum Likelihood Estimation of the disjoint-cell cardinalities of the
    /// hypersphere sketch for `M` nested left counters and `N` nested right counters.
    ///
    /// Given `lefts = [A_0 subset ... subset A_{M-1}]` and `rights = [B_0 subset ... subset
    /// B_{N-1}]`, this jointly estimates, in a single optimization over the disjoint-region model,
    /// all `M*N + M + N` non-negative cell cardinalities:
    /// * `overlap[i][j] = |L_i intersect R_j|`, the exclusive overlap grid, where `L_i = A_i \
    ///   A_{i-1}` and `R_j = B_j \ B_{j-1}` are the left/right shells.
    /// * `left_diff[i] = |L_i \ B_{N-1}|` and `right_diff[j] = |R_j \ A_{M-1}|`, the margins.
    ///
    /// Because the parameters are the disjoint regions themselves (optimized in log-space), the
    /// returned cells are non-negative and globally consistent by construction. At `M = N = 1`
    /// this reduces to the three-region model of the 2-set joint union MLE.
    ///
    /// This is the crate-internal engine behind the public [`JointSketch::estimate`] /
    /// [`JointSketch::estimate_with`], reached by passing [`mle`](HyperLogLog::mle) views as operands.
    ///
    /// # Implementative details
    /// When every operand is still a hash list, the disjoint cells are counted exactly from the
    /// stored composite hashes, with no optimization. Otherwise any hash-list operand is
    /// materialized into registers first, and the optimization is warm-started from the pairwise
    /// sketch and refined with the default `Chain<Adam, Lbfgs>` optimizer, driven by the exact
    /// forward-mode gradient of the joint per-register log-likelihood. See `docs/joint_mle_math.md`.
    #[inline]
    pub(crate) fn joint_sketch_mle<const M: usize, const N: usize>(
        lefts: &[Self; M],
        rights: &[Self; N],
    ) -> JointSketch<M, N> {
        #[cfg(feature = "exact")]
        if lefts.iter().all(Self::is_exact) && rights.iter().all(Self::is_exact) {
            return joint_sketch_exact_from_values::<P, B, R, H, M, N>(lefts, rights);
        }
        if lefts.iter().all(Self::is_hash_list) && rights.iter().all(Self::is_hash_list) {
            return joint_sketch_exact_from_hash_lists::<P, B, R, H, M, N>(lefts, rights);
        }

        let lefts: [Self; M] = core::array::from_fn(|i| Self::materialize_to_registers(&lefts[i]));
        let rights: [Self; N] =
            core::array::from_fn(|j| Self::materialize_to_registers(&rights[j]));

        joint_sketch_mle_from_registers::<P, B, R, H, M, N>(&lefts, &rights)
    }

    /// Same as [`HyperLogLog::joint_sketch_mle`] but with a caller-chosen optimizer type driving the
    /// refinement, selected at compile time by turbofish. Compose optimizers with [`Chain`], for
    /// example `Chain<Adam, Lbfgs>`, or implement [`JointOptimizer`] for a custom strategy. The
    /// default uses `Chain<Adam, Lbfgs>`; `Lbfgs` alone is fastest where the objective is unimodal.
    ///
    /// This is the crate-internal engine behind the public [`JointSketch::estimate_with`].
    #[inline]
    pub(crate) fn joint_sketch_mle_with<O: JointOptimizer, const M: usize, const N: usize>(
        lefts: &[Self; M],
        rights: &[Self; N],
    ) -> JointSketch<M, N> {
        // When every operand is in a recoverable pre-dense representation, the exact set-algebra
        // paths are used and the optimizer type O is irrelevant: the result is exact and
        // optimizer-independent.
        #[cfg(feature = "exact")]
        if lefts.iter().all(Self::is_exact) && rights.iter().all(Self::is_exact) {
            return joint_sketch_exact_from_values::<P, B, R, H, M, N>(lefts, rights);
        }
        if lefts.iter().all(Self::is_hash_list) && rights.iter().all(Self::is_hash_list) {
            return joint_sketch_exact_from_hash_lists::<P, B, R, H, M, N>(lefts, rights);
        }

        let lefts: [Self; M] = core::array::from_fn(|i| Self::materialize_to_registers(&lefts[i]));
        let rights: [Self; N] =
            core::array::from_fn(|j| Self::materialize_to_registers(&rights[j]));

        joint_sketch_mle_from_registers_with::<P, B, R, H, O, M, N>(&lefts, &rights)
    }

    /// Clones `counter` and materializes it into dense register mode, stepping an exact-values
    /// operand through the hash list first.
    #[inline]
    fn materialize_to_registers(counter: &Self) -> Self {
        let mut counter = counter.clone();
        #[cfg(feature = "exact")]
        if counter.is_exact() {
            counter.convert_exact_to_hash_list().unwrap();
        }
        if counter.is_hash_list() {
            counter.convert_hash_list_to_hyperloglog().unwrap();
        }
        counter
    }

    /// Joint MLE union estimate assuming both counters are in HyperLogLog (register) mode.
    fn mle_union_from_registers(&self, other: &Self) -> f64 {
        // Maps a union harmonic sum to the HyperLogLog++ corrected cardinality, exactly as the
        // default register-based union estimator does.
        let estimate = |harmonic_sum: f64, _zeros: u32| {
            correct_cardinality::<P, B>(
                P::ALPHA * f64::integer_exp2(P::EXPONENT + P::EXPONENT) / harmonic_sum,
                &HYPERLOGLOG_CORRECTION_CARDINALITIES[P::EXPONENT as usize - 4]
                    [B::NUMBER_OF_BITS as usize - 4],
                &HYPERLOGLOG_CORRECTION_BIAS[P::EXPONENT as usize - 4]
                    [B::NUMBER_OF_BITS as usize - 4],
            )
        };

        mle_union_cardinality::<P, B, _>(
            self.registers.iter_registers_zipped(&other.registers),
            self.estimate_cardinality(),
            other.estimate_cardinality(),
            estimate,
            2,
        )
    }
}
