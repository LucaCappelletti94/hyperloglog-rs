//! Trait definition for a set.

use hyperloglog_rs::prelude::*;
use mem_dbg::MemSize;
use std::collections::HashSet;
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

    #[inline]
    fn precision(&self) -> Option<u8> {
        None
    }

    #[inline]
    fn bits(&self) -> Option<u8> {
        None
    }
}

impl Set for HashSet<u64> {
    #[inline]
    fn insert_element(&mut self, value: u64) {
        self.insert(value);
    }

    #[inline]
    fn cardinality(&self) -> f64 {
        self.len() as f64
    }

    #[inline]
    fn union(&self, other: &Self) -> f64 {
        self.union(other).count() as f64
    }

    #[inline]
    fn model_name(&self) -> String {
        "HashSet".to_string()
    }
}

impl<H: HasherType, P: Precision, B: Bits, R: Registers<P, B>> Set for HyperLogLog<P, B, R, H> {
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

    #[inline]
    fn precision(&self) -> Option<u8> {
        Some(P::EXPONENT)
    }

    #[inline]
    fn bits(&self) -> Option<u8> {
        Some(B::NUMBER_OF_BITS)
    }
}

#[derive(Clone, MemSize)]
/// Wrapper for the Uncorrected implementation
pub struct UncorrectedFullyImprinted<P: Precision + PackedRegister<B>, B: Bits> {
    hll: HyperLogLog<P, B, <P as PackedRegister<B>>::Array, WyHash>,
}

impl<P: Precision + PackedRegister<B>, B: Bits> Default for UncorrectedFullyImprinted<P, B> {
    #[inline]
    fn default() -> Self {
        Self {
            hll: HyperLogLog::default(),
        }
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

    #[inline]
    fn precision(&self) -> Option<u8> {
        Some(P::EXPONENT)
    }

    #[inline]
    fn bits(&self) -> Option<u8> {
        Some(B::NUMBER_OF_BITS)
    }
}

#[derive(Clone, MemSize, Default)]
/// Wrapper for the Uncorrected implementation
pub struct Uncorrected<P: Precision + PackedRegister<B>, B: Bits> {
    hll: HyperLogLog<P, B, <P as PackedRegister<B>>::Array, WyHash>,
}

impl<P: Precision + PackedRegister<B>, B: Bits> Uncorrected<P, B> {
    #[inline]
    pub fn is_hash_list(&self) -> bool {
        self.hll.is_hash_list()
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.hll.is_full()
    }
}

impl<P: Precision + PackedRegister<B>, B: Bits> Set for Uncorrected<P, B> {
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
        format!("Uncorrected<P{}, B{}>", P::EXPONENT, B::NUMBER_OF_BITS)
    }

    #[inline]
    fn union(&self, _other: &Self) -> f64 {
        unimplemented!()
    }

    #[inline]
    fn precision(&self) -> Option<u8> {
        Some(P::EXPONENT)
    }

    #[inline]
    fn bits(&self) -> Option<u8> {
        Some(B::NUMBER_OF_BITS)
    }
}

#[derive(Clone, MemSize, Default)]
/// Wrapper using exclusively linear counting
/// for the estimation of the cardinality.
pub struct LinearCountingHashList<P: Precision + PackedRegister<B>, B: Bits> {
    hll: HyperLogLog<P, B, <P as PackedRegister<B>>::Array, WyHash>,
}

impl<P: Precision + PackedRegister<B>, B: Bits> Set for LinearCountingHashList<P, B> {
    #[inline]
    fn cardinality(&self) -> f64 {
        if self.hll.is_hash_list() {
            let hash_list_cardinality = self.hll.uncorrected_estimate_cardinality();
            // The bits part of the hash, being geometric, only contributes two bits
            // to the hash list bits entropy.
            let hash_list_bits = self.hll.get_hash_bits().unwrap() + 2 - B::NUMBER_OF_BITS as u8;
            f64::from(1 << hash_list_bits)
                * f64::ln(
                    f64::from(1 << hash_list_bits)
                        / (f64::from(1 << hash_list_bits) - hash_list_cardinality),
                )
        } else {
            self.hll.uncorrected_estimate_cardinality()
        }
    }

    #[inline]
    fn insert_element(&mut self, value: u64) {
        self.hll.insert(&value);
    }

    #[inline]
    fn model_name(&self) -> String {
        format!("LinearCounting<P{}, B{}>", P::EXPONENT, B::NUMBER_OF_BITS)
    }

    #[inline]
    fn union(&self, _other: &Self) -> f64 {
        unimplemented!()
    }

    #[inline]
    fn precision(&self) -> Option<u8> {
        Some(P::EXPONENT)
    }

    #[inline]
    fn bits(&self) -> Option<u8> {
        Some(B::NUMBER_OF_BITS)
    }
}
