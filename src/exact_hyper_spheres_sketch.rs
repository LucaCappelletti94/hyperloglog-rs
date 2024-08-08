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
    fn get_union_cardinality(&self, other: &Self) -> C {
        self.union(other).count().try_into().unwrap()
    }

    fn get_cardinality(&self) -> C {
        self.len().try_into().unwrap()
    }
}

impl<C: Eq + Hash> HyperSpheresSketch for HashSet<C> {}
