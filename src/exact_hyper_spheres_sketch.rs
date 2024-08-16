//! Submodule providing the implementation of `HyperSphere` sketches for `HashSets`.
use crate::prelude::*;
use core::hash::Hash;
use std::collections::HashSet;

impl<I, S: Default + Send + Sync + ::std::hash::BuildHasher> Named for HashSet<I, S>
where
    I: Eq + Hash,
{
    fn name(&self) -> String {
        "HashSet".to_string()
    }
}

macro_rules! impl_estimator_for_hashset {
    ($($typ:ty)*,) => {
        $(
            impl<I, S: Default + Send + Sync + ::std::hash::BuildHasher> Estimator<$typ> for HashSet<I, S>
                where
                    I: Eq + Hash + Send + Sync,
                {
                    #[allow(clippy::cast_precision_loss)]
                    fn estimate_union_cardinality(&self, other: &Self) -> $typ {
                        self.union(other).count() as $typ
                    }

                    #[allow(clippy::cast_precision_loss)]
                    fn estimate_cardinality(&self) -> $typ {
                        self.len() as $typ
                    }

                    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
                        false
                    }
                }
        )*
    }
}

impl_estimator_for_hashset! {
    usize f32 f64,
}

impl<T, S: Default + Send + Sync + ::std::hash::BuildHasher> SetProperties for HashSet<T, S> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn is_full(&self) -> bool {
        false
    }
}

impl<T, S: Default + Send + Sync + ::std::hash::BuildHasher> MutableSet for HashSet<T, S> {
    fn clear(&mut self) {
        self.clear();
    }
}

impl<T, S: Default + Send + Sync + ::std::hash::BuildHasher> ApproximatedSet<T> for HashSet<T, S>
where
    T: Hash + Eq,
{
    fn may_contain(&self, element: &T) -> bool {
        self.contains(element)
    }
}

impl<T, S: Default + Send + Sync + ::std::hash::BuildHasher> ExtendableApproximatedSet<T> for HashSet<T, S>
where
    T: Hash + Eq + Clone,
{
    fn insert(&mut self, element: &T) -> bool {
        self.insert(element.clone())
    }
}
