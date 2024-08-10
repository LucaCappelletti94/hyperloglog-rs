//! Submodule providing the trait registers

use core::fmt::Debug;

use crate::prelude::*;
use crate::utils::*;
mod array;

pub use array::ArrayRegister;

pub trait Registers<P: Precision, B: Bits>: Eq + PartialEq + Clone + Debug + Send + Sync {
    type Iter<'a>: Iterator<Item = u32>
    where
        Self: 'a;

    type IterZipped<'a>: Iterator<Item = (u32, u32)>
    where
        Self: 'a;

    // type IterMax<'a> : Iterator<Item = u32>
    //     where
    //         Self: 'a;

    fn iter_registers(&self) -> Self::Iter<'_>;

    fn iter_registers_zipped<'a>(&'a self, other: &'a Self) -> Self::IterZipped<'a>;

    // fn iter_registers_max<'a>(&'a self, other: &'a Self) -> Self::IterMax<'a>;

    fn get_harmonic_sum_and_zeros<F: FloatNumber>(
        &self,
        other: &Self,
    ) -> (F, P::NumberOfZeros)
    where
        P: PrecisionConstants<F>;

    fn apply<F>(&mut self, f: F)
    where
        F: FnMut(u32) -> u32;

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
    /// # Safety
    /// The caller must ensure that the index is within the bounds of the data structure.
    unsafe fn set_greater(&mut self, index: usize, value: u32) -> (u32, u32);

    /// Returns the value of the register at the given index.
    fn get_register(&self, index: usize) -> u32;

    /// Clears the registers to zero.
    fn clear(&mut self);
}
