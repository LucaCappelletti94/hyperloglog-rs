//! Submodule implementing the registers trait for the array data structure.
use super::*;

pub struct ArrayRegisterIter<'a, P: Precision, B: Bits, R: Words + Registers<P, B>>
where
    R: 'a,
{
    current_register: usize,
    current_register_in_word: usize,
    words: R::WordIter<'a>,
    current_word: Option<R::Word>,
    _phantom: core::marker::PhantomData<(P, B, R)>,
}

impl<'a, P: Precision, B: Bits, R: Words + Registers<P, B>> ArrayRegisterIter<'a, P, B, R> {
    pub(super) fn new(registers: &'a R) -> Self {
        let mut words = registers.words();
        let current_word = words.next();
        Self {
            current_register: 0,
            words,
            current_register_in_word: 0,
            current_word,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<'a, P: Precision, B: Bits, R: Words<Word = u64> + Registers<P, B>> Iterator
    for ArrayRegisterIter<'a, P, B, R>
where
    R::Word: RegisterWord<B>,
{
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_register == P::NUMBER_OF_REGISTERS {
            return None;
        }

        self.current_word.map(|word| {
            let register: R::Word = (word >> (self.current_register_in_word * B::NUMBER_OF_BITS))
                & R::Word::LOWER_REGISTER_MASK;
            self.current_register_in_word += 1;
            self.current_register += 1;
            if self.current_register_in_word == <u64 as RegisterWord<B>>::NUMBER_OF_REGISTERS {
                self.current_register_in_word = 0;
                self.current_word = self.words.next();
            }
            register as u32
        })
    }
}

pub struct ArrayRegisterTupleIter<'a, P: Precision, B: Bits, R: Words + Registers<P, B>>
where
    R: 'a,
{
    current_register: usize,
    current_register_in_word: usize,
    left: R::WordIter<'a>,
    right: R::WordIter<'a>,
    current_word: Option<(R::Word, R::Word)>,
    _phantom: core::marker::PhantomData<(P, B, R)>,
}

impl<'a, P: Precision, B: Bits, R: Words + Registers<P, B>> ArrayRegisterTupleIter<'a, P, B, R> {
    pub(super) fn new(left: &'a R, right: &'a R) -> Self {
        let mut left = left.words();
        let mut right = right.words();
        let current_word = left
            .next()
            .map(|left_word| unsafe { (left_word, right.next().unwrap_unchecked()) });
        Self {
            current_register: 0,
            left,
            right,
            current_register_in_word: 0,
            current_word,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<'a, P: Precision, B: Bits, R: Words<Word = u64> + Registers<P, B>> Iterator
    for ArrayRegisterTupleIter<'a, P, B, R>
where
    R::Word: RegisterWord<B>,
{
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_register == P::NUMBER_OF_REGISTERS {
            return None;
        }

        self.current_word.map(|(left, right)| {
            let left_register: R::Word = (left
                >> (self.current_register_in_word * B::NUMBER_OF_BITS))
                & R::Word::LOWER_REGISTER_MASK;
            let right_register: R::Word = (right
                >> (self.current_register_in_word * B::NUMBER_OF_BITS))
                & R::Word::LOWER_REGISTER_MASK;
            self.current_register_in_word += 1;
            self.current_register += 1;
            if self.current_register_in_word == <u64 as RegisterWord<B>>::NUMBER_OF_REGISTERS {
                self.current_register_in_word = 0;
                self.current_word = self
                    .left
                    .next()
                    .map(|left_word| unsafe { (left_word, self.right.next().unwrap_unchecked()) });
            }
            (left_register as u32, right_register as u32)
        })
    }
}

/// Trait marker to associate a specific register array with a combination of precision and bits.
///
/// Meant to be associated with a specific Precision.
pub trait ArrayRegister<B: Bits>: Precision {
    /// The type of the array register associated with the precision and bits.
    type ArrayRegister: Registers<Self, B>;
}

impl<const N: usize, W> Named for [W; N]
{
    fn name(&self) -> String {
        "Array".to_string()
    }
}

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
                    type Iter<'a> = ArrayRegisterIter<'a, [<Precision $exponent>], [<Bits $bits>], Self>;
                    type IterZipped<'a> = ArrayRegisterTupleIter<'a, [<Precision $exponent>], [<Bits $bits>], Self>
                        where
                            Self: 'a;

                    fn zeroed() -> Self {
                        [0; crate::utils::ceil(usize::pow(2, $exponent), 64 / $bits)]
                    }

                    fn iter_registers(&self) -> Self::Iter<'_> {
                        ArrayRegisterIter::new(self)
                    }

                    fn iter_registers_zipped<'a>(&'a self, other: &'a Self) -> Self::IterZipped<'a>{
                        ArrayRegisterTupleIter::new(self, other)
                    }

                    fn get_harmonic_sum_and_zeros<F: FloatNumber>(
                        &self,
                        other: &Self,
                    ) -> (F, <[<Precision $exponent>] as Precision>::NumberOfZeros)
                    where
                    [<Precision $exponent>]: PrecisionConstants<F> {
                        let mut harmonic_sum = F::ZERO;
                        let mut union_zeros = <[<Precision $exponent>] as Precision>::NumberOfZeros::ZERO;

                        for i in 0..self.len() {
                            let mut left = self[i];
                            let mut right = other[i];
                            let mut partial_sum = F::ZERO;
                            let mut partial_zeros = <[<Precision $exponent>] as Precision>::NumberOfZeros::ZERO;

                            for _ in 0..<u64 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS {
                                let left_register = left & <u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK;
                                let right_register = right & <u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK;
                                left >>= [<Bits $bits>]::NUMBER_OF_BITS;
                                right >>= [<Bits $bits>]::NUMBER_OF_BITS;
                                let max_register = left_register.max(right_register);
                                partial_sum += F::inverse_register(max_register as i32);
                                partial_zeros += <[<Precision $exponent>] as Precision>::NumberOfZeros::from_bool(max_register == 0);
                            }
                            harmonic_sum += partial_sum;
                            union_zeros += partial_zeros;
                        }

                        const NUMBER_OF_PADDING_REGISTERS: usize = ceil(<[<Precision $exponent>] as Precision>::NUMBER_OF_REGISTERS, 64 / [<Bits $bits>]::NUMBER_OF_BITS)
                        * <u64 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS
                        - <[<Precision $exponent>] as Precision>::NUMBER_OF_REGISTERS;

                        union_zeros -= unsafe{<[<Precision $exponent>] as Precision>::NumberOfZeros::try_from(NUMBER_OF_PADDING_REGISTERS).unwrap_unchecked()};
                        harmonic_sum -= F::from_usize(NUMBER_OF_PADDING_REGISTERS);

                        (harmonic_sum, union_zeros)
                    }

                    fn apply<F>(&mut self, mut f: F)
                    where
                        F: FnMut(u32) -> u32,
                    {
                        let mut number_of_registers = 0;
                        for word in self.iter_mut() {
                            let word_copy = *word;
                            let mut tmp_word: u64 = 0;
                            for step in 0..<u64 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS {
                                if number_of_registers == [<Precision $exponent>]::NUMBER_OF_REGISTERS {
                                    break;
                                }
                                let register = (word_copy >> (step * [<Bits $bits>]::NUMBER_OF_BITS)) & <u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK;
                                let new_register = f(register as u32) as u64;
                                debug_assert!(new_register < 1 << [<Bits $bits>]::NUMBER_OF_BITS);
                                tmp_word |= new_register << (step * [<Bits $bits>]::NUMBER_OF_BITS);
                                number_of_registers += 1;
                            }
                            *word = tmp_word;
                        }
                    }

                    #[inline(always)]
                    unsafe fn set_greater(&mut self, index: usize, new_register: u32) -> (u32, u32) {
                        debug_assert!(index < [<Precision $exponent>]::NUMBER_OF_REGISTERS);
                        debug_assert!(new_register < 1 << [<Bits $bits>]::NUMBER_OF_BITS);

                        // Calculate the position of the register in the internal buffer array.
                        let word_position = index / <u64 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS;
                        let register_position = index - word_position * <u64 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS;

                        // Extract the current value of the register at `index`.
                        let register_value: u64 =
                            (self[word_position] >> (register_position * [<Bits $bits>]::NUMBER_OF_BITS)) & <u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK;

                        if register_value as u32 >= new_register {
                            return (register_value as u32, register_value as u32);
                        }

                        self[word_position] &= !(<u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK << (register_position * [<Bits $bits>]::NUMBER_OF_BITS));
                        self[word_position] |= (new_register as u64) << (register_position * [<Bits $bits>]::NUMBER_OF_BITS);

                        (register_value as u32, new_register)
                    }

                    fn get_register(&self, index: usize) -> u32 {
                        // Calculate the position of the register in the internal buffer array.
                        let word_position = index / <u64 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS;
                        let register_position = index - word_position * <u64 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS;

                        // Extract the current value of the register at `index`.
                        let register = (self[word_position] >> (register_position * [<Bits $bits>]::NUMBER_OF_BITS)) & <u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK;
                        register as u32
                    }

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

macro_rules! impl_registers_for_precisions {
    ($($exponent: expr),*) => {
        $(
            impl_register_for_precision_and_bits!($exponent, 1, 2, 3, 4, 5, 6, 7, 8);
        )*
    };
}

impl_registers_for_precisions!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);
