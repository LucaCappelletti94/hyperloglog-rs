//! Submodule implementing `HyperLogLog++`.
use crate::basicloglog::BasicLogLog;
use crate::hll_impl;
use crate::prelude::*;

#[cfg(feature = "std")]
use core::any::type_name;

#[derive(Debug, Clone, Copy, Default)]
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

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> AsRef<BasicLogLog<P, B, R, Hasher>>
    for PlusPlus<P, B, R, Hasher>
{
    #[inline]
    fn as_ref(&self) -> &BasicLogLog<P, B, R, Hasher> {
        &self.counter
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> AsMut<BasicLogLog<P, B, R, Hasher>>
    for PlusPlus<P, B, R, Hasher>
{
    #[inline]
    fn as_mut(&mut self) -> &mut BasicLogLog<P, B, R, Hasher> {
        &mut self.counter
    }
}

#[cfg(feature = "std")]
impl<P: Precision + Named, B: Bits + Named, R: Registers<P, B> + Named, Hasher: HasherType> Named
    for PlusPlus<P, B, R, Hasher>
{
    #[inline]
    fn name(&self) -> String {
        #[cfg(all(feature = "integer_plusplus", not(feature = "plusplus_kmeans")))]
        let estimator_name = "PPI";
        #[cfg(all(feature = "integer_plusplus", feature = "plusplus_kmeans"))]
        let estimator_name = "PPIK";
        #[cfg(all(not(feature = "integer_plusplus"), not(feature = "plusplus_kmeans")))]
        let estimator_name = "PP";
        #[cfg(all(not(feature = "integer_plusplus"), feature = "plusplus_kmeans"))]
        let estimator_name = "PPK";
        #[cfg(feature = "std_ln")]
        let estimator_name = format!("{estimator_name}-std-ln");

        format!(
            "{estimator_name}<{}, {}, {}> + {}",
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

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> HyperLogLog
    for PlusPlus<P, B, R, Hasher>
{
    type Registers = R;
    type Precision = P;
    type Bits = B;
    type Hasher = Hasher;

    #[inline]
    fn registers(&self) -> &Self::Registers {
        self.counter.registers()
    }

    #[inline]
    fn get_number_of_zero_registers(&self) -> u32 {
        self.counter.get_number_of_zero_registers()
    }

    #[inline]
    fn get_register(&self, index: usize) -> u8 {
        self.counter.get_register(index)
    }

    #[inline]
    fn harmonic_sum(&self) -> f64 {
        self.counter.harmonic_sum()
    }

    #[inline]
    fn insert_register_value_and_index(
            &mut self,
            new_register_value: u8,
            index: usize,
        ) -> bool {
        self.counter.insert_register_value_and_index(new_register_value, index)
    }

    #[inline]
    fn from_registers(registers: R) -> Self {
        Self {
            counter: HyperLogLog::from_registers(registers),
        }
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> Correction
    for PlusPlus<P, B, R, Hasher>
{
    #[inline]
    fn correction(
        harmonic_sum: f64,
        number_of_zero_registers: u32,
    ) -> f64 {
        P::plusplus_estimate(harmonic_sum, number_of_zero_registers)
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> Estimator<f64>
    for PlusPlus<P, B, R, Hasher>
where
    Self: HyperLogLog<Precision = P, Bits = B, Registers = R, Hasher = Hasher>,
{
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        P::plusplus_estimate(self.harmonic_sum(), self.get_number_of_zero_registers())
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
            P::plusplus_estimate(harmonic_sum, number_of_zero_registers),
        )
    }
}
