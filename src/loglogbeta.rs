//! Submodule implementing [`LogLogBeta`].
use crate::basicloglog::BasicLogLog;
use crate::hll_impl;
use crate::prelude::*;
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
    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        let (harmonic_sum, number_of_zero_registers) = self
            .registers()
            .get_harmonic_sum_and_zeros(other.registers());

        correct_union_estimate(
            P::beta_estimate(self.harmonic_sum(), self.get_number_of_zero_registers()),
            P::beta_estimate(other.harmonic_sum(), other.get_number_of_zero_registers()),
            P::beta_estimate(harmonic_sum, number_of_zero_registers),
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
        let mut hll = LogLogBeta::<
            Precision5,
            Bits6,
            <Precision5 as ArrayRegister<Bits6>>::ArrayRegister,
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

        let mut hll1 = LogLogBeta::<
            Precision5,
            Bits6,
            <Precision5 as ArrayRegister<Bits6>>::ArrayRegister,
        >::default();
        let mut hll2 = LogLogBeta::<
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
                let union_inverted_estimate: f64 = hll2.estimate_union_cardinality(&hll1);
                let bitor_estimate: f64 = (hll1 | hll2).estimate_cardinality();
                let bitor_inverted_estimate: f64 = (hll2 | hll1).estimate_cardinality();

                assert_eq!(bitor_inverted_estimate, bitor_estimate);
                assert_eq!(union_estimate, union_inverted_estimate);
            }
        }
    }
}
