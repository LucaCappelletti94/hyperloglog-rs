//! Trait definition for a set.

use std::collections::HashSet;
use hyperloglog_rs::prelude::*;

/// A trait to represent a set.
pub trait Set {
    /// Inserts an element into the set.
    fn insert_element(&mut self, value: u64);
    /// Returns the cardinality of the set.
    fn cardinality(&self) -> f64;
    /// Returns the union of two sets.
    fn union(&self, other: &Self) -> f64;
    /// Returns the name of the model.
    fn model_name(&self) -> String;
}

impl Set for HashSet<u64> {
    fn insert_element(&mut self, value: u64) {
        self.insert(value);
    }

    fn cardinality(&self) -> f64 {
        self.len() as f64
    }

    fn union(&self, other: &Self) -> f64 {
        self.union(other).count() as f64
    }

    fn model_name(&self) -> String {
        "HashSet".to_string()
    }
}

impl<H: HasherType, P: Precision, B: Bits, R: Registers<P, B>> Set
    for HyperLogLog<P, B, R, H>
{   
    #[inline]
    fn insert_element(&mut self, value: u64) {
        self.insert(&value);
    }

    #[inline]
    fn cardinality(&self) -> f64 {
        self.estimate_cardinality()
    }

    #[inline]
    fn union(&self, other: &Self) -> f64 {
        self.estimate_union_cardinality(other)
    }

    #[inline]
    fn model_name(&self) -> String {
        format!(
            "HLL<P{}, B{}> + {}",
            P::EXPONENT,
            B::NUMBER_OF_BITS,
            core::any::type_name::<H>().split("::").last().unwrap()
        )
    }
}