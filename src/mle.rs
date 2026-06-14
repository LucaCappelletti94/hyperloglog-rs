//! Maximum Likelihood Estimation for HyperLogLog cardinalities and set sketches.
//!
//! This module is behind the `mle` feature, which works in no_std + alloc: it stores joint patterns
//! in an `alloc::collections::BTreeMap` and routes the float transcendentals to `libm` (via
//! `num-traits`) when `std` is unavailable, and to the standard library otherwise. It provides three
//! estimators on
//! [`HyperLogLog`], all maximizing a register-multiplicity likelihood:
//! - [`HyperLogLog::estimate_union_cardinality_mle`]: Ertl's 2-set joint union MLE.
//! - [`HyperLogLog::estimate_cardinality_mle`]: Ertl's single-counter cardinality MLE.
//! - [`HyperLogLog::joint_sketch_mle`] / [`HyperLogLog::joint_sketch_mle_with`]: the generalized
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

pub use optimizers::{Adam, Chain, JointOptimizer, Lbfgs, RmsProp};

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
    /// type Hll = HyperLogLog<Precision12, Bits6, <Precision12 as PackedRegister<Bits6>>::Array>;
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
    /// let union = a.estimate_union_cardinality_mle(&b);
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
    pub fn estimate_union_cardinality_mle(&self, other: &Self) -> f64 {
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
    /// type Hll = HyperLogLog<Precision12, Bits6, <Precision12 as PackedRegister<Bits6>>::Array>;
    ///
    /// let mut counter = Hll::default();
    /// for x in 0u64..40_000 {
    ///     counter.insert(&x); // true cardinality is 40000
    /// }
    ///
    /// let estimate = counter.estimate_cardinality_mle();
    /// assert!((estimate - 40_000.0).abs() / 40_000.0 < 0.1);
    /// ```
    ///
    /// # Implementative details
    /// This is Ertl's secant-method maximum-likelihood estimator over the register multiplicities.
    /// A hash-list operand returns the corrected hash-list estimate directly
    /// ([`HyperLogLog::estimate_cardinality`]) rather than being materialized into registers: the
    /// register multiplicities are a function of the stored hashes, so the MLE cannot beat the
    /// near-exact hash-list count. In register mode the MLE is provided for completeness and
    /// comparison: it is less accurate, and substantially slower, than the default corrected
    /// estimate.
    #[inline]
    pub fn estimate_cardinality_mle(&self) -> f64 {
        if self.is_hash_list() {
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
    /// this reduces to the three-region model of [`HyperLogLog::estimate_union_cardinality_mle`].
    ///
    /// # Examples
    /// The `M = N = 1` case decomposes two sets into intersection and the two differences. With
    /// `A = [0, 4000)` and `B = [2000, 6000)`, the intersection `[2000, 4000)` is about 2000 and the
    /// union is about 6000.
    /// ```
    /// use hyperloglog_rs::prelude::*;
    /// type Hll = HyperLogLog<Precision12, Bits6, <Precision12 as PackedRegister<Bits6>>::Array>;
    ///
    /// let mut a = Hll::default();
    /// let mut b = Hll::default();
    /// for x in 0u64..4_000 {
    ///     a.insert(&x);
    /// }
    /// for x in 2_000u64..6_000 {
    ///     b.insert(&x);
    /// }
    ///
    /// let (overlap, left_diff, right_diff) = Hll::joint_sketch_mle(&[a], &[b]);
    /// // overlap[i][j] = |L_i intersect R_j|; here the single intersection cell.
    /// assert!((overlap[0][0] - 2_000.0).abs() / 2_000.0 < 0.25);
    /// let union = overlap[0][0] + left_diff[0] + right_diff[0];
    /// assert!((union - 6_000.0).abs() / 6_000.0 < 0.2);
    /// ```
    ///
    /// # Implementative details
    /// When every operand is still a hash list, the disjoint cells are counted exactly from the
    /// stored composite hashes, with no optimization. Otherwise any hash-list operand is
    /// materialized into registers first, and the optimization is warm-started from the pairwise
    /// sketch and refined with the default `Chain<Adam, Lbfgs>` optimizer, driven by the exact
    /// forward-mode gradient of the joint per-register log-likelihood. Use
    /// [`HyperLogLog::joint_sketch_mle_with`] to pick a different optimizer. See
    /// `docs/joint_mle_math.md`.
    #[inline]
    pub fn joint_sketch_mle<const M: usize, const N: usize>(
        lefts: &[Self; M],
        rights: &[Self; N],
    ) -> ([[f64; N]; M], [f64; M], [f64; N]) {
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
    /// default method uses `Chain<Adam, Lbfgs>`; `Lbfgs` alone is fastest where the objective is
    /// unimodal.
    ///
    /// # Examples
    /// ```
    /// use hyperloglog_rs::prelude::*;
    /// type Hll = HyperLogLog<Precision12, Bits6, <Precision12 as PackedRegister<Bits6>>::Array>;
    ///
    /// let mut a = Hll::default();
    /// let mut b = Hll::default();
    /// for x in 0u64..4_000 {
    ///     a.insert(&x);
    /// }
    /// for x in 2_000u64..6_000 {
    ///     b.insert(&x);
    /// }
    ///
    /// // Fastest: plain L-BFGS (M and N are inferred from the arrays).
    /// let (overlap, left_diff, right_diff) =
    ///     Hll::joint_sketch_mle_with::<Lbfgs, 1, 1>(&[a.clone()], &[b.clone()]);
    /// let union = overlap[0][0] + left_diff[0] + right_diff[0];
    /// assert!((union - 6_000.0).abs() / 6_000.0 < 0.2);
    ///
    /// // Robust: an Adam warmup composed with L-BFGS (this is also the default).
    /// let (overlap, ..) = Hll::joint_sketch_mle_with::<Chain<Adam, Lbfgs>, 1, 1>(&[a], &[b]);
    /// assert!(overlap[0][0] > 0.0);
    /// ```
    #[inline]
    pub fn joint_sketch_mle_with<O: JointOptimizer, const M: usize, const N: usize>(
        lefts: &[Self; M],
        rights: &[Self; N],
    ) -> ([[f64; N]; M], [f64; M], [f64; N]) {
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
