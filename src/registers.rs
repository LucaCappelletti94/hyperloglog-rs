//! Submodule providing the trait registers

use core::fmt::Debug;

use crate::prelude::*;
use crate::utils::*;
mod array;
mod packed_array;
mod vector;

pub use array::ArrayRegister;
pub use packed_array::{PackedArray, PackedArrayRegister};

/// Trait marker for the registers.
pub trait Registers<P: Precision, B: Bits>:
    Eq + PartialEq + Clone + Debug + Send + Sync + Named
{
    /// Iterator over the registers.
    type Iter<'a>: Iterator<Item = u32>
    where
        Self: 'a;

    /// Iterator over the registers zipped with another set of registers.
    type IterZipped<'a>: Iterator<Item = (u32, u32)>
    where
        Self: 'a;

    /// Returns an iterator over the registers.
    fn iter_registers(&self) -> Self::Iter<'_>;

    /// Returns an iterator over the registers zipped with another set of registers.
    fn iter_registers_zipped<'a>(&'a self, other: &'a Self) -> Self::IterZipped<'a>;

    /// Returns the harmonic sum of the maximum value of the registers and the number of zero registers.
    fn get_harmonic_sum_and_zeros<F: Float>(&self, other: &Self) -> (F, P::NumberOfZeros)
    where
        P: PrecisionConstants<F>;

    /// Applies a function to each register.
    fn apply<F>(&mut self, f: F)
    where
        F: FnMut(u32) -> u32;

    /// Returns a new instance of the registers with all the values set to zero.
    fn zeroed() -> Self;

    /// Updates the register at the given index with the given value,
    /// if the value is greater than the current value in the register.
    ///
    /// # Arguments
    /// * `index` - The index of the register to be updated.
    /// * `value` - The value to be set in the register.
    ///
    /// # Returns
    /// The previous value of the register, and the larger of the two values.
    ///
    fn set_greater(&mut self, index: usize, value: u32) -> (u32, u32);

    /// Returns the value of the register at the given index.
    fn get_register(&self, index: usize) -> u32;

    /// Clears the registers to zero.
    fn clear(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_register_iterator<P: Precision, B: Bits, R: Registers<P, B>>() {
        let mut registers = R::zeroed();
        let collected_values = registers.iter_registers().collect::<Vec<_>>();
        assert_eq!(collected_values.len(), P::NUMBER_OF_REGISTERS);
        // All the values should be zeroed.
        assert!(collected_values.iter().all(|&value| value == 0));
        // We check that each collected value is identical to what we obtain using the get method.
        assert!(collected_values
            .iter()
            .enumerate()
            .all(|(index, &value)| value == registers.get_register(index)));

        // We check that, given all registers are currently zeroed, when we set them to the maximum value
        // we get always returned a value and that value is equal to zero.
        for index in 0..P::NUMBER_OF_REGISTERS {
            let old_value = 
                registers.set_greater(index, <u32 as RegisterWord<B>>::LOWER_REGISTER_MASK);
            assert_eq!(
                old_value,
                (0, <u32 as RegisterWord<B>>::LOWER_REGISTER_MASK)
            );
            // If we try to do it again, we should receive the new value
            let old_value = registers.set_greater(index, <u32 as RegisterWord<B>>::LOWER_REGISTER_MASK);
            assert_eq!(
                old_value,
                (
                    <u32 as RegisterWord<B>>::LOWER_REGISTER_MASK,
                    <u32 as RegisterWord<B>>::LOWER_REGISTER_MASK
                )
            );
        }
    }

    fn test_registers_self_consistency<
        P: Precision + ArrayRegister<B> + PackedArrayRegister<B>,
        B: Bits,
    >()
    where
        Vec<u64>: Registers<P, B>,
    {
        let iterations = 50;
        let mut random_state = splitmix64(324564567865354);
        let mut index_random_state = splitmix64(324566754567865354);
        let mut vector_registers = <Vec<u64> as Registers<P, B>>::zeroed();
        let mut packed_registers = <P as PackedArrayRegister<B>>::PackedArrayRegister::zeroed();
        let mut array_registers = <P as ArrayRegister<B>>::ArrayRegister::zeroed();

        let maximal_register_value = <u64 as RegisterWord<B>>::LOWER_REGISTER_MASK;
        let number_of_registers = P::NUMBER_OF_REGISTERS;

        for _ in 0..iterations {
            random_state = splitmix64(random_state);
            index_random_state = splitmix64(index_random_state);

            for (index, value) in
                iter_random_values(1_000_000, Some(number_of_registers), random_state).zip(
                    iter_random_values(
                        1_000_000,
                        Some(maximal_register_value as usize),
                        random_state,
                    ),
                )
            {
                // We set the values in all of the registers, and we check that the values are consistent.
                let old_value_vector =
                    vector_registers.set_greater(index as usize, value as u32);
                let old_value_packed =
                    packed_registers.set_greater(index as usize, value as u32);
                let old_value_array =
                    array_registers.set_greater(index as usize, value as u32);
                assert_eq!(old_value_vector, old_value_packed);
                assert_eq!(old_value_vector, old_value_array);
                assert_eq!(old_value_packed, old_value_array);

                let largest_value = old_value_vector.1;

                // We check that the values are consistent with the get method.
                assert_eq!(
                    array_registers.get_register(index as usize),
                    largest_value as u32
                );
                assert_eq!(
                    vector_registers.get_register(index as usize),
                    largest_value as u32
                );
                assert_eq!(
                    packed_registers.get_register(index as usize),
                    largest_value as u32
                );
            }

            for index in 0..number_of_registers {
                // We check that the values are consistent with the get method.
                assert_eq!(
                    vector_registers.get_register(index),
                    packed_registers.get_register(index)
                );
                assert_eq!(
                    vector_registers.get_register(index),
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
                    fn [< test_registers_self_consistency_ $precision:lower _and_ $bits:lower _bits >]() {
                        test_registers_self_consistency::<$precision, $bits>();
                    }

                    #[test]
                    fn [< test_array_register_iterator_ $precision:lower _and_ $bits:lower _bits >]() {
                        test_register_iterator::<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister>();
                    }

                    #[test]
                    fn [< test_packed_array_register_iterator_ $precision:lower _and_ $bits:lower _bits >]() {
                        test_register_iterator::<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister>();
                    }

                    #[test]
                    fn [< test_vector_register_iterator_ $precision:lower _and_ $bits:lower _bits >]() {
                        test_register_iterator::<$precision, $bits, Vec<u64>>();
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
