//! This module contains the `ArrayDefault` trait, which is used to set the default value of an array.
//! This trait is necessary as the standard library only provides a `Default` implementation for arrays
//! of limited length, while we need this for objects of several differenty lengths.
use core::ops::Index;

pub trait ArrayIter<T: Default + PartialEq>: Index<usize, Output = T> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the partition point of the array.
    fn partition_point<P>(&self, predicate: P) -> usize
    where
        P: FnMut(&T) -> bool;
}

impl<T: Default + PartialEq, const N: usize> ArrayIter<T> for [T; N] {
    #[inline(always)]
    fn len(&self) -> usize {
        N
    }

    #[inline(always)]
    fn partition_point<P>(&self, predicate: P) -> usize
    where
        P: FnMut(&T) -> bool,
    {
        <[T]>::partition_point(self, predicate)
    }
}
