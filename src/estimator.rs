//! This module defines the traits for the cardinality estimators.
use core::hash::Hash;

use crate::prelude::*;

/// Trait for properties of a set.
pub trait SetProperties {
    /// Returns whether the set is empty.
    fn is_empty(&self) -> bool;

    /// Returns whether the set is full.
    fn is_full(&self) -> bool;
}

/// Trait for an approximated set.
pub trait ApproximatedSet<T: Hash>: SetProperties {
    /// Returns whether the set contains the element.
    fn may_contain(&self, element: &T) -> bool;
}

/// Trait for a mutable set.
pub trait MutableSet: SetProperties {
    /// Empties the set.
    fn clear(&mut self);
}

/// Trait for an extendable approximated set.
pub trait ExtendableApproximatedSet<T: Hash> {
    /// Insert an element into the set and return whether the element has been inserted.
    fn insert(&mut self, element: &T) -> bool;

    #[inline]
    /// Extend the [`HyperLogLog`] counter with the elements from an iterator.
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.insert(&value);
        }
    }
}

/// Trait for a cardinality estimator.
pub trait Estimator<F: Number>: Sized + Default + Send + Sync {
    /// Estimates the cardinality.
    fn estimate_cardinality(&self) -> F;

    /// Returns an estimate of two [`HyperLogLog`] counters union cardinality.
    fn estimate_union_cardinality(&self, other: &Self) -> F;

    /// Returns whether the union estimate is currently non-deterministic.
    fn is_union_estimate_non_deterministic(&self, other: &Self) -> bool;

    #[inline]
    /// Returns an estimate of the intersection cardinality between two counters.
    fn estimate_intersection_cardinality(&self, other: &Self) -> F {
        let self_cardinality = self.estimate_cardinality();
        let other_cardinality = other.estimate_cardinality();
        let union_cardinality = self.estimate_union_cardinality(other);

        // We apply correction to the union cardinality to get the intersection cardinality.
        if self_cardinality + other_cardinality < union_cardinality {
            F::ZERO
        } else {
            self_cardinality + other_cardinality - union_cardinality
        }
    }

    #[inline]
    /// Returns an estimate of the Jaccard index between two counters.
    fn estimate_jaccard_index(&self, other: &Self) -> F {
        let self_cardinality = self.estimate_cardinality();
        let other_cardinality = other.estimate_cardinality();
        let union_cardinality = self.estimate_union_cardinality(other);

        // We apply correction to the union cardinality to get the intersection cardinality.
        if self_cardinality + other_cardinality < union_cardinality || union_cardinality.is_zero() {
            F::ZERO
        } else {
            (self_cardinality + other_cardinality - union_cardinality) / union_cardinality
        }
    }

    #[inline]
    /// Returns an estimate of the cardinality of the current counter minus the cardinality of the other counter.
    fn estimate_difference_cardinality(&self, other: &Self) -> F {
        let union_cardinality = self.estimate_union_cardinality(other);
        let other_cardinality = other.estimate_cardinality();
        if union_cardinality < other_cardinality {
            F::ZERO
        } else {
            union_cardinality - other_cardinality
        }
    }
}
