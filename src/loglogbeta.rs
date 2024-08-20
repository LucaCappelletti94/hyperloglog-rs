//! Submodule implementing [`LogLogBeta`].
use crate::basicloglog::BasicLogLog;
use crate::hll_impl;
use crate::prelude::*;

#[cfg(feature = "std")]
use core::any::type_name;

#[cfg(feature = "std")]
use crate::utils::Named;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
/// A struct implementing the [`LogLogBeta`] algorithm.
pub struct LogLogBeta<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    Hasher: HasherType = twox_hash::XxHash64,
> {
    /// The underlying `BasicLogLog` counter.
    counter: BasicLogLog<P, B, R, Hasher>,
}

hll_impl!(LogLogBeta<P, B, R, Hasher>);

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType>
    From<BasicLogLog<P, B, R, Hasher>> for LogLogBeta<P, B, R, Hasher>
{
    #[inline]
    fn from(counter: BasicLogLog<P, B, R, Hasher>) -> Self {
        Self { counter }
    }
}

#[cfg(feature = "std")]
impl<P: Precision + Named, B: Bits + Named, R: Registers<P, B> + Named, Hasher: HasherType> Named
    for LogLogBeta<P, B, R, Hasher>
{
    #[inline]
    fn name(&self) -> String {
        #[cfg(feature = "precomputed_beta")]
        let model_name = "LLPB";
        #[cfg(not(feature = "precomputed_beta"))]
        let model_name = "LLB";

        format!(
            "{model_name}<{}, {}, {}> + {}",
            P::default().name(),
            B::default().name(),
            self.registers().name(),
            type_name::<Hasher>().split("::").last().unwrap()
        )
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> Estimator<f64>
    for LogLogBeta<P, B, R, Hasher>
where
    Self: HyperLogLog<P, B, Hasher>,
{
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        P::beta_estimate(self.harmonic_sum(), self.get_number_of_zero_registers())
    }

    #[inline]
    fn estimate_union_cardinality_with_cardinalities(
        &self,
        other: &Self,
        self_cardinality: f64,
        other_cardinality: f64,
    ) -> f64 {
        let (harmonic_sum, number_of_zero_registers) = self
            .registers()
            .get_harmonic_sum_and_zeros(other.registers());

        correct_union_estimate(
            self_cardinality,
            other_cardinality,
            P::beta_estimate(harmonic_sum, number_of_zero_registers),
        )
    }
}
