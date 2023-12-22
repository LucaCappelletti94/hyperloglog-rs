use crate::precisions::{Precision, WordType};
use crate::HyperLogLog;
use core::hash::Hash;
use core::ops::{BitOr, BitOrAssign};

impl<
        Item: Hash,
        I: IntoIterator<Item = Item>,
        P: Precision + WordType<BITS>,
        const BITS: usize,
    > BitOrAssign<I> for HyperLogLog<P, BITS>
{
    #[inline(always)]
    /// Computes inplace union between an HLL counter and an iterator.
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    /// # use core::ops::BitOrAssign;
    ///
    /// let mut hll = HyperLogLog::<Precision8, 6>::default();
    ///
    /// hll |= [1u8, 2u8];
    ///
    /// assert!(hll.estimate_cardinality() > 2.0 - 0.1, "The cardinality is {}, we were expecting 2.", hll.estimate_cardinality());
    /// assert!(hll.estimate_cardinality() < 2.0 + 0.1, "The cardinality is {}, we were expecting 2.", hll.estimate_cardinality());
    ///
    /// hll |= [2u8, 3u8];
    ///
    /// assert!(hll3.estimate_cardinality() > 3.0 - 0.1, "Expected a value equal to around 3, got {}", hll.estimate_cardinality());
    /// assert!(hll3.estimate_cardinality() < 3.0 + 0.1, "Expected a value equal to around 3, got {}", hll.estimate_cardinality());
    /// ```
    fn bitor_assign(&mut self, rhs: I) {
        rhs.into_iter().for_each(|item| self.insert(item));
    }
}

impl<Item: Hash, I: Iterator<Item = Item>, P: Precision + WordType<BITS>, const BITS: usize>
    BitOr<I> for HyperLogLog<P, BITS>
{
    type Output = Self;

    #[inline(always)]
    /// Computes the union between an HLL counter and an iterator.
    ///
    fn bitor(mut self, rhs: I) -> Self {
        self.bitor_assign(rhs);
        self
    }
}
