//! Submodule implementing `HyperLogLog++`.
use crate::basicloglog::BasicLogLog;
use crate::hll_impl;
use crate::prelude::*;
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
    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        let (harmonic_sum, number_of_zero_registers) = self
            .registers()
            .get_harmonic_sum_and_zeros(other.registers());

        correct_union_estimate(
            P::plusplus_estimate(self.harmonic_sum(), self.get_number_of_zero_registers()),
            P::plusplus_estimate(other.harmonic_sum(), other.get_number_of_zero_registers()),
            P::plusplus_estimate(harmonic_sum, number_of_zero_registers),
        )
    }

    #[inline]
    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "precision_5")]
    fn test_estimate_cardinality() {
        let mut hll = PlusPlus::<
            Precision5,
            Bits6,
            <Precision5 as ArrayRegister<Bits6>>::Array,
            twox_hash::XxHash64,
        >::default();
        hll.extend(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let estimate: f64 = hll.estimate_cardinality();
        assert!(estimate > 10.0 * (1.0 - Precision5::error_rate()));
        assert!(estimate < 10.0 * (1.0 + Precision5::error_rate()));
    }

    #[test]
    #[cfg(feature = "precision_5")]
    /// In this test we verify that the output of the `estimate_union_cardinality` method always
    /// yields the same result as the `estimate_cardinality` run on the bitor of the two sets.
    fn test_union_bitor() {
        let iterations = 10;
        let mut random_state = splitmix64(6545345645876_u64);

        let mut hll1 = PlusPlus::<
            Precision5,
            Bits6,
            <Precision5 as ArrayRegister<Bits6>>::Array,
            twox_hash::XxHash64,
        >::default();
        let mut hll2 = PlusPlus::<
            Precision5,
            Bits6,
            <Precision5 as ArrayRegister<Bits6>>::Array,
            twox_hash::XxHash64,
        >::default();

        for _ in 0..iterations {
            random_state = splitmix64(random_state);
            for (i, value) in iter_random_values(100_000, None, random_state).enumerate() {
                if i % 2 == 0 {
                    hll1.insert(&value);
                } else {
                    hll2.insert(&value);
                }

                let union_estimate: f64 = hll1.estimate_union_cardinality(&hll2);
                let union_inverted_estimate: f64 = hll2.estimate_union_cardinality(&hll1);
                let bitor_estimate: f64 = (hll1 | hll2).estimate_cardinality();
                let bitor_inverted_estimate: f64 = (hll2 | hll1).estimate_cardinality();

                assert_eq!(bitor_inverted_estimate, bitor_estimate);
                assert_eq!(union_estimate, union_inverted_estimate);
            }
        }
    }
}
