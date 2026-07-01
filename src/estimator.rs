//! The [`CardinalityEstimator`] trait: the HyperLogLog-specific estimator interface.
//!
//! It extends the shared [`sketching_core::CardinalityEstimator`] trait (which provides
//! `estimate_cardinality`, `estimate_union_cardinality`, and the derived intersection, Jaccard, and
//! difference estimates) with HyperLogLog-specific error and bias analysis methods.
//!
//! It is implemented by [`HyperLogLog`] (using its default, HyperLogLog++ corrected estimators) and
//! by the [`Mle`](crate::mle::Mle) mode wrapper (using the maximum-likelihood estimators).

use crate::prelude::{Bits, HasherType, HyperLogLog, Precision, Registers};

/// Estimates set cardinalities with HyperLogLog-specific error and bias analysis.
///
/// Extends [`sketching_core::CardinalityEstimator`] to add methods for predicting the theoretical
/// relative standard error and systematic bias of the register estimator. The base trait provides
/// `estimate_cardinality`, `estimate_union_cardinality`, and the derived intersection, Jaccard, and
/// difference estimates.
pub trait CardinalityEstimator: sketching_core::CardinalityEstimator {
    /// The theoretical relative standard error (the sampling error, one standard deviation as a
    /// fraction of the cardinality) of [`estimate_cardinality`](sketching_core::CardinalityEstimator::estimate_cardinality) at this
    /// counter's current estimate. This is the variance term only; the systematic bias is reported
    /// separately by [`predicted_bias`](Self::predicted_bias).
    fn predicted_relative_standard_error(&self) -> f64;

    /// The theoretical systematic relative bias of the register estimator, evaluated at this counter's
    /// current estimate. It models the raw `alpha * m^2 / harmonic_sum` estimator, so it is meaningful
    /// in the saturation regime (above the correction bound `7.5 * 2^P`, where
    /// [`estimate_cardinality`](sketching_core::CardinalityEstimator::estimate_cardinality) returns the raw value), and overstates the
    /// true bias below that bound, where the empirical HyperLogLog++ correction has already removed
    /// most of it. It is zero for the exact value-list and the unbiased hash-list representations.
    /// Because bias is a function of the true (unknown) cardinality, this evaluates the model at the
    /// current estimate; prefer [`bias_at`](Self::bias_at) with a known cardinality for planning.
    fn predicted_bias(&self) -> f64;

    /// The theoretical relative standard error at an arbitrary `cardinality`, using the dense-register
    /// model (the regime where error analysis is meaningful). Takes `&self` only to select the
    /// estimation method and the precision and bits of the type.
    fn relative_standard_error_at(&self, cardinality: f64) -> f64;

    /// The theoretical systematic relative bias at an arbitrary true `cardinality`, using the
    /// dense-register model. Takes `&self` only to select the estimation method and the precision and
    /// bits of the type.
    fn bias_at(&self, cardinality: f64) -> f64;
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> sketching_core::CardinalityEstimator
    for HyperLogLog<P, B, R, H>
{
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        // Resolves to the inherent method (inherent methods take precedence over trait methods), so
        // this delegates to the default HyperLogLog++ estimate rather than recursing.
        self.estimate_cardinality()
    }

    #[inline]
    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        self.estimate_union_cardinality(other)
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> CardinalityEstimator
    for HyperLogLog<P, B, R, H>
{
    #[inline]
    fn predicted_relative_standard_error(&self) -> f64 {
        // A sorted value list is an exact count, so it has no sampling error.
        if self.is_sorted_value_list() {
            return 0.0;
        }
        // A sorted hash list is corrected by the occupancy model; its error follows from it.
        if self.is_sorted_hash_list() {
            return Self::hash_list_relative_standard_error(
                self.get_number_of_hashes().unwrap(),
                self.get_hash_bits().unwrap(),
            );
        }
        crate::error_model::register_default_relative_standard_error::<P>()
    }

    #[inline]
    fn predicted_bias(&self) -> f64 {
        // Only the raw register estimator carries a non-negligible bias (near saturation). The value
        // list is exact and the hash-list occupancy inverse is unbiased in expectation.
        if self.is_hyperloglog() {
            crate::error_model::register_raw_bias::<P, B>(self.estimate_cardinality())
        } else {
            0.0
        }
    }

    #[inline]
    fn relative_standard_error_at(&self, _cardinality: f64) -> f64 {
        // The default estimator's variance is flat across its operating range; its saturation error is
        // the bias, see `bias_at`. The `cardinality` argument is unused for this method.
        crate::error_model::register_default_relative_standard_error::<P>()
    }

    #[inline]
    fn bias_at(&self, cardinality: f64) -> f64 {
        crate::error_model::register_raw_bias::<P, B>(cardinality)
    }
}
