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
    Debug, Float, Number, One, Precision, RegisterWord, Registers, WordLike, Words, Zero,
};

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


fn extract_bridge_register_from_word<B: Bits, const N: usize, W: WordLike + RegisterWord<B>>(
    lower_word: [W; N],
    upper_word: [W; N],
    offset: u8,
) -> [u8; N]
where
    u8: TryFrom<W>,
    <u8 as TryFrom<W>>::Error: core::fmt::Debug,
{
    debug_assert!(
        offset + B::NUMBER_OF_BITS > W::NUMBER_OF_BITS,
        "Offset + bits ({} + {}) should be greater than {}",
        offset,
        B::NUMBER_OF_BITS,
        W::NUMBER_OF_BITS
    );
    debug_assert!(
        offset <= W::NUMBER_OF_BITS,
        "Offset {} should be less than {}",
        offset,
        W::NUMBER_OF_BITS
    );
    let mut registers = [0_u8; N];
    for i in 0..N {
        let number_of_bits_in_lower_register = W::NUMBER_OF_BITS - offset;
        let number_of_bits_in_upper_register = B::NUMBER_OF_BITS - number_of_bits_in_lower_register;
        let upper_register_mask = (W::ONE << number_of_bits_in_upper_register) - W::ONE;
        let lower_register = if offset == W::NUMBER_OF_BITS {
            W::ZERO
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

pub struct RegisterIter<'a, P: Precision, B: Bits, R: Registers<P, B> + Words>
where
    R: 'a,
{
    /// The current register index across the packed array.
    current_register: P::NumberOfRegisters,
    /// The offset in bits of the current word. In the case of bridge registers, this will be the
    /// offset of the bridge size from the previous word.
    word_offset: u8,
    /// The iterator over the words of the packed array.
    words: R::WordIter<'a>,
    /// The current register being processed.
    current_word: Option<R::Word>,
    _phantom: core::marker::PhantomData<(P, B, R)>,
}

/// New constructor for [`RegisterIter`].
impl<'a, P: Precision, B: Bits, R: Registers<P, B> + Words> RegisterIter<'a, P, B, R>
where
    R: 'a,
{
    fn new(registers: &'a R) -> Self {
        let mut words = registers.words();
        let current_word = words.next();
        Self {
            current_register: P::NumberOfRegisters::ZERO,
            word_offset: 0,
            words,
            current_word,
            _phantom: core::marker::PhantomData,
        }
    }
}

/// Implementation of the Iterator trait for [`RegisterIter`].
impl<'a, P: Precision, B: Bits, R: Registers<P, B> + Words<Word = u64>> Iterator
    for RegisterIter<'a, P, B, R>
where
    R: 'a,
    R::Word: RegisterWord<B>,
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_register == P::NUMBER_OF_REGISTERS {
            return None;
        }

        self.current_word.map(|word| {
            self.current_register += P::NumberOfRegisters::ONE;
            // If the current register is inside the current word and not a bridge register, we can
            // extract the register directly from the word.
            if self.word_offset + B::NUMBER_OF_BITS <= R::Word::NUMBER_OF_BITS {
                let [register] = extract_register_from_word::<B, 1, u64>([word], self.word_offset);
                self.word_offset += B::NUMBER_OF_BITS;
                return register;
            }
            // Otherwise, we need to extract the register from the bridge between the current word
            // and the next one. Since we are guaranteed that the current word is not the last one,
            // we can safely unwrap the next word after having stored what we need from the current.
            self.current_word = self.words.next();
            let next_word = self.current_word.unwrap();

            let register = extract_bridge_register_from_word::<B, 1, u64>(
                [word],
                [next_word],
                self.word_offset,
            )[0];

            self.word_offset = B::NUMBER_OF_BITS - (R::Word::NUMBER_OF_BITS - self.word_offset);

            register
        })
    }
}

pub struct RegisterTupleIter<'a, P: Precision, B: Bits, R: Registers<P, B> + Words<Word = u64>>
where
    R: 'a,
{
    current_register: P::NumberOfRegisters,
    word_offset: u8,
    left: R::WordIter<'a>,
    right: R::WordIter<'a>,
    current_word: Option<(R::Word, R::Word)>,
    _phantom: core::marker::PhantomData<(P, B, R)>,
}

/// Constructor for [`RegisterTupleIter`].
impl<'a, P: Precision, B: Bits, R: Registers<P, B> + Words<Word = u64>>
    RegisterTupleIter<'a, P, B, R>
{
    pub fn new(left: &'a R, right: &'a R) -> Self {
        let mut left = left.words();
        let mut right = right.words();
        let current_word = Some((left.next().unwrap(), right.next().unwrap()));
        Self {
            current_register: P::NumberOfRegisters::ZERO,
            word_offset: 0,
            left,
            right,
            current_word,
            _phantom: core::marker::PhantomData,
        }
    }
}

/// Implementation of the Iterator trait for [`RegisterTupleIter`].
impl<'a, P: Precision, B: Bits, R: Registers<P, B> + Words<Word = u64>> Iterator
    for RegisterTupleIter<'a, P, B, R>
where
    R::Word: RegisterWord<B>,
{
    type Item = (u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_register == P::NUMBER_OF_REGISTERS {
            return None;
        }

        self.current_word.map(|(left_word, right_word)| {
            self.current_register += P::NumberOfRegisters::ONE;
            // If the current register is inside the current word and not a bridge register, we can
            // extract the register directly from the word.
            if self.word_offset + B::NUMBER_OF_BITS <= R::Word::NUMBER_OF_BITS {
                let [left_register, right_register] = extract_register_from_word::<B, 2, u64>(
                    [left_word, right_word],
                    self.word_offset,
                );
                self.word_offset += B::NUMBER_OF_BITS;
                return (left_register, right_register);
            }
            // Otherwise, we need to extract the register from the bridge between the current word
            // and the next one. Since we are guaranteed that the current word is not the last one,
            // we can safely unwrap the next word after having stored what we need from the current.
            self.current_word = Some((self.left.next().unwrap(), self.right.next().unwrap()));
            let (next_left_word, next_right_word) = self.current_word.unwrap();
            let [left_register, right_register] = extract_bridge_register_from_word::<B, 2, u64>(
                [left_word, right_word],
                [next_left_word, next_right_word],
                self.word_offset,
            );

            self.word_offset = B::NUMBER_OF_BITS - (R::Word::NUMBER_OF_BITS - self.word_offset);

            (left_register, right_register)
        })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
/// Register implementation for the packed array registers.
pub struct PackedArray<W, const N: usize> {
    words: [W; N],
}

#[cfg(feature = "std")]
impl<const N: usize, W> crate::prelude::Named for PackedArray<W, N> {
    fn name(&self) -> String {
        "PackedArray".to_string()
    }
}

impl<const N: usize, W: WordLike> Words for PackedArray<W, N> {
    type Word = W;
    type WordIter<'a> = core::iter::Copied<core::slice::Iter<'a, Self::Word>> where Self: 'a;

    fn number_of_words(&self) -> usize {
        N
    }

    fn find_sorted_with_len(&self, value: Self::Word, len: usize) -> bool {
        self.words.find_sorted_with_len(value, len)
    }

    fn sorted_insert_with_len(&mut self, value: Self::Word, len: usize) -> bool {
        self.words.sorted_insert_with_len(value, len)
    }

    fn words(&self) -> Self::WordIter<'_> {
        self.words.iter().copied()
    }
}

impl<W, const N: usize> From<[W; N]> for PackedArray<W, N> {
    fn from(words: [W; N]) -> Self {
        Self { words }
    }
}

/// Trait marker to associate a specific register array with a combination of precision and bits.
///
/// Meant to be associated with a specific Precision.
pub trait PackedArrayRegister<B: Bits>: Precision {
    /// The type of the packed array register.
    type PackedArrayRegister: Registers<Self, B>;
}

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

macro_rules! impl_packed_array_register_for_precision_and_bits {
    ($exponent: expr, $($bits: expr),*) => {
        $(
            paste::paste! {
                #[cfg(feature = "precision_" $exponent)]
                impl PackedArrayRegister<[<Bits $bits>]> for [<Precision $exponent>] {
                    type PackedArrayRegister = PackedArray<u64, {crate::utils::ceil(usize::pow(2, $exponent) * $bits, 64)}>;
                }

                #[cfg(feature = "precision_" $exponent)]
                impl Registers<[<Precision $exponent>], [<Bits $bits>]> for PackedArray<u64, {crate::utils::ceil(usize::pow(2, $exponent) * $bits, 64)}> {
                    type Iter<'a> = RegisterIter<'a, [<Precision $exponent>], [<Bits $bits>], Self>;
                    type IterZipped<'a> = RegisterTupleIter<'a, [<Precision $exponent>], [<Bits $bits>], Self>
                        where
                            Self: 'a;

                    fn zeroed() -> Self {
                        PackedArray::from([0; crate::utils::ceil(usize::pow(2, $exponent) * $bits, 64)])
                    }

                    fn iter_registers(&self) -> Self::Iter<'_> {
                        RegisterIter::new(self)
                    }

                    fn iter_registers_zipped<'a>(&'a self, other: &'a Self) -> Self::IterZipped<'a>{
                        RegisterTupleIter::new(self, other)
                    }

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
                                let registers = extract_register_from_word::<[<Bits $bits>], 2, u64>(words, word_offset);
                                word_offset += [<Bits $bits>]::NUMBER_OF_BITS;
                                registers
                            } else {
                                // If we are dealing with a bridge register, we need to extract the registers
                                // from the bridge between the current word and the next one.
                                let old_words = words;
                                words = [self.words[word_index + 1], other.words[word_index + 1]];
                                let registers = extract_bridge_register_from_word::<[<Bits $bits>], 2, u64>(old_words, words, word_offset);
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

                    fn apply<F>(&mut self, mut f: F)
                    where
                        F: FnMut(u8) -> u8,
                    {
                        let mut number_of_registers = 0;
                        let mut register_offset = 0;
                        let mut word_index = 0;
                        while number_of_registers < [<Precision $exponent>]::NUMBER_OF_REGISTERS {
                            if register_offset + [<Bits $bits>]::NUMBER_OF_BITS <= 64 {
                                let [register] = extract_register_from_word::<[<Bits $bits>], 1, u64>([self.words[word_index]], register_offset);
                                let new_register = f(u8::try_from(register).unwrap());
                                self.words[word_index] &= !((<u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK as u64) << register_offset);
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
                                let new_register = f(register);
                                self.words[word_index] &= !((<u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK as u64) << register_offset);
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
                            let [old_register] = extract_register_from_word::<[<Bits $bits>], 1, u64>([*word], relative_register_offset);
                            if new_register > old_register {
                                // We delete the old register
                                *word &= !((<u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK as u64) << relative_register_offset);
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

                    /// Returns the value of the register at the given index in the packed array.
                    fn get_register(&self, index: <[<Precision $exponent>] as Precision>::NumberOfRegisters) -> u8 {
                        // We determine the word which contains the register and the position of the register,
                        // taking into account the bridge registers.
                        let (word_position, relative_register_offset) = split_packed_index::<[<Precision $exponent>], [<Bits $bits>]>(index);
                        // Now we determine whether the register is a bridge register or not, i.e. if it spans
                        // two words.
                        if relative_register_offset + [<Bits $bits>]::NUMBER_OF_BITS <= 64 {
                            // If it is not a bridge register, we can extract the register directly from the word.
                            extract_register_from_word::<[<Bits $bits>], 1, u64>([self.words[word_position]], relative_register_offset)[0]
                        } else {
                            // Otherwise, we need to extract the register from the bridge between the current word
                            // and the next one. We start by extracting the lower portion of the register from the
                            // current word.
                            extract_bridge_register_from_word::<[<Bits $bits>], 1, u64>([self.words[word_position]], [self.words[word_position + 1]], relative_register_offset)[0]
                        }
                    }

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

macro_rules! impl_registers_for_precisions {
    ($($exponent: expr),*) => {
        $(
            impl_packed_array_register_for_precision_and_bits!($exponent, 1, 2, 3, 4, 5, 6, 7, 8);
        )*
    };
}

impl_registers_for_precisions!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);
