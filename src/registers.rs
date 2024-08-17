//! Submodule providing the trait registers

use core::fmt::Debug;

use crate::prelude::*;
use crate::utils::{FloatOps, Number, One, Words, Zero};
mod array;
mod packed_array;

pub use array::{ArrayRegister, ArrayRegisters};
pub use packed_array::{PackedArray, PackedArrayRegister, PackedArrayRegisters};

/// Trait for a register word.
pub(super) trait RegisterWord<B: Bits> {
    /// The mask for the lower register.
    const LOWER_REGISTER_MASK: Self;
    /// The number of bits in the word.
    const NUMBER_OF_BITS: u8;
    /// The number of registers in the word.
    const NUMBER_OF_REGISTERS_IN_WORD: u8 = Self::NUMBER_OF_BITS / B::NUMBER_OF_BITS;
}

impl<B: Bits> RegisterWord<B> for u64 {
    const LOWER_REGISTER_MASK: Self = (1 << B::NUMBER_OF_BITS) - 1;
    const NUMBER_OF_BITS: u8 = 64;
}

/// Extracts the register from one or more words at the given offset.
///
/// # Arguments
/// * `word` - The word array from which the register is to be extracted.
/// * `offset` - The offset at which the register starts.
fn extract_register_from_word<B: Bits, const N: usize>(word: [u64; N], offset: u8) -> [u8; N]
where
    u64: RegisterWord<B>,
{
    debug_assert!(
        offset + B::NUMBER_OF_BITS <= u64::NUMBER_OF_BITS,
        "The offset is too large."
    );
    let mut registers = [0_u8; N];
    for i in 0..N {
        registers[i] = u8::try_from((word[i] >> offset) & u64::LOWER_REGISTER_MASK).unwrap();
    }
    registers
}

/// Trait marker for the registers.
pub trait Registers<P: Precision, B: Bits>: Eq + PartialEq + Clone + Debug + Send + Sync {
    /// Iterator over the registers.
    type Iter<'register>: Iterator<Item = u8>
    where
        Self: 'register;

    /// Iterator over the registers zipped with another set of registers.
    type IterZipped<'registers>: Iterator<Item = (u8, u8)>
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
    fn apply<F>(&mut self, f: F)
    where
        F: FnMut(u8) -> u8;

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
    fn set_greater(&mut self, index: P::NumberOfRegisters, value: u8) -> (u8, u8);

    /// Returns the value of the register at the given index.
    fn get_register(&self, index: P::NumberOfRegisters) -> u8;

    /// Clears the registers to zero.
    fn clear(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "std")]
    fn test_register_iterator<P: Precision, B: Bits, R: Registers<P, B>>() {
        let mut registers = R::zeroed();
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

    #[cfg(feature = "std")]
    fn test_registers_self_consistency<
        P: Precision + ArrayRegister<B> + PackedArrayRegister<B>,
        B: Bits,
    >() {
        let iterations = 50;
        let mut random_state = splitmix64(324564567865354);
        let mut index_random_state = splitmix64(324566754567865354);
        let mut packed_registers = <P as PackedArrayRegister<B>>::PackedArrayRegister::zeroed();
        let mut array_registers = <P as ArrayRegister<B>>::ArrayRegister::zeroed();

        let maximal_register_value = <u64 as RegisterWord<B>>::LOWER_REGISTER_MASK;

        for _ in 0..iterations {
            random_state = splitmix64(random_state);
            index_random_state = splitmix64(index_random_state);

            for (index, value) in
                iter_random_values(1_000_000, Some(1 << P::EXPONENT), random_state).zip(
                    iter_random_values(
                        1_000_000,
                        Some(maximal_register_value.into()),
                        random_state,
                    ),
                )
            {
                let index: P::NumberOfRegisters =
                    P::NumberOfRegisters::try_from_u64(index).unwrap();
                let value: u8 = u8::try_from(value).unwrap();

                // We set the values in all of the registers, and we check that the values are consistent.
                let old_value_packed = packed_registers.set_greater(index, value);
                let old_value_array = array_registers.set_greater(index, value);
                assert_eq!(old_value_packed, old_value_array);

                let largest_value = old_value_array.1;

                // We check that the values are consistent with the get method.
                assert_eq!(array_registers.get_register(index), largest_value);
                assert_eq!(packed_registers.get_register(index), largest_value);
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
                        test_register_iterator::<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister>();
                    }

                    #[test]
                    #[cfg(feature = "std")]
                    fn [< test_packed_array_register_iterator_ $precision:lower _and_ $bits:lower _bits >]() {
                        test_register_iterator::<$precision, $bits, <$precision as PackedArrayRegister<$bits>>::PackedArrayRegister>();
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
