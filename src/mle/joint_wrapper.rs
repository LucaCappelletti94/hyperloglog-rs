//! The [`JointMle`] mode wrapper: a view over a [`HyperLogLog`] whose joint hypersphere sketch runs
//! the generalized joint maximum-likelihood estimator over the disjoint-cell region model.
//!
//! Obtain it with [`HyperLogLog::jmle`] and return to the inner counter with [`JointMle::into_inner`].
//! Unlike the [`Mle`](crate::mle::Mle) wrapper, which builds the joint sketch by pairwise
//! inclusion-exclusion over the 2-set union MLE, [`JointMle`] fits every disjoint region (the
//! `M*N` overlap grid and the `M + N` margins) in a single optimization. The scalar cardinality and
//! union estimates fall back to the inner counter's default (`HyperLogLog`++) estimators, so only the
//! joint sketch differs from a bare [`HyperLogLog`].

use super::sketch::joint_sketch_mle_from_registers;
use crate::estimator::HllCardinalityEstimator;
use crate::prelude::{Bits, HasherType, HyperLogLog, Precision, Registers};
use sketching_core::sparse_value_list::SparseValueCodec;
use crate::sketches::{HyperSpheresSketch, JointSketch};

/// A generalized-joint-MLE view over a [`HyperLogLog`] (here a borrowed one, produced by
/// [`HyperLogLog::jmle`]). See the module documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JointMle<H>(pub H);

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> HyperLogLog<P, B, R, H, C>
where
    C: SparseValueCodec,
{
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

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> sketching_core::CardinalityEstimator
    for JointMle<&HyperLogLog<P, B, R, H, C>>
where
    C: SparseValueCodec,
{
    /// The scalar cardinality is the inner counter's default (`HyperLogLog`++) estimate: the joint MLE
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

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> HllCardinalityEstimator
    for JointMle<&HyperLogLog<P, B, R, H, C>>
where
    C: SparseValueCodec,
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

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> HyperSpheresSketch
    for JointMle<&HyperLogLog<P, B, R, H, C>>
where
    C: SparseValueCodec,
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
        let left_counters: [HyperLogLog<P, B, R, H, C>; L] =
            core::array::from_fn(|i| lefts[i].0.clone());
        let right_counters: [HyperLogLog<P, B, R, H, C>; N] =
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
            return HyperLogLog::<P, B, R, H, C>::joint_sketch_mle::<L, N>(
                &left_counters,
                &right_counters,
            );
        }

        // Materialize every operand to registers (a no-op for those already dense) so all cells share
        // the same footing, then fit the disjoint-region model jointly. The owned arrays above are
        // consumed here, so there is no second copy.
        let left_counters = left_counters.map(HyperLogLog::into_hll);
        let right_counters = right_counters.map(HyperLogLog::into_hll);
        joint_sketch_mle_from_registers::<P, B, R, H, C, L, N>(&left_counters, &right_counters)
    }
}

#[cfg(test)]
mod tests {
    //! Delegator coverage for the `CardinalityEstimator` and `HllCardinalityEstimator` methods on
    //! `JointMle`. The joint-sketch path is exercised by the top-level joint MLE tests; this
    //! module focuses on the scalar delegators (which all forward to the inner counter's default
    //! `HyperLogLog`++ estimators) and `into_inner`.
    #![allow(clippy::float_cmp)]
    use crate::estimator::HllCardinalityEstimator;
    use crate::prelude::*;
    use sketching_core::CardinalityEstimator;

    fn build_hll() -> HyperLogLog<Precision10, Bits6> {
        let mut h = HyperLogLog::<Precision10, Bits6>::default();
        let mut state = 0x2020_2020_2020_2020u64;
        for _ in 0..50_000u64 {
            state = splitmix64(state);
            h.insert(&state);
        }
        h
    }

    /// The scalar cardinality is the inner counter's default `HyperLogLog`++ estimate.
    #[test]
    fn jmle_estimate_cardinality_matches_default() {
        let h = build_hll();
        assert_eq!(h.jmle().estimate_cardinality(), h.estimate_cardinality());
    }

    /// The pairwise union is the inner counter's default estimate; only the joint sketch differs.
    #[test]
    fn jmle_estimate_union_cardinality_matches_default() {
        let mut a = HyperLogLog::<Precision10, Bits6>::default();
        let mut b = HyperLogLog::<Precision10, Bits6>::default();
        let mut state_a = 0xAAAA_AAAAu64;
        let mut state_b = 0xBBBB_BBBBu64;
        for _ in 0..50_000u64 {
            state_a = splitmix64(state_a);
            state_b = splitmix64(state_b);
            a.insert(&state_a);
            b.insert(&state_b);
        }
        assert_eq!(
            a.jmle().estimate_union_cardinality(&b.jmle()),
            a.estimate_union_cardinality(&b),
        );
    }

    /// The error-model methods all forward to the inner counter's default implementation.
    #[test]
    fn jmle_error_model_matches_default() {
        let h = build_hll();
        let view = h.jmle();
        assert_eq!(
            view.predicted_relative_standard_error(),
            h.predicted_relative_standard_error(),
        );
        assert_eq!(view.predicted_bias(), h.predicted_bias());
        assert_eq!(
            view.relative_standard_error_at(100_000.0),
            h.relative_standard_error_at(100_000.0),
        );
        assert_eq!(view.bias_at(100_000.0), h.bias_at(100_000.0));
    }

    /// `into_inner()` returns the wrapped reference.
    #[test]
    fn jmle_into_inner_returns_wrapped_reference() {
        let h = build_hll();
        let view = h.jmle();
        let inner: &HyperLogLog<Precision10, Bits6> = view.into_inner();
        assert_eq!(inner.estimate_cardinality(), h.estimate_cardinality());
    }
}
