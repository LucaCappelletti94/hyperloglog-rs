//! Submodule implementing the registers trait for the array data structure.
use super::{
    extract_register_from_word, Bits, Bits1, Bits2, Bits3, Bits4, Bits5, Bits6, Bits7, Bits8,
    FloatOps, Number, One, Precision, RegisterWord, Registers, Zero,
};
use core::marker::PhantomData;
use core::slice::Iter;

#[cfg(feature = "std")]
use crate::prelude::Named;

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
use crate::utils::Words;

/// Iterator over the registers.
pub struct RegisterIter<'words, P: Precision, B: Bits> {
    /// The current register.
    current_register: P::NumberOfRegisters,
    /// The current register in the word.
    current_register_in_word: u8,
    /// The iterator over the words.
    words: Iter<'words, u64>,
    /// The current word.
    current_word: Option<u64>,
    /// The phantom data.
    _phantom: PhantomData<(P, B)>,
}

impl<'words, P: Precision, B: Bits> RegisterIter<'words, P, B> {
    /// Creates a new instance of the register iterator.
    fn new(mut words: Iter<'words, u64>) -> Self {
        let current_word = words.next().copied();
        Self {
            current_register: P::NumberOfRegisters::ZERO,
            words,
            current_register_in_word: 0,
            current_word,
            _phantom: PhantomData,
        }
    }
}

impl<'words, P: Precision, B: Bits> Iterator for RegisterIter<'words, P, B> {
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
                self.current_word = self.words.next().copied();
            }
            register
        })
    }
}

/// Iterator over the registers zipped with another set of registers.
pub struct TupleIter<'words, P: Precision, B: Bits> {
    /// The current register.
    current_register: P::NumberOfRegisters,
    /// The current register in the word.
    current_register_in_word: u8,
    /// The iterator over the left registers.
    left: Iter<'words, u64>,
    /// The iterator over the right registers.
    right: Iter<'words, u64>,
    /// The current words.
    current_word: Option<(u64, u64)>,
    /// The phantom data.
    _phantom: PhantomData<(P, B)>,
}

impl<'words, P: Precision, B: Bits> TupleIter<'words, P, B> {
    /// Creates a new instance of the tuple iterator.
    fn new(mut left: Iter<'words, u64>, mut right: Iter<'words, u64>) -> Self {
        let current_word = left.next().copied().and_then(|left_word| {
            right
                .next()
                .copied()
                .map(|right_word| (left_word, right_word))
        });
        Self {
            current_register: P::NumberOfRegisters::ZERO,
            left,
            right,
            current_register_in_word: 0,
            current_word,
            _phantom: PhantomData,
        }
    }
}

impl<'words, P: Precision, B: Bits> Iterator for TupleIter<'words, P, B> {
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
                self.current_word = self.left.next().copied().and_then(|left_word| {
                    self.right
                        .next()
                        .copied()
                        .map(|right_word| (left_word, right_word))
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
    #[cfg(all(feature = "std", feature = "mem_dbg"))]
    type ArrayRegister: Registers<Self, B> + Words + Named + MemDbg + MemSize;
    #[cfg(all(feature = "std", not(feature = "mem_dbg")))]
    type ArrayRegister: Registers<Self, B> + Words + Named;
    #[cfg(not(feature = "std"))]
    type ArrayRegister: Registers<Self, B> + Words;
}

/// Trait marker to say that a given precision implements all of the array registers.
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

impl<P: Precision> AllArrays for P where
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
                    type Iter<'words> = RegisterIter<'words, [<Precision $exponent>], [<Bits $bits>]>;
                    type IterZipped<'words> = TupleIter<'words, [<Precision $exponent>], [<Bits $bits>]>
                        where
                            Self: 'words;

                    #[inline]
                    fn zeroed() -> Self {
                        [0; crate::utils::ceil(usize::pow(2, $exponent), 64 / $bits)]
                    }

                    #[inline]
                    fn iter_registers(&self) -> Self::Iter<'_> {
                        RegisterIter::new(self.iter())
                    }

                    #[inline]
                    fn iter_registers_zipped<'words>(&'words self, other: &'words Self) -> Self::IterZipped<'words>{
                        TupleIter::new(self.iter(), other.iter())
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
