//! Submodule implementing the registers trait for the array data structure.
use super::*;

pub struct ArrayRegisterIter<'a, P: Precision, B: Bits, R: Words + Registers<P, B>>
where
    R: 'a,
{
    current_register: P::NumberOfRegisters,
    current_register_in_word: u8,
    words: R::WordIter<'a>,
    current_word: Option<R::Word>,
    _phantom: core::marker::PhantomData<(P, B, R)>,
}

impl<'a, P: Precision, B: Bits, R: Words + Registers<P, B>> ArrayRegisterIter<'a, P, B, R> {
    pub(super) fn new(registers: &'a R) -> Self {
        let mut words = registers.words();
        let current_word = words.next();
        Self {
            current_register: P::NumberOfRegisters::ZERO,
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
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_register == P::NUMBER_OF_REGISTERS {
            return None;
        }

        self.current_word.map(|word| {
            let [register] = extract_register_from_word(
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

pub struct ArrayRegisterTupleIter<'a, P: Precision, B: Bits, R: Words + Registers<P, B>>
where
    R: 'a,
{
    current_register: P::NumberOfRegisters,
    current_register_in_word: u8,
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
            .and_then(|left_word| right.next().map(|right_word| (left_word, right_word)));
        Self {
            current_register: P::NumberOfRegisters::ZERO,
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
    type Item = (u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_register == P::NUMBER_OF_REGISTERS {
            return None;
        }

        self.current_word.map(|(left, right)| {
            let [left_register, right_register] = extract_register_from_word(
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

impl<const N: usize, W> Named for [W; N] {
    fn name(&self) -> String {
        "Array".to_string()
    }
}

fn split_index<P: Precision, B: Bits>(index: P::NumberOfRegisters) -> (usize, u8)
where
    u64: RegisterWord<B>,
{
    let mask: u64 = <u64 as RegisterWord<B>>::NUMBER_OF_REGISTERS_IN_WORD.into();
    let word_position: u64 = index.into() / mask;
    let register_position: u8 = u8::try_from(index.into() - word_position * mask).unwrap();
    (usize::try_from(word_position).unwrap(), register_position)
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

                    fn get_harmonic_sum_and_zeros<F: Float>(
                        &self,
                        other: &Self,
                    ) -> (F, <[<Precision $exponent>] as Precision>::NumberOfRegisters)
                    where
                    [<Precision $exponent>]: PrecisionConstants<F> {
                        let mut harmonic_sum = F::ZERO;
                        let mut union_zeros = <[<Precision $exponent>] as Precision>::NumberOfRegisters::ZERO;

                        for i in 0..self.len() {
                            let left = self[i];
                            let right = other[i];
                            let mut partial_sum = F::ZERO;
                            let mut partial_zeros = <[<Precision $exponent>] as Precision>::NumberOfRegisters::ZERO;

                            for step in 0..<u64 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS_IN_WORD {
                                let [left_register, right_register] = extract_register_from_word::<[<Bits $bits>], 2, u64>([left, right], step * [<Bits $bits>]::NUMBER_OF_BITS);
                                let max_register = left_register.max(right_register);
                                partial_sum += F::exp2_minus(max_register);
                                partial_zeros += <[<Precision $exponent>] as Precision>::NumberOfRegisters::from_bool(max_register == 0);
                            }
                            harmonic_sum += partial_sum;
                            union_zeros += partial_zeros;
                        }

                        let number_of_padding_registers: u8 = u8::try_from((self.len() as u64)
                        * u64::from(<u64 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS_IN_WORD)
                        - u64::from(<[<Precision $exponent>] as Precision>::NUMBER_OF_REGISTERS)).unwrap();

                        union_zeros -= <[<Precision $exponent>] as Precision>::NumberOfRegisters::from(number_of_padding_registers);
                        harmonic_sum -= F::from(number_of_padding_registers);

                        (harmonic_sum, union_zeros)
                    }

                    fn apply<F>(&mut self, mut f: F)
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
                                let [register] = extract_register_from_word::<[<Bits $bits>], 1, u64>([word_copy], step * [<Bits $bits>]::NUMBER_OF_BITS);
                                let new_register = f(register);
                                debug_assert!(new_register < 1 << [<Bits $bits>]::NUMBER_OF_BITS);
                                tmp_word |= u64::from(new_register) << (step * [<Bits $bits>]::NUMBER_OF_BITS);
                                number_of_registers += <[<Precision $exponent>] as Precision>::NumberOfRegisters::ONE;
                            }
                            *word = tmp_word;
                        }
                    }

                    #[inline(always)]
                    fn set_greater(&mut self, index: <[<Precision $exponent>] as Precision>::NumberOfRegisters, new_register: u8) -> (u8, u8) {
                        debug_assert!(index < [<Precision $exponent>]::NUMBER_OF_REGISTERS);
                        debug_assert!(new_register < 1 << [<Bits $bits>]::NUMBER_OF_BITS);

                        // Calculate the position of the register in the internal buffer array.
                        let (word_position, register_position) = split_index::<[<Precision $exponent>], [<Bits $bits>]>(index);

                        // Extract the current value of the register at `index`.
                        let register_value: u8 =
                            u8::try_from((self[word_position] >> (register_position * [<Bits $bits>]::NUMBER_OF_BITS)) & <u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK).unwrap();

                        if register_value >= new_register {
                            return (register_value , register_value );
                        }

                        self[word_position] &= !(<u64 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK << (register_position * [<Bits $bits>]::NUMBER_OF_BITS));
                        self[word_position] |= (new_register as u64) << (register_position * [<Bits $bits>]::NUMBER_OF_BITS);

                        (register_value, new_register)
                    }

                    fn get_register(&self, index: <[<Precision $exponent>] as Precision>::NumberOfRegisters) -> u8 {
                        // Calculate the position of the register in the internal buffer array.
                        let (word_position, register_position) = split_index::<[<Precision $exponent>], [<Bits $bits>]>(index);

                        // Extract the current value of the register at `index`.
                        extract_register_from_word::<[<Bits $bits>], 1, u64>([self[word_position]], register_position * [<Bits $bits>]::NUMBER_OF_BITS)[0]
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
