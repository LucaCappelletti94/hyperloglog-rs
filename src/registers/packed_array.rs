//! Packed array for registers.
//!
//! The principal difference between this implementation and the one in either `array.rs` or
//! `vector.rs` is that this implementation uses a packed array to store the registers. This means
//! that while in the other implementations we store as many registers as they fit in a word and we
//! discard the padding bits (e.g. when using a 64-bit word and a 6-bit register, we store 10 registers and
//! discard 4 bits), in this implementation we store the registers in a packed array, so we don't discard
//! any bits. This will tendentially make the packed array implementation more memory efficient, but
//! it will also make it slower, as we need to perform more operations to extract the registers from the
//! packed array, expecially in the case of bridge registers, i.e. registers that span two words.

use super::{
    Bits, Bits1, Bits2, Bits3, Bits4, Bits5, Bits6, Bits7, Bits8, FloatOps, Matrix, Precision,
    Registers, Zero,
};
use crate::utils::PositiveInteger;
use crate::utils::VariableWord;
use core::fmt::Debug;
use core::marker::PhantomData;

#[cfg(feature = "std")]
use crate::utils::Named;

#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

#[cfg(feature = "precision_10")]
use crate::prelude::Precision10;
#[cfg(feature = "precision_11")]
use crate::prelude::Precision11;
#[cfg(feature = "precision_12")]
use crate::prelude::Precision12;
#[cfg(feature = "precision_13")]
use crate::prelude::Precision13;
#[cfg(feature = "precision_14")]
use crate::prelude::Precision14;
#[cfg(feature = "precision_15")]
use crate::prelude::Precision15;
#[cfg(feature = "precision_16")]
use crate::prelude::Precision16;
#[cfg(feature = "precision_17")]
use crate::prelude::Precision17;
#[cfg(feature = "precision_18")]
use crate::prelude::Precision18;
#[cfg(feature = "precision_4")]
use crate::prelude::Precision4;
#[cfg(feature = "precision_5")]
use crate::prelude::Precision5;
#[cfg(feature = "precision_6")]
use crate::prelude::Precision6;
#[cfg(feature = "precision_7")]
use crate::prelude::Precision7;
#[cfg(feature = "precision_8")]
use crate::prelude::Precision8;
#[cfg(feature = "precision_9")]
use crate::prelude::Precision9;

#[allow(unsafe_code)]
#[inline]
/// Extracts the register from one or more words at the given offset.
///
/// # Arguments
/// * `word` - The word array from which the register is to be extracted.
/// * `offset` - The offset (from the right) at which the register starts.
///
/// # Implementative details
/// We store the values starting from the left-side of the word, so the offset is the number of bits
/// from the right side of the word at which the register starts. We then shift the word to the right
/// by the offset and apply a mask to extract the register.
///
/// # Safety
/// This method uses an unsafe conversion from `u64` to `V::Word`, as we do not check
/// whether the value extracted from the word is a valid value for the register type.
/// This is okay because we apply a mask to the value, and it is not possible for the
/// value we cast to be greater than the mask.
fn extract_value_from_word<V: VariableWord>(word: u64, offset: u8) -> V::Word {
    debug_assert!(
        offset + V::NUMBER_OF_BITS <= 64,
        "The offset ({offset} + {}) should be less than or equal to 64",
        V::NUMBER_OF_BITS,
    );
    unsafe { V::unchecked_from_u64((word >> (64 - V::NUMBER_OF_BITS - offset)) & V::MASK) }
}

#[inline]
/// We insert the value into the word at the given offset.
///
/// # Arguments
/// * `word` - The word in which the value is to be inserted.
/// * `offset` - The offset (from the right) at which the value is to be inserted.
/// * `value` - The value to be inserted.
fn insert_value_into_word<V: VariableWord>(word: &mut u64, offset: u8, value: u64) {
    debug_assert!(
        offset + V::NUMBER_OF_BITS <= 64,
        "The offset ({offset} + {}) should be less than or equal to 64",
        V::NUMBER_OF_BITS,
    );

    let flipped_offset = 64 - V::NUMBER_OF_BITS - offset;
    *word &= !(V::MASK << flipped_offset);
    *word |= value << flipped_offset;
}

#[inline]
/// Extracts the register from one or more words at the given offset.
///
/// # Arguments
/// * `word` - The word array from which the register is to be extracted.
/// * `offset` - The offset (from the right) at which the register starts.
fn extract_value_from_words<V: VariableWord, const N: usize>(
    words: [u64; N],
    offset: u8,
) -> [V::Word; N] {
    let mut values = [V::Word::ZERO; N];
    for i in 0..N {
        values[i] = extract_value_from_word::<V>(words[i], offset);
    }
    values
}

#[cfg(test)]
/// Test module for the [`extract_value_from_word`], [`extract_value_from_words`] and [`insert_value_into_word`] functions.
mod test_extract_value_from_word {
    use super::*;
    use crate::prelude::*;
    use hyperloglog_derive::test_variable_words;

    #[test_variable_words]
    /// Test the extraction of a V::Word from an u64 word.
    fn test_extract_value_from_word<V: VariableWord>() {
        let mut word = 0_u64;
        // We sample 250 random values of the word.
        for value in iter_random_values::<V>(V::MASK.min(200), None, None) {
            // For each word, we iter all possible offset values.
            for offset in 0_u8..=(64_u8 - V::NUMBER_OF_BITS) {
                insert_value_into_word::<V>(&mut word, offset, value.into());
                assert_eq!(
                    extract_value_from_word::<V>(word, offset),
                    value,
                    "The value extracted from the word {} at offset {} should be equal to the value {}",
                    word,
                    offset,
                    value
                );
            }
        }
    }
}

#[inline]
#[allow(unsafe_code)]
/// Returns the number of bits in the upper and lower value of a bridge value.
///
/// # Arguments
/// * `lower_word` - The lower word of the bridge value.
/// * `upper_word` - The upper word of the bridge value.
/// * `offset` - The offset (from the right) of the bridge value.
///
/// # Safety
/// * The method converts in an unchecked manner the value from a `u64` to a `V::Word`.
fn extract_bridge_value_from_word<V: VariableWord>(
    lower_word: u64,
    upper_word: u64,
    offset: u8,
) -> V::Word {
    debug_assert!(offset != 0, "Offset should be greater than 0");
    debug_assert!(offset != 64, "Offset should be less than 64");
    debug_assert!(
        offset > 64 - V::NUMBER_OF_BITS,
        "Offset should be greater than 64 - V::NUMBER_OF_BITS"
    );

    let number_of_high_bits_in_lower_value: u8 = 64 - offset;
    let number_of_low_bits_in_upper_value = V::NUMBER_OF_BITS - number_of_high_bits_in_lower_value;
    let higher_bits_mask = V::MASK >> number_of_low_bits_in_upper_value;

    let higher_bits = (lower_word & higher_bits_mask) << number_of_low_bits_in_upper_value;
    let lower_bits = upper_word >> (64 - number_of_low_bits_in_upper_value);

    let word = higher_bits | lower_bits;

    unsafe { V::unchecked_from_u64(word) }
}

/// Extracts a bridge register from a starting word and an ending word.
fn extract_bridge_value_from_words<V: VariableWord, const N: usize>(
    lower_word: [u64; N],
    upper_word: [u64; N],
    offset: u8,
) -> [V::Word; N] {
    let mut values = [V::Word::ZERO; N];
    for i in 0..N {
        values[i] = extract_bridge_value_from_word::<V>(lower_word[i], upper_word[i], offset);
    }
    values
}

fn insert_bridge_value_into_word<V: VariableWord>(
    lower_word: &mut u64,
    upper_word: &mut u64,
    offset: u8,
    value: u64,
) {
    debug_assert!(
        offset + V::NUMBER_OF_BITS > 64,
        "Offset + bits ({} + {}) should be greater than {}",
        offset,
        V::NUMBER_OF_BITS,
        64
    );

    debug_assert!(offset < 64, "Offset {} should be less than {}", offset, 64);

    let number_of_lower_bits = V::NUMBER_OF_BITS + offset - 64;
    let lower_bits_mask = (1 << number_of_lower_bits) - 1;
    let higher_bits_mask = V::MASK >> number_of_lower_bits;
    let lower_bits = value & lower_bits_mask;
    let higher_bits = value >> number_of_lower_bits;

    // First, we clear the bits that will be replaced by the new value.
    *lower_word &= !higher_bits_mask;
    // Then, we insert the lower part of the new value.
    *lower_word |= higher_bits;
    // We do the same for the upper part of the new value.
    *upper_word &= !(lower_bits_mask << (64 - number_of_lower_bits));
    *upper_word |= lower_bits << (64 - number_of_lower_bits);
}

#[cfg(test)]
/// Test module for the [`extract_bridge_value_from_word`], [`extract_bridge_value_from_words`] and [`insert_bridge_value_into_word`] functions.
mod test_extract_bridge_value_from_word {
    use super::*;
    use crate::prelude::*;
    use hyperloglog_derive::test_variable_words;

    #[test_variable_words]
    /// Test the extraction of a V::Word from an u64 word.
    fn test_extract_bridge_value_from_word<V: VariableWord>() {
        let mut lower_word = 0_u64;
        let mut upper_word = 0_u64;
        // We sample 250 random values of the word.
        for value in iter_random_values::<V>(V::MASK.min(200), None, None) {
            // For each value, we iter all possible offset values.
            for offset in (65_u8 - V::NUMBER_OF_BITS)..64_u8 {
                insert_bridge_value_into_word::<V>(
                    &mut lower_word,
                    &mut upper_word,
                    offset,
                    value.into(),
                );
                assert_eq!(
                    extract_bridge_value_from_words::<V, 2>([lower_word, lower_word], [upper_word, upper_word], offset),
                    [value, value],
                    "The value extracted from the word {} at offset {} should be equal to the value {}",
                    lower_word,
                    offset,
                    value
                );
            }
        }
    }
}

/// Iterator over the registers of two packed arrays.
pub struct ArrayIter<A, const M: usize> {
    /// Number of values to be iterated in total.
    total_values: usize,
    /// The current register being processed.
    value_index: usize,
    /// The current column of the matrix being processed.
    word_index: usize,
    /// The offset in bits of the current word. In the case of bridge registers, this will be the
    /// offset of the bridge size from the previous word.
    word_offset: u8,
    /// The arrays being processed.
    arrays: [A; M],
    /// The current n-uple of words being processed.
    column: [u64; M],
}

/// Constructor for [`ArrayIter`].
impl<A, const M: usize> ArrayIter<A, M> {
    #[inline]
    /// Creates a new instance of the register tuple iterator.
    fn new<const N: usize>(arrays: [A; M], total_values: usize) -> Self
    where
        [A; M]: Matrix<u64, M, N>,
    {
        Self {
            total_values,
            value_index: 0,
            word_offset: 0,
            word_index: 0,
            column: if N == 0 {
                [u64::ZERO; M]
            } else {
                arrays.column(0)
            },
            arrays,
        }
    }
}

/// Implementation of the `Iterator` trait for [`ArrayIter`].
impl<'array, const PACKED: bool, const N: usize, V: VariableWord> Iterator
    for ArrayIter<&'array Array<N, PACKED, V>, 2>
{
    type Item = [V::Word; 2];

    fn next(&mut self) -> Option<Self::Item> {
        if self.total_values == self.value_index {
            return None;
        }

        self.value_index += 1;

        // If the current register is inside the current word and not a bridge register, we can
        // extract the register directly from the word.
        Some(
            if <Array<N, PACKED, V>>::is_bridge_offset(self.word_offset) {
                let current_column = self.column;
                self.word_index += 1;
                self.column = self.arrays.column(self.word_index);
                let values = extract_bridge_value_from_words::<V, 2>(
                    current_column,
                    self.column,
                    self.word_offset,
                );

                self.word_offset = V::NUMBER_OF_BITS - (64 - self.word_offset);
                values
            } else {
                let values = extract_value_from_words::<V, 2>(self.column, self.word_offset);
                self.word_offset += V::NUMBER_OF_BITS;
                if self.value_index < self.total_values
                    && (PACKED && self.word_offset == 64
                        || !PACKED && self.word_offset == V::NUMBER_OF_BITS * V::NUMBER_OF_ENTRIES)
                {
                    self.word_offset = 0;
                    self.word_index += 1;
                    self.column = self.arrays.column(self.word_index);
                }
                values
            },
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_values = self.total_values - self.value_index;
        (remaining_values, Some(remaining_values))
    }
}

/// Implementation of the `Iterator` trait for [`ArrayIter`].
impl<'array, const PACKED: bool, const N: usize, V: VariableWord> Iterator
    for ArrayIter<&'array Array<N, PACKED, V>, 1>
{
    type Item = V::Word;

    fn next(&mut self) -> Option<Self::Item> {
        if self.total_values == self.value_index {
            return None;
        }

        self.value_index += 1;

        // If the current register is inside the current word and not a bridge register, we can
        // extract the register directly from the word.
        Some(
            if <Array<N, PACKED, V>>::is_bridge_offset(self.word_offset) {
                let current_column = self.column;
                self.word_index += 1;
                self.column = self.arrays.column(self.word_index);
                let [value] = extract_bridge_value_from_words::<V, 1>(
                    current_column,
                    self.column,
                    self.word_offset,
                );

                self.word_offset = V::NUMBER_OF_BITS - (64 - self.word_offset);
                value
            } else {
                let [value] = extract_value_from_words::<V, 1>(self.column, self.word_offset);
                self.word_offset += V::NUMBER_OF_BITS;
                if self.value_index < self.total_values
                    && (PACKED && self.word_offset == 64
                        || !PACKED && self.word_offset == V::NUMBER_OF_BITS * V::NUMBER_OF_ENTRIES)
                {
                    self.word_offset = 0;
                    self.word_index += 1;
                    self.column = self.arrays.column(self.word_index);
                }
                value
            },
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_values = self.total_values - self.value_index;
        (remaining_values, Some(remaining_values))
    }
}

/// Implementation of the `ExactSizeIterator` trait for [`ArrayIter`].
impl<'array, const PACKED: bool, const N: usize, const M: usize, V: VariableWord> ExactSizeIterator
    for ArrayIter<&'array Array<N, PACKED, V>, M>
where
    Self: Iterator,
{
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
/// Register implementation for the packed array registers.
pub struct Array<const N: usize, const PACKED: bool, V> {
    /// The packed array of registers.
    words: [u64; N],
    /// Phantom data to keep track of the variable word type.
    _phantom: PhantomData<V>,
}

impl<const N: usize, const PACKED: bool, V: VariableWord> AsRef<[u64; N]> for Array<N, PACKED, V> {
    #[inline]
    fn as_ref(&self) -> &[u64; N] {
        &self.words
    }
}

macro_rules! impl_as_ref_mut {
    ($($typ:ty),*) => {
        $(
            impl<const N: usize, const PACKED: bool, V2> AsRef<[$typ]>
                for Array<N, PACKED, V2>
            {
                #[inline]
                #[allow(unsafe_code)]
                fn as_ref(&self) -> &[$typ] {
                    let words_u64: &[u64] = self.words.as_ref();
                    unsafe { core::slice::from_raw_parts(words_u64.as_ptr().cast::<$typ>(), words_u64.len() * 8 / core::mem::size_of::<$typ>()) }
                }
            }

            impl<const N: usize, const PACKED: bool, V2> AsMut<[$typ]>
                for Array<N, PACKED, V2>
            {
                #[inline]
                #[allow(unsafe_code)]
                fn as_mut(&mut self) -> &mut [$typ] {
                    let words_u64: &mut [u64] = self.words.as_mut();
                    unsafe { core::slice::from_raw_parts_mut(words_u64.as_mut_ptr().cast::<$typ>(), words_u64.len() * 8 / core::mem::size_of::<$typ>()) }
                }
            }
        )*
    };
}

impl_as_ref_mut!(u8, u16, u32, u64);

macro_rules! impl_to_bytes_ref_mut {
    ($($number:expr),*) => {
        $(
            impl<const N: usize, const PACKED: bool, V2> AsRef<[[u8; $number]]>
                for Array<N, PACKED, V2>
            {
                #[inline]
                #[allow(unsafe_code)]
                fn as_ref(&self) -> &[[u8; $number]] {
                    let words_u64: &[u64] = self.words.as_ref();
                    unsafe { core::slice::from_raw_parts(words_u64.as_ptr().cast::<[u8; $number]>(), words_u64.len() * 8 / $number) }
                }
            }

            impl<const N: usize, const PACKED: bool, V2> AsMut<[[u8; $number]]>
                for Array<N, PACKED, V2>
            {
                #[inline]
                #[allow(unsafe_code)]
                fn as_mut(&mut self) -> &mut [[u8; $number]] {
                    let words_u64: &mut [u64] = self.words.as_mut();
                    unsafe { core::slice::from_raw_parts_mut(words_u64.as_mut_ptr().cast::<[u8; $number]>(), words_u64.len() * 8 / $number) }
                }
            }
        )*
    };
}

impl_to_bytes_ref_mut!(3, 5, 6, 7);

impl<const N: usize, const PACKED: bool, V: VariableWord> Array<N, PACKED, V> {
    #[inline]
    fn iter_values(&self, len: usize) -> ArrayIter<&Self, 1> {
        ArrayIter::new([self], len)
    }
}

#[cfg(test)]
mod test_iter_values {
    use super::*;
    use crate::prelude::*;
    use hyperloglog_derive::test_array;

    #[test_array]
    /// Test the extraction of a V::Word from an u64 word.
    fn test_iter_values<const M: usize, const N: usize, const PACKED: bool, V: VariableWord>(
        reference: [V::Word; M],
    ) {
        let mut array = Array::<N, PACKED, V>::default();

        // We populate the array with the values from the reference.
        for (i, value) in reference.iter().enumerate() {
            array.set(i, *value);
        }

        let iter: ArrayIter<&Array<N, PACKED, V>, 1> = array.iter_values(M);
        assert_eq!(iter.len(), M, "The iterator should have a length of {}", M);

        for (i, (reference_value, array_value)) in reference.iter().zip(iter).enumerate() {
            assert_eq!(reference_value, &array_value, "The value ({array_value}) extracted from position ({i}) should be equal to the reference value ({reference_value}).");
        }
    }
}

impl<const N: usize, const PACKED: bool, V: VariableWord> Array<N, PACKED, V> {
    #[inline]
    fn iter_values_zipped<'words>(
        &'words self,
        other: &'words Self,
        len: usize,
    ) -> ArrayIter<&'_ Self, 2> {
        ArrayIter::new([self, other], len)
    }
}

#[cfg(test)]
mod test_iter_values_zipped {
    use super::*;
    use crate::prelude::*;
    use hyperloglog_derive::test_array;

    #[test_array]
    /// Test the extraction of a V::Word from an u64 word.
    fn test_iter_values_zipped<
        const M: usize,
        const N: usize,
        const PACKED: bool,
        V: VariableWord,
    >(
        reference: [V::Word; M],
    ) {
        let mut array = Array::<N, PACKED, V>::default();
        let mut rev_array = Array::<N, PACKED, V>::default();

        // We populate an array with the values from the reference, and
        // we populate the reverse array with the values in reverse order.
        for (i, value) in reference.iter().enumerate() {
            array.set(i, *value);
            rev_array.set(M - 1 - i, *value);
        }

        let iter: ArrayIter<&Array<N, PACKED, V>, 2> = array.iter_values_zipped(&rev_array, M);
        let swapped_iter: ArrayIter<&Array<N, PACKED, V>, 2> =
            rev_array.iter_values_zipped(&array, M);
        assert_eq!(iter.len(), M, "The iterator should have a length of {}", M);
        assert_eq!(
            swapped_iter.len(),
            M,
            "The iterator should have a length of {}",
            M
        );

        for (i, (reference_values, (array_value, swapped_array_value))) in (reference
            .iter()
            .copied()
            .zip(reference.iter().copied().rev())
            .map(|(a, b)| [a, b]))
        .zip(iter.zip(swapped_iter))
        .enumerate()
        {
            let swapped_reference = [reference_values[1], reference_values[0]];
            assert_eq!(reference_values, array_value, "The value ({array_value:?}) extracted from position ({i}) should be equal to the reference value ({reference_values:?}).");
            assert_eq!(swapped_reference, swapped_array_value, "The value ({swapped_array_value:?}) extracted from position ({i}) should be equal to the reference value ({reference_values:?}).");
        }
    }
}

impl<const N: usize, const PACKED: bool, V: VariableWord> Array<N, PACKED, V> {
    /// Clears the packed array of registers.
    #[inline]
    fn clear(&mut self) {
        self.words.fill(0_u64);
    }
}

#[cfg(test)]
mod test_clear_array {
    use super::*;
    use crate::prelude::*;
    use hyperloglog_derive::test_array;

    #[test_array]
    /// Test the extraction of a V::Word from an u64 word.
    fn test_clear_array<const M: usize, const N: usize, const PACKED: bool, V: VariableWord>(
        reference: [V::Word; M],
    ) {
        let mut array = Array::<N, PACKED, V>::default();

        // We populate the array with the values from the reference.
        for (i, value) in reference.iter().enumerate() {
            array.set(i, *value);
        }

        // We clear the array.
        array.clear();

        // We check that all the values in the array are zero.
        for i in 0..M {
            assert_eq!(
                array.get(i),
                V::Word::ZERO,
                "The value at position ({i}) should be zero."
            );
        }
    }
}

impl<const N: usize, const PACKED: bool, V: VariableWord> Array<N, PACKED, V> {
    #[inline]
    /// Returns whether a given offset is a bridge offset.
    const fn is_bridge_offset(offset: u8) -> bool {
        PACKED
            && (V::NUMBER_OF_BITS_U64 * V::NUMBER_OF_ENTRIES_U64 < 64)
            && (offset + V::NUMBER_OF_BITS > 64)
    }

    #[inline]
    /// Returns whether the array has padding for a given length.
    const fn has_padding(len: u64) -> bool {
        if PACKED {
            len * V::NUMBER_OF_BITS_U64 < N as u64 * 64
        } else {
            len < N as u64 * V::NUMBER_OF_ENTRIES_U64
        }
    }

    #[inline]
    /// Returns the number of values in the array.
    const fn number_of_values() -> usize {
        if PACKED {
            N * 64 / V::NUMBER_OF_BITS_USIZE
        } else {
            N * V::NUMBER_OF_ENTRIES_USIZE
        }
    }

    #[inline]
    /// Returns the value stored at the given index.
    fn get(&self, index: usize) -> V::Word {
        debug_assert!(
            index < Self::number_of_values(),
            "The index {index} should be less than {} (the number of registers) in an object of type {}",
            N as u64 * V::NUMBER_OF_ENTRIES_U64,
            core::any::type_name::<Self>()
        );

        // We determine the word which contains the value and the position of the value,
        // taking into account the bridge values.
        let (word_index, relative_value_offset) = split_index::<PACKED, V>(index);

        debug_assert!(
            word_index < N,
            "The word index {} (started out as {}) should be less than {} (the number of words) in an object of type {}",
            word_index,
            index,
            N,
            core::any::type_name::<Self>()
        );

        // Now we determine whether the value is a bridge value or not, i.e. if it spans
        // two words.
        if Self::is_bridge_offset(relative_value_offset) {
            extract_bridge_value_from_word::<V>(
                self.words[word_index],
                self.words[word_index + 1],
                relative_value_offset,
            )
        } else {
            extract_value_from_word::<V>(self.words[word_index], relative_value_offset)
        }
    }

    #[inline]
    /// Set the value at the given index.
    ///
    /// # Arguments
    /// * `index` - The index at which the value is to be set.
    /// * `value` - The value to be set.
    pub fn set(&mut self, index: usize, value: V::Word) {
        let (word_index, relative_value_offset) = split_index::<PACKED, V>(index);

        if Self::is_bridge_offset(relative_value_offset) {
            let (low, high) = self.words.split_at_mut(word_index + 1);
            let low = &mut low[word_index];
            let high = &mut high[0];
            insert_bridge_value_into_word::<V>(low, high, relative_value_offset, value.into());
        } else {
            insert_value_into_word::<V>(
                &mut self.words[word_index],
                relative_value_offset,
                value.into(),
            );
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    /// Applies a function to the value at the given index.
    ///
    /// # Arguments
    /// * `index` - The index at which the value is to be set.
    /// * `ops` - The function to apply to the value at the given index.
    ///
    /// # Returns
    /// The previous value at the given index and the new value.
    /// 
    /// # Safety
    /// This method accesses values in the underlying array without checking whether the index is valid,
    /// as it is guaranteed to be valid by the split_index method.
    pub fn set_apply<F>(&mut self, index: usize, ops: F) -> (V::Word, V::Word)
    where
        F: Fn(V::Word) -> V::Word,
    {
        let (word_index, relative_value_offset) = split_index::<PACKED, V>(index);

        if Self::is_bridge_offset(relative_value_offset) {
            let (low, high) = unsafe {self.words.split_at_mut_unchecked(word_index + 1)};
            let low = &mut low[word_index];
            let high = &mut high[0];
            let value = extract_bridge_value_from_word::<V>(*low, *high, relative_value_offset);
            let new_value = ops(value);
            insert_bridge_value_into_word::<V>(low, high, relative_value_offset, new_value.into());

            debug_assert_eq!(self.get(index), new_value);

            (value, new_value)
        } else {
            let value = extract_value_from_word::<V>(self.words[word_index], relative_value_offset);
            let new_value = ops(value);
            insert_value_into_word::<V>(
                &mut self.words[word_index],
                relative_value_offset,
                new_value.into(),
            );

            debug_assert_eq!(self.get(index), new_value);

            (value, new_value)
        }
    }

    #[inline]
    /// Applies the given function to up to `len` values in the packed array.
    ///
    /// # Arguments
    /// * `ops` - The function to apply to the values.
    /// * `len` - The number of values to apply the function to.
    fn apply<F>(&mut self, mut ops: F, len: u64)
    where
        F: FnMut(V::Word) -> V::Word,
    {
        let mut number_of_values = 0;
        let mut value_offset = 0;
        for i in 0..N {
            let mut number_of_values_in_word = if PACKED {
                (64 - u64::from(value_offset)) / V::NUMBER_OF_BITS_U64
            } else {
                V::NUMBER_OF_ENTRIES_U64
            };

            if Self::has_padding(len) && number_of_values + number_of_values_in_word > len {
                number_of_values_in_word = len - number_of_values;
            }

            let word = &mut self.words[i];
            for _ in 0..number_of_values_in_word {
                let register = extract_value_from_word::<V>(*word, value_offset);
                let new_register = ops(register);
                insert_value_into_word::<V>(word, value_offset, new_register.into());
                value_offset += V::NUMBER_OF_BITS;
            }
            number_of_values += number_of_values_in_word;

            if Self::is_bridge_offset(value_offset) && (!Self::has_padding(len) || i != N - 1) {
                let (low, high) = self.words.split_at_mut(i + 1);
                let low = &mut low[i];
                let high = &mut high[0];
                let value = extract_bridge_value_from_word::<V>(*low, *high, value_offset);
                let new_value = ops(value);
                insert_bridge_value_into_word::<V>(low, high, value_offset, new_value.into());
                value_offset = V::NUMBER_OF_BITS - (64 - value_offset);
                number_of_values += 1;
            } else {
                value_offset = 0;
            }
        }
    }
}

impl<const N: usize, const PACKED: bool, V: VariableWord> Default for Array<N, PACKED, V> {
    #[inline]
    fn default() -> Self {
        Self {
            words: [0; N],
            _phantom: PhantomData,
        }
    }
}

#[cfg(feature = "std")]
impl<const N: usize, const PACKED: bool, V: VariableWord> Named for Array<N, PACKED, V> {
    #[inline]
    fn name(&self) -> String {
        format!(
            "{}<{}>",
            if PACKED { "Packed" } else { "Array" },
            V::NUMBER_OF_BITS
        )
    }
}

/// Trait marker to associate a specific register array with a combination of precision and bits.
///
/// Meant to be associated with a specific Precision.
pub trait ArrayRegister<B: Bits>: Precision {
    #[cfg(all(feature = "std", feature = "mem_dbg"))]
    /// The type of the packed array register.
    type Array: Registers<Self, B> + Named + MemDbg + MemSize;
    #[cfg(all(feature = "std", not(feature = "mem_dbg")))]
    /// The type of the packed array register.
    type Array: Registers<Self, B> + Named;
    #[cfg(not(feature = "std"))]
    /// The type of the packed array register.
    type Array: Registers<Self, B>;

    #[cfg(all(feature = "std", feature = "mem_dbg"))]
    /// The type of the packed array register.
    type Packed: Registers<Self, B> + Named + MemDbg + MemSize;
    #[cfg(all(feature = "std", not(feature = "mem_dbg")))]
    /// The type of the packed array register.
    type Packed: Registers<Self, B> + Named;
    #[cfg(not(feature = "std"))]
    /// The type of the packed array register.
    type Packed: Registers<Self, B>;
}

/// Trait marker to associate a precision to all possible packed array registers.
pub trait AllArrays:
    ArrayRegister<Bits1>
    + ArrayRegister<Bits2>
    + ArrayRegister<Bits3>
    + ArrayRegister<Bits4>
    + ArrayRegister<Bits5>
    + ArrayRegister<Bits6>
    + ArrayRegister<Bits7>
    + ArrayRegister<Bits8>
{
}

impl<P> AllArrays for P where
    P: ArrayRegister<Bits1>
        + ArrayRegister<Bits2>
        + ArrayRegister<Bits3>
        + ArrayRegister<Bits4>
        + ArrayRegister<Bits5>
        + ArrayRegister<Bits6>
        + ArrayRegister<Bits7>
        + ArrayRegister<Bits8>
{
}

#[allow(unsafe_code)]
#[inline]
#[expect(
    clippy::cast_possible_truncation,
    reason = "The value is guaranteed to be less than 256"
)]
/// Method to convert a usize to a u8.
///
/// # Arguments
/// * `value` - The value to be converted.
///
/// # Safety
/// This method needs to be used with caution, as it will truncate values
/// that are greater than 255.
const unsafe fn usize_to_u8(value: usize) -> u8 {
    debug_assert!(value <= 255, "The value should be less than 256");
    value as u8
}

const LOG2_USIZE: usize = (core::mem::size_of::<usize>() * 8).trailing_zeros() as usize;

#[allow(unsafe_code)]
/// Extracts the word position and the relative register offset from the packed index.
///
/// # Safety
/// This method employs unsafe code to convert a usize to a u8, as it guarantees
/// that the value is less than 256.
const fn split_packed_index<V: VariableWord>(index: usize) -> (usize, u8) {
    let absolute_register_offset: usize = (V::NUMBER_OF_BITS_USIZE) * index;
    let word_index: usize = absolute_register_offset >> LOG2_USIZE;
    let relative_register_offset =
        unsafe { usize_to_u8(absolute_register_offset - word_index * 64) };
    (word_index, relative_register_offset)
}

#[allow(unsafe_code)]
/// Extracts the word position and the relative register offset from a non-packed index.
///
/// # Safety
/// This method employs unsafe code to convert a usize to a u8, as it guarantees
/// that the value is less than 256.
const fn split_not_packed_index<V: VariableWord>(index: usize) -> (usize, u8) {
    let word_index: usize = index / V::NUMBER_OF_ENTRIES_USIZE;
    let relative_register_offset: u8 =
        V::NUMBER_OF_BITS * unsafe { usize_to_u8(index - word_index * V::NUMBER_OF_ENTRIES_USIZE) };
    (word_index, relative_register_offset)
}

#[cfg(test)]
/// Test module for the [`split_packed_index`] and [`split_not_packed_index`] functions.
mod test_split_index {
    use super::*;
    use crate::prelude::*;
    use hyperloglog_derive::test_variable_words;

    #[test_variable_words]
    /// Test the extraction of the word index and the relative register offset from the packed index.
    fn test_split_packed_index<V: VariableWord>() {
        let minimum_index = 0_usize;
        // The maximal precision is 18, so the maximum number of registers is 2^18,
        // hence the maximum index is 2^18 - 1.
        let maximum_index = 1_usize << 18;
        // We iter all possible values of the index.
        for index in minimum_index..maximum_index {
            let expected_word_index = (usize::from(V::NUMBER_OF_BITS) * index) / 64;
            let expected_relative_register_offset = (usize::from(V::NUMBER_OF_BITS) * index) % 64;
            let (word_index, relative_register_offset) = split_packed_index::<V>(index);
            assert_eq!(
                word_index, expected_word_index as usize,
                "The word index {} should be equal to the word index {}",
                word_index, expected_word_index
            );
            assert_eq!(
                relative_register_offset,
                expected_relative_register_offset as u8,
                "The relative register offset {} should be equal to the relative register offset {}",
                relative_register_offset,
                expected_relative_register_offset
            );
        }
    }

    #[test_variable_words]
    /// Test the extraction of the word index and the relative register offset from the non-packed index.
    fn test_split_not_packed_index<V: VariableWord>() {
        let minimum_index = 0_usize;
        // The maximal precision is 18, so the maximum number of registers is 2^18,
        // hence the maximum index is 2^18 - 1.
        let maximum_index = 1_usize << 18;
        // We iter all possible values of the index.
        for index in minimum_index..maximum_index {
            let expected_word_index = index / usize::from(V::NUMBER_OF_ENTRIES);
            let expected_relative_register_offset =
                (index % usize::from(V::NUMBER_OF_ENTRIES)) as u8 * V::NUMBER_OF_BITS;
            let (word_index, relative_register_offset) = split_not_packed_index::<V>(index);
            assert_eq!(
                word_index, expected_word_index as usize,
                "The word index {} should be equal to the word index {}",
                word_index, expected_word_index
            );
            assert_eq!(
                relative_register_offset,
                expected_relative_register_offset as u8,
                "The relative register offset {} should be equal to the relative register offset {}",
                relative_register_offset,
                expected_relative_register_offset
            );
        }
    }
}

const fn split_index<const PACKED: bool, V: VariableWord>(index: usize) -> (usize, u8) {
    if PACKED {
        split_packed_index::<V>(index)
    } else {
        split_not_packed_index::<V>(index)
    }
}

/// Implement the packed array registers for a specific combination of precision and bits.
macro_rules! impl_packed_array_register_for_precision_and_bits {
    ($exponent: expr, $($bits: expr),*) => {
        $(
            paste::paste! {
                #[cfg(feature = "precision_" $exponent)]
                impl ArrayRegister<[<Bits $bits>]> for [<Precision $exponent>] {
                    type Array = Array<{crate::utils::ceil(usize::pow(2, $exponent), 64 / $bits)}, false, [<Bits $bits>]>;
                    type Packed = Array<{crate::utils::ceil(usize::pow(2, $exponent) * $bits, 64)}, true, [<Bits $bits>]>;
                }

                #[cfg(feature = "precision_" $exponent)]
                impl Registers<[<Precision $exponent>], [<Bits $bits>]> for Array<{crate::utils::ceil(usize::pow(2, $exponent) * $bits, 64)}, true, [<Bits $bits>]> {
                    type Iter<'words> = ArrayIter<&'words Self, 1> where Self: 'words;
                    type IterZipped<'words> = ArrayIter<&'words Self, 2>
                        where
                            Self: 'words;

                    #[inline]
                    fn iter_registers(&self) -> Self::Iter<'_> {
                        self.iter_values(1 << [<Precision $exponent>]::EXPONENT)
                    }

                    #[inline]
                    fn iter_registers_zipped<'words>(&'words self, other: &'words Self) -> Self::IterZipped<'words>{
                        self.iter_values_zipped(other, 1 << [<Precision $exponent>]::EXPONENT)
                    }

                    #[inline]
                    fn get_harmonic_sum_and_zeros(
                        &self,
                        other: &Self,
                    ) -> (f64, <[<Precision $exponent>] as Precision>::NumberOfRegisters)
                    {
                        let mut harmonic_sum = f64::ZERO;
                        let mut union_zeros = <[<Precision $exponent>] as Precision>::NumberOfRegisters::ZERO;

                        for [left, right] in <Self as Registers<[<Precision $exponent>], [<Bits $bits>]>>::iter_registers_zipped(self, other) {
                            let max_register = core::cmp::max(left, right);
                            harmonic_sum += f64::integer_exp2_minus(max_register);
                            union_zeros += <[<Precision $exponent>] as Precision>::NumberOfRegisters::from(max_register.is_zero());
                        }

                        (harmonic_sum, union_zeros)
                    }

                    #[inline]
                    fn apply_to_registers<F>(&mut self, register_function: F)
                    where
                        F: FnMut(u8) -> u8,
                    {
                        self.apply(register_function, <[<Precision $exponent>] as Precision>::NUMBER_OF_REGISTERS.into());
                    }

                    #[inline]
                    fn set_greater(&mut self, index: <[<Precision $exponent>] as Precision>::NumberOfRegisters, new_register: u8) -> (u8, u8) {
                        self.set_apply(index.to_usize(), |register| core::cmp::max(register, new_register))
                    }

                    #[inline]
                    /// Returns the value of the register at the given index in the packed array.
                    fn get_register(&self, index: <[<Precision $exponent>] as Precision>::NumberOfRegisters) -> u8 {
                        self.get(index.to_usize())
                    }

                    #[inline]
                    fn clear_registers(&mut self) {
                        self.clear();
                    }
                }

                #[cfg(feature = "precision_" $exponent)]
                impl Registers<[<Precision $exponent>], [<Bits $bits>]> for Array<{crate::utils::ceil(usize::pow(2, $exponent), 64 / $bits)}, false, [<Bits $bits>]> {
                    type Iter<'words> = ArrayIter<&'words Self, 1> where Self: 'words;
                    type IterZipped<'words> = ArrayIter<&'words Self, 2>
                        where
                            Self: 'words;

                    #[inline]
                    fn iter_registers(&self) -> Self::Iter<'_> {
                        self.iter_values(1 << [<Precision $exponent>]::EXPONENT)
                    }

                    #[inline]
                    fn iter_registers_zipped<'words>(&'words self, other: &'words Self) -> Self::IterZipped<'words>{
                        self.iter_values_zipped(other, 1 << [<Precision $exponent>]::EXPONENT)
                    }

                    #[inline]
                    fn get_harmonic_sum_and_zeros(
                        &self,
                        other: &Self,
                    ) -> (f64, <[<Precision $exponent>] as Precision>::NumberOfRegisters)
                    {
                        let mut harmonic_sum = f64::ZERO;
                        let mut union_zeros = <[<Precision $exponent>] as Precision>::NumberOfRegisters::ZERO;

                        for [left, right] in <Self as Registers<[<Precision $exponent>], [<Bits $bits>]>>::iter_registers_zipped(self, other) {
                            let max_register = core::cmp::max(left, right);
                            harmonic_sum += f64::integer_exp2_minus(max_register);
                            union_zeros += <[<Precision $exponent>] as Precision>::NumberOfRegisters::from(max_register.is_zero());
                        }

                        (harmonic_sum, union_zeros)
                    }

                    #[inline]
                    fn apply_to_registers<F>(&mut self, register_function: F)
                    where
                        F: FnMut(u8) -> u8,
                    {
                        self.apply(register_function, <[<Precision $exponent>] as Precision>::NUMBER_OF_REGISTERS.into());
                    }

                    #[inline]
                    fn set_greater(&mut self, index: <[<Precision $exponent>] as Precision>::NumberOfRegisters, new_register: u8) -> (u8, u8) {
                        self.set_apply(index.to_usize(), |register| core::cmp::max(register, new_register))
                    }

                    #[inline]
                    /// Returns the value of the register at the given index in the packed array.
                    fn get_register(&self, index: <[<Precision $exponent>] as Precision>::NumberOfRegisters) -> u8 {
                        self.get(index.to_usize())
                    }

                    #[inline]
                    fn clear_registers(&mut self) {
                        self.clear();
                    }
                }
            }
        )*
    };
}

/// Implement the packed array registers for all the possible combinations of precision and bits.
macro_rules! impl_registers_for_precisions {
    ($($exponent: expr),*) => {
        $(
            impl_packed_array_register_for_precision_and_bits!($exponent, 1, 2, 3, 4, 5, 6, 7, 8);
        )*
    };
}

impl_registers_for_precisions!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);
