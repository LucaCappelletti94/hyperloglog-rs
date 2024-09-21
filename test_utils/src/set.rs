//! Trait definition for a set.

use std::collections::HashSet;
use hyperloglog_rs::prelude::*;
use mem_dbg::MemSize;
use wyhash::WyHash;

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

#[derive(Clone, MemSize)]
/// Wrapper for the Uncorrected implementation
pub struct UncorrectedFullyImprinted<P: Precision + PackedRegister<B>, B: Bits> {
    hll: HyperLogLog<P, B, <P as PackedRegister<B>>::Array, WyHash>,
}

impl<P: Precision + PackedRegister<B>, B: Bits> Default for UncorrectedFullyImprinted<P, B> {
    fn default() -> Self {
        Self {
            hll: HyperLogLog::default(),
        }
    }
}

impl<P: Precision + PackedRegister<B>, B: Bits> UncorrectedFullyImprinted<P, B> {
    pub fn is_hash_list(&self) -> bool {
        self.hll.is_hash_list()
    }
}

impl<P: Precision + PackedRegister<B>, B: Bits> Set for UncorrectedFullyImprinted<P, B> {
    #[inline]
    fn cardinality(&self) -> f64 {
        self.hll.uncorrected_estimate_cardinality()
    }

    #[inline]
    fn insert_element(&mut self, value: u64) {
        self.hll.insert(&value);
    }

    #[inline]
    fn model_name(&self) -> String {
        format!(
            "UncorrectedFullyImprinted<P{}, B{}>",
            P::EXPONENT,
            B::NUMBER_OF_BITS
        )
    }

    #[inline]
    fn union(&self, _other: &Self) -> f64 {
        unimplemented!()
    }
}


#[derive(Clone, MemSize)]
/// Wrapper for the Uncorrected implementation
pub struct UncorrectedImprintedUp<P: Precision + PackedRegister<B>, B: Bits> {
    hll: HyperLogLog<P, B, <P as PackedRegister<B>>::Array, WyHash>,
}

impl<P: Precision + PackedRegister<B>, B: Bits> Default for UncorrectedImprintedUp<P, B> {
    fn default() -> Self {
        Self {
            hll: HyperLogLog::default(),
        }
    }
}

impl<P: Precision + PackedRegister<B>, B: Bits> Set for UncorrectedImprintedUp<P, B> {
    #[inline]
    fn cardinality(&self) -> f64 {
        self.hll.uncorrected_estimate_cardinality()
    }

    #[inline]
    fn insert_element(&mut self, value: u64) {
        if self.hll.is_hash_list() {
            let mut copy = self.hll.clone();
            self.hll.insert(&value);
            if !self.hll.is_hash_list() {
                copy.convert_hash_list_to_hyperloglog(true, false).unwrap();
                self.hll = copy;
                self.hll.insert(&value);
            }
        } else {
            self.hll.insert(&value);
        }
    }

    #[inline]
    fn model_name(&self) -> String {
        format!(
            "UncorrectedImprintedUp<P{}, B{}>",
            P::EXPONENT,
            B::NUMBER_OF_BITS
        )
    }

    #[inline]
    fn union(&self, _other: &Self) -> f64 {
        unimplemented!()
    }
}

#[derive(Clone, MemSize)]
/// Wrapper for the Uncorrected implementation
pub struct UncorrectedImprintedDown<P: Precision + PackedRegister<B>, B: Bits> {
    hll: HyperLogLog<P, B, <P as PackedRegister<B>>::Array, WyHash>,
}

impl<P: Precision + PackedRegister<B>, B: Bits> Default for UncorrectedImprintedDown<P, B> {
    fn default() -> Self {
        Self {
            hll: HyperLogLog::default(),
        }
    }
}

impl<P: Precision + PackedRegister<B>, B: Bits> Set for UncorrectedImprintedDown<P, B> {
    #[inline]
    fn cardinality(&self) -> f64 {
        self.hll.uncorrected_estimate_cardinality()
    }

    #[inline]
    fn insert_element(&mut self, value: u64) {
        if self.hll.is_hash_list() {
            let mut copy = self.hll.clone();
            self.hll.insert(&value);
            if !self.hll.is_hash_list() {
                copy.convert_hash_list_to_hyperloglog(false, true).unwrap();
                self.hll = copy;
                self.hll.insert(&value);
            }
        } else {
            self.hll.insert(&value);
        }
    }

    #[inline]
    fn model_name(&self) -> String {
        format!(
            "UncorrectedImprintedDown<P{}, B{}>",
            P::EXPONENT,
            B::NUMBER_OF_BITS
        )
    }

    #[inline]
    fn union(&self, _other: &Self) -> f64 {
        unimplemented!()
    }
}

#[derive(Clone, MemSize)]
/// Wrapper for the Uncorrected implementation
pub struct UncorrectedNotImprinted<P: Precision + PackedRegister<B>, B: Bits> {
    hll: HyperLogLog<P, B, <P as PackedRegister<B>>::Array, WyHash>,
}

impl<P: Precision + PackedRegister<B>, B: Bits> UncorrectedNotImprinted<P, B> {
    pub fn is_hash_list(&self) -> bool {
        self.hll.is_hash_list()
    }

    pub fn is_full(&self) -> bool {
        self.hll.is_full()
    }
}

impl<P: Precision + PackedRegister<B>, B: Bits> Default for UncorrectedNotImprinted<P, B> {
    fn default() -> Self {
        Self {
            hll: HyperLogLog::default(),
        }
    }
}

impl<P: Precision + PackedRegister<B>, B: Bits> Set for UncorrectedNotImprinted<P, B> {
    #[inline]
    fn cardinality(&self) -> f64 {
        self.hll.uncorrected_estimate_cardinality()
    }

    #[inline]
    fn insert_element(&mut self, value: u64) {
        if self.hll.is_hash_list() {
            let mut copy = self.hll.clone();
            self.hll.insert(&value);
            if !self.hll.is_hash_list() {
                copy.convert_hash_list_to_hyperloglog(false, false).unwrap();
                self.hll = copy;
                self.hll.insert(&value);
            }
        } else {
            self.hll.insert(&value);
        }
    }

    #[inline]
    fn model_name(&self) -> String {
        format!(
            "UncorrectedNotImprinted<P{}, B{}>",
            P::EXPONENT,
            B::NUMBER_OF_BITS
        )
    }

    #[inline]
    fn union(&self, _other: &Self) -> f64 {
        unimplemented!()
    }
}