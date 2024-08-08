use core::{
    hash::Hash,
    ops::{Index, IndexMut},
};

use crate::{prelude::*, utils::FloatNumber};

#[repr(transparent)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
pub struct HyperLogLogArray<
    P: Precision,
    B: Bits,
    H: HyperLogLogTrait<P, B, Hasher>,
    Hasher: core::hash::Hasher + Default,
    const N: usize,
> {
    counters: [H; N],
    _phantom: core::marker::PhantomData<(P, B, Hasher)>,
}

impl<
        P: Precision,
        B: Bits,
        H: HyperLogLogTrait<P, B, Hasher>,
        Hasher: core::hash::Hasher + Default,
        const N: usize,
    > PartialEq for HyperLogLogArray<P, B, H, Hasher, N>
{
    #[inline(always)]
    /// Returns true if the two HyperLogLogArrays are equal.
    ///
    /// # Arguments
    /// * `other`: The other HyperLogLogArray to compare with.
    ///
    /// # Returns
    /// True if the two HyperLogLogArrays are equal.
    fn eq(&self, other: &Self) -> bool {
        self.counters == other.counters
    }
}

impl<
        P: Precision,
        B: Bits,
        H: HyperLogLogTrait<P, B, Hasher>,
        Hasher: core::hash::Hasher + Default,
        const N: usize,
    > Eq for HyperLogLogArray<P, B, H, Hasher, N>
{
}

impl<
        P: Precision,
        B: Bits,
        Hasher: core::hash::Hasher + Default,
        H: HyperLogLogTrait<P, B, Hasher> + Copy,
        const N: usize,
    > Default for HyperLogLogArray<P, B, H, Hasher, N>
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
    /// let hll_array = HyperLogLogArray::<
    ///     Precision12,
    ///     Bits6,
    ///     HyperLogLog<Precision12, Bits6, <Precision12 as ArrayRegister<Bits6>>::ArrayRegister>,
    ///     3,
    /// >::default();
    /// ```
    fn default() -> Self {
        Self {
            counters: [H::default(); N],
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<
        P: Precision,
        B: Bits,
        H: HyperLogLogTrait<P, B, Hasher>,
        Hasher: core::hash::Hasher + Default,
        const N: usize,
    > AsRef<[H; N]> for HyperLogLogArray<P, B, H, Hasher, N>
{
    #[inline(always)]
    /// Returns a reference to the underlying array of HyperLogLog counters.
    ///
    /// # Returns
    /// A reference to the underlying array of HyperLogLog counters.
    fn as_ref(&self) -> &[H; N] {
        &self.counters
    }
}

impl<
        P: Precision,
        B: Bits,
        H: HyperLogLogTrait<P, B, Hasher>,
        Hasher: core::hash::Hasher + Default,
        const N: usize,
    > AsMut<[H; N]> for HyperLogLogArray<P, B, H, Hasher, N>
{
    #[inline(always)]
    /// Returns a mutable reference to the underlying array of HyperLogLog counters.
    ///
    /// # Returns
    /// A mutable reference to the underlying array of HyperLogLog counters.
    fn as_mut(&mut self) -> &mut [H; N] {
        &mut self.counters
    }
}

impl<
        P: Precision,
        B: Bits,
        H: HyperLogLogTrait<P, B, Hasher>,
        Hasher: core::hash::Hasher + Default,
        const N: usize,
    > Index<usize> for HyperLogLogArray<P, B, H, Hasher, N>
{
    type Output = H;

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
    /// let mut hll_array = HyperLogLogArray::<
    ///     Precision12,
    ///     Bits6,
    ///     HyperLogLog<Precision12, Bits6, <Precision12 as ArrayRegister<Bits6>>::ArrayRegister>,
    ///     4,
    /// >::default();
    /// hll_array.insert(0, &1);
    /// hll_array.insert(1, &2);
    /// hll_array.insert(2, &3);
    ///
    /// assert!(
    ///     hll_array[0].estimate_cardinality::<f32>() > 0.9
    ///         && hll_array[1].estimate_cardinality::<f32>() < 1.1
    /// );
    /// assert!(
    ///     hll_array[1].estimate_cardinality::<f32>() > 0.9
    ///         && hll_array[1].estimate_cardinality::<f32>() < 1.1
    /// );
    /// assert!(
    ///     hll_array[2].estimate_cardinality::<f32>() > 0.9
    ///         && hll_array[2].estimate_cardinality::<f32>() < 1.1
    /// );
    /// assert!(
    ///     hll_array[3].estimate_cardinality::<f32>() > -0.1
    ///         && hll_array[3].estimate_cardinality::<f32>() < 0.1
    /// );
    /// ```
    fn index(&self, index: usize) -> &Self::Output {
        &self.counters[index]
    }
}

impl<
        P: Precision,
        B: Bits,
        H: HyperLogLogTrait<P, B, Hasher>,
        Hasher: core::hash::Hasher + Default,
        const N: usize,
    > IndexMut<usize> for HyperLogLogArray<P, B, H, Hasher, N>
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
    /// let mut hll_array = HyperLogLogArray::<
    ///     Precision12,
    ///     Bits6,
    ///     HyperLogLog<Precision12, Bits6, <Precision12 as ArrayRegister<Bits6>>::ArrayRegister>,
    ///     4,
    /// >::default();
    /// hll_array[0].insert(&1);
    /// hll_array[1].insert(&2);
    /// hll_array[2].insert(&3);
    ///
    /// assert!(
    ///     hll_array[0].estimate_cardinality::<f32>() > 0.9
    ///         && hll_array[1].estimate_cardinality::<f32>() < 1.1
    /// );
    /// assert!(
    ///     hll_array[1].estimate_cardinality::<f32>() > 0.9
    ///         && hll_array[1].estimate_cardinality::<f32>() < 1.1
    /// );
    /// assert!(
    ///     hll_array[2].estimate_cardinality::<f32>() > 0.9
    ///         && hll_array[2].estimate_cardinality::<f32>() < 1.1
    /// );
    /// assert!(
    ///     hll_array[3].estimate_cardinality::<f32>() > -0.1
    ///         && hll_array[3].estimate_cardinality::<f32>() < 0.1
    /// );
    /// ```
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.counters[index]
    }
}

impl<
        P: Precision,
        B: Bits,
        H: HyperLogLogTrait<P, B, Hasher>,
        Hasher: core::hash::Hasher + Default,
        const N: usize,
    > From<[H; N]> for HyperLogLogArray<P, B, H, Hasher, N>
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
    /// let hll_array = HyperLogLogArray::<
    ///     Precision12,
    ///     Bits6,
    ///     HyperLogLog<Precision12, Bits6, <Precision12 as ArrayRegister<Bits6>>::ArrayRegister>,
    ///     3,
    /// >::from([
    ///     HyperLogLog::default(),
    ///     HyperLogLog::default(),
    ///     HyperLogLog::default(),
    /// ]);
    /// ```
    fn from(counters: [H; N]) -> Self {
        Self {
            counters,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<
        P: Precision,
        B: Bits,
        H: HyperLogLogTrait<P, B, Hasher> + HyperSpheresSketch + NormalizedHyperSpheresSketch + Copy,
        const N: usize,
        Hasher: core::hash::Hasher + Default + Clone,
    > HyperLogLogArrayTrait<P, B, H, Hasher, N> for HyperLogLogArray<P, B, H, Hasher, N>
{
    #[inline(always)]
    /// Returns the estimated overlap and difference cardinality matrices and vectors with the provided HyperLogLogArray.
    ///
    /// # Arguments
    /// * `other`: The HyperLogLogArray to estimate the overlap and difference cardinality matrices and vectors with.
    ///
    /// # Returns
    /// The estimated overlap and difference cardinality matrices and vectors with the provided HyperLogLogArray.
    fn overlap_and_differences_cardinality_matrices<F: FloatNumber>(
        &self,
        other: &Self,
    ) -> ([[F; N]; N], [F; N], [F; N])
    where
        H: SetLike<F>,
        P: PrecisionConstants<F>,
    {
        H::overlap_and_differences_cardinality_matrices(self.as_ref(), other.as_ref())
    }

    #[inline(always)]
    /// Returns the estimated normalized overlap and difference cardinality matrices and vectors with the provided HyperLogLogArray.
    ///
    /// # Arguments
    /// * `other`: The HyperLogLogArray to estimate the normalized overlap and difference cardinality matrices and vectors with.
    ///
    /// # Returns
    /// The estimated normalized overlap and difference cardinality matrices and vectors with the provided HyperLogLogArray.
    fn normalized_overlap_and_differences_cardinality_matrices<F: FloatNumber>(
        &self,
        other: &Self,
    ) -> ([[F; N]; N], [F; N], [F; N])
    where
        H: SetLike<F>,
        P: PrecisionConstants<F>,
    {
        H::normalized_overlap_and_differences_cardinality_matrices(self.as_ref(), other.as_ref())
    }

    #[inline(always)]
    fn insert<T: Hash>(&mut self, i: usize, value: T) {
        self[i].insert(&value);
    }
}
