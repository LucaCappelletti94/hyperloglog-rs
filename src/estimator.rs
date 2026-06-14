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
}
