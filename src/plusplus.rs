//! Submodule implementing `HyperLogLog++`.
use crate::basicloglog::BasicLogLog;
use crate::hll_impl;
use crate::prelude::*;

#[cfg(feature = "std")]
use core::any::type_name;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
/// A struct implementing the `HyperLogLog++` algorithm.
pub struct PlusPlus<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    Hasher: HasherType = twox_hash::XxHash64,
> {
    /// The underlying `BasicLogLog` counter.
    counter: BasicLogLog<P, B, R, Hasher>,
}

#[cfg(feature = "std")]
impl<P: Precision + Named, B: Bits + Named, R: Registers<P, B> + Named, Hasher: HasherType> Named
    for PlusPlus<P, B, R, Hasher>
{
    #[inline]
    fn name(&self) -> String {
        #[cfg(all(feature = "integer_plusplus", not(feature = "plusplus_kmeans")))]
        let model_name = "PPI";
        #[cfg(all(feature = "integer_plusplus", feature = "plusplus_kmeans"))]
        let model_name = "PPIK";
        #[cfg(all(not(feature = "integer_plusplus"), not(feature = "plusplus_kmeans")))]
        let model_name = "PP";
        #[cfg(all(not(feature = "integer_plusplus"), feature = "plusplus_kmeans"))]
        let model_name = "PPK";

        format!(
            "{model_name}<{}, {}, {}> + {}",
            P::default().name(),
            B::default().name(),
            self.registers().name(),
            type_name::<Hasher>().split("::").last().unwrap()
        )
    }
}

hll_impl!(PlusPlus<P, B, R, Hasher>);

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType>
    From<BasicLogLog<P, B, R, Hasher>> for PlusPlus<P, B, R, Hasher>
{
    #[inline]
    fn from(counter: BasicLogLog<P, B, R, Hasher>) -> Self {
        Self { counter }
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> Estimator<f64>
    for PlusPlus<P, B, R, Hasher>
where
    Self: HyperLogLog<P, B, Hasher>,
{
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        P::plusplus_estimate(self.harmonic_sum(), self.get_number_of_zero_registers())
    }

    #[inline]
    fn estimate_union_cardinality_with_cardinalities(&self, other: &Self, self_cardinality: f64, other_cardinality: f64) -> f64 {
        let (harmonic_sum, number_of_zero_registers) = self
            .registers()
            .get_harmonic_sum_and_zeros(other.registers());

        correct_union_estimate(
            self_cardinality,
            other_cardinality,
            P::plusplus_estimate(harmonic_sum, number_of_zero_registers),
        )
    }
}
