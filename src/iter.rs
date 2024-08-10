//! This module defines a trait and an implementation for estimating the cardinality of an iterator
//! using a HyperLogLog data structure.
//!
//! # Example
//! You can estimate the cardinality of an iterator using the `estimate_cardinality` method.
//! ```
//! use hyperloglog_rs::prelude::*;
//!
//! let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
//! let cardinality_estimate = v.iter().estimate_cardinality::<f32, Precision12, Bits5>();
//! assert!((cardinality_estimate - 10.0).abs() < 1.0);
//! ```
//!
//! You can merge multiple HyperLogLog counters from iterators using the `union` method.
//!
//! ```
//! use hyperloglog_rs::prelude::*;
//!
//! let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
//! let hll: HyperLogLog<Precision12, Bits6, <Precision12 as ArrayRegister<Bits6>>::ArrayRegister> =
//!     v.into_iter()
//!         .map(|index| HyperLogLog::from_iter([index]))
//!         .union();
//! let cardinality_estimate: f32 = hll.estimate_cardinality();
//! assert!((cardinality_estimate - 10.0).abs() < 1.0);
//! ```
use core::hash::Hash;

use crate::{prelude::*, utils::FloatNumber};

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
    fn estimate_cardinality<
        F: FloatNumber,
        P: Precision + PrecisionConstants<F> + ArrayRegister<B>,
        B: Bits,
        Hasher: HasherType,
    >(
        self,
    ) -> F;
}

impl<I, T: Hash> EstimateIterCardinality for I
where
    I: Iterator<Item = T>,
{
    fn estimate_cardinality<
        F: FloatNumber,
        P: Precision + PrecisionConstants<F> + ArrayRegister<B>,
        B: Bits,
        Hasher: HasherType,
    >(
        self,
    ) -> F {
        let hll: HyperLogLog<
            P,
            B,
            <P as ArrayRegister<B>>::ArrayRegister,
            Hasher
        > = self.collect();
        hll.estimate_cardinality()
    }
}

pub trait HyperLogLogIterator<P: Precision, B: Bits, Hasher: HasherType>: Sized + IntoIterator
where
    <Self as IntoIterator>::Item: HyperLogLogTrait<P, B, Hasher>,
{
    /// Returns a HyperLogLog that is the union of all HyperLogLogs in the iterator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    ///
    /// let mut hll2 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll2.insert(&3);
    /// hll2.insert(&4);
    ///
    /// let mut hll3 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll3.insert(&5);
    /// hll3.insert(&6);
    ///
    /// let hll_union = vec![hll1, hll2, hll3].into_iter().union();
    ///
    /// assert!(
    ///     hll_union.estimate_cardinality::<f32>() - 6.0 < 1.0,
    ///     "Expected 6.0, got {}",
    ///     hll_union.estimate_cardinality::<f32>()
    /// );
    /// ```
    fn union(self) -> Self::Item {
        let mut hll = Self::Item::default();
        for h in self {
            hll |= h;
        }
        hll
    }
}

impl<P: Precision, B: Bits, I, Hasher: HasherType> HyperLogLogIterator<P, B, Hasher> for I
where
    I: IntoIterator,
    I::Item: HyperLogLogTrait<P, B, Hasher>,
{
}
