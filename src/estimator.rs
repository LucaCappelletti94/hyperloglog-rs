//! The [`CardinalityEstimator`] trait: the common interface for estimating set cardinalities and
//! their pairwise relations.
//!
//! It is implemented by [`HyperLogLog`] (using its default, HyperLogLog++ corrected estimators) and
//! by the [`Mle`](crate::mle::Mle) mode wrapper (using the maximum-likelihood estimators). The
//! intersection, Jaccard and difference estimates are derived once here, as provided methods on top
//! of `estimate_cardinality` and `estimate_union_cardinality`, so every estimator mode gets them
//! consistently.

use crate::prelude::{Bits, HasherType, HyperLogLog, Precision, Registers};

/// Estimates set cardinalities and the cardinalities of pairwise set relations.
///
/// Only [`estimate_cardinality`](CardinalityEstimator::estimate_cardinality) and
/// [`estimate_union_cardinality`](CardinalityEstimator::estimate_union_cardinality) are required;
/// the intersection, difference and Jaccard estimates are derived from those two by inclusion and
/// exclusion (clamped to remain non-negative).
pub trait CardinalityEstimator {
    /// Estimates the cardinality of this set.
    fn estimate_cardinality(&self) -> f64;

    /// Estimates the cardinality of the union of this set and `other`.
    fn estimate_union_cardinality(&self, other: &Self) -> f64;

    /// The theoretical relative standard error (the sampling error, one standard deviation as a
    /// fraction of the cardinality) of [`estimate_cardinality`](Self::estimate_cardinality) at this
    /// counter's current estimate. This is the variance term only; the systematic bias is reported
    /// separately by [`predicted_bias`](Self::predicted_bias).
    fn predicted_relative_standard_error(&self) -> f64;

    /// The theoretical systematic relative bias of the register estimator, evaluated at this counter's
    /// current estimate. It models the raw `alpha * m^2 / harmonic_sum` estimator, so it is meaningful
    /// in the saturation regime (above the correction bound `7.5 * 2^P`, where
    /// [`estimate_cardinality`](Self::estimate_cardinality) returns the raw value), and overstates the
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

    #[inline]
    /// Estimates the cardinality of the intersection, derived as `max(0, |A| + |B| - |A union B|)`.
    fn estimate_intersection_cardinality(&self, other: &Self) -> f64 {
        let self_cardinality = self.estimate_cardinality();
        let other_cardinality = other.estimate_cardinality();
        let union_cardinality = self.estimate_union_cardinality(other);
        if self_cardinality + other_cardinality < union_cardinality {
            0.0
        } else {
            self_cardinality + other_cardinality - union_cardinality
        }
    }

    #[inline]
    /// Estimates the Jaccard index, derived as `intersection / union` (zero when the union is empty
    /// or the inclusion-exclusion intersection would be negative).
    fn estimate_jaccard_index(&self, other: &Self) -> f64 {
        let self_cardinality = self.estimate_cardinality();
        let other_cardinality = other.estimate_cardinality();
        let union_cardinality = self.estimate_union_cardinality(other);
        if self_cardinality + other_cardinality < union_cardinality || union_cardinality == 0.0 {
            0.0
        } else {
            (self_cardinality + other_cardinality - union_cardinality) / union_cardinality
        }
    }

    #[inline]
    /// Estimates the cardinality of this set minus `other`, derived as `max(0, |A union B| - |B|)`.
    fn estimate_difference_cardinality(&self, other: &Self) -> f64 {
        let union_cardinality = self.estimate_union_cardinality(other);
        let other_cardinality = other.estimate_cardinality();
        if union_cardinality < other_cardinality {
            0.0
        } else {
            union_cardinality - other_cardinality
        }
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> CardinalityEstimator
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
