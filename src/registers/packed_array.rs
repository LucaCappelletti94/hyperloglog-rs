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

use super::Registers;
use super::{Bits, Bits4, Bits5, Bits6, Matrix, Precision, Zero};
use super::{
    Precision10, Precision11, Precision12, Precision13, Precision14, Precision15, Precision16,
    Precision17, Precision18, Precision4, Precision5, Precision6, Precision7, Precision8,
    Precision9,
};
use crate::utils::VariableWord;
use core::fmt::Debug;
use core::hash::Hash;
use core::marker::PhantomData;
use core::mem::size_of;

#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

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

#[inline]
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

#[inline]
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

/// Iterator over the registers of two packed arrays.
pub struct PackedIter<A, const M: usize> {
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

/// Constructor for [`PackedIter`].
impl<A, const M: usize> PackedIter<A, M> {
    #[inline]
    /// Creates a new instance of the register tuple iterator.
    fn new(arrays: [A; M], total_values: usize) -> Self
    where
        [A; M]: Matrix<u64, M>,
    {
        Self {
            total_values,
            value_index: 0,
            word_offset: 0,
            word_index: 0,
            column: arrays.column(0),
            arrays,
        }
    }
}

/// Implementation of the `Iterator` trait for [`PackedIter`].
impl<'array, W: Debug + AsRef<[u64]> + AsMut<[u64]>, V: VariableWord> Iterator
    for PackedIter<&'array Packed<W, V>, 2>
{
    type Item = [V::Word; 2];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.total_values == self.value_index {
            return None;
        }

        self.value_index += 1;

        // If the current register is inside the current word and not a bridge register, we can
        // extract the register directly from the word.
        Some(if <Packed<W, V>>::is_bridge_offset(self.word_offset) {
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
            if self.value_index < self.total_values && self.word_offset == 64 {
                self.word_offset = 0;
                self.word_index += 1;
                self.column = self.arrays.column(self.word_index);
            }
            values
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_values = self.total_values - self.value_index;
        (remaining_values, Some(remaining_values))
    }
}

/// Implementation of the `Iterator` trait for [`PackedIter`].
impl<'array, W: Debug + AsRef<[u64]> + AsMut<[u64]>, V: VariableWord> Iterator
    for PackedIter<&'array Packed<W, V>, 1>
{
    type Item = V::Word;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.total_values == self.value_index {
            return None;
        }

        self.value_index += 1;

        // If the current register is inside the current word and not a bridge register, we can
        // extract the register directly from the word.
        Some(if <Packed<W, V>>::is_bridge_offset(self.word_offset) {
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
            if self.value_index < self.total_values && self.word_offset == 64 {
                self.word_offset = 0;
                self.word_index += 1;
                self.column = self.arrays.column(self.word_index);
            }
            value
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_values = self.total_values - self.value_index;
        (remaining_values, Some(remaining_values))
    }
}

/// Implementation of the `ExactSizeIterator` trait for [`PackedIter`].
impl<'array, W: AsRef<[u64]>, const M: usize, V: VariableWord> ExactSizeIterator
    for PackedIter<&'array Packed<W, V>, M>
where
    Self: Iterator,
{
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
/// Register implementation for the packed array registers.
pub struct Packed<W, V> {
    /// The packed array of registers.
    words: W,
    /// Phantom data to keep track of the variable word type.
    _phantom: PhantomData<V>,
}

impl<W: AsRef<[u64]>, V> AsRef<[u64]> for Packed<W, V> {
    #[inline]
    fn as_ref(&self) -> &[u64] {
        self.words.as_ref()
    }
}

impl<W: AsRef<[u64]>, V> AsRef<[u8]> for Packed<W, V> {
    #[inline]
    #[allow(unsafe_code)]
    fn as_ref(&self) -> &[u8] {
        let words_u64: &[u64] = self.words.as_ref();
        unsafe { core::slice::from_raw_parts(words_u64.as_ptr().cast::<u8>(), words_u64.len() * 8) }
    }
}

impl<W: AsMut<[u64]>, V> AsMut<[u8]> for Packed<W, V> {
    #[inline]
    #[allow(unsafe_code)]
    fn as_mut(&mut self) -> &mut [u8] {
        let words_u64: &mut [u64] = self.words.as_mut();
        let slice = unsafe {
            core::slice::from_raw_parts_mut(
                words_u64.as_mut_ptr().cast::<u8>(),
                words_u64.len() * 8,
            )
        };
        debug_assert_eq!(slice.len() % size_of::<u64>(), 0);
        slice
    }
}

impl<W: Debug + AsRef<[u64]>, V: VariableWord> Packed<W, V> {
    #[inline]
    fn iter_values(&self, len: usize) -> PackedIter<&Self, 1> {
        PackedIter::new([self], len)
    }
}

impl<W: Debug + AsRef<[u64]>, V: VariableWord> Packed<W, V> {
    #[inline]
    fn iter_values_zipped<'words>(
        &'words self,
        other: &'words Self,
        len: usize,
    ) -> PackedIter<&'_ Self, 2> {
        PackedIter::new([self, other], len)
    }
}

impl<W: AsMut<[u64]>, V: VariableWord> Packed<W, V> {
    /// Clears the packed array of registers.
    #[inline]
    fn clear(&mut self) {
        self.words.as_mut().fill(0_u64);
    }
}

impl<W: AsRef<[u64]> + AsMut<[u64]>, V: VariableWord> Packed<W, V> {
    #[inline]
    /// Returns whether a given offset is a bridge offset.
    const fn is_bridge_offset(offset: u8) -> bool {
        (V::NUMBER_OF_BITS_USIZE * V::NUMBER_OF_ENTRIES < 64) && (offset + V::NUMBER_OF_BITS > 64)
    }

    #[inline]
    /// Returns the value stored at the given index.
    fn get(&self, index: usize) -> V::Word {
        // We determine the word which contains the value and the position of the value,
        // taking into account the bridge values.
        let (word_index, relative_value_offset) = split_packed_index::<V>(index);

        debug_assert!(
            word_index < self.words.as_ref().len(),
            "The word index {} (started out as {}) should be less than {} (the number of words) in an object of type {}",
            word_index,
            index,
            self.words.as_ref().len(),
            core::any::type_name::<Self>()
        );

        // Now we determine whether the value is a bridge value or not, i.e. if it spans
        // two words.
        if Self::is_bridge_offset(relative_value_offset) {
            extract_bridge_value_from_word::<V>(
                self.words.as_ref()[word_index],
                self.words.as_ref()[word_index + 1],
                relative_value_offset,
            )
        } else {
            extract_value_from_word::<V>(self.words.as_ref()[word_index], relative_value_offset)
        }
    }

    #[inline]
    /// Set the value at the given index.
    ///
    /// # Arguments
    /// * `index` - The index at which the value is to be set.
    /// * `value` - The value to be set.
    pub fn set(&mut self, index: usize, value: V::Word) {
        let (word_index, relative_value_offset) = split_packed_index::<V>(index);

        if Self::is_bridge_offset(relative_value_offset) {
            let (low, high) = self.words.as_mut().split_at_mut(word_index + 1);
            let low = &mut low[word_index];
            let high = &mut high[0];
            insert_bridge_value_into_word::<V>(low, high, relative_value_offset, value.into());
        } else {
            insert_value_into_word::<V>(
                &mut self.words.as_mut()[word_index],
                relative_value_offset,
                value.into(),
            );
        }
    }

    #[inline]
    /// Applies the given function to up to `len` values in the packed array.
    ///
    /// # Arguments
    /// * `ops` - The function to apply to the values.
    /// * `len` - The number of values to apply the function to.
    fn apply<F>(&mut self, mut ops: F, len: usize)
    where
        F: FnMut(V::Word) -> V::Word,
    {
        let mut number_of_values: usize = 0;
        let mut value_offset = 0;
        for i in 0..self.words.as_ref().len() {
            let mut number_of_values_in_word =
                (64 - usize::from(value_offset)) / V::NUMBER_OF_BITS_USIZE;

            if number_of_values + number_of_values_in_word > len {
                number_of_values_in_word = len - number_of_values;
            }

            let word = &mut self.words.as_mut()[i];
            for _ in 0..number_of_values_in_word {
                let register = extract_value_from_word::<V>(*word, value_offset);
                let new_register = ops(register);
                insert_value_into_word::<V>(word, value_offset, new_register.into());
                value_offset += V::NUMBER_OF_BITS;
            }
            number_of_values += number_of_values_in_word;

            if Self::is_bridge_offset(value_offset) && i != self.words.as_ref().len() - 1 {
                let (low, high) = self.words.as_mut().split_at_mut(i + 1);
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

impl<const N: usize, V: VariableWord> Default for Packed<[u64; N], V> {
    #[inline]
    fn default() -> Self {
        Self {
            words: [0; N],
            _phantom: PhantomData,
        }
    }
}

#[cfg(feature = "alloc")]
impl<V: VariableWord> Default for Packed<Vec<u64>, V> {
    #[inline]
    fn default() -> Self {
        Self {
            words: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

/// Trait marker to associate a specific register array with a combination of precision and bits.
///
/// Meant to be associated with a specific Precision.
pub trait PackedRegister<B: Bits>: Precision {
    #[cfg(all(feature = "mem_dbg"))]
    /// The type of the packed array register.
    type Array: Registers<Self, B> + MemDbg + MemSize;
    #[cfg(not(feature = "mem_dbg"))]
    /// The type of the packed array register.
    type Array: Registers<Self, B>;
    #[cfg(all(feature = "mem_dbg", feature = "alloc"))]
    /// The type of the packed vector register.
    type Vec: Registers<Self, B> + MemDbg + MemSize;
    #[cfg(all(not(feature = "mem_dbg"), feature = "alloc"))]
    /// The type of the packed vector register.
    type Vec: Registers<Self, B>;
}

const LOG2_USIZE: usize = (size_of::<usize>() * 8).trailing_zeros() as usize;

#[inline]
/// Extracts the word position and the relative register offset from the packed index.
///
/// # Safety
/// This method employs unsafe code to convert a usize to a u8, as it guarantees
/// that the value is less than 256.
const fn split_packed_index<V: VariableWord>(index: usize) -> (usize, u8) {
    let absolute_register_offset: usize = V::NUMBER_OF_BITS_USIZE * index;
    let word_index: usize = absolute_register_offset >> LOG2_USIZE;
    let relative_register_offset = (absolute_register_offset - word_index * 64) as u8;
    (word_index, relative_register_offset)
}

pub trait IncreaseCapacity {
    fn increase_capacity(&mut self, maximal_size: usize);
}

impl<const N: usize> IncreaseCapacity for [u64; N] {
    #[inline]
    fn increase_capacity(&mut self, _maximal_size: usize) {
        unimplemented!("The increase_capacity method is not implemented for [u64; N]");
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
    type Iter<'words> = PackedIter<&'words Self, 1> where Self: 'words;
    type IterZipped<'words> = PackedIter<&'words Self, 2>
        where
            Self: 'words;

    #[inline]
    fn increase_capacity(&mut self) {
        self.words
            .increase_capacity(((1 << P::EXPONENT) * B::NUMBER_OF_BITS_USIZE).div_ceil(64));
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

        if Self::is_bridge_offset(relative_value_offset) {
            let (low, high) = unsafe { self.words.as_mut().split_at_mut_unchecked(word_index + 1) };
            let low = unsafe { low.get_unchecked_mut(word_index) };
            let high = unsafe { high.get_unchecked_mut(0) };
            let value = extract_bridge_value_from_word::<B>(*low, *high, relative_value_offset);
            let new_value = core::cmp::max(value, new_register);
            insert_bridge_value_into_word::<B>(low, high, relative_value_offset, new_value.into());

            debug_assert_eq!(self.get(index), new_value);

            (value, new_value)
        } else {
            let value = extract_value_from_word::<B>(
                unsafe { *self.words.as_ref().get_unchecked(word_index) },
                relative_value_offset,
            );
            let new_value = core::cmp::max(value, new_register);
            insert_value_into_word::<B>(
                unsafe { self.words.as_mut().get_unchecked_mut(word_index) },
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

        if Self::is_bridge_offset(relative_value_offset) {
            let (low, high) = unsafe { self.words.as_mut().split_at_mut_unchecked(word_index + 1) };
            let low = unsafe { low.get_unchecked_mut(word_index) };
            let high = unsafe { high.get_unchecked_mut(0) };
            insert_bridge_value_into_word::<B>(low, high, relative_value_offset, new_register.into());

            debug_assert_eq!(self.get(index), new_register);
        } else {
            insert_value_into_word::<B>(
                unsafe { self.words.as_mut().get_unchecked_mut(word_index) },
                relative_value_offset,
                new_register.into(),
            );

            debug_assert_eq!(self.get(index), new_register);
        }
    }

    #[inline]
    /// Returns the value of the register at the given index in the packed array.
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
                    type Array = Packed<[u64; {(usize::pow(2, $exponent) * $bits).div_ceil(64)}], [<Bits $bits>]>;
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
}
