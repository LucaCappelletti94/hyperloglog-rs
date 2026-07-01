//! The [`JointMle`] mode wrapper: a view over a [`HyperLogLog`] whose joint hypersphere sketch runs
//! the generalized joint maximum-likelihood estimator over the disjoint-cell region model.
//!
//! Obtain it with [`HyperLogLog::jmle`] and return to the inner counter with [`JointMle::into_inner`].
//! Unlike the [`Mle`](crate::mle::Mle) wrapper, which builds the joint sketch by pairwise
//! inclusion-exclusion over the 2-set union MLE, [`JointMle`] fits every disjoint region (the
//! `M*N` overlap grid and the `M + N` margins) in a single optimization. The scalar cardinality and
//! union estimates fall back to the inner counter's default (HyperLogLog++) estimators, so only the
//! joint sketch differs from a bare [`HyperLogLog`].

use super::sketch::joint_sketch_mle_from_registers;
use crate::estimator::CardinalityEstimator;
use crate::prelude::{Bits, HasherType, HyperLogLog, Precision, Registers};
use crate::sketches::{HyperSpheresSketch, JointSketch};

/// A generalized-joint-MLE view over a [`HyperLogLog`] (here a borrowed one, produced by
/// [`HyperLogLog::jmle`]). See the module documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JointMle<H>(pub H);

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperLogLog<P, B, R, H> {
    /// Returns a generalized-joint-MLE view over this counter. The view borrows the counter (no
    /// copy). Its joint hypersphere sketch ([`JointSketch::estimate`]) runs the joint maximum
    /// likelihood optimization over the disjoint-cell region model, which fits every overlap cell and
    /// margin together rather than by pairwise inclusion-exclusion. Use [`JointMle::into_inner`] to go
    /// back to the bare counter.
    ///
    /// # Examples
    /// ```
    /// use hyperloglog_rs::prelude::*;
    /// type Hll = HyperLogLog<Precision10, Bits6>;
    ///
    /// let mut a0 = Hll::default();
    /// let mut a1 = Hll::default();
    /// let mut b0 = Hll::default();
    /// let mut b1 = Hll::default();
    /// for x in 0u64..20_000 {
    ///     a0.insert(&x);
    ///     a1.insert(&x);
    /// }
    /// for x in 20_000u64..40_000 {
    ///     a1.insert(&x);
    /// }
    /// for x in 10_000u64..30_000 {
    ///     b0.insert(&x);
    ///     b1.insert(&x);
    /// }
    /// for x in 30_000u64..50_000 {
    ///     b1.insert(&x);
    /// }
    ///
    /// let sketch = JointSketch::estimate(&[a0.jmle(), a1.jmle()], &[b0.jmle(), b1.jmle()]);
    /// assert!((sketch.union() - 50_000.0).abs() / 50_000.0 < 0.2);
    /// ```
    #[inline]
    pub fn jmle(&self) -> JointMle<&Self> {
        JointMle(self)
    }
}

impl<H> JointMle<H> {
    /// Returns the wrapped counter (or reference), switching back from the joint MLE to the bare
    /// [`HyperLogLog`].
    #[inline]
    pub fn into_inner(self) -> H {
        self.0
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> sketching_core::CardinalityEstimator
    for JointMle<&HyperLogLog<P, B, R, H>>
{
    /// The scalar cardinality is the inner counter's default (HyperLogLog++) estimate: the joint MLE
    /// refines the disjoint-region decomposition, not the single-counter cardinality.
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        self.0.estimate_cardinality()
    }

    /// The pairwise union is the inner counter's default estimate. The joint MLE only changes the
    /// multi-set [`joint_sketch`](HyperSpheresSketch::joint_sketch).
    #[inline]
    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        self.0.estimate_union_cardinality(other.0)
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> CardinalityEstimator
    for JointMle<&HyperLogLog<P, B, R, H>>
{
    #[inline]
    fn predicted_relative_standard_error(&self) -> f64 {
        self.0.predicted_relative_standard_error()
    }

    #[inline]
    fn predicted_bias(&self) -> f64 {
        self.0.predicted_bias()
    }

    #[inline]
    fn relative_standard_error_at(&self, cardinality: f64) -> f64 {
        self.0.relative_standard_error_at(cardinality)
    }

    #[inline]
    fn bias_at(&self, cardinality: f64) -> f64 {
        self.0.bias_at(cardinality)
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperSpheresSketch
    for JointMle<&HyperLogLog<P, B, R, H>>
{
    /// Runs the generalized joint MLE over the disjoint-region model. The dispatch mirrors the bare
    /// [`joint_sketch_mle`](HyperLogLog::joint_sketch_mle): all-value-list operands are counted
    /// exactly, an all-pre-dense (value or hash list) mix uses the near-exact pairwise
    /// inclusion-exclusion path, and once any operand is dense every operand is materialized to
    /// registers and the cells are fit jointly by [`joint_sketch_mle_from_registers`]. The register
    /// path is what makes this wrapper distinct from [`Mle`](crate::mle::Mle), which always
    /// decomposes pairwise.
    #[inline]
    fn joint_sketch<const L: usize, const N: usize>(
        lefts: &[Self; L],
        rights: &[Self; N],
    ) -> JointSketch<L, N> {
        let left_counters: [HyperLogLog<P, B, R, H>; L] =
            core::array::from_fn(|i| lefts[i].0.clone());
        let right_counters: [HyperLogLog<P, B, R, H>; N] =
            core::array::from_fn(|j| rights[j].0.clone());

        // Pre-dense operands (all value lists, or any value/hash-list mix with no dense operand) are
        // resolved by the bare dispatcher's near-exact set algebra: materializing them to registers
        // would only add register noise. The joint MLE register optimization is the win once an
        // operand is genuinely dense.
        let any_dense = left_counters
            .iter()
            .chain(right_counters.iter())
            .any(HyperLogLog::is_hyperloglog);
        if !any_dense {
            return HyperLogLog::<P, B, R, H>::joint_sketch_mle::<L, N>(
                &left_counters,
                &right_counters,
            );
        }

        // Materialize every operand to registers (a no-op for those already dense) so all cells share
        // the same footing, then fit the disjoint-region model jointly. The owned arrays above are
        // consumed here, so there is no second copy.
        let left_counters = left_counters.map(HyperLogLog::into_hll);
        let right_counters = right_counters.map(HyperLogLog::into_hll);
        joint_sketch_mle_from_registers::<P, B, R, H, L, N>(&left_counters, &right_counters)
    }
}
