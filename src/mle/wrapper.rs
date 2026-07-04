//! The [`Mle`] mode wrapper: a view over a [`HyperLogLog`] whose set estimates use the
//! maximum-likelihood estimators instead of the default `HyperLogLog`++ ones.
//!
//! Obtain it with [`HyperLogLog::mle`] and return to the default estimators with
//! [`Mle::into_inner`]. Because it implements [`CardinalityEstimator`] (and
//! [`HyperSpheresSketch`]), the derived intersection / Jaccard / difference estimates and the
//! overlap matrices come for free, all computed from the MLE primitives, and `Mle` can be passed to
//! any code generic over those traits.

use crate::estimator::HllCardinalityEstimator;
use crate::prelude::{Bits, HasherType, HyperLogLog, Precision, Registers};
use crate::sketches::{HyperSpheresSketch, JointSketch};
use sketching_core::sparse_value_list::SparseValueCodec;

/// A maximum-likelihood-estimation view over a [`HyperLogLog`] (here a borrowed one, produced by
/// [`HyperLogLog::mle`]). See the module documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mle<H>(pub H);

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> HyperLogLog<P, B, R, H, C>
where
    C: SparseValueCodec,
{
    /// Returns a maximum-likelihood-estimation view over this counter. The view borrows the counter
    /// (no copy) and routes its estimates through the MLE estimators. Use [`Mle::into_inner`] to go
    /// back to the default estimators.
    #[inline]
    pub fn mle(&self) -> Mle<&Self> {
        Mle(self)
    }
}

impl<H> Mle<H> {
    /// Returns the wrapped counter (or reference), switching back from MLE to the default
    /// (`HyperLogLog`++) estimators, which the bare [`HyperLogLog`] provides.
    #[inline]
    pub fn into_inner(self) -> H {
        self.0
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C>
    sketching_core::CardinalityEstimator for Mle<&HyperLogLog<P, B, R, H, C>>
where
    C: SparseValueCodec,
{
    /// Estimates the cardinality via the single-counter maximum-likelihood estimator.
    ///
    /// # Warning
    /// This estimator is dominated by the default [`HyperLogLog::estimate_cardinality`]: it is both
    /// slower and less accurate. It exists for completeness and comparison. Prefer the default
    /// (i.e. `self.into_inner().estimate_cardinality()`) unless you specifically want the MLE value.
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        self.0.estimate_cardinality_mle()
    }

    #[inline]
    /// Estimates the union cardinality via the joint maximum-likelihood estimator.
    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        self.0.estimate_union_cardinality_mle(other.0)
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> HllCardinalityEstimator
    for Mle<&HyperLogLog<P, B, R, H, C>>
where
    C: SparseValueCodec,
{
    #[inline]
    fn predicted_relative_standard_error(&self) -> f64 {
        // For a pre-dense operand the MLE falls back to the default estimate, so its error model does
        // too.
        if !self.0.is_hyperloglog() {
            return self.0.predicted_relative_standard_error();
        }
        // A fully saturated counter carries no information: the MLE estimate diverges and so does its
        // standard error.
        if self.0.is_full() {
            return f64::INFINITY;
        }
        crate::error_model::register_crlb_relative_standard_error::<P, B>(
            self.0.estimate_cardinality_mle(),
        )
    }

    #[inline]
    fn predicted_bias(&self) -> f64 {
        // The MLE is asymptotically unbiased until it diverges at full saturation; the pre-dense
        // fallback follows the default.
        if !self.0.is_hyperloglog() {
            return self.0.predicted_bias();
        }
        0.0
    }

    #[inline]
    fn relative_standard_error_at(&self, cardinality: f64) -> f64 {
        crate::error_model::register_crlb_relative_standard_error::<P, B>(cardinality)
    }

    #[inline]
    fn bias_at(&self, _cardinality: f64) -> f64 {
        0.0
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> HyperSpheresSketch
    for Mle<&HyperLogLog<P, B, R, H, C>>
where
    C: SparseValueCodec,
{
    #[inline]
    /// Overridden so the joint sketch over MLE views uses the maximum-likelihood union estimates:
    /// exact set algebra while the operands are still pre-dense, otherwise pairwise
    /// inclusion-exclusion over the 2-set union MLE (see [`HyperLogLog::joint_sketch_mle`]). Without
    /// this override the trait default would use the operands' own (HLL++) estimators.
    fn joint_sketch<const L: usize, const N: usize>(
        lefts: &[Self; L],
        rights: &[Self; N],
    ) -> JointSketch<L, N> {
        let left_counters: [HyperLogLog<P, B, R, H, C>; L] =
            core::array::from_fn(|i| lefts[i].0.clone());
        let right_counters: [HyperLogLog<P, B, R, H, C>; N] =
            core::array::from_fn(|j| rights[j].0.clone());
        HyperLogLog::<P, B, R, H, C>::joint_sketch_mle::<L, N>(&left_counters, &right_counters)
    }
}

#[cfg(test)]
mod tests {
    //! Delegator coverage for the `HllCardinalityEstimator` and `into_inner` methods on `Mle`.
    //! The `estimate_cardinality` / `estimate_union_cardinality` delegators are already exercised
    //! by the top-level MLE tests; this module focuses on the error-model forwarders and
    //! `into_inner`, which no other test currently touches under `--lib`.
    #![allow(clippy::float_cmp)]
    use crate::prelude::*;

    /// Pre-dense (hash-list) counter: the MLE view falls back to the default error model.
    #[test]
    fn mle_error_model_pre_dense_matches_default() {
        type Hll = HyperLogLog<Precision10, Bits6>;
        let mut h = Hll::default();
        for value in 0u64..40 {
            h.insert(&value);
        }
        assert!(!h.is_hyperloglog());
        let view = h.mle();
        assert_eq!(
            view.predicted_relative_standard_error(),
            h.predicted_relative_standard_error(),
        );
        assert_eq!(view.predicted_bias(), h.predicted_bias());
    }

    /// Dense, unsaturated counter: the MLE view uses the register-CRLB error model with a zero
    /// asymptotic bias.
    #[test]
    fn mle_error_model_dense_uses_crlb_and_zero_bias() {
        type Hll = HyperLogLog<Precision10, Bits6>;
        let mut h = Hll::default();
        let mut state = 0x123u64;
        for _ in 0..50_000u64 {
            state = splitmix64(state);
            h.insert(&state);
        }
        assert!(h.is_hyperloglog() && !h.is_full());
        let view = h.mle();
        // `predicted_relative_standard_error` calls `crlb(cardinality_mle)`; positive and finite.
        let rse = view.predicted_relative_standard_error();
        assert!(rse.is_finite() && rse > 0.0);
        // Reference implementation of the same call.
        let expected = crate::error_model::register_crlb_relative_standard_error::<
            Precision10,
            Bits6,
        >(h.estimate_cardinality_mle());
        assert_eq!(rse, expected);
        // The MLE is asymptotically unbiased.
        assert_eq!(view.predicted_bias(), 0.0);
        // `bias_at` is zero for any cardinality by construction.
        assert_eq!(view.bias_at(1_000_000.0), 0.0);
        // `relative_standard_error_at` matches the CRLB at the supplied cardinality.
        assert_eq!(
            view.relative_standard_error_at(100_000.0),
            crate::error_model::register_crlb_relative_standard_error::<Precision10, Bits6>(
                100_000.0
            ),
        );
    }

    /// Full saturated counter: the CRLB diverges, so the MLE view returns `f64::INFINITY` for the
    /// standard error.
    #[test]
    fn mle_error_model_full_counter_reports_infinite_error() {
        // `Precision4, Bits4` saturates quickly at ~5M distinct inputs (m = 16, q + 1 = 15).
        type Hll = HyperLogLog<Precision4, Bits4>;
        let mut h = Hll::default();
        let mut state = 0x7u64;
        for _ in 0..5_000_000u64 {
            state = splitmix64(state);
            h.insert(&state);
        }
        assert!(h.is_hyperloglog() && h.is_full());
        assert_eq!(h.mle().predicted_relative_standard_error(), f64::INFINITY);
    }

    /// `into_inner()` returns the wrapped reference; downstream calls on it match calls on the
    /// bare counter.
    #[test]
    fn mle_into_inner_returns_wrapped_reference() {
        type Hll = HyperLogLog<Precision10, Bits6>;
        let mut h = Hll::default();
        for value in 0u64..50 {
            h.insert(&value);
        }
        let view = h.mle();
        let inner: &Hll = view.into_inner();
        assert_eq!(inner.estimate_cardinality(), h.estimate_cardinality());
    }
}
