//! Submodule implementing the registers trait for the array data structure.
use core::marker::PhantomData;

use super::{
    extract_register_from_word, Bits, Bits1, Bits2, Bits3, Bits4, Bits5, Bits6, Bits7, Bits8,
    FloatOps, Number, One, Precision, RegisterWord, Registers, Words, Zero,
};

#[cfg(feature = "std")]
use crate::prelude::Named;

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

/// Iterator over the registers.
pub struct RegisterIter<'register, P: Precision, B: Bits, R: Words + Registers<P, B>>
where
    R: 'register,
{
    /// The current register.
    current_register: P::NumberOfRegisters,
    /// The current register in the word.
    current_register_in_word: u8,
    /// The iterator over the words.
    words: R::WordIter<'register>,
    /// The current word.
    current_word: Option<u64>,
    /// The phantom data.
    _phantom: PhantomData<(P, B, R)>,
}

impl<'register, P: Precision, B: Bits, R: Words + Registers<P, B>> RegisterIter<'register, P, B, R> {
    /// Creates a new instance of the register iterator.
    fn new(registers: &'register R) -> Self {
        let mut words = registers.words();
        let current_word = words.next();
        Self {
            current_register: P::NumberOfRegisters::ZERO,
            words,
            current_register_in_word: 0,
            current_word,
            _phantom: PhantomData,
        }
    }
}

impl<'register, P: Precision, B: Bits, R: Words + Registers<P, B>> Iterator
    for RegisterIter<'register, P, B, R>
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_register == P::NUMBER_OF_REGISTERS {
            return None;
        }

        self.current_word.map(|word| {
            let [register] = extract_register_from_word::<B, 1>(
                [word],
                self.current_register_in_word * B::NUMBER_OF_BITS,
            );
            self.current_register_in_word += 1;
            self.current_register += P::NumberOfRegisters::ONE;
            if self.current_register_in_word
                == <u64 as RegisterWord<B>>::NUMBER_OF_REGISTERS_IN_WORD
            {
                self.current_register_in_word = 0;
                self.current_word = self.words.next();
            }
            register
        })
    }
}

/// Iterator over the registers zipped with another set of registers.
pub struct TupleIter<'registers, P: Precision, B: Bits, R: Words + Registers<P, B>>
where
    R: 'registers,
{
    /// The current register.
    current_register: P::NumberOfRegisters,
    /// The current register in the word.
    current_register_in_word: u8,
    /// The iterator over the left registers.
    left: R::WordIter<'registers>,
    /// The iterator over the right registers.
    right: R::WordIter<'registers>,
    /// The current words.
    current_word: Option<(u64, u64)>,
    /// The phantom data.
    _phantom: PhantomData<(P, B, R)>,
}

impl<'registers, P: Precision, B: Bits, R: Words + Registers<P, B>> TupleIter<'registers, P, B, R> {
    /// Creates a new instance of the tuple iterator.
    fn new(left: &'registers R, right: &'registers R) -> Self {
        let mut left_iterator = left.words();
        let mut right_iterator = right.words();
        let current_word = left_iterator
            .next()
            .and_then(|left_word| right_iterator.next().map(|right_word| (left_word, right_word)));
        Self {
            current_register: P::NumberOfRegisters::ZERO,
            left: left_iterator,
            right: right_iterator,
            current_register_in_word: 0,
            current_word,
            _phantom: PhantomData,
        }
    }
}

impl<'registers, P: Precision, B: Bits, R: Words + Registers<P, B>> Iterator
    for TupleIter<'registers, P, B, R>
{
    type Item = (u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_register == P::NUMBER_OF_REGISTERS {
            return None;
        }

        self.current_word.map(|(left, right)| {
            let [left_register, right_register] = extract_register_from_word::<B, 2>(
                [left, right],
                self.current_register_in_word * B::NUMBER_OF_BITS,
            );
            self.current_register_in_word += 1;
            self.current_register += P::NumberOfRegisters::ONE;
            if self.current_register_in_word
                == <u64 as RegisterWord<B>>::NUMBER_OF_REGISTERS_IN_WORD
            {
                self.current_register_in_word = 0;
                self.current_word = self.left.next().and_then(|left_word| {
                    self.right.next().map(|right_word| (left_word, right_word))
                });
            }
            (left_register, right_register)
        })
    }
}

/// Trait marker to associate a specific register array with a combination of precision and bits.
///
/// Meant to be associated with a specific Precision.
pub trait ArrayRegister<B: Bits>: Precision {
    /// The type of the array register.
    type ArrayRegister: Registers<Self, B>;
}

#[cfg(feature = "std")]
impl<const N: usize, W> Named for [W; N] {
    #[inline]
    fn name(&self) -> String {
        "Array".to_owned()
    }
}

/// Splits the index into the word position and the register position.
fn split_index<P: Precision, B: Bits>(index: P::NumberOfRegisters) -> (usize, u8)
where
    u64: RegisterWord<B>,
{
    let mask: u64 = <u64 as RegisterWord<B>>::NUMBER_OF_REGISTERS_IN_WORD.into();
    let word_position: u64 = index.into() / mask;
    let register_position: u8 = u8::try_from(index.into() - word_position * mask).unwrap();
    (usize::try_from(word_position).unwrap(), register_position)
}

/// Implement the registers trait for the different combinations of precision and bits.
macro_rules! impl_register_for_precision_and_bits {
    ($exponent: expr, $($bits: expr),*) => {
        $(
            paste::paste! {
                #[cfg(feature = "precision_" $exponent)]
                impl ArrayRegister<[<Bits $bits>]> for [<Precision $exponent>] {
                    type ArrayRegister = [u64; crate::utils::ceil(usize::pow(2, $exponent), 64 / $bits)];
                }

                #[cfg(feature = "precision_" $exponent)]
                impl Registers<[<Precision $exponent>], [<Bits $bits>]> for [u64; crate::utils::ceil(usize::pow(2, $exponent), 64 / $bits)] {
                    type Iter<'register> = RegisterIter<'register, [<Precision $exponent>], [<Bits $bits>], Self>;
                    type IterZipped<'registers> = TupleIter<'registers, [<Precision $exponent>], [<Bits $bits>], Self>
                        where
                            Self: 'registers;

                    #[inline]
                    fn zeroed() -> Self {
                        [0; crate::utils::ceil(usize::pow(2, $exponent), 64 / $bits)]
                    }

                    #[inline]
                    fn iter_registers(&self) -> Self::Iter<'_> {
                        RegisterIter::new(self)
                    }

                    #[inline]
                    fn iter_registers_zipped<'registers>(&'registers self, other: &'registers Self) -> Self::IterZipped<'registers>{
                        TupleIter::new(self, other)
                    }

                    #[inline]
                    fn get_harmonic_sum_and_zeros(
                        &self,
                        other: &Self,
                    ) -> (f64, <[<Precision $exponent>] as Precision>::NumberOfRegisters)
        {
                        let mut harmonic_sum = f64::ZERO;
                        let mut union_zeros = <[<Precision $exponent>] as Precision>::NumberOfRegisters::ZERO;

                        for i in 0..self.len() {
                            let left = self[i];
                            let right = other[i];
                            let mut partial_sum = f64::ZERO;
                            let mut partial_zeros = <[<Precision $exponent>] as Precision>::NumberOfRegisters::ZERO;

                            for step in 0..<u64 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS_IN_WORD {
                                let [left_register, right_register] = extract_register_from_word::<[<Bits $bits>], 2>([left, right], step * [<Bits $bits>]::NUMBER_OF_BITS);
                                let max_register = left_register.max(right_register);
                                partial_sum += f64::integer_exp2_minus(max_register);
                                partial_zeros += <[<Precision $exponent>] as Precision>::NumberOfRegisters::from_bool(max_register == 0);
                            }
                            harmonic_sum += partial_sum;
                            union_zeros += partial_zeros;
                        }

                        let number_of_padding_registers: u8 = u8::try_from(u64::try_from(self.len()).unwrap()
                        * u64::from(<u64 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS_IN_WORD)
                        - u64::from(<[<Precision $exponent>] as Precision>::NUMBER_OF_REGISTERS)).unwrap();

                        union_zeros -= <[<Precision $exponent>] as Precision>::NumberOfRegisters::from(number_of_padding_registers);
                        harmonic_sum -= f64::from(number_of_padding_registers);

                        (harmonic_sum, union_zeros)
                    }

                    #[inline]
                    fn apply<F>(&mut self, mut register_function: F)
                    where
                        F: FnMut(u8) -> u8,
                    {
                        let mut number_of_registers: <[<Precision $exponent>] as Precision>::NumberOfRegisters = <[<Precision $exponent>] as Precision>::NumberOfRegisters::ZERO;
                        for word in self.iter_mut() {
                            let word_copy = *word;
                            let mut tmp_word: u64 = 0;
                            for step in 0..<u64 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS_IN_WORD {
                                if number_of_registers == [<Precision $exponent>]::NUMBER_OF_REGISTERS {
                                    break;
                                }
                                let [register] = extract_register_from_word::<[<Bits $bits>], 1>([word_copy], step * [<Bits $bits>]::NUMBER_OF_BITS);
                                let new_register = register_function(register);
                                debug_assert!(
                                    new_register <= u8::try_from(<u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK).unwrap(),
                                    "Expected the new register at precision {} and bits {} to be <= {} but got {}.",
                                    [<Precision $exponent>]::EXPONENT,
                                    [<Bits $bits>]::NUMBER_OF_BITS,
                                    u8::try_from(<u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK).unwrap(),
                                    new_register
                                );
                                tmp_word |= u64::from(new_register) << (step * [<Bits $bits>]::NUMBER_OF_BITS);
                                number_of_registers += <[<Precision $exponent>] as Precision>::NumberOfRegisters::ONE;
                            }
                            *word = tmp_word;
                        }
                    }

                    #[inline]
                    fn set_greater(&mut self, index: <[<Precision $exponent>] as Precision>::NumberOfRegisters, new_register: u8) -> (u8, u8) {
                        debug_assert!(index < [<Precision $exponent>]::NUMBER_OF_REGISTERS);
                        debug_assert!(
                            new_register <= u8::try_from(<u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK).unwrap(),
                            "Expected the new register at precision {} and bits {} to be <= {} but got {}.",
                            [<Precision $exponent>]::EXPONENT,
                            [<Bits $bits>]::NUMBER_OF_BITS,
                            u8::try_from(<u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK).unwrap(),
                            new_register
                        );

                        // Calculate the position of the register in the internal buffer array.
                        let (word_position, register_position) = split_index::<[<Precision $exponent>], [<Bits $bits>]>(index);

                        // Extract the current value of the register at `index`.
                        let register_value: u8 =
                            u8::try_from((self[word_position] >> (register_position * [<Bits $bits>]::NUMBER_OF_BITS)) & <u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK).unwrap();

                        if register_value >= new_register {
                            return (register_value , register_value );
                        }

                        self[word_position] &= !(<u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK << (register_position * [<Bits $bits>]::NUMBER_OF_BITS));
                        self[word_position] |= u64::from(new_register) << (register_position * [<Bits $bits>]::NUMBER_OF_BITS);

                        (register_value, new_register)
                    }

                    #[inline]
                    fn get_register(&self, index: <[<Precision $exponent>] as Precision>::NumberOfRegisters) -> u8 {
                        // Calculate the position of the register in the internal buffer array.
                        let (word_position, register_position) = split_index::<[<Precision $exponent>], [<Bits $bits>]>(index);

                        // Extract the current value of the register at `index`.
                        extract_register_from_word::<[<Bits $bits>], 1>([self[word_position]], register_position * [<Bits $bits>]::NUMBER_OF_BITS)[0]
                    }

                    #[inline]
                    fn clear(&mut self) {
                        for word in self.iter_mut() {
                            *word = 0;
                        }
                    }
                }
            }
        )*
    };
}

/// Implement the registers trait for the different combinations of precision and bits.
macro_rules! impl_registers_for_precisions {
    ($($exponent: expr),*) => {
        $(
            impl_register_for_precision_and_bits!($exponent, 1, 2, 3, 4, 5, 6, 7, 8);
        )*
    };
}

impl_registers_for_precisions!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);
