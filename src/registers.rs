//! Submodule providing the trait registers

use core::fmt::Debug;
use core::hash::Hash;

use crate::prelude::*;
use crate::utils::{FloatOps, Zero};
mod packed_array;

pub use packed_array::{Packed, PackedRegister};

/// Trait marker for the registers.
pub trait Registers<P: Precision, B: Bits>:
    Hash + Eq + PartialEq + Clone + Debug + Send + Sync + Default + AsMut<[u8]> + AsRef<[u8]>
{
    /// Iterator over the registers.
    type Iter<'register>: ExactSizeIterator<Item = u8>
    where
        Self: 'register;

    /// Iterator over the registers zipped with another set of registers.
    type IterZipped<'registers>: ExactSizeIterator<Item = [u8; 2]>
    where
        Self: 'registers;

    /// Doubles or saturates the size of the registers.
    fn increase_capacity(&mut self);

    /// Returns an iterator over the registers.
    fn iter_registers(&self) -> Self::Iter<'_>;

    /// Returns a random register.
    fn random(&self, random_state: u64) -> (usize, u8) {
        let index = xorshift64(random_state) as usize % (1 << P::EXPONENT);
        (index, self.get_register(index))
    }
    
    /// Returns the minimum register, including zero.
    fn min(&self) -> (usize, u8) {
        let mut min_register = u8::MAX;
        let mut min_index = 0;

        for (index, register) in self.iter_registers().enumerate() {
            if register == 0 {
                // If the register is 0, we can return immediately
                // as this is necessatily one of the smallest registers.
                return (index, 0);
            }
            if register < min_register {
                min_register = register;
                min_index = index;
            }
        }

        (min_index, min_register)
    }

    /// Returns the minimum register, excluding zero.
    fn min_non_zero(&self) -> Option<(usize, u8)> {
        let mut min_register = u8::MAX;
        let mut min_index = 0;
        let mut found = false;

        for (index, register) in self.iter_registers().enumerate() {
            if register == 0 {
                continue;
            }
            if register == 1 {
                // If the register is 1, we can return immediately
                // as this is necessatily one of the smallest registers.
                return Some((index, 1));
            }
            if register < min_register {
                min_register = register;
                min_index = index;
                found = true;
            }
        }

        if found {
            Some((min_index, min_register))
        } else {
            None
        }
    }

    /// Returns the maximum register.
    fn max(&self) -> (usize, u8) {
        let mut max_register = 0;
        let mut max_index = 0;

        for (index, register) in self.iter_registers().enumerate() {
            if register > max_register {
                max_register = register;
                max_index = index;
            }
        }

        (max_index, max_register)
    }

    /// Returns an iterator over the registers zipped with another set of registers.
    fn iter_registers_zipped<'registers>(
        &'registers self,
        other: &'registers Self,
    ) -> Self::IterZipped<'registers>;

    /// Returns the harmonic sum of the maximum value of the registers and the number of zero registers.
    fn get_union_harmonic_sum(&self, other: &Self) -> f64 {
        let mut harmonic_sum = f64::ZERO;

        for [left, right] in Self::iter_registers_zipped(self, other) {
            harmonic_sum += f64::integer_exp2_minus(core::cmp::max(left, right));
        }

        harmonic_sum
    }

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
    fn set_greater(&mut self, index: usize, value: u8) -> (u8, u8);

    /// Updates the register at the given index with the given value.
    ///
    /// # Arguments
    /// * `index` - The index of the register to be updated.
    /// * `value` - The value to be set in the register.
    ///
    fn set(&mut self, index: usize, value: u8);

    /// Returns the value of the register at the given index.
    fn get_register(&self, index: usize) -> u8;

    /// Clears the registers to zero.
    fn clear_registers(&mut self);

    /// Returns the struct bitsize.
    fn bitsize() -> usize;
}
