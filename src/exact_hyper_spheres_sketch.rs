//! Submodule providing the implementation of HyperSphere sketches for HashSets.
use crate::prelude::*;
use core::hash::Hash;
use std::collections::HashSet;

impl<I, C> SetLike<C> for HashSet<I>
where
    C: Primitive<f32>,
    I: Eq + Hash,
{
    fn get_estimated_union_cardinality(
        &self,
        self_cardinality: C,
        other: &Self,
        other_cardinality: C,
    ) -> EstimatedUnionCardinalities<C> {
        let union_cardinality = C::reverse(self.union(other).count() as f32);

        EstimatedUnionCardinalities::from((self_cardinality, other_cardinality, union_cardinality))
    }

    fn get_cardinality(&self) -> C {
        C::reverse(self.len() as f32)
    }
}

impl<C: Eq + Hash, I: Primitive<f32> + One> HyperSpheresSketch<I> for HashSet<C> {}
