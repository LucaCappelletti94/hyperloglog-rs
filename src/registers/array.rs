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
    fn new(registers: &'a R) -> Self {
        Self {
            current_register: 0,
            words: registers.words(),
            current_register_in_word: 0,
            current_word: None,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<'a, P: Precision, B: Bits, R: Words + Registers<P, B>> Iterator
    for ArrayRegisterIter<'a, P, B, R>
where
    R::Word: RegisterWord<B>,
{
    type Item = R::Word;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_register == P::NUMBER_OF_REGISTERS {
            return None;
        }
        if self.current_word.is_none() {
            self.current_word = self.words.next();
        }

        self.current_word.map(|word| {
            let register: R::Word = (word >> (self.current_register_in_word * B::NUMBER_OF_BITS))
                & R::Word::LOWER_REGISTER_MASK;
            self.current_register_in_word += 1;
            self.current_register += 1;
            if self.current_register_in_word == 32 / B::NUMBER_OF_BITS {
                self.current_register_in_word = 0;
                self.current_word = None;
            }
            register
        })
    }
}

/// Trait marker to associate a specific register array with a combination of precision and bits.
///
/// Meant to be associated with a specific Precision.
pub trait ArrayRegister<B: Bits>: Precision {
    type ArrayRegister: Registers<Self, B>;

    fn initialize_with(word: u32) -> Self::ArrayRegister;
}

macro_rules! impl_register_for_precision_and_bits {
    ($exponent: expr, $($bits: expr),*) => {
        $(
            paste::paste! {
                impl ArrayRegister<[<Bits $bits>]> for [<Precision $exponent>] {
                    type ArrayRegister = [u32; crate::utils::ceil(usize::pow(2, $exponent), 32 / $bits)];

                    fn initialize_with(word: u32) -> Self::ArrayRegister {
                        [word; crate::utils::ceil(usize::pow(2, $exponent), 32 / $bits)]
                    }
                }

                impl Registers<[<Precision $exponent>], [<Bits $bits>]> for [u32; crate::utils::ceil(usize::pow(2, $exponent), 32 / $bits)] {
                    type Iter<'a> = ArrayRegisterIter<'a, [<Precision $exponent>], [<Bits $bits>], Self>;

                    fn zeroed() -> Self {
                        [0; crate::utils::ceil(usize::pow(2, $exponent), 32 / $bits)]
                    }

                    fn iter_registers(&self) -> Self::Iter<'_> {
                        ArrayRegisterIter::new(self)
                    }

                    fn apply<F>(&mut self, mut f: F)
                    where
                        F: FnMut(u32) -> u32,
                    {
                        let mut number_of_registers = 0;
                        for word in self.iter_mut() {
                            let word_copy = *word;
                            let mut tmp_word = 0;
                            for step in 0..<u32 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS {
                                if number_of_registers == [<Precision $exponent>]::NUMBER_OF_REGISTERS {
                                    break;
                                }
                                let register = (word_copy >> (step * [<Bits $bits>]::NUMBER_OF_BITS)) & <u32 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK;
                                let new_register = f(register);
                                debug_assert!(new_register < 1 << [<Bits $bits>]::NUMBER_OF_BITS);
                                tmp_word |= new_register << (step * [<Bits $bits>]::NUMBER_OF_BITS);
                                number_of_registers += 1;
                            }
                            *word = tmp_word;
                        }
                    }

                    unsafe fn set_greater(&mut self, index: usize, value: u32) -> Option<u32> {
                        // Calculate the position of the register in the internal buffer array.
                        let word_position = index / <u32 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS;
                        let register_position = index - word_position * <u32 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS;

                        // Extract the current value of the register at `index`.
                        let register_value: u32 =
                            (self[word_position] >> (register_position * [<Bits $bits>]::NUMBER_OF_BITS)) & <u32 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK;

                        // Otherwise, update the register using a bit mask.
                        (value > register_value).then(||{
                            self[word_position] &= !(<u32 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK << (register_position * [<Bits $bits>]::NUMBER_OF_BITS));
                            self[word_position] |= value << (register_position * [<Bits $bits>]::NUMBER_OF_BITS);
                            register_value
                        })
                    }

                    fn get_register(&self, index: usize) -> u32 {
                        // Calculate the position of the register in the internal buffer array.
                        let word_position = index / <u32 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS;
                        let register_position = index - word_position * <u32 as RegisterWord<[<Bits $bits>]>>::NUMBER_OF_REGISTERS;

                        // Extract the current value of the register at `index`.
                        (self[word_position] >> (register_position * [<Bits $bits>]::NUMBER_OF_BITS)) & <u32 as RegisterWord<[<Bits $bits>]>>::LOWER_REGISTER_MASK
                    }
                }
            }
        )*
    };
}

macro_rules! impl_registers_for_precisions {
    ($($exponent: expr),*) => {
        $(
            impl_register_for_precision_and_bits!($exponent, 1, 2, 3, 4, 5, 6);
        )*
    };
}

impl_registers_for_precisions!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

#[cfg(test)]
mod tests {
    use core::u32;

    use super::*;

    fn test_register_iterator<P: Precision + ArrayRegister<B>, B: Bits>() {
        let mut registers = <P as ArrayRegister<B>>::initialize_with(0);
        let collected_values = registers.iter_registers().collect::<Vec<_>>();
        assert_eq!(collected_values.len(), P::NUMBER_OF_REGISTERS);
        // All the values should be zeroed.
        assert!(collected_values.iter().all(|&value| value == 0));
        // We check that each collected value is identical to what we obtain using the get method.
        assert!(collected_values
            .iter()
            .enumerate()
            .all(|(index, &value)| value == registers.get_register(index)));

        // We check that, given all registers are currently zeroed, when we set them to the maximum value
        // we get always returned a value and that value is equal to zero.
        for index in 0..P::NUMBER_OF_REGISTERS {
            let old_value = unsafe {
                registers.set_greater(index, <u32 as RegisterWord<B>>::LOWER_REGISTER_MASK)
            };
            assert_eq!(old_value, Some(0));
            // If we try to do it again, we should receive a None
            let old_value = unsafe {
                registers.set_greater(index, <u32 as RegisterWord<B>>::LOWER_REGISTER_MASK)
            };
            assert_eq!(old_value, None);
        }

        // ==========================================

        let mut registers = <P as ArrayRegister<B>>::initialize_with(u32::MAX);
        let collected_values = registers.iter_registers().collect::<Vec<_>>();
        assert_eq!(collected_values.len(), P::NUMBER_OF_REGISTERS);
        // All the values should be the maximum value allowed by the register,
        // as determined by the number of bits.
        let expected_value = <u32 as RegisterWord<B>>::LOWER_REGISTER_MASK;
        assert!(collected_values
            .iter()
            .all(|&value| value == expected_value));
        // We check that each collected value is identical to what we obtain using the get method.
        assert!(collected_values
            .iter()
            .enumerate()
            .all(|(index, &value)| value == registers.get_register(index)));

        // We check that, given all registers are currently maxxed, when we set them to the maximum value
        // we get always returned None.
        for index in 0..P::NUMBER_OF_REGISTERS {
            let old_value = unsafe {
                registers.set_greater(index, <u32 as RegisterWord<B>>::LOWER_REGISTER_MASK)
            };
            assert_eq!(old_value, None);
        }

        // ==================================================

        if B::NUMBER_OF_BITS == 1 || B::NUMBER_OF_BITS == 2 || B::NUMBER_OF_BITS == 4 {
            // We prepare now a test to create a word that has alternated values of 0 and max value.
            let word: u32 = match B::NUMBER_OF_BITS {
                1 => 0b0101_0101_0101_0101_0101_0101_0101_0101,
                2 => 0b0011_0011_0011_0011_0011_0011_0011_0011,
                4 => 0b0000_1111_0000_1111_0000_1111_0000_1111,
                _ => unreachable!(),
            };

            let registers = <P as ArrayRegister<B>>::initialize_with(word);
            let collected_values = registers.iter_registers().collect::<Vec<_>>();
            assert_eq!(collected_values.len(), P::NUMBER_OF_REGISTERS);
            // We check that the values are alternating between 0 and the maximum value.
            let expected_values = (0..P::NUMBER_OF_REGISTERS)
                .map(|index| {
                    if index % 2 == 0 {
                        <u32 as RegisterWord<B>>::LOWER_REGISTER_MASK
                    } else {
                        0
                    }
                })
                .collect::<Vec<_>>();
            assert!(collected_values
                .iter()
                .copied()
                .zip(expected_values)
                .all(|(a, b)| a == b));
            // We check that each collected value is identical to what we obtain using the get method.
            assert!(collected_values
                .iter()
                .enumerate()
                .all(|(index, &value)| value == registers.get_register(index)));
        }
    }

    macro_rules! test_register_iterator {
        ($precision:ty, $($bits:ty),*) => {
            $(
                paste::item! {
                    #[test]
                    fn [< test_register_iterator_ $precision:lower _and_ $bits:lower _bits >]() {
                        test_register_iterator::<$precision, $bits>();
                    }
                }
            )*
        };
    }

    macro_rules! test_register_iterators_by_precision {
        ($($precision:ty),*) => {
            $(
                test_register_iterator!($precision, Bits1, Bits2, Bits3, Bits4, Bits5, Bits6);
            )*
        };
    }

    test_register_iterators_by_precision!(
        Precision4,
        Precision5,
        Precision6,
        Precision7,
        Precision8,
        Precision9,
        Precision10,
        Precision11,
        Precision12,
        Precision13,
        Precision14,
        Precision15,
        Precision16
    );
}
