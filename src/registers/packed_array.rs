//! Packed array register implementation for HLL.
//!
//! Provides the [`Registers`] trait implementation for [`Packed`]
//! and the [`PackedRegister`] association trait.
//!
//! The [`Packed`] struct and [`PackedIter`] live in the shared crate [`sketching_core`].

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use super::Registers;
use super::{Bits, Precision};
use crate::prelude::{
    Bits4, Bits5, Bits6, Precision10, Precision11, Precision12, Precision13, Precision14,
    Precision15, Precision16, Precision17, Precision18, Precision4, Precision5, Precision6,
    Precision7, Precision8, Precision9,
};
use crate::VariableWord;
use core::fmt::Debug;
use core::hash::Hash;
use sketching_core::{
    extract_bridge_value_from_word, extract_value_from_word, insert_bridge_value_into_word,
    insert_value_into_word, split_packed_index, Packed, PackedShape, Words,
};

/// Trait marker to associate a specific register array with a combination of precision and bits.
/// Extends [`PackedShape`], which fixes the underlying `[u64; N]` word storage; this trait adds
/// the array-backed and (optional) `Vec`-backed `Registers` associations that HLL needs.
pub trait PackedRegister<B: Bits>: PackedShape<B> {
    /// The type of the packed array register.
    type Array: Registers<Self, B>;
    #[cfg(feature = "alloc")]
    /// The type of the packed vector register.
    type Vec: Registers<Self, B>;
}

pub trait IncreaseCapacity {
    fn increase_capacity(&mut self, maximal_size: usize);
}

impl<const N: usize> IncreaseCapacity for Words<N> {
    #[inline]
    fn increase_capacity(&mut self, _maximal_size: usize) {
        unimplemented!("The increase_capacity method is not implemented for Words<N>");
    }
}

#[cfg(feature = "alloc")]
impl IncreaseCapacity for Vec<u64> {
    #[inline]
    fn increase_capacity(&mut self, maximal_size: usize) {
        let new_length = if self.is_empty() { 1 } else { self.len() * 2 }.min(maximal_size);
        self.resize(new_length, 0);
    }
}

impl<
        W: Hash + IncreaseCapacity + Clone + Eq + Send + Sync + Debug + AsRef<[u64]> + AsMut<[u64]>,
        P: Precision,
        B: Bits,
    > Registers<P, B> for Packed<W, B>
where
    Self: Default,
    B: VariableWord<Word = u8>,
{
    type Iter<'words>
        = sketching_core::PackedIter<&'words Self, 1>
    where
        Self: 'words;
    type IterZipped<'words>
        = sketching_core::PackedIter<&'words Self, 2>
    where
        Self: 'words;

    #[inline]
    fn increase_capacity(&mut self) {
        // No-op for array-backed; Vec-backed handles via IncreaseCapacity.
    }

    #[inline]
    fn iter_registers(&self) -> Self::Iter<'_> {
        self.iter_values(1 << P::EXPONENT)
    }

    #[inline]
    fn iter_registers_zipped<'words>(
        &'words self,
        other: &'words Self,
    ) -> Self::IterZipped<'words> {
        self.iter_values_zipped(other, 1 << P::EXPONENT)
    }

    #[inline]
    fn apply_to_registers<F>(&mut self, register_function: F)
    where
        F: FnMut(u8) -> u8,
    {
        self.apply(register_function, 1 << <P as Precision>::EXPONENT);
    }

    #[inline]
    #[allow(unsafe_code)]
    fn set_greater(&mut self, index: usize, new_register: u8) -> (u8, u8) {
        let (word_index, relative_value_offset) = split_packed_index::<B>(index);

        if <Packed<W, B>>::is_bridge_offset(relative_value_offset) {
            let words: &mut [u64] = AsMut::<[u64]>::as_mut(self);
            let (low, high) = unsafe { words.split_at_mut_unchecked(word_index + 1) };
            let low = unsafe { low.get_unchecked_mut(word_index) };
            let high = unsafe { high.get_unchecked_mut(0) };
            let value = extract_bridge_value_from_word::<B>(*low, *high, relative_value_offset);
            let new_value = core::cmp::max(value, new_register);
            insert_bridge_value_into_word::<B>(low, high, relative_value_offset, new_value.into());

            debug_assert_eq!(self.get(index), new_value);

            (value, new_value)
        } else {
            let words: &[u64] = AsRef::<[u64]>::as_ref(self);
            let value = extract_value_from_word::<B>(
                unsafe { *words.get_unchecked(word_index) },
                relative_value_offset,
            );
            let new_value = core::cmp::max(value, new_register);
            let words: &mut [u64] = AsMut::<[u64]>::as_mut(self);
            insert_value_into_word::<B>(
                unsafe { words.get_unchecked_mut(word_index) },
                relative_value_offset,
                new_value.into(),
            );

            debug_assert_eq!(self.get(index), new_value);

            (value, new_value)
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn set(&mut self, index: usize, new_register: u8) {
        let (word_index, relative_value_offset) = split_packed_index::<B>(index);

        if <Packed<W, B>>::is_bridge_offset(relative_value_offset) {
            let words: &mut [u64] = AsMut::<[u64]>::as_mut(self);
            let (low, high) = unsafe { words.split_at_mut_unchecked(word_index + 1) };
            let low = unsafe { low.get_unchecked_mut(word_index) };
            let high = unsafe { high.get_unchecked_mut(0) };
            insert_bridge_value_into_word::<B>(
                low,
                high,
                relative_value_offset,
                new_register.into(),
            );

            debug_assert_eq!(self.get(index), new_register);
        } else {
            let words: &mut [u64] = AsMut::<[u64]>::as_mut(self);
            insert_value_into_word::<B>(
                unsafe { words.get_unchecked_mut(word_index) },
                relative_value_offset,
                new_register.into(),
            );

            debug_assert_eq!(self.get(index), new_register);
        }
    }

    #[inline]
    fn get_register(&self, index: usize) -> u8 {
        self.get(index)
    }

    #[inline]
    fn clear_registers(&mut self) {
        self.clear();
    }

    #[inline]
    fn bitsize() -> usize {
        64 * ((1 << P::EXPONENT) * B::NUMBER_OF_BITS_USIZE).div_ceil(64)
    }
}

/// Implement the packed array registers for a specific combination of precision and bits.
macro_rules! impl_packed_array_register_for_precision_and_bits {
    ($exponent: expr, $($bits: expr),*) => {
        $(
            paste::paste! {
                impl PackedRegister<[<Bits $bits>]> for [<Precision $exponent>] {
                    type Array = Packed<<[<Precision $exponent>] as PackedShape<[<Bits $bits>]>>::Words, [<Bits $bits>]>;
                    #[cfg(feature = "alloc")]
                    type Vec = Packed<Vec<u64>, [<Bits $bits>]>;
                }
            }
        )*
    };
}

/// Implement the packed array registers for all the possible combinations of precision and bits.
macro_rules! impl_registers_for_precisions {
    ($($exponent: expr),*) => {
        $(
            impl_packed_array_register_for_precision_and_bits!($exponent, 4, 5, 6);
        )*
    };
}

impl_registers_for_precisions!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);

#[cfg(test)]
mod test_extract_bridge_value_from_word {
    use super::*;
    use crate::prelude::*;
    use hyperloglog_derive::test_variable_words;

    #[test_variable_words]
    #[allow(unsafe_code)]
    fn test_extract_bridge_value_from_word<V: VariableWord>() {
        let mut lower_word = 0_u64;
        let mut upper_word = 0_u64;
        for value in iter_random_values::<V>(V::MASK.min(200), None, None) {
            for offset in (65_u8 - V::NUMBER_OF_BITS)..64_u8 {
                insert_bridge_value_into_word::<V>(
                    &mut lower_word,
                    &mut upper_word,
                    offset,
                    value.into(),
                );
                assert_eq!(
                    extract_bridge_value_from_word::<V>(lower_word, upper_word, offset),
                    value,
                    "The value extracted from the word {lower_word} at offset {offset} should be equal to the value {value}"
                );
            }
        }
    }
}

#[cfg(test)]
mod test_extract_value_from_word {
    use super::*;
    use crate::prelude::*;
    use hyperloglog_derive::test_variable_words;

    #[test_variable_words]
    #[allow(unsafe_code)]
    fn test_extract_value_from_word<V: VariableWord>() {
        let mut word = 0_u64;
        for value in iter_random_values::<V>(V::MASK.min(200), None, None) {
            for offset in 0_u8..=(64_u8 - V::NUMBER_OF_BITS) {
                insert_value_into_word::<V>(&mut word, offset, value.into());
                assert_eq!(
                    extract_value_from_word::<V>(word, offset),
                    value,
                    "The value extracted from the word {word} at offset {offset} should be equal to the value {value}"
                );
            }
        }
    }
}

#[cfg(test)]
mod test_split_index {
    use super::*;
    use crate::prelude::*;
    use hyperloglog_derive::test_variable_words;

    #[test_variable_words]
    fn test_split_packed_index<V: VariableWord>() {
        let minimum_index = 0_usize;
        let maximum_index = 1_usize << 18;
        for index in minimum_index..maximum_index {
            let expected_word_index = (usize::from(V::NUMBER_OF_BITS) * index) / 64;
            let expected_relative_register_offset = (usize::from(V::NUMBER_OF_BITS) * index) % 64;
            let (word_index, relative_register_offset) = split_packed_index::<V>(index);
            assert_eq!(
                word_index, expected_word_index,
                "The word index {word_index} should be equal to the word index {expected_word_index}"
            );
            assert_eq!(
                relative_register_offset,
                expected_relative_register_offset as u8,
                "The relative register offset {relative_register_offset} should be equal to the relative register offset {expected_relative_register_offset}"
            );
        }
    }
}
