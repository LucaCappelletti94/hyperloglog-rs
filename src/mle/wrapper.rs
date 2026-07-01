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

/// A maximum-likelihood-estimation view over a [`HyperLogLog`] (here a borrowed one, produced by
/// [`HyperLogLog::mle`]). See the module documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mle<H>(pub H);

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperLogLog<P, B, R, H> {
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

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> sketching_core::CardinalityEstimator
    for Mle<&HyperLogLog<P, B, R, H>>
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

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HllCardinalityEstimator
    for Mle<&HyperLogLog<P, B, R, H>>
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

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperSpheresSketch
    for Mle<&HyperLogLog<P, B, R, H>>
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
        let left_counters: [HyperLogLog<P, B, R, H>; L] =
            core::array::from_fn(|i| lefts[i].0.clone());
        let right_counters: [HyperLogLog<P, B, R, H>; N] =
            core::array::from_fn(|j| rights[j].0.clone());
        HyperLogLog::<P, B, R, H>::joint_sketch_mle::<L, N>(&left_counters, &right_counters)
    }
}
