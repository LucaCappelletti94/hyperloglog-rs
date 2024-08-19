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
    Bits, Bits1, Bits2, Bits3, Bits4, Bits5, Bits6, Bits7, Bits8, FloatOps, Matrix, Number,
    Precision, Registers, Zero,
};
use crate::utils::{PositiveInteger, VariableWord};
use core::fmt::Debug;
use core::iter::Map;
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

/// Extracts the register from one or more words at the given offset.
///
/// # Arguments
/// * `word` - The word array from which the register is to be extracted.
/// * `offset` - The offset at which the register starts.
fn extract_value_from_word<V: VariableWord>(word: u64, offset: u8) -> V::Word {
    debug_assert!(
        offset + V::NUMBER_OF_BITS <= 64,
        "The offset ({offset} + {}) should be less than or equal to 64",
        V::NUMBER_OF_BITS,
    );
    V::Word::try_from_u64((word >> offset) & V::MASK).unwrap()
}

/// Extracts the register from one or more words at the given offset.
///
/// # Arguments
/// * `word` - The word array from which the register is to be extracted.
/// * `offset` - The offset at which the register starts.
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

fn insert_value_into_word<V: VariableWord>(word: &mut u64, offset: u8, value: u64) {
    *word &= !(V::MASK << offset);
    *word |= value << offset;
}

/// Returns the number of bits in the upper and lower value of a bridge value.
const fn value_from_bridge<V: VariableWord>(lower_word: u64, upper_word: u64, offset: u8) -> u64 {
    debug_assert!(offset != 0, "Offset should be greater than 0");
    debug_assert!(offset != 64, "Offset should be less than 64");
    debug_assert!(
        offset > 64 - V::NUMBER_OF_BITS,
        "Offset should be greater than 64 - V::NUMBER_OF_BITS"
    );
    let number_of_bits_in_lower_value: u8 = 64 - offset;
    let number_of_bits_in_upper_value = V::NUMBER_OF_BITS - number_of_bits_in_lower_value;
    let upper_value_mask: u64 = (1 << number_of_bits_in_upper_value) - 1;

    let lower_value = lower_word >> offset;

    let upper_value = upper_word & upper_value_mask;

    (upper_value << number_of_bits_in_lower_value) | lower_value
}

/// Extracts a bridge register from a starting word and an ending word.
fn extract_bridge_value_from_words<V: VariableWord, const N: usize>(
    lower_word: [u64; N],
    upper_word: [u64; N],
    offset: u8,
) -> [V::Word; N] {
    debug_assert!(
        offset + V::NUMBER_OF_BITS > 64,
        "Offset + bits ({} + {}) should be greater than {}",
        offset,
        V::NUMBER_OF_BITS,
        64
    );
    debug_assert!(offset <= 64, "Offset {} should be less than {}", offset, 64);
    let mut values = [V::Word::ZERO; N];
    for i in 0..N {
        values[i] =
            V::Word::try_from_u64(value_from_bridge::<V>(lower_word[i], upper_word[i], offset))
                .unwrap();
    }
    values
}

fn extract_bridge_value_from_word<V: VariableWord>(
    lower_word: u64,
    upper_word: u64,
    offset: u8,
) -> V::Word {
    extract_bridge_value_from_words::<V, 1>([lower_word], [upper_word], offset)[0]
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

    // First, we clear the bits that will be replaced by the new value.
    *lower_word &= !(V::MASK << offset);
    // Then, we insert the lower part of the new value.
    *lower_word |= value << offset;
    // We do the same for the upper part of the new value.
    *upper_word &= !(V::MASK >> (64 - offset));
    *upper_word |= value >> (64 - offset);
}

/// Iterator over the registers of two packed arrays.
pub struct ArrayIter<A, const M: usize> {
    /// Number of values to be iterated in total.
    total_values: u64,
    /// The current register being processed.
    value_index: u64,
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
    fn new<const N: usize>(arrays: [A; M], total_values: u64) -> Self
    where
        [A; M]: Matrix<u64, M, N>,
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

/// Implementation of the Iterator trait for [`ArrayIter`].
impl<'array, const PACKED: bool, const N: usize, const M: usize, V: VariableWord> Iterator
    for ArrayIter<&'array Array<N, PACKED, V>, M>
{
    type Item = [V::Word; M];

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
                let values = extract_bridge_value_from_words::<V, M>(
                    current_column,
                    self.column,
                    self.word_offset,
                );

                self.word_offset = V::NUMBER_OF_BITS - (64 - self.word_offset);
                values
            } else {
                debug_assert!(self.word_offset + V::NUMBER_OF_BITS <= 64, "While iterating on an object of type {}, the offset ({} + {}) should be less than or equal to 64", core::any::type_name::<Self>(), self.word_offset, V::NUMBER_OF_BITS);
                let values = extract_value_from_words::<V, M>(self.column, self.word_offset);
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

impl<
        const N: usize,
        const PACKED1: bool,
        const PACKED2: bool,
        V1: VariableWord,
        V2: VariableWord,
    > AsRef<Array<N, PACKED1, V1>> for Array<N, PACKED2, V2>
{
    #[inline]
    #[allow(unsafe_code)]
    fn as_ref(&self) -> &Array<N, PACKED1, V1> {
        unsafe { &*(self as *const Array<N, PACKED2, V2> as *const Array<N, PACKED1, V1>) }
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
                    unsafe { core::slice::from_raw_parts(words_u64.as_ptr() as *const $typ, words_u64.len() * core::mem::size_of::<u64>() / core::mem::size_of::<$typ>()) }
                }
            }

            impl<const N: usize, const PACKED: bool, V2> AsMut<[$typ]>
                for Array<N, PACKED, V2>
            {
                #[inline]
                #[allow(unsafe_code)]
                fn as_mut(&mut self) -> &mut [$typ] {
                    let words_u64: &mut [u64] = self.words.as_mut();
                    unsafe { core::slice::from_raw_parts_mut(words_u64.as_mut_ptr() as *mut $typ, words_u64.len() * core::mem::size_of::<u64>() / core::mem::size_of::<$typ>()) }
                }
            }
        )*
    };
}

impl_as_ref_mut!(u8, u16, u32, u64);

impl<
        const N: usize,
        const PACKED1: bool,
        const PACKED2: bool,
        V1: VariableWord,
        V2: VariableWord,
    > AsMut<Array<N, PACKED1, V1>> for Array<N, PACKED2, V2>
{
    #[inline]
    #[allow(unsafe_code)]
    fn as_mut(&mut self) -> &mut Array<N, PACKED1, V1> {
        unsafe { &mut *(self as *mut Array<N, PACKED2, V2> as *mut Array<N, PACKED1, V1>) }
    }
}

impl<const N: usize, const PACKED: bool, V: VariableWord> Array<N, PACKED, V> {
    #[inline]
    fn iter_values(&self, len: u64) -> Map<ArrayIter<&Self, 1>, fn([V::Word; 1]) -> V::Word> {
        ArrayIter::new([self], len).map(|values| values[0])
    }

    #[inline]
    fn iter_values_zipped<'words>(
        &'words self,
        other: &'words Self,
        len: u64,
    ) -> ArrayIter<&'_ Self, 2> {
        ArrayIter::new([self, other], len)
    }

    /// Clears the packed array of registers.
    #[inline]
    fn clear(&mut self) {
        for word in self.words.iter_mut() {
            *word = u64::ZERO;
        }
    }

    #[inline]
    /// Returns whether a given offset is a bridge offset.
    const fn is_bridge_offset(offset: u8) -> bool {
        PACKED
            && (V::NUMBER_OF_BITS_U64 * V::NUMBER_OF_ENTRIES_U64 < 64)
            && (offset + V::NUMBER_OF_BITS > 64)
    }

    #[inline]
    /// Returns whether the packed array has padding for a given length.
    const fn has_padding(len: u64) -> bool {
        if PACKED {
            len * V::NUMBER_OF_BITS_U64 < N as u64 * 64
        } else {
            len < N as u64 * V::NUMBER_OF_ENTRIES_U64
        }
    }

    #[inline]
    /// Returns the value stored at the given index.
    fn get(&self, index: u64) -> V::Word {
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
    /// Applies a function to the value at the given index.
    ///
    /// # Arguments
    /// * `index` - The index at which the value is to be set.
    /// * `ops` - The function to apply to the value at the given index.
    ///
    /// # Returns
    /// The previous value at the given index and the new value.
    fn set_apply<F>(&mut self, index: u64, ops: F) -> (V::Word, V::Word)
    where
        F: Fn(V::Word) -> V::Word,
    {
        let (word_index, relative_value_offset) = split_index::<PACKED, V>(index);

        if Self::is_bridge_offset(relative_value_offset) {
            let (low, high) = self.words.split_at_mut(word_index + 1);
            let low = &mut low[word_index];
            let high = &mut high[0];
            let value = extract_bridge_value_from_word::<V>(*low, *high, relative_value_offset);
            let new_value = ops(value);
            insert_bridge_value_into_word::<V>(low, high, relative_value_offset, new_value.into());

            debug_assert_eq!(
                self.get(index),
                new_value,
                "The value at index {} should be equal to the new value {}",
                index,
                new_value
            );

            (value, new_value)
        } else {
            let value = extract_value_from_word::<V>(self.words[word_index], relative_value_offset);
            let new_value = ops(value);
            insert_value_into_word::<V>(
                &mut self.words[word_index],
                relative_value_offset,
                new_value.into(),
            );

            debug_assert_eq!(
                self.get(index),
                new_value,
                "The value at index {} should be equal to the new value {}",
                index,
                new_value
            );

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
                (64 - value_offset as u64) / V::NUMBER_OF_BITS_U64
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

/// Extracts the word position and the relative register offset from the packed index.
const fn split_packed_index<V: VariableWord>(index: u64) -> (usize, u8) {
    let absolute_register_offset: u64 = V::NUMBER_OF_BITS_U64 * index;
    let word_index: u64 = absolute_register_offset / 64;
    let relative_register_offset = (absolute_register_offset - word_index * 64) as u8;
    (word_index as usize, relative_register_offset)
}

/// Extracts the word position and the relative register offset from a non-packed index.
const fn split_not_packed_index<V: VariableWord>(index: u64) -> (usize, u8) {
    let word_index: u64 = index / V::NUMBER_OF_ENTRIES_U64;
    let relative_register_offset: u8 =
        V::NUMBER_OF_BITS * (index - word_index * V::NUMBER_OF_ENTRIES_U64) as u8;
    (word_index as usize, relative_register_offset)
}

const fn split_index<const PACKED: bool, V: VariableWord>(index: u64) -> (usize, u8) {
    if PACKED {
        split_packed_index::<V>(index)
    } else {
        split_not_packed_index::<V>(index)
    }
}

// impl<const PACKED: bool, const N: usize, W: VariableWord, V: VariableWord> VariableWords<W>
//     for Array<N, PACKED, V>
// where
//     <W as VariableWord>::Word: 'static,
// {
//     type Iter<'words> = Map<ArrayIter<&'words Array<N, PACKED, W>, 1>, fn([W::Word; 1]) -> W::Word> where Self: 'words, W: 'words;

//     #[must_use]
//     #[inline]
//     fn number_of_words(&self) -> usize {
//         if PACKED {
//             N * 64 / W::NUMBER_OF_BITS_U64 as usize
//         } else {
//             N * W::NUMBER_OF_ENTRIES_U64 as usize
//         }
//     }

//     #[must_use]
//     #[inline]
//     #[allow(unsafe_code)]
//     fn find_sorted_with_len(&self, value: W::Word, len: usize) -> bool {
//         debug_assert!(
//             len <= <Self as VariableWords<W>>::number_of_words(self),
//             "The length must be less than or equal to the number of words."
//         );
//         debug_assert!(
//             <Self as VariableWords<W>>::variable_words(self, len)
//                 .zip(<Self as VariableWords<W>>::variable_words(self, len).skip(1))
//                 .all(|(a, b)| a <= b),
//             "The array must be sorted."
//         );

//         // TODO! Actually implement a binary search!
//         match core::any::TypeId::of::<W::Word>() {
//             t if t == core::any::TypeId::of::<u8>()
//                 || t == core::any::TypeId::of::<u16>()
//                 || t == core::any::TypeId::of::<u32>()
//                 || t == core::any::TypeId::of::<u64>() =>
//             unsafe { W::transmutative_binary_search(self.words.as_slice(), len, value).is_ok() },
//             _ => <Self as VariableWords<W>>::variable_words(self, len).any(|v| v == value),
//         }
//     }

//     #[must_use]
//     #[inline]
//     #[allow(unsafe_code)]
//     fn sorted_insert_with_len(&mut self, value: W::Word, len: usize) -> bool {
//         debug_assert!(
//             len <= <Self as VariableWords<W>>::number_of_words(self),
//             "The length must be less than or equal to the number of words."
//         );

//         debug_assert!(
//             <Self as VariableWords<W>>::variable_words(self, len)
//                 .zip(<Self as VariableWords<W>>::variable_words(self, len).skip(1))
//                 .all(|(a, b)| a <= b),
//             "The array must be sorted."
//         );

//         // TODO! Actually implement a binary search!
//         match core::any::TypeId::of::<W::Word>() {
//             t if t == core::any::TypeId::of::<u8>()
//                 || t == core::any::TypeId::of::<u16>()
//                 || t == core::any::TypeId::of::<u32>()
//                 || t == core::any::TypeId::of::<u64>() =>
//             unsafe { W::transmutative_sorted_insert(self.words.as_mut(), len, value) },
//             _ => todo!(),
//         }
//     }

//     #[must_use]
//     #[inline]
//     fn variable_words(&self, len: usize) -> Self::Iter<'_> {
//         <Self as AsRef<Array<N, PACKED, W>>>::as_ref(self).iter_values(len as u64)
//     }
// }

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
                    type Iter<'words> = Map<ArrayIter<&'words Self, 1>, fn([u8; 1]) -> u8> where Self: 'words;
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
                            union_zeros += <[<Precision $exponent>] as Precision>::NumberOfRegisters::from_bool(max_register.is_zero());
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

                    #[inline(always)]
                    fn set_greater(&mut self, index: <[<Precision $exponent>] as Precision>::NumberOfRegisters, new_register: u8) -> (u8, u8) {
                        self.set_apply(index.into(), |register| core::cmp::max(register, new_register))
                    }

                    #[inline]
                    /// Returns the value of the register at the given index in the packed array.
                    fn get_register(&self, index: <[<Precision $exponent>] as Precision>::NumberOfRegisters) -> u8 {
                        self.get(index.into())
                    }

                    #[inline]
                    fn clear_registers(&mut self) {
                        self.clear();
                    }
                }

                #[cfg(feature = "precision_" $exponent)]
                impl Registers<[<Precision $exponent>], [<Bits $bits>]> for Array<{crate::utils::ceil(usize::pow(2, $exponent), 64 / $bits)}, false, [<Bits $bits>]> {
                    type Iter<'words> = Map<ArrayIter<&'words Self, 1>, fn([u8; 1]) -> u8> where Self: 'words;
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
                            union_zeros += <[<Precision $exponent>] as Precision>::NumberOfRegisters::from_bool(max_register.is_zero());
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

                    #[inline(always)]
                    fn set_greater(&mut self, index: <[<Precision $exponent>] as Precision>::NumberOfRegisters, new_register: u8) -> (u8, u8) {
                        self.set_apply(index.into(), |register| core::cmp::max(register, new_register))
                    }

                    #[inline]
                    /// Returns the value of the register at the given index in the packed array.
                    fn get_register(&self, index: <[<Precision $exponent>] as Precision>::NumberOfRegisters) -> u8 {
                        self.get(index.into())
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
