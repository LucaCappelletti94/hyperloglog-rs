//! Submodule implementing HyperLogLog++.
use crate::basicloglog::BasicLogLog;
use crate::hll_impl;
use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
pub struct PlusPlus<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    Hasher: HasherType = twox_hash::XxHash64,
> {
    counter: BasicLogLog<P, B, R, Hasher>,
}

hll_impl!(PlusPlus<P, B, R, Hasher>);

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType>
    From<BasicLogLog<P, B, R, Hasher>> for PlusPlus<P, B, R, Hasher>
{
    fn from(counter: BasicLogLog<P, B, R, Hasher>) -> Self {
        Self { counter }
    }
}

impl<F: FloatNumber, P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> Estimator<F>
    for PlusPlus<P, B, R, Hasher>
where
    P: PrecisionConstants<F>,
    Self: HyperLogLog<P, B, Hasher>,
{
    fn estimate_cardinality(&self) -> F {
        P::plusplus_estimate(self.harmonic_sum(), self.get_number_of_zero_registers())
    }

    fn estimate_union_cardinality(&self, other: &Self) -> F {
        let (harmonic_sum, number_of_zero_registers) = self
            .registers()
            .get_harmonic_sum_and_zeros(other.registers());
        P::plusplus_estimate(harmonic_sum, number_of_zero_registers)
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_cardinality() {
        let mut hll = PlusPlus::<
            Precision5,
            Bits6,
            <Precision5 as ArrayRegister<Bits6>>::ArrayRegister,
        >::default();
        hll.extend(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let estimate: f32 = hll.estimate_cardinality();
        assert!(estimate > 10.0 * (1.0 - Precision5::error_rate() as f32));
        assert!(estimate < 10.0 * (1.0 + Precision5::error_rate() as f32));
        let estimate: f64 = hll.estimate_cardinality();
        assert!(estimate > 10.0 * (1.0 - Precision5::error_rate()));
        assert!(estimate < 10.0 * (1.0 + Precision5::error_rate()));
    }

    #[test]
    /// In this test we verify that the output of the `estimate_union_cardinality` method always
    /// yields the same result as the `estimate_cardinality` run on the bitor of the two sets.
    fn test_union_bitor() {
        let iterations = 100;
        let mut random_state = splitmix64(6545345645876_u64);

        let mut hll1 = PlusPlus::<
            Precision5,
            Bits6,
            <Precision5 as ArrayRegister<Bits6>>::ArrayRegister,
        >::default();
        let mut hll2 = PlusPlus::<
            Precision5,
            Bits6,
            <Precision5 as ArrayRegister<Bits6>>::ArrayRegister,
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
                let bitor_estimate: f64 = (hll1 | hll2).estimate_cardinality();

                assert_eq!(union_estimate, bitor_estimate);
            }
        }
    }
}
