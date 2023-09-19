use core::{
    hash::Hash,
    ops::{Index, IndexMut},
};

use siphasher::sip::SipHasher13;

use crate::prelude::*;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Eq)]
pub struct HyperLogLogArray<
    PRECISION: Precision + WordType<BITS>,
    const BITS: usize,
    const N: usize,
    M: HasherMethod + Copy = SipHasher13,
> {
    counters: [HyperLogLog<PRECISION, BITS, M>; N],
}

impl<
        PRECISION: Precision + WordType<BITS>,
        const BITS: usize,
        const N: usize,
        M: HasherMethod + Copy,
    > PartialEq for HyperLogLogArray<PRECISION, BITS, N, M>
{
    #[inline(always)]
    /// Returns true if the HyperLogLogArray is equal to the other HyperLogLogArray.
    ///
    /// # Arguments
    /// * `other`: The other HyperLogLogArray to compare to.
    ///
    /// # Returns
    /// True if the HyperLogLogArray is equal to the other HyperLogLogArray.
    fn eq(&self, other: &Self) -> bool {
        self.counters.eq(&other.counters)
    }
}

impl<
        PRECISION: Precision + WordType<BITS>,
        const BITS: usize,
        const N: usize,
        M: HasherMethod + Copy,
    > Default for HyperLogLogArray<PRECISION, BITS, N, M>
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
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll_array = HyperLogLogArray::<Precision12, 6, 3>::default();
    /// ```
    fn default() -> Self {
        Self {
            counters: [HyperLogLog::new(); N],
        }
    }
}

impl<
        PRECISION: Precision + WordType<BITS>,
        const BITS: usize,
        const N: usize,
        M: HasherMethod + Copy,
    > HyperLogLogArray<PRECISION, BITS, N, M>
{
    #[inline(always)]
    /// Creates a new HyperLogLogArray with the given precision and number of bits.
    ///
    /// # Example
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll_array = HyperLogLogArray::<Precision12, 6, 3>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            counters: [HyperLogLog::new(); N],
        }
    }

    #[inline(always)]
    /// Returns the estimated overlap and difference cardinality matrices and vectors with the provided HyperLogLogArray.
    ///
    /// # Arguments
    /// * `other`: The HyperLogLogArray to estimate the overlap and difference cardinality matrices and vectors with.
    ///
    /// # Returns
    /// The estimated overlap and difference cardinality matrices and vectors with the provided HyperLogLogArray.
    pub fn overlap_and_differences_cardinality_matrices<F: Primitive<f32>>(
        &self,
        other: &Self,
    ) -> ([[F; N]; N], [F; N], [F; N]) {
        HyperLogLog::overlap_and_differences_cardinality_matrices(self.as_ref(), other.as_ref())
    }
}

impl<
        PRECISION: Precision + WordType<BITS>,
        const BITS: usize,
        const N: usize,
        M: HasherMethod + Copy,
    > AsRef<[HyperLogLog<PRECISION, BITS, M>; N]> for HyperLogLogArray<PRECISION, BITS, N, M>
{
    #[inline(always)]
    /// Returns a reference to the underlying array of HyperLogLog counters.
    ///
    /// # Returns
    /// A reference to the underlying array of HyperLogLog counters.
    fn as_ref(&self) -> &[HyperLogLog<PRECISION, BITS, M>; N] {
        &self.counters
    }
}

impl<
        PRECISION: Precision + WordType<BITS>,
        const BITS: usize,
        const N: usize,
        M: HasherMethod + Copy,
    > AsMut<[HyperLogLog<PRECISION, BITS, M>; N]> for HyperLogLogArray<PRECISION, BITS, N, M>
{
    #[inline(always)]
    /// Returns a mutable reference to the underlying array of HyperLogLog counters.
    ///
    /// # Returns
    /// A mutable reference to the underlying array of HyperLogLog counters.
    fn as_mut(&mut self) -> &mut [HyperLogLog<PRECISION, BITS, M>; N] {
        &mut self.counters
    }
}

impl<
        PRECISION: Precision + WordType<BITS>,
        const BITS: usize,
        const N: usize,
        M: HasherMethod + Copy,
    > Index<usize> for HyperLogLogArray<PRECISION, BITS, N, M>
{
    type Output = HyperLogLog<PRECISION, BITS, M>;

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
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll_array = HyperLogLogArray::<Precision12, 6, 4>::new();
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

impl<
        PRECISION: Precision + WordType<BITS>,
        const BITS: usize,
        const N: usize,
        M: HasherMethod + Copy,
    > IndexMut<usize> for HyperLogLogArray<PRECISION, BITS, N, M>
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
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll_array = HyperLogLogArray::<Precision12, 6, 4>::new();
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

impl<
        PRECISION: Precision + WordType<BITS>,
        const BITS: usize,
        const N: usize,
        M: HasherMethod + Copy,
    > From<[HyperLogLog<PRECISION, BITS, M>; N]> for HyperLogLogArray<PRECISION, BITS, N, M>
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
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll_array = HyperLogLogArray::<Precision12, 6, 3>::from([
    ///     HyperLogLog::new(),
    ///     HyperLogLog::new(),
    ///     HyperLogLog::new(),
    /// ]);
    /// ```
    fn from(counters: [HyperLogLog<PRECISION, BITS, M>; N]) -> Self {
        Self { counters }
    }
}

impl<
        PRECISION: Precision + WordType<BITS>,
        const BITS: usize,
        const N: usize,
        H: Hash,
        M: HasherMethod + Copy,
    > From<&[&[H]; N]> for HyperLogLogArray<PRECISION, BITS, N, M>
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
    /// use core::hash::Hash;
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll_array = HyperLogLogArray::<Precision12, 6, 3>::from(&[
    ///     [1, 2, 3].as_slice(),
    ///     [4, 5, 6].as_slice(),
    ///     [7, 8, 9].as_slice(),
    /// ]);
    /// ```
    fn from(items: &[&[H]; N]) -> Self {
        let mut array = [HyperLogLog::new(); N];
        for (i, item) in items.iter().enumerate() {
            for item in item.iter() {
                array[i].insert(item);
            }
        }
        Self { counters: array }
    }
}

impl<
        PRECISION: Precision + WordType<BITS>,
        const BITS: usize,
        const N: usize,
        M: HasherMethod + Copy,
    > From<&[HyperLogLog<PRECISION, BITS, M>]> for HyperLogLogArray<PRECISION, BITS, N, M>
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
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll_array = HyperLogLogArray::<Precision12, 6, 3>::from(vec![
    ///     HyperLogLog::new(),
    ///     HyperLogLog::new(),
    ///     HyperLogLog::new(),
    /// ].as_slice());
    /// ```
    fn from(counters: &[HyperLogLog<PRECISION, BITS, M>]) -> Self {
        assert_eq!(counters.len(), N, concat!(
            "The length of the vector of HyperLogLog counters must be equal to the number of counters ",
            "in the HyperLogLogArray."
        ));
        let mut array = [HyperLogLog::new(); N];
        array.copy_from_slice(&counters[..N]);
        Self { counters: array }
    }
}

impl<
        PRECISION: Precision + WordType<BITS>,
        const BITS: usize,
        const N: usize,
        H: Hash,
        M: HasherMethod + Copy,
    > From<&[&[H]]> for HyperLogLogArray<PRECISION, BITS, N, M>
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
    /// use core::hash::Hash;
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll_array = HyperLogLogArray::<Precision12, 6, 3>::from(&[
    ///     [1, 2, 3].as_slice(),
    ///     [4, 5, 6].as_slice(),
    ///     [7, 8, 9].as_slice(),
    /// ]);
    /// ```
    fn from(items: &[&[H]]) -> Self {
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

impl<
        PRECISION: Precision + WordType<BITS>,
        const BITS: usize,
        const N: usize,
        H: Hash,
        M: HasherMethod + Copy,
    > From<[&[H]; N]> for HyperLogLogArray<PRECISION, BITS, N, M>
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
    /// use core::hash::Hash;
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll_array = HyperLogLogArray::<Precision12, 6, 3>::from([
    ///     vec![1_usize, 2, 3].as_slice(),
    ///     vec![4, 5, 6].as_slice(),
    ///     vec![7, 8, 9].as_slice(),
    /// ]);
    /// ```
    fn from(items: [&[H]; N]) -> Self {
        let mut array = [HyperLogLog::new(); N];
        for (i, item) in items.iter().enumerate() {
            for item in item.iter() {
                array[i].insert(item);
            }
        }
        Self { counters: array }
    }
}
