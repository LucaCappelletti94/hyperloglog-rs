//! Submodule providing the implementation of HyperSphere sketches for HashSets.
//!
use crate::prelude::*;
#[cfg(feature = "std")]
use std::collections::HashSet;

use core::hash::Hash;

impl<I> SetLike<usize> for HashSet<I>
where
    I: Hash + Eq + Clone,
{
    fn get_estimated_union_cardinality(
        &self,
        self_cardinality: usize,
        other: &Self,
        other_cardinality: usize,
    ) -> EstimatedUnionCardinalities<usize> {
        let union_cardinality = self.union(other).count();

        EstimatedUnionCardinalities::from((
            self_cardinality,
            other_cardinality,
            union_cardinality,
        ))
    }

    fn get_cardinality(&self) -> usize {
        self.len()
    }
}

impl<I> HyperSpheresSketch for HashSet<I> {}
