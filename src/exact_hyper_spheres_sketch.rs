//! Submodule providing the implementation of HyperSphere sketches for HashSets.
//!
use crate::prelude::*;
#[cfg(feature = "std")]
use std::collections::HashSet;

use core::hash::Hash;

impl<I> SetLike<I> for HashSet<I>
where
    I: Hash + Eq + Clone,
{
    fn get_estimated_union_cardinality(
        &self,
        self_cardinality: I,
        other: &Self,
        other_cardinality: I,
    ) -> EstimatedUnionCardinalities<I> {
        let intersection_cardinality = self.intersection(other).count();
        let union_cardinality = self.union(other).count();
        let left_difference_cardinality = self.difference(other).count();
        let right_difference_cardinality = other.difference(self).count();

        EstimatedUnionCardinalities {
            intersection_cardinality,
            union_cardinality,
            left_difference_cardinality,
            right_difference_cardinality,
        }
    }

    fn get_cardinality(&self) -> I {
        self.len()
    }
}

impl<I> HyperSpheresSketch for HashSet<I> {}
