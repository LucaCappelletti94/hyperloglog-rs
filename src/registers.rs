//! Submodule providing the trait registers

use core::fmt::Debug;

use crate::prelude::*;
use crate::utils::{FloatOps, Number, Zero};
mod packed_array;

pub use packed_array::{AllArrays, Array, ArrayRegister};

/// Trait for a register word.
pub(super) trait RegisterWord<B: Bits> {
    /// The mask for the lower register.
    const LOWER_REGISTER_MASK: Self;
    /// The number of bits in the word.
    const NUMBER_OF_BITS: u8;
}

impl<B: Bits> RegisterWord<B> for u64 {
    const LOWER_REGISTER_MASK: Self = (1 << B::NUMBER_OF_BITS) - 1;
    const NUMBER_OF_BITS: u8 = 64;
}

/// Trait marker for the registers.
pub trait Registers<P: Precision, B: Bits>:
    Eq + PartialEq + Clone + Debug + Send + Sync + Default
{
    /// Iterator over the registers.
    type Iter<'register>: Iterator<Item = u8>
    where
        Self: 'register;

    /// Iterator over the registers zipped with another set of registers.
    type IterZipped<'registers>: Iterator<Item = [u8; 2]>
    where
        Self: 'registers;

    /// Returns an iterator over the registers.
    fn iter_registers(&self) -> Self::Iter<'_>;

    /// Returns an iterator over the registers zipped with another set of registers.
    fn iter_registers_zipped<'registers>(
        &'registers self,
        other: &'registers Self,
    ) -> Self::IterZipped<'registers>;

    /// Returns the harmonic sum of the maximum value of the registers and the number of zero registers.
    fn get_harmonic_sum_and_zeros(&self, other: &Self) -> (f64, P::NumberOfRegisters);

    /// Applies a function to each register.
    fn apply_to_registers<F>(&mut self, f: F)
    where
        F: FnMut(u8) -> u8;

    /// Updates the register at the given index with the given value,
    /// if the value is greater than the current value in the register.
    ///
    /// # Arguments
    /// * `index` - The index of the register to be updated.
    /// * `value` - The value to be set in the register.
    ///
    /// # Returns
    /// The previous value of the register, and the larger of the two values.
    fn set_greater(&mut self, index: P::NumberOfRegisters, value: u8) -> (u8, u8);

    /// Returns the value of the register at the given index.
    fn get_register(&self, index: P::NumberOfRegisters) -> u8;

    /// Clears the registers to zero.
    fn clear_registers(&mut self);
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;

    fn test_register_iterator<P: Precision, B: Bits, R: Registers<P, B>>() {
        let mut registers = R::default();
        let collected_values = registers.iter_registers().collect::<Vec<_>>();
        assert_eq!(
            P::NumberOfRegisters::try_from_u64(collected_values.len() as u64).unwrap(),
            P::NUMBER_OF_REGISTERS
        );
        // All the values should be zeroed.
        assert!(collected_values.iter().all(|&value| value == 0));
        // We check that each collected value is identical to what we obtain using the get method.
        assert!(collected_values
            .iter()
            .enumerate()
            .all(|(index, &value)| value
                == registers
                    .get_register(P::NumberOfRegisters::try_from_u64(index as u64).unwrap())));

        // We check that, given all registers are currently zeroed, when we set them to the maximum value
        // we get always returned a value and that value is equal to zero.
        for index in 0_u64..P::NUMBER_OF_REGISTERS.into() {
            let index = P::NumberOfRegisters::try_from_u64(index).unwrap();
            let max_value = u8::try_from(<u64 as RegisterWord<B>>::LOWER_REGISTER_MASK).unwrap();
            let old_value = registers.set_greater(index, max_value);
            assert_eq!(old_value, (0, max_value));
            // If we try to do it again, we should receive the new value
            let old_value = registers.set_greater(index, max_value);
            assert_eq!(old_value, (max_value, max_value));
        }
    }

    fn test_registers_self_consistency<P: Precision + ArrayRegister<B>, B: Bits>() {
        let iterations = 50;
        let mut random_state = splitmix64(324564567865354);
        let mut index_random_state = splitmix64(324566754567865354);
        let mut packed_registers = <P as ArrayRegister<B>>::Packed::default();
        let mut array_registers = <P as ArrayRegister<B>>::Array::default();
        let mut reference = vec![0_u8; 1 << P::EXPONENT];

        let maximal_register_value = <u64 as RegisterWord<B>>::LOWER_REGISTER_MASK;

        // We check that the arrays are full of zeros.
        assert!(packed_registers.iter_registers().all(|value| value == 0));
        assert!(array_registers.iter_registers().all(|value| value == 0));

        // We check that if we call get_register on all index we get zeros.
        for index in 0..(1 << P::EXPONENT) {
            let index = P::NumberOfRegisters::try_from_u64(index).unwrap();
            assert_eq!(packed_registers.get_register(index), 0);
            assert_eq!(array_registers.get_register(index), 0);
        }

        for i in 0..iterations {
            random_state = splitmix64(random_state);
            index_random_state = splitmix64(index_random_state);

            for (j, (index, value)) in
                iter_random_values(1_000_000, Some(1 << P::EXPONENT), random_state)
                    .zip(iter_random_values(
                        1_000_000,
                        Some(maximal_register_value.into()),
                        random_state,
                    ))
                    .enumerate()
            {
                let index_usize = index as usize;
                let index: P::NumberOfRegisters =
                    P::NumberOfRegisters::try_from_u64(index).unwrap();
                let value: u8 = u8::try_from(value).unwrap();

                // We expect that the values at index are the same in both packed and array registers.
                assert_eq!(
                    packed_registers.get_register(index),
                    array_registers.get_register(index),
                    "Registers are dis-aligned at index {}, at inner iteration {} and outer iteration {}. Expected value {}.",
                    index, j, i, reference[index_usize]
                );

                // We retrieve the values immediately before and after the change and we verify that they
                // have not been changed.
                let prev_value_packed = (index_usize > 0)
                    .then(|| packed_registers.get_register(index - P::NumberOfRegisters::ONE));
                let next_value_packed = (index_usize < (1 << P::EXPONENT))
                    .then(|| packed_registers.get_register(index + P::NumberOfRegisters::ONE));

                let prev_value_array = (index_usize > 0)
                    .then(|| array_registers.get_register(index - P::NumberOfRegisters::ONE));
                let next_value_array = (index_usize < (1 << P::EXPONENT))
                    .then(|| array_registers.get_register(index + P::NumberOfRegisters::ONE));

                // We set the values in all of the registers, and we check that the values are consistent.
                let old_value_packed = packed_registers.set_greater(index, value);
                let old_value_array = array_registers.set_greater(index, value);
                reference[index_usize] = reference[index_usize].max(value);

                assert_eq!(
                    old_value_packed, old_value_array,
                    "Failed while trying to set index {} with value {}.",
                    index, value
                );

                // Then, we check that the before and after values have not been changed.
                if let Some(old_value) = prev_value_array {
                    assert_eq!(
                        old_value,
                        array_registers.get_register(index - P::NumberOfRegisters::ONE),
                        "Failed while trying to set index {} with value {} - Reference value {}.",
                        index,
                        value,
                        reference[index_usize - 1]
                    );
                }
                if let Some(old_value) = next_value_array {
                    assert_eq!(
                        old_value,
                        array_registers.get_register(index + P::NumberOfRegisters::ONE),
                        "Failed while trying to set index {} with value {} - Reference value {}.",
                        index,
                        value,
                        reference[index_usize + 1]
                    );
                }
                if let Some(old_value) = prev_value_packed {
                    assert_eq!(
                        old_value,
                        packed_registers.get_register(index - P::NumberOfRegisters::ONE),
                        "Failed while trying to set index {} with value {} - Reference value {}.",
                        index,
                        value,
                        reference[index_usize - 1]
                    );
                }
                if let Some(old_value) = next_value_packed {
                    assert_eq!(
                        old_value,
                        packed_registers.get_register(index + P::NumberOfRegisters::ONE),
                        "Failed while trying to set index {} with value {} - Reference value {}.",
                        index,
                        value,
                        reference[index_usize + 1]
                    );
                }

                let largest_value = old_value_array.1;

                assert_eq!(
                    packed_registers.get_register(index),
                    array_registers.get_register(index)
                );

                // We check that the values are consistent with the get method.
                assert_eq!(array_registers.get_register(index), largest_value);
                assert_eq!(packed_registers.get_register(index), largest_value);
            }

            // We check that the iterator works as expected.
            for (index, value) in array_registers.iter_registers().enumerate() {
                assert_eq!(
                    value,
                    array_registers
                        .get_register(P::NumberOfRegisters::try_from_u64(index as u64).unwrap()),
                    "Failed at index {}.",
                    index
                );
                assert_eq!(
                    value, reference[index],
                    "Failed at index {}. Expected value {}.",
                    index, reference[index]
                );
            }
            for (index, value) in packed_registers.iter_registers().enumerate() {
                assert_eq!(
                    value,
                    packed_registers
                        .get_register(P::NumberOfRegisters::try_from_u64(index as u64).unwrap()),
                    "Failed at index {}.",
                    index
                );
                assert_eq!(
                    value, reference[index],
                    "Failed at index {}. Expected value {}.",
                    index, reference[index]
                );
            }

            for index in 0..(1 << P::EXPONENT) {
                let index = P::NumberOfRegisters::try_from_u64(index).unwrap();

                // We check that the values are consistent with the get method.
                assert_eq!(
                    packed_registers.get_register(index),
                    array_registers.get_register(index)
                );
            }
        }
    }

    macro_rules! test_register_iterator {
        ($precision:ty, $($bits:ty),*) => {
            $(
                paste::item! {
                    #[test]
                    #[cfg(feature = "std")]
                    fn [< test_registers_self_consistency_ $precision:lower _and_ $bits:lower _bits >]() {
                        test_registers_self_consistency::<$precision, $bits>();
                    }

                    #[test]
                    #[cfg(feature = "std")]
                    fn [< test_array_register_iterator_ $precision:lower _and_ $bits:lower _bits >]() {
                        test_register_iterator::<$precision, $bits, <$precision as ArrayRegister<$bits>>::Array>();
                    }

                    #[test]
                    #[cfg(feature = "std")]
                    fn [< test_packed_array_register_iterator_ $precision:lower _and_ $bits:lower _bits >]() {
                        test_register_iterator::<$precision, $bits, <$precision as ArrayRegister<$bits>>::Packed>();
                    }
                }
            )*
        };
    }

    macro_rules! test_register_iterators_by_precision {
        ($($precision:ty),*) => {
            $(
                test_register_iterator!($precision, Bits1, Bits2, Bits3, Bits4, Bits5, Bits6, Bits7, Bits8);
            )*
        };
    }

    #[cfg(feature = "low_precisions")]
    test_register_iterators_by_precision!(
        Precision4,
        Precision5,
        Precision6,
        Precision7,
        Precision8,
        Precision9,
        Precision10
    );

    #[cfg(feature = "medium_precisions")]
    test_register_iterators_by_precision!(
        Precision11,
        Precision12,
        Precision13,
        Precision14,
        Precision15,
        Precision16
    );

    #[cfg(feature = "high_precisions")]
    test_register_iterators_by_precision!(Precision17, Precision18);
}
