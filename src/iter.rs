//! This module defines a trait and an implementation for estimating the cardinality of an iterator
//! using a HyperLogLog data structure.
//!
//! # Example
//!
//! ```
//! use hyperloglog_rs::prelude::*;
//!
//! let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
//! let cardinality_estimate = v.iter().estimate_cardinality::<12, 5>();
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
