//! The [`NoLinearCounting`] mode wrapper: a view over a [`HyperLogLog`] whose register estimates
//! skip the low-load linear-counting correction and always use the bias-corrected raw estimate.
//!
//! Obtain it with [`HyperLogLog::no_linear_counting`] and return to the default estimators with
//! [`NoLinearCounting::into_inner`]. It exists to measure how much the linear-counting branch of the
//! register estimator contributes at low load. In the default estimator linear counting and the
//! empirical bias correction are mutually exclusive: at low load it returns the bare `m * ln(m /
//! zeros)` with NO bias-table correction, and the bias correction is applied only to the raw estimate
//! when linear counting is not used (see `corrected_register_cardinality`). This view always takes
//! the bias-corrected raw estimate, so comparing it against the default measures exactly the gap
//! between the two competing low-load estimators (linear counting versus the bias-corrected raw
//! fallback). It only changes behavior for register-mode counters, because the pre-dense
//! representations (sorted value list and sorted hash list) never use linear counting. For those it
//! delegates to the default estimators.
//!
//! Because it implements [`CardinalityEstimator`] (and [`HyperSpheresSketch`]), the derived
//! intersection / Jaccard / difference estimates and the joint sketch come for free, computed from
//! the bypassed cardinality and union primitives, and it can be passed to any code generic over those
//! traits. One edge case is not specially handled: the union of a register-mode counter with a
//! pre-dense one falls back to the default union estimate, which can still apply linear counting on
//! the reconstructed union. The bypass is exact for register-mode cardinalities and for the union of
//! two register-mode counters, which is the regime the wrapper is meant to measure.

use crate::estimator::HllCardinalityEstimator;
use crate::prelude::{Bits, HasherType, HyperLogLog, Precision, Registers};
use crate::sketches::HyperSpheresSketch;

/// A view over a [`HyperLogLog`] (here a borrowed one, produced by
/// [`HyperLogLog::no_linear_counting`]) whose register estimates bypass linear counting. See the
/// module documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoLinearCounting<H>(pub H);

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperLogLog<P, B, R, H> {
    /// Returns a view over this counter whose register estimates skip linear counting and always use
    /// the bias-corrected raw estimate. The view borrows the counter (no copy). Use
    /// [`NoLinearCounting::into_inner`] to go back to the default estimators.
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
    /// let estimate = counter.no_linear_counting().estimate_cardinality();
    /// assert!((estimate - 40_000.0).abs() / 40_000.0 < 0.1);
    /// ```
    #[inline]
    pub fn no_linear_counting(&self) -> NoLinearCounting<&Self> {
        NoLinearCounting(self)
    }
}

impl<H> NoLinearCounting<H> {
    /// Returns the wrapped counter (or reference), switching back from the no-linear-counting view to
    /// the default estimators, which the bare [`HyperLogLog`] provides.
    #[inline]
    pub fn into_inner(self) -> H {
        self.0
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> sketching_core::CardinalityEstimator
    for NoLinearCounting<&HyperLogLog<P, B, R, H>>
{
    /// Estimates the cardinality with the register linear-counting branch bypassed (bias-corrected
    /// raw in register mode, the default estimate while pre-dense).
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        self.0.estimate_cardinality_no_linear_counting()
    }

    /// Estimates the union cardinality with linear counting bypassed for register-mode operands.
    #[inline]
    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        self.0
            .estimate_union_cardinality_no_linear_counting(other.0)
    }
}

// The error model is independent of the linear-counting choice (linear counting only swaps the
// estimator at low register load, not its variance or the raw-estimator bias), so these delegate
// to the inner default-estimator error model.
impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HllCardinalityEstimator
    for NoLinearCounting<&HyperLogLog<P, B, R, H>>
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

// Empty body: inherits the default inclusion-exclusion `joint_sketch`, which runs over this view's
// (linear-counting-free) cardinality and union estimates, so the joint sketch bypasses linear
// counting without any per-sketch code.
impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperSpheresSketch
    for NoLinearCounting<&HyperLogLog<P, B, R, H>>
{
}

#[cfg(test)]
mod tests {
    // The assertions compare estimates that are the bit-exact result of the same computation (the
    // view must equal `bias_corrected_raw_cardinality`, or equal the default), so exact float
    // comparison is intentional here.
    #![allow(clippy::float_cmp)]
    use crate::prelude::*;
    use sketching_core::CardinalityEstimator;

    type Hll = HyperLogLog<Precision12, Bits6>;

    /// Forced dense at a low cardinality, the default uses linear counting while the view uses the
    /// bias-corrected raw estimate, so the two must differ and the view must equal
    /// `bias_corrected_raw_cardinality` exactly.
    #[test]
    fn bypasses_linear_counting_when_dense_at_low_load() {
        let mut hll = Hll::default();
        for x in 0u64..200 {
            hll.insert(&x);
        }
        let dense = hll.into_hll();
        assert!(dense.is_hyperloglog(), "operand must be forced dense");
        assert_eq!(
            dense.estimation_regime(),
            EstimationRegime::HyperLogLogLinearCounted,
            "low-load forced-dense counter must be in the linear-counting regime",
        );

        let default = dense.estimate_cardinality();
        let bypass = dense.no_linear_counting().estimate_cardinality();
        assert_ne!(
            default, bypass,
            "the view must skip linear counting where the default applies it",
        );
        assert_eq!(
            bypass,
            Hll::bias_corrected_raw_cardinality(dense.dense_harmonic_sum()),
            "the view must equal the bias-corrected raw estimate directly",
        );
    }

    /// Pre-dense (hash list) operands never use linear counting, so the view is identical to the
    /// default for both cardinality and union.
    #[test]
    fn matches_default_when_pre_dense() {
        let mut a = Hll::default();
        for x in 0u64..200 {
            a.insert(&x);
        }
        let mut b = Hll::default();
        for x in 100u64..300 {
            b.insert(&x);
        }
        assert!(
            a.is_sorted_hash_list() && b.is_sorted_hash_list(),
            "operands must be pre-dense hash lists",
        );

        assert_eq!(
            a.estimate_cardinality(),
            a.no_linear_counting().estimate_cardinality(),
        );
        assert_eq!(
            a.estimate_union_cardinality(&b),
            a.no_linear_counting()
                .estimate_union_cardinality(&b.no_linear_counting()),
        );
    }
}
