//! Submodule providing the implementation of HyperSphere sketches for HashSets.
use crate::{prelude::*, utils::Number};
use core::hash::Hash;
use std::collections::HashSet;

impl<I, C> Estimator<C> for HashSet<I>
where
    C: Number + TryFrom<usize>,
    I: Eq + Hash + Send + Sync,
    <C as TryFrom<usize>>::Error: core::fmt::Debug,
{
    fn estimate_union_cardinality(&self, other: &Self) -> C {
        self.union(other).count().try_into().unwrap()
    }

    fn estimate_cardinality(&self) -> C {
        self.len().try_into().unwrap()
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
    }
}

impl<T> SetProperties for HashSet<T> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn is_full(&self) -> bool {
        false
    }
}

impl<T> MutableSet for HashSet<T> {
    fn clear(&mut self) {
        self.clear()
    }
}

impl<T> ApproximatedSet<T> for HashSet<T>
where
    T: Hash + Eq,
{
    fn may_contain(&self, element: &T) -> bool {
        self.contains(element)
    }
}

impl<T> ExtendableApproximatedSet<T> for HashSet<T>
where
    T: Hash + Eq + Clone,
{
    fn insert(&mut self, element: &T) -> bool {
        self.insert(element.clone())
    }
}