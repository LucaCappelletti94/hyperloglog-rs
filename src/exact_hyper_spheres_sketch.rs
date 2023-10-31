//! Submodule providing the implementation of HyperSphere sketches for HashSets.
//!
use crate::prelude::*;
#[cfg(feature = "std")]
use std::collections::HashSet;

use core::hash::Hash;

impl<I, C> SetLike<C> for HashSet<I>
where
    I: Hash + Eq + Clone,
    C: Primitive<usize>,
{
    fn get_estimated_union_cardinality(
        &self,
        self_cardinality: C,
        other: &Self,
        other_cardinality: C,
    ) -> EstimatedUnionCardinalities<C> {
        let union_cardinality = C::reverse(self.union(other).count());

        EstimatedUnionCardinalities::from((
            self_cardinality,
            other_cardinality,
            union_cardinality,
        ))
    }

    fn get_cardinality(&self) -> C {
        C::reverse(self.len())
    }
}

impl<I> HyperSpheresSketch for HashSet<I> {}
