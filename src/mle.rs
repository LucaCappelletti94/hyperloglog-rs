//! Struct marker MLE.

use crate::prelude::*;

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
pub struct MLE<H, const ERROR: i32 = 2> {
    inner: H,
}

impl<H, const ERROR: i32> From<H> for MLE<H, ERROR> {
    fn from(inner: H) -> Self {
        Self { inner }
    }
}

impl<H: BitOrAssign, const ERROR: i32> BitOrAssign for MLE<H, ERROR> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.inner |= rhs.inner;
    }
}

impl<H: BitOr<H, Output = H>, const ERROR: i32> BitOr for MLE<H, ERROR> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner | rhs.inner,
        }
    }
}

impl<P: Precision, B: Bits, Hasher: core::hash::Hasher + Default, H: HyperLogLogTrait<P, B, Hasher>, const ERROR: i32> HyperLogLogTrait<P, B, Hasher>
    for MLE<H, ERROR>
{
    type Registers = H::Registers;

    fn registers(&self) -> &Self::Registers {
        self.inner.registers()
    }

    fn harmonic_sum<F: FloatNumber>(&self) -> F
    where
        P: PrecisionConstants<F>,
    {
        self.inner.harmonic_sum()
    }

    fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    fn get_number_of_zero_registers(&self) -> <P as Precision>::NumberOfZeros {
        self.inner.get_number_of_zero_registers()
    }

    fn insert<T: core::hash::Hash>(&mut self, value: T) -> bool {
        self.inner.insert(value)
    }

    fn get_register(&self, index: usize) -> u32 {
        self.inner.get_register(index)
    }

    fn estimate_union_cardinality<F: FloatNumber>(
        &self,
        other: &Self,
    ) -> F
    where
        P: PrecisionConstants<F>,
    {
        self.inner
            .estimate_union_cardinality_with_mle::<ERROR, F>(&other.inner)
            .get_union_cardinality()
    }

    fn estimate_intersection_cardinality<F: FloatNumber>(
        &self,
        other: &Self,
    ) -> F
    where
        P: PrecisionConstants<F>,
    {
        self.inner
            .estimate_union_cardinality_with_mle::<ERROR, F>(&other.inner)
            .get_intersection_cardinality()
    }

    fn estimate_difference_cardinality<F: FloatNumber>(
        &self,
        other: &Self,
    ) -> F
    where
        P: PrecisionConstants<F>,
    {
        self.inner
            .estimate_union_cardinality_with_mle::<ERROR, F>(&other.inner)
            .get_left_difference_cardinality()
    }

    fn estimate_jaccard_index<F: FloatNumber>(&self, other: &Self) -> F
    where
        P: PrecisionConstants<F>,
    {
        self.inner
            .estimate_union_cardinality_with_mle::<ERROR, F>(&other.inner)
            .get_jaccard_index()
    }
}
