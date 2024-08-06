//! Submodule providing the trait registers

use core::fmt::Debug;

use crate::prelude::*;
use crate::utils::*;
mod array;

pub use array::ArrayRegister;

pub trait Registers<P: Precision, B: Bits>: Eq + PartialEq + Clone + Debug {
    type Iter<'a>: Iterator<Item = u32>
    where
        Self: 'a;

    fn iter_registers(&self) -> Self::Iter<'_>;

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
    /// The previous value of the register, if the new value is greater than the current value.
    ///
    /// # Safety
    /// The caller must ensure that the index is within the bounds of the data structure.
    unsafe fn set_greater(&mut self, index: usize, value: u32) -> Option<u32>;

    /// Returns the value of the register at the given index.
    fn get_register(&self, index: usize) -> u32;
}
