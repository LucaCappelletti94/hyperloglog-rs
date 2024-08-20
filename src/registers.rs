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
