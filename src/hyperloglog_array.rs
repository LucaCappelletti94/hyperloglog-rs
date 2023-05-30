use std::{
    hash::Hash,
    ops::{Index, IndexMut},
};

use crate::prelude::*;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HyperLogLogArray<const PRECISION: usize, const BITS: usize, const N: usize>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    counters: [HyperLogLog<PRECISION, BITS>; N],
}

impl<const PRECISION: usize, const BITS: usize, const N: usize> Default
    for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Creates a new HyperLogLogArray with the given precision and number of bits.
    ///
    /// # Returns
    /// A new HyperLogLogArray with the given precision and number of bits.
    ///
    /// # Example
    ///
    /// ```rust
    /// #![feature(generic_const_exprs)]
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll_array = HyperLogLogArray::<12, 6, 3>::default();
    /// ```
    fn default() -> Self {
        Self {
            counters: [HyperLogLog::new(); N],
        }
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize> HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Creates a new HyperLogLogArray with the given precision and number of bits.
    ///
    /// # Example
    ///
    /// ```
    /// #![feature(generic_const_exprs)]
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll_array = HyperLogLogArray::<12, 6, 3>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            counters: [HyperLogLog::new(); N],
        }
    }

    #[inline(always)]
    /// Returns the estimated overlap cardinality matrices with the provided HyperLogLogArray.
    ///
    /// # Arguments
    /// * `other`: The HyperLogLogArray to estimate the overlap cardinality matrices with.
    ///
    /// # Returns
    /// The estimated overlap cardinality matrices with the provided HyperLogLogArray.
    ///
    pub fn estimate_overlap_cardinalities(&self, other: &Self) -> [[f32; N]; N] {
        HyperLogLog::estimated_overlap_cardinality_matrix(self.as_ref(), other.as_ref())
    }

    #[inline(always)]
    /// Returns the estimated difference cardinality matrices with the provided HyperLogLog.
    ///
    /// # Arguments
    /// * `other`: The HyperLogLog to estimate the difference cardinality matrices with.
    ///
    /// # Returns
    /// The estimated difference cardinality matrices with the provided HyperLogLogArray.
    ///
    pub fn estimated_difference_cardinality_vector(
        &self,
        other: &HyperLogLog<PRECISION, BITS>,
    ) -> [f32; N] {
        HyperLogLog::estimated_difference_cardinality_vector(self.as_ref(), other)
    }

    #[inline(always)]
    /// Returns the estimated overlap and difference cardinality matrices and vectors with the provided HyperLogLogArray.
    ///
    /// # Arguments
    /// * `other`: The HyperLogLogArray to estimate the overlap and difference cardinality matrices and vectors with.
    ///
    /// # Returns
    /// The estimated overlap and difference cardinality matrices and vectors with the provided HyperLogLogArray.
    pub fn estimated_overlap_and_differences_cardinality_matrices(
        &self,
        other: &Self,
    ) -> ([[f32; N]; N], [f32; N], [f32; N]) {
        HyperLogLog::estimated_overlap_and_differences_cardinality_matrices(
            self.as_ref(),
            other.as_ref(),
        )
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize>
    AsRef<[HyperLogLog<PRECISION, BITS>; N]> for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Returns a reference to the underlying array of HyperLogLog counters.
    ///
    /// # Returns
    /// A reference to the underlying array of HyperLogLog counters.
    fn as_ref(&self) -> &[HyperLogLog<PRECISION, BITS>; N] {
        &self.counters
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize>
    AsMut<[HyperLogLog<PRECISION, BITS>; N]> for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Returns a mutable reference to the underlying array of HyperLogLog counters.
    ///
    /// # Returns
    /// A mutable reference to the underlying array of HyperLogLog counters.
    fn as_mut(&mut self) -> &mut [HyperLogLog<PRECISION, BITS>; N] {
        &mut self.counters
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize> Index<usize>
    for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    type Output = HyperLogLog<PRECISION, BITS>;

    #[inline(always)]
    /// Returns a reference to the HyperLogLog counter at the given index.
    ///
    /// # Arguments
    /// * `index`: The index of the HyperLogLog counter to return.
    ///
    /// # Returns
    /// A reference to the HyperLogLog counter at the given index.
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// #![feature(generic_const_exprs)]
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll_array = HyperLogLogArray::<12, 6, 4>::new();
    /// hll_array[0].insert(&1);
    /// hll_array[1].insert(&2);
    /// hll_array[2].insert(&3);
    ///
    /// assert!(hll_array[0].estimate_cardinality() > 0.9
    ///     && hll_array[1].estimate_cardinality() < 1.1
    /// );
    /// assert!(hll_array[1].estimate_cardinality() > 0.9
    ///    && hll_array[1].estimate_cardinality() < 1.1
    /// );
    /// assert!(hll_array[2].estimate_cardinality() > 0.9
    ///   && hll_array[2].estimate_cardinality() < 1.1
    /// );
    /// assert!(hll_array[3].estimate_cardinality() > -0.1
    ///  && hll_array[3].estimate_cardinality() < 0.1
    /// );
    ///
    /// ```
    fn index(&self, index: usize) -> &Self::Output {
        &self.counters[index]
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize> IndexMut<usize>
    for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Returns a mutable reference to the HyperLogLog counter at the given index.
    ///
    /// # Arguments
    /// * `index`: The index of the HyperLogLog counter to return.
    ///
    /// # Returns
    /// A mutable reference to the HyperLogLog counter at the given index.
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// #![feature(generic_const_exprs)]
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll_array = HyperLogLogArray::<12, 6, 4>::new();
    /// hll_array[0].insert(&1);
    /// hll_array[1].insert(&2);
    /// hll_array[2].insert(&3);
    ///
    /// assert!(hll_array[0].estimate_cardinality() > 0.9
    ///    && hll_array[1].estimate_cardinality() < 1.1
    /// );
    /// assert!(hll_array[1].estimate_cardinality() > 0.9
    ///  && hll_array[1].estimate_cardinality() < 1.1
    /// );
    /// assert!(hll_array[2].estimate_cardinality() > 0.9
    /// && hll_array[2].estimate_cardinality() < 1.1
    /// );
    /// assert!(hll_array[3].estimate_cardinality() > -0.1
    /// && hll_array[3].estimate_cardinality() < 0.1
    /// );
    ///
    /// ```
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.counters[index]
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize>
    From<[HyperLogLog<PRECISION, BITS>; N]> for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Creates a new HyperLogLogArray from the given array of HyperLogLog counters.
    ///
    /// # Arguments
    /// * `counters`: The array of HyperLogLog counters to create the HyperLogLogArray from.
    ///
    /// # Returns
    /// A new HyperLogLogArray from the given array of HyperLogLog counters.
    ///
    /// # Example
    ///
    /// ```rust
    /// #![feature(generic_const_exprs)]
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll_array = HyperLogLogArray::<12, 6, 3>::from([
    ///     HyperLogLog::new(),
    ///     HyperLogLog::new(),
    ///     HyperLogLog::new(),
    /// ]);
    /// ```
    fn from(counters: [HyperLogLog<PRECISION, BITS>; N]) -> Self {
        Self { counters }
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize, H: Hash>
    From<&[Vec<H>; N]> for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Creates a new HyperLogLogArray from the given array of vectors of hashable items.
    ///
    /// # Arguments
    /// * `items`: The array of vectors of hashable items to create the HyperLogLogArray from.
    ///
    /// # Returns
    /// A new HyperLogLogArray from the given array of vectors of hashable items.
    ///
    /// # Example
    ///
    /// ```rust
    /// #![feature(generic_const_exprs)]
    /// use std::hash::Hash;
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll_array = HyperLogLogArray::<12, 6, 3>::from(&[
    ///     vec![1, 2, 3],
    ///     vec![4, 5, 6],
    ///     vec![7, 8, 9],
    /// ]);
    /// ```
    fn from(items: &[Vec<H>; N]) -> Self {
        let mut array = [HyperLogLog::new(); N];
        for (i, item) in items.iter().enumerate() {
            for item in item.iter() {
                array[i].insert(item);
            }
        }
        Self { counters: array }
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize>
    From<Vec<HyperLogLog<PRECISION, BITS>>> for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Creates a new HyperLogLogArray from the given vector of HyperLogLog counters.
    ///
    /// # Arguments
    /// * `counters`: The vector of HyperLogLog counters to create the HyperLogLogArray from.
    ///
    /// # Returns
    /// A new HyperLogLogArray from the given vector of HyperLogLog counters.
    ///
    /// # Example
    ///
    /// ```rust
    /// #![feature(generic_const_exprs)]
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll_array = HyperLogLogArray::<12, 6, 3>::from(vec![
    ///     HyperLogLog::new(),
    ///     HyperLogLog::new(),
    ///     HyperLogLog::new(),
    /// ]);
    /// ```
    fn from(counters: Vec<HyperLogLog<PRECISION, BITS>>) -> Self {
        assert_eq!(counters.len(), N, concat!(
            "The length of the vector of HyperLogLog counters must be equal to the number of counters ",
            "in the HyperLogLogArray."
        ));
        let mut array = [HyperLogLog::new(); N];
        array.copy_from_slice(&counters[..N]);
        Self { counters: array }
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize>
    From<&[HyperLogLog<PRECISION, BITS>]> for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Creates a new HyperLogLogArray from the given slice of HyperLogLog counters.
    ///
    /// # Arguments
    /// * `counters`: The slice of HyperLogLog counters to create the HyperLogLogArray from.
    ///
    /// # Returns
    /// A new HyperLogLogArray from the given slice of HyperLogLog counters.
    ///
    /// # Example
    ///
    /// ```rust
    /// #![feature(generic_const_exprs)]
    /// use hyperloglog_rs::prelude::*;
    /// 
    /// let counters = vec![HyperLogLog::new(), HyperLogLog::new(), HyperLogLog::new()];
    /// 
    /// let hll_array = HyperLogLogArray::<12, 6, 3>::from(counters.as_slice());
    /// ```
    fn from(counters: &[HyperLogLog<PRECISION, BITS>]) -> Self {
        assert_eq!(counters.len(), N, concat!(
            "The length of the slice of HyperLogLog counters must be equal to the number of counters ",
            "in the HyperLogLogArray."
        ));
        let mut array = [HyperLogLog::new(); N];
        array.copy_from_slice(&counters[..N]);
        Self { counters: array }
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize, H: Hash> From<Vec<Vec<H>>>
    for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Creates a new HyperLogLogArray from the given vector of vectors of hashable items.
    ///
    /// # Arguments
    /// * `items`: The vector of vectors of hashable items to create the HyperLogLogArray from.
    ///
    /// # Returns
    /// A new HyperLogLogArray from the given vector of vectors of hashable items.
    ///
    /// # Example
    ///
    /// ```rust
    /// #![feature(generic_const_exprs)]
    /// use std::hash::Hash;
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll_array = HyperLogLogArray::<12, 6, 3>::from(vec![
    ///     vec![1, 2, 3],
    ///     vec![4, 5, 6],
    ///     vec![7, 8, 9],
    /// ]);
    /// ```
    fn from(items: Vec<Vec<H>>) -> Self {
        assert_eq!(items.len(), N, concat!(
            "The length of the vector of vectors of hashable items must be equal to the number of counters ",
            "in the HyperLogLogArray."
        ));
        let mut array = [HyperLogLog::new(); N];
        for (i, item) in items.into_iter().enumerate() {
            for item in item.into_iter() {
                array[i].insert(&item);
            }
        }
        Self { counters: array }
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize, H: Hash> From<&[Vec<H>]>
    for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Creates a new HyperLogLogArray from the given slice of vectors of hashable items.
    ///
    /// # Arguments
    /// * `items`: The slice of vectors of hashable items to create the HyperLogLogArray from.
    ///
    /// # Returns
    /// A new HyperLogLogArray from the given slice of vectors of hashable items.
    ///
    /// # Example
    ///
    /// ```rust
    /// #![feature(generic_const_exprs)]
    /// use std::hash::Hash;
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll_array = HyperLogLogArray::<12, 6, 3>::from(&[
    ///     vec![1, 2, 3],
    ///     vec![4, 5, 6],
    ///     vec![7, 8, 9],
    /// ]);
    /// ```
    fn from(items: &[Vec<H>]) -> Self {
        assert_eq!(items.len(), N, concat!(
            "The length of the slice of vectors of hashable items must be equal to the number of counters ",
            "in the HyperLogLogArray."
        ));
        let mut array = [HyperLogLog::new(); N];
        for (i, item) in items.iter().enumerate() {
            for item in item.iter() {
                array[i].insert(item);
            }
        }
        Self { counters: array }
    }
}
