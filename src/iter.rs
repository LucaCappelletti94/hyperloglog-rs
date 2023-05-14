//! This module defines a trait and an implementation for estimating the cardinality of an iterator
//! using a HyperLogLog data structure.
//!
//! # Example
//! You can estimate the cardinality of an iterator using the `estimate_cardinality` method.
//! ```
//! use hyperloglog_rs::prelude::*;
//!
//! let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
//! let cardinality_estimate = v.iter().estimate_cardinality::<12, 5>();
//! assert!((cardinality_estimate - 10.0).abs() < 1.0);
//! ```
//!
//! You can merge multiple HyperLogLog counters from iterators using the `union` method.
//!
//! ```
//! use hyperloglog_rs::prelude::*;
//!
//! let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
//! let hll: HyperLogLog<12, 6> = v.into_iter().map(|index|{
//!     HyperLogLog::from(index)
//! }).union();
//! let cardinality_estimate = hll.estimate_cardinality();
//! assert!((cardinality_estimate - 10.0).abs() < 1.0);
//! ```
use core::hash::Hash;

use crate::prelude::*;

/// A trait for estimating the cardinality of an iterator.
pub trait EstimateIterCardinality {
    /// Estimate the cardinality of the iterator.
    ///
    /// # Arguments
    ///
    /// * `self` - An iterator over elements to estimate the cardinality of.
    ///
    /// # Type parameters
    ///
    /// * `PRECISION` - The precision to use for the HyperLogLog counter.
    /// * `BITS` - The number of bits per register in the HyperLogLog counter.
    ///
    fn estimate_cardinality<const PRECISION: usize, const BITS: usize>(self) -> f32
    where
        [(); ceil(1 << PRECISION, 32 / BITS)]:;
}

impl<I, T: Hash> EstimateIterCardinality for I
where
    I: Iterator<Item = T>,
{
    fn estimate_cardinality<const PRECISION: usize, const BITS: usize>(self) -> f32
    where
        [(); ceil(1 << PRECISION, 32 / BITS)]:,
    {
        let hll: HyperLogLog<PRECISION, BITS> = self.collect();
        hll.estimate_cardinality()
    }
}

pub trait HyperLogLogIterator<const PRECISION: usize, const BITS: usize> {
    /// Returns a HyperLogLog that is the union of all HyperLogLogs in the iterator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1 = HyperLogLog::<12, 6>::new();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    ///
    /// let mut hll2 = HyperLogLog::<12, 6>::new();
    /// hll2.insert(&3);
    /// hll2.insert(&4);
    ///
    /// let mut hll3 = HyperLogLog::<12, 6>::new();
    /// hll3.insert(&5);
    /// hll3.insert(&6);
    ///
    /// let hll_union = vec![hll1, hll2, hll3].into_iter().union();
    ///
    /// assert!(hll_union.estimate_cardinality() - 6.0 < 1.0, "Expected 6.0, got {}", hll_union.estimate_cardinality());
    /// ```
    fn union(self) -> HyperLogLog<PRECISION, BITS>
    where
        [(); ceil(1 << PRECISION, 32 / BITS)]:;
}

impl<const PRECISION: usize, const BITS: usize, I> HyperLogLogIterator<PRECISION, BITS> for I
where
    I: Iterator<Item = HyperLogLog<PRECISION, BITS>>,
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    fn union(self) -> HyperLogLog<PRECISION, BITS>
    where
        [(); ceil(1 << PRECISION, 32 / BITS)]:,
    {
        self.fold(HyperLogLog::<PRECISION, BITS>::default(), |acc, hll| {
            acc | hll
        })
    }
}