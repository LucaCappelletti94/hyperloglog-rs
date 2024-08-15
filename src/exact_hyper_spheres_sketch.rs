//! Submodule providing the implementation of HyperSphere sketches for HashSets.
use crate::prelude::*;
use core::hash::Hash;
use std::collections::HashSet;

impl<I> Named for HashSet<I>
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
            impl<I> Estimator<$typ> for HashSet<I>
                where
                    I: Eq + Hash + Send + Sync,
                {
                    fn estimate_union_cardinality(&self, other: &Self) -> $typ {
                        self.union(other).count() as $typ
                    }

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
    u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64,
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
