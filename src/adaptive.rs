//! The [`Adaptive`] mode wrapper: a view over a [`HyperLogLog`] that auto-selects the more accurate
//! estimator per counter, the fast default everywhere except the saturation window where the
//! maximum-likelihood estimator is more accurate.
//!
//! Obtain it with [`HyperLogLog::adaptive`] and return to the plain default estimators with
//! [`Adaptive::into_inner`]. The default register estimate is O(1) and accurate over the whole normal
//! range, but as tiny registers saturate it develops a large bias (it flatlines at its ceiling) while
//! the maximum-likelihood estimate stays accurate for a window before it too diverges. This view spends
//! the MLE's much higher cost only inside that window, picking the estimator with the smaller predicted
//! error for the counter's current state, and otherwise uses the cheap default.
//!
//! The switch is decided by [`HyperLogLog::prefers_mle`]: the counter must be dense and not fully
//! saturated, and the MLE's predicted standard error must beat the raw estimator's bias. For wide
//! registers (for example `Bits6`) that never saturate in range this is never true, so the view is
//! identical to the default and free.
//!
//! Because it implements [`CardinalityEstimator`] (and [`HyperSpheresSketch`]), the derived
//! intersection / Jaccard / difference estimates and the joint sketch come for free, each computed
//! from the adaptively chosen primitives, and it can be passed to any code generic over those traits.

use crate::error_model::{register_crlb_relative_standard_error, register_raw_bias};
use crate::estimator::CardinalityEstimator;
use crate::prelude::{Bits, HasherType, HyperLogLog, Precision, Registers};
use crate::sketches::HyperSpheresSketch;
use crate::utils::FloatOps;

/// A view over a [`HyperLogLog`] (here a borrowed one, produced by [`HyperLogLog::adaptive`]) that
/// auto-selects the default or maximum-likelihood estimator per counter. See the module documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Adaptive<H>(pub H);

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperLogLog<P, B, R, H> {
    /// Returns a view over this counter that auto-selects the more accurate estimator: the fast
    /// default everywhere except the saturation window where the maximum-likelihood estimator is more
    /// accurate. The view borrows the counter (no copy). Use [`Adaptive::into_inner`] to go back to the
    /// plain default estimators.
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
    /// // With 6-bit registers nothing saturates in range, so this matches the default estimate.
    /// let estimate = counter.adaptive().estimate_cardinality();
    /// assert!((estimate - 40_000.0).abs() / 40_000.0 < 0.1);
    /// ```
    #[inline]
    pub fn adaptive(&self) -> Adaptive<&Self> {
        Adaptive(self)
    }

    /// Whether the maximum-likelihood estimator is the more accurate choice for this counter's current
    /// state, used by the [`Adaptive`] view. True only when the counter is dense, not fully saturated
    /// (the MLE diverges to infinity there), and the MLE's predicted standard error
    /// ([`register_crlb_relative_standard_error`]) is below the raw estimator's systematic bias
    /// ([`register_raw_bias`]), which is exactly the saturation window. Cheap: an O(1) estimate plus an
    /// O(2^B) error evaluation, no register scan.
    #[inline]
    pub(crate) fn prefers_mle(&self) -> bool {
        if !self.is_hyperloglog() || self.is_full() {
            return false;
        }
        let estimate = self.estimate_cardinality();
        estimate > 0.0
            && register_crlb_relative_standard_error::<P, B>(estimate)
                < FloatOps::abs(register_raw_bias::<P, B>(estimate))
    }
}

impl<H> Adaptive<H> {
    /// Returns the wrapped counter (or reference), switching back from the adaptive view to the plain
    /// default estimators that the bare [`HyperLogLog`] provides.
    #[inline]
    pub fn into_inner(self) -> H {
        self.0
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> sketching_core::CardinalityEstimator
    for Adaptive<&HyperLogLog<P, B, R, H>>
{
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        if self.0.prefers_mle() {
            self.0.estimate_cardinality_mle()
        } else {
            self.0.estimate_cardinality()
        }
    }

    #[inline]
    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        if self.0.prefers_mle() || other.0.prefers_mle() {
            self.0.estimate_union_cardinality_mle(other.0)
        } else {
            self.0.estimate_union_cardinality(other.0)
        }
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> CardinalityEstimator
    for Adaptive<&HyperLogLog<P, B, R, H>>
{
    #[inline]
    fn predicted_relative_standard_error(&self) -> f64 {
        if self.0.prefers_mle() {
            self.0.mle().predicted_relative_standard_error()
        } else {
            self.0.predicted_relative_standard_error()
        }
    }

    #[inline]
    fn predicted_bias(&self) -> f64 {
        if self.0.prefers_mle() {
            self.0.mle().predicted_bias()
        } else {
            self.0.predicted_bias()
        }
    }

    #[inline]
    fn relative_standard_error_at(&self, cardinality: f64) -> f64 {
        if self.0.prefers_mle() {
            self.0.mle().relative_standard_error_at(cardinality)
        } else {
            self.0.relative_standard_error_at(cardinality)
        }
    }

    #[inline]
    fn bias_at(&self, cardinality: f64) -> f64 {
        if self.0.prefers_mle() {
            self.0.mle().bias_at(cardinality)
        } else {
            self.0.bias_at(cardinality)
        }
    }
}

// Empty body: inherits the default inclusion-exclusion `joint_sketch`, which runs over this view's
// adaptively chosen cardinality and union estimates.
impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperSpheresSketch
    for Adaptive<&HyperLogLog<P, B, R, H>>
{
}

#[cfg(test)]
mod tests {
    // The adaptive estimate is the bit-exact result of the same call the MLE (or default) view makes,
    // so exact float comparison is intentional here.
    #![allow(clippy::float_cmp)]
    use crate::prelude::*;
    use sketching_core::CardinalityEstimator;

    #[test]
    fn adaptive_uses_mle_in_the_saturation_window() {
        // Tiny registers (Bits4) saturate, so a counter pushed into the MLE-preferred window must
        // switch to the MLE, matching the `.mle()` estimate and differing from the biased default.
        type Hll = HyperLogLog<Precision6, Bits4>;
        let mut h = Hll::default();
        let mut state = 0x1234_5678u64;
        for _ in 0..1_200_000u64 {
            state = splitmix64(state);
            h.insert(&state);
        }
        assert!(h.is_hyperloglog() && !h.is_full());
        assert!(
            h.prefers_mle(),
            "P6B4 at ~1.2M should be inside the MLE-preferred window",
        );
        let adaptive = h.adaptive().estimate_cardinality();
        assert_eq!(
            adaptive,
            h.mle().estimate_cardinality(),
            "adaptive must use the MLE inside the window",
        );
        assert_ne!(
            adaptive,
            h.estimate_cardinality(),
            "adaptive must differ from the biased default estimate here",
        );
    }

    #[test]
    fn adaptive_uses_default_outside_the_window() {
        // Wide registers never saturate, so the MLE is never preferred and the view matches the default.
        type Hll = HyperLogLog<Precision10, Bits6>;
        let mut h = Hll::default();
        let mut state = 0x99u64;
        for _ in 0..100_000u64 {
            state = splitmix64(state);
            h.insert(&state);
        }
        assert!(h.is_hyperloglog() && !h.prefers_mle());
        assert_eq!(
            h.adaptive().estimate_cardinality(),
            h.estimate_cardinality(),
        );

        // A pre-dense (hash list) counter never prefers the MLE either.
        let mut small = Hll::default();
        small.insert(&1u64);
        assert!(!small.is_hyperloglog() && !small.prefers_mle());
        assert_eq!(
            small.adaptive().estimate_cardinality(),
            small.estimate_cardinality(),
        );
    }
}
