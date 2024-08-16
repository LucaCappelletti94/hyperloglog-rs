//! Submodule providing the implementation of `HyperSphere` sketches for `HashSets`.
use crate::prelude::*;
use core::hash::Hash;
use std::collections::HashSet;
use core::hash::BuildHasher;

impl<I, S: Default + Send + Sync + BuildHasher> Named for HashSet<I, S>
where
    I: Eq + Hash,
{
    #[inline]
    fn name(&self) -> String {
        "HashSet".to_owned()
    }
}

/// Implementation of the `Estimator` trait for `HashSet`.
impl<I, S: Default + Send + Sync + BuildHasher> Estimator<f64> for HashSet<I, S>
    where
        I: Eq + Hash + Send + Sync,
    {
        #[inline]
        #[expect(clippy::cast_precision_loss, reason = "This is an adapter trait for tests.")]
        #[expect(clippy::as_conversions, reason = "There are no better options.")]
        fn estimate_union_cardinality(&self, other: &Self) -> f64 {
            self.union(other).count() as f64
        }

        #[inline]
        #[expect(clippy::cast_precision_loss, reason = "This is an adapter trait for tests.")]
        #[expect(clippy::as_conversions, reason = "There are no better options.")]
        fn estimate_cardinality(&self) -> f64 {
            self.len() as f64
        }

        #[inline]
        fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
            false
        }
    }

impl<I, S: Default + Send + Sync + BuildHasher> Estimator<usize> for HashSet<I, S>
        where
    I: Eq + Hash + Send + Sync,
{
    #[inline]
    fn estimate_union_cardinality(&self, other: &Self) -> usize {
        self.union(other).count()
    }

    #[inline]
    fn estimate_cardinality(&self) -> usize {
        self.len()
    }

    #[inline]
    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
    }
}

impl<T, S: Default + Send + Sync + BuildHasher> SetProperties for HashSet<T, S> {
    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    #[inline]
    fn is_full(&self) -> bool {
        false
    }
}

impl<T, S: Default + Send + Sync + BuildHasher> MutableSet for HashSet<T, S> {
    #[inline]
    fn clear(&mut self) {
        self.clear();
    }
}

impl<T, S: Default + Send + Sync + BuildHasher> ApproximatedSet<T> for HashSet<T, S>
where
    T: Hash + Eq,
{
    #[inline]
    fn may_contain(&self, element: &T) -> bool {
        self.contains(element)
    }
}

impl<T, S: Default + Send + Sync + BuildHasher> ExtendableApproximatedSet<T> for HashSet<T, S>
where
    T: Hash + Eq + Clone,
{
    #[inline]
    fn insert(&mut self, element: &T) -> bool {
        self.insert(element.clone())
    }
}
