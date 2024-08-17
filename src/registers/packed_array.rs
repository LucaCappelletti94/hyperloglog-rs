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
    extract_register_from_word, Bits, Bits1, Bits2, Bits3, Bits4, Bits5, Bits6, Bits7, Bits8,
    FloatOps, Number, One, Precision, RegisterWord, Registers, Words, Zero,
};
use core::fmt::Debug;
use core::iter::Copied;
use core::marker::PhantomData;
use core::slice::Iter;

#[cfg(feature = "std")]
use crate::utils::Named;

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

/// Extracts a bridge register from a starting word and an ending word.
fn extract_bridge_register_from_word<B: Bits, const N: usize>(
    lower_word: [u64; N],
    upper_word: [u64; N],
    offset: u8,
) -> [u8; N]
where
    u64: RegisterWord<B>,
{
    debug_assert!(
        offset + B::NUMBER_OF_BITS > u64::NUMBER_OF_BITS,
        "Offset + bits ({} + {}) should be greater than {}",
        offset,
        B::NUMBER_OF_BITS,
        u64::NUMBER_OF_BITS
    );
    debug_assert!(
        offset <= u64::NUMBER_OF_BITS,
        "Offset {} should be less than {}",
        offset,
        u64::NUMBER_OF_BITS
    );
    let mut registers = [0_u8; N];
    for i in 0..N {
        let number_of_bits_in_lower_register = u64::NUMBER_OF_BITS - offset;
        let number_of_bits_in_upper_register = B::NUMBER_OF_BITS - number_of_bits_in_lower_register;
        let upper_register_mask = (u64::ONE << number_of_bits_in_upper_register) - u64::ONE;
        let lower_register = if offset == u64::NUMBER_OF_BITS {
            u64::ZERO
        } else {
            lower_word[i] >> offset
        };
        let upper_register = upper_word[i] & upper_register_mask;
        registers[i] =
            u8::try_from((upper_register << number_of_bits_in_lower_register) | lower_register)
                .unwrap();
    }
    registers
}

/// Iterator over the registers of a packed array.
pub struct RegisterIter<'words, P: Precision, B: Bits> {
    /// The current register index across the packed array.
    current_register: P::NumberOfRegisters,
    /// The offset in bits of the current word. In the case of bridge registers, this will be the
    /// offset of the bridge size from the previous word.
    word_offset: u8,
    /// The iterator over the words of the packed array.
    words: Iter<'words, u64>,
    /// The current register being processed.
    current_word: Option<u64>,
    /// Phantom data to keep track of the precision, bits, and registers.
    _phantom: PhantomData<(P, B)>,
}

/// New constructor for [`RegisterIter`].
impl<'words, P: Precision, B: Bits> RegisterIter<'words, P, B> {
    /// Creates a new instance of the register iterator.
    fn new(mut words: Iter<'words, u64>) -> Self {
        Self {
            current_register: P::NumberOfRegisters::ZERO,
            word_offset: 0,
            current_word: words.next().copied(),
            words,
            _phantom: PhantomData,
        }
    }
}

/// Implementation of the Iterator trait for [`RegisterIter`].
impl<'words, P: Precision, B: Bits> Iterator for RegisterIter<'words, P, B> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_register == P::NUMBER_OF_REGISTERS {
            return None;
        }

        self.current_word.and_then(|word| {
            self.current_register += P::NumberOfRegisters::ONE;
            // If the current register is inside the current word and not a bridge register, we can
            // extract the register directly from the word.
            if self.word_offset + B::NUMBER_OF_BITS <= 64 {
                let [register] = extract_register_from_word::<B, 1>([word], self.word_offset);
                self.word_offset += B::NUMBER_OF_BITS;
                return Some(register);
            }
            // Otherwise, we need to extract the register from the bridge between the current word
            // and the next one. Since we are guaranteed that the current word is not the last one,
            // we can safely unwrap the next word after having stored what we need from the current.
            self.current_word = self.words.next().copied();
            self.current_word.map(|next_word| {
                let register = extract_bridge_register_from_word::<B, 1>(
                    [word],
                    [next_word],
                    self.word_offset,
                )[0];

                self.word_offset = B::NUMBER_OF_BITS - (64 - self.word_offset);

                register
            })
        })
    }
}

/// Iterator over the registers of two packed arrays.
pub struct RegisterTupleIter<'words, P: Precision, B: Bits> {
    /// The current register index across the packed array.
    current_register: P::NumberOfRegisters,
    /// The offset in bits of the current word. In the case of bridge registers, this will be the
    /// offset of the bridge size from the previous word.
    word_offset: u8,
    /// The iterator over the words of the left packed array.
    left: Iter<'words, u64>,
    /// The iterator over the words of the right packed array.
    right: Iter<'words, u64>,
    /// The current word tuple being processed.
    current_word: Option<(u64, u64)>,
    /// Phantom data to keep track of the precision, bits, and registers.
    _phantom: PhantomData<(P, B)>,
}

/// Constructor for [`RegisterTupleIter`].
impl<'words, P: Precision, B: Bits> RegisterTupleIter<'words, P, B> {
    #[inline]
    /// Creates a new instance of the register tuple iterator.
    fn new(mut left: Iter<'words, u64>, mut right: Iter<'words, u64>) -> Self {
        let current_word = left.next().copied().and_then(|left_word| {
            right
                .next()
                .copied()
                .map(|right_word| (left_word, right_word))
        });
        Self {
            current_register: P::NumberOfRegisters::ZERO,
            word_offset: 0,
            left,
            right,
            current_word,
            _phantom: PhantomData,
        }
    }
}

/// Implementation of the Iterator trait for [`RegisterTupleIter`].
impl<'words, P: Precision, B: Bits> Iterator for RegisterTupleIter<'words, P, B> {
    type Item = (u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_register == P::NUMBER_OF_REGISTERS {
            return None;
        }

        self.current_word.and_then(|(left_word, right_word)| {
            self.current_register += P::NumberOfRegisters::ONE;
            // If the current register is inside the current word and not a bridge register, we can
            // extract the register directly from the word.
            if self.word_offset + B::NUMBER_OF_BITS <= 64 {
                let [left_register, right_register] =
                    extract_register_from_word::<B, 2>([left_word, right_word], self.word_offset);
                self.word_offset += B::NUMBER_OF_BITS;
                return Some((left_register, right_register));
            }
            // Otherwise, we need to extract the register from the bridge between the current word
            // and the next one. Since we are guaranteed that the current word is not the last one,
            // we can safely unwrap the next word after having stored what we need from the current.
            self.current_word = self.left.next().copied().and_then(|new_left_word| {
                self.right
                    .next().copied()
                    .map(|new_right_word| (new_left_word, new_right_word))
            });
            self.current_word.map(|(next_left_word, next_right_word)| {
                let [left_register, right_register] = extract_bridge_register_from_word::<B, 2>(
                    [left_word, right_word],
                    [next_left_word, next_right_word],
                    self.word_offset,
                );

                self.word_offset = B::NUMBER_OF_BITS - (64 - self.word_offset);

                (left_register, right_register)
            })
        })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
/// Register implementation for the packed array registers.
pub struct PackedArray<const N: usize> {
    /// The packed array of registers.
    words: [u64; N],
}

#[cfg(feature = "std")]
impl<const N: usize> Named for PackedArray<N> {
    #[inline]
    fn name(&self) -> String {
        "PackedArray".to_owned()
    }
}

impl<const N: usize> Words for PackedArray<N> {
    type WordIter<'words> = Copied<Iter<'words, u64>> where Self: 'words;

    #[inline]
    fn number_of_words(&self) -> usize {
        N
    }

    #[inline]
    fn find_sorted_with_len(&self, value: u64, len: usize) -> bool {
        self.words.find_sorted_with_len(value, len)
    }

    #[inline]
    fn sorted_insert_with_len(&mut self, value: u64, len: usize) -> bool {
        self.words.sorted_insert_with_len(value, len)
    }

    #[inline]
    fn words(&self) -> Self::WordIter<'_> {
        self.words.iter().copied()
    }
}

/// Trait marker to associate a specific register array with a combination of precision and bits.
///
/// Meant to be associated with a specific Precision.
pub trait PackedArrayRegister<B: Bits>: Precision {
    /// The type of the packed array register.
    type PackedArrayRegister: Registers<Self, B>;
}

/// Extracts the word position and the relative register offset from the packed index.
fn split_packed_index<P: Precision, B: Bits>(index: P::NumberOfRegisters) -> (usize, u8)
where
    u64: RegisterWord<B>,
{
    let number_of_bits: u64 = B::NUMBER_OF_BITS.into();
    let absolute_register_offset: u64 = number_of_bits * index.into();
    let word_position: u64 = absolute_register_offset / 64;
    let relative_register_offset =
        u8::try_from(absolute_register_offset - word_position * 64).unwrap();
    (
        usize::try_from(word_position).unwrap(),
        relative_register_offset,
    )
}

/// Implement the packed array registers for a specific combination of precision and bits.
macro_rules! impl_packed_array_register_for_precision_and_bits {
    ($exponent: expr, $($bits: expr),*) => {
        $(
            paste::paste! {
                #[cfg(feature = "precision_" $exponent)]
                impl PackedArrayRegister<[<Bits $bits>]> for [<Precision $exponent>] {
                    type PackedArrayRegister = PackedArray<{crate::utils::ceil(usize::pow(2, $exponent) * $bits, 64)}>;
                }

                #[cfg(feature = "precision_" $exponent)]
                impl Registers<[<Precision $exponent>], [<Bits $bits>]> for PackedArray<{crate::utils::ceil(usize::pow(2, $exponent) * $bits, 64)}> {
                    type Iter<'words> = RegisterIter<'words, [<Precision $exponent>], [<Bits $bits>]>;
                    type IterZipped<'words> = RegisterTupleIter<'words, [<Precision $exponent>], [<Bits $bits>]>
                        where
                            Self: 'words;

                    #[inline]
                    fn zeroed() -> Self {
                        PackedArray {
                            words: [0; crate::utils::ceil(usize::pow(2, $exponent) * $bits, 64)]
                        }
                    }

                    #[inline]
                    fn iter_registers(&self) -> Self::Iter<'_> {
                        RegisterIter::new(self.words.iter())
                    }

                    #[inline]
                    fn iter_registers_zipped<'words>(&'words self, other: &'words Self) -> Self::IterZipped<'words>{
                        RegisterTupleIter::new(self.words.iter(), other.words.iter())
                    }

                    #[inline]
                    fn get_harmonic_sum_and_zeros(
                        &self,
                        other: &Self,
                    ) -> (f64, <[<Precision $exponent>] as Precision>::NumberOfRegisters)
                    {
                        let mut harmonic_sum = f64::ZERO;
                        let mut union_zeros = <[<Precision $exponent>] as Precision>::NumberOfRegisters::ZERO;

                        let mut word_offset = 0;
                        let mut word_index = 0;
                        let mut number_of_registers = 0;
                        let mut words = [self.words[word_index], other.words[word_index]];

                        while number_of_registers < [<Precision $exponent>]::NUMBER_OF_REGISTERS {
                            // Up until the word offset is less than 64, we can extract the registers
                            // knowing that they are not bridge registers.
                            let [left_register, right_register] = if word_offset + [<Bits $bits>]::NUMBER_OF_BITS <= 64 {
                                let registers = extract_register_from_word::<[<Bits $bits>], 2>(words, word_offset);
                                word_offset += [<Bits $bits>]::NUMBER_OF_BITS;
                                registers
                            } else {
                                // If we are dealing with a bridge register, we need to extract the registers
                                // from the bridge between the current word and the next one.
                                let old_words = words;
                                words = [self.words[word_index + 1], other.words[word_index + 1]];
                                let registers = extract_bridge_register_from_word::<[<Bits $bits>], 2>(old_words, words, word_offset);
                                word_offset = word_offset + [<Bits $bits>]::NUMBER_OF_BITS - 64;
                                word_index += 1;
                                registers
                            };
                            number_of_registers += 1;
                            let max_register = core::cmp::max(left_register, right_register);
                            harmonic_sum += f64::integer_exp2_minus(max_register);
                            union_zeros += <[<Precision $exponent>] as Precision>::NumberOfRegisters::from_bool(max_register.is_zero());
                        }

                        (harmonic_sum, union_zeros)
                    }

                    #[inline]
                    fn apply<F>(&mut self, mut register_function: F)
                    where
                        F: FnMut(u8) -> u8,
                    {
                        let mut number_of_registers = 0;
                        let mut register_offset = 0;
                        let mut word_index = 0;
                        while number_of_registers < [<Precision $exponent>]::NUMBER_OF_REGISTERS {
                            if register_offset + [<Bits $bits>]::NUMBER_OF_BITS <= 64 {
                                let [register] = extract_register_from_word::<[<Bits $bits>], 1>([self.words[word_index]], register_offset);
                                let new_register = register_function(u8::try_from(register).unwrap());
                                self.words[word_index] &= !(u64::from(<u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK) << register_offset);
                                self.words[word_index] |= u64::from(new_register) << register_offset;
                                register_offset += [<Bits $bits>]::NUMBER_OF_BITS;
                                number_of_registers += 1;
                            } else {
                                let number_of_bits_in_lower_register = 64 - register_offset;
                                let number_of_bits_in_upper_register = [<Bits $bits>]::NUMBER_OF_BITS - number_of_bits_in_lower_register;
                                let upper_register_mask = (1 << number_of_bits_in_upper_register) - 1;
                                let lower_register = self.words[word_index] >> register_offset;
                                let upper_register = self.words[word_index + 1] & upper_register_mask;
                                let register = u8::try_from((upper_register << number_of_bits_in_lower_register) | lower_register).unwrap();
                                let new_register = register_function(register);
                                self.words[word_index] &= !(u64::from(<u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK) << register_offset);
                                self.words[word_index] |= u64::from(new_register) << register_offset;
                                self.words[word_index + 1] &= !upper_register_mask;
                                self.words[word_index + 1] |= u64::from(new_register) >> number_of_bits_in_lower_register;
                                register_offset = number_of_bits_in_upper_register;
                                number_of_registers += 1;
                                word_index += 1;
                            }
                        }
                    }

                    #[inline(always)]
                    fn set_greater(&mut self, index: <[<Precision $exponent>] as Precision>::NumberOfRegisters, new_register: u8) -> (u8, u8) {
                        debug_assert!(index < [<Precision $exponent>]::NUMBER_OF_REGISTERS);
                        debug_assert!(new_register <= u8::try_from(<u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK).unwrap());

                        let (word_position, relative_register_offset) = split_packed_index::<[<Precision $exponent>], [<Bits $bits>]>(index);

                        if relative_register_offset + [<Bits $bits>]::NUMBER_OF_BITS <= 64 {
                            let word = &mut self.words[word_position];
                            let [old_register] = extract_register_from_word::<[<Bits $bits>], 1>([*word], relative_register_offset);
                            if new_register > old_register {
                                // We delete the old register
                                *word &= !(u64::from(<u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK) << relative_register_offset);
                                // We insert the new register
                                *word |= u64::from(new_register) << relative_register_offset;
                                (old_register, new_register)
                            } else {
                                (old_register, old_register)
                            }
                        } else {
                            // We are dealing with a bridge register
                            let number_of_bits_in_lower_register = 64 - relative_register_offset;
                            let number_of_bits_in_upper_register = [<Bits $bits>]::NUMBER_OF_BITS - number_of_bits_in_lower_register;
                            let upper_register_mask = (1 << number_of_bits_in_upper_register) - 1;
                            let lower_register = self.words[word_position] >> relative_register_offset;
                            let upper_register = self.words[word_position + 1] & upper_register_mask;
                            let old_register = u8::try_from((upper_register << number_of_bits_in_lower_register) | lower_register).unwrap();
                            if new_register > old_register {
                                // We delete the old register
                                self.words[word_position] &= !(<u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK << relative_register_offset);
                                self.words[word_position + 1] &= !upper_register_mask;
                                // We insert the new register
                                self.words[word_position] |= u64::from(new_register) << relative_register_offset;
                                self.words[word_position + 1] |= u64::from(new_register) >> number_of_bits_in_lower_register;
                                (old_register, new_register)
                            } else {
                                (old_register, old_register)
                            }
                        }
                    }

                    #[inline]
                    /// Returns the value of the register at the given index in the packed array.
                    fn get_register(&self, index: <[<Precision $exponent>] as Precision>::NumberOfRegisters) -> u8 {
                        // We determine the word which contains the register and the position of the register,
                        // taking into account the bridge registers.
                        let (word_position, relative_register_offset) = split_packed_index::<[<Precision $exponent>], [<Bits $bits>]>(index);
                        // Now we determine whether the register is a bridge register or not, i.e. if it spans
                        // two words.
                        if relative_register_offset + [<Bits $bits>]::NUMBER_OF_BITS <= 64 {
                            // If it is not a bridge register, we can extract the register directly from the word.
                            extract_register_from_word::<[<Bits $bits>], 1>([self.words[word_position]], relative_register_offset)[0]
                        } else {
                            // Otherwise, we need to extract the register from the bridge between the current word
                            // and the next one. We start by extracting the lower portion of the register from the
                            // current word.
                            extract_bridge_register_from_word::<[<Bits $bits>], 1>([self.words[word_position]], [self.words[word_position + 1]], relative_register_offset)[0]
                        }
                    }

                    #[inline]
                    fn clear(&mut self) {
                        for word in self.words.iter_mut() {
                            *word = 0;
                        }
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
