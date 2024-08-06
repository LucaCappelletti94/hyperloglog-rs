//! Submodule providing the implementation of HyperSphere sketches for HashSets.
use crate::{prelude::*, utils::Number};
use core::hash::Hash;
use std::collections::HashSet;

impl<I, C> SetLike<C> for HashSet<I>
where
    C: Number + TryFrom<usize>,
    I: Eq + Hash,
    <C as TryFrom<usize>>::Error: core::fmt::Debug,
{
    fn get_estimated_union_cardinality(
        &self,
        self_cardinality: C,
        other: &Self,
        other_cardinality: C,
    ) -> EstimatedUnionCardinalities<C> {
        let union_cardinality = self.union(other).count().try_into().unwrap();

        EstimatedUnionCardinalities::from((self_cardinality, other_cardinality, union_cardinality))
    }

    fn get_cardinality(&self) -> C {
        self.len().try_into().unwrap()
    }
}

impl<C: Eq + Hash> HyperSpheresSketch for HashSet<C> {}
