//! Register implementation for the vector registers.
use super::array::*;
use crate::prelude::*;

impl<W> Named for Vec<W> {
    fn name(&self) -> String {
        "Vec".to_string()
    }
}

impl<P: Precision, B: Bits> Registers<P, B> for Vec<u64> {
    type Iter<'a> = ArrayRegisterIter<'a, P, B, Self>;
    type IterZipped<'a> = ArrayRegisterTupleIter<'a, P, B, Self>
        where
            Self: 'a;

    fn zeroed() -> Self {
        vec![0; crate::utils::ceil(usize::pow(2, P::EXPONENT as u32), 64 / B::NUMBER_OF_BITS)]
    }

    fn iter_registers(&self) -> Self::Iter<'_> {
        ArrayRegisterIter::new(self)
    }

    fn iter_registers_zipped<'a>(&'a self, other: &'a Self) -> Self::IterZipped<'a> {
        ArrayRegisterTupleIter::new(self, other)
    }

    fn get_harmonic_sum_and_zeros<F: FloatNumber>(
        &self,
        other: &Self,
    ) -> (F, <P as Precision>::NumberOfZeros)
    where
        P: PrecisionConstants<F>,
    {
        let mut harmonic_sum = F::ZERO;
        let mut union_zeros = <P as Precision>::NumberOfZeros::ZERO;

        for i in 0..self.len() {
            let mut left = self[i];
            let mut right = other[i];
            let mut partial_sum = F::ZERO;
            let mut partial_zeros = <P as Precision>::NumberOfZeros::ZERO;

            for _ in 0..<u64 as RegisterWord<B>>::NUMBER_OF_REGISTERS {
                let left_register = left & <u64 as RegisterWord<B>>::LOWER_REGISTER_MASK;
                let right_register = right & <u64 as RegisterWord<B>>::LOWER_REGISTER_MASK;
                left >>= B::NUMBER_OF_BITS;
                right >>= B::NUMBER_OF_BITS;
                let max_register = left_register.max(right_register);
                partial_sum += F::inverse_register(max_register as i32);
                partial_zeros += <P as Precision>::NumberOfZeros::from_bool(max_register == 0);
            }
            harmonic_sum += partial_sum;
            union_zeros += partial_zeros;
        }

        let number_of_padding_registers = self.len() * <u64 as RegisterWord<B>>::NUMBER_OF_REGISTERS
            - P::NUMBER_OF_REGISTERS;

        union_zeros -= 
            <P as Precision>::NumberOfZeros::from_usize(number_of_padding_registers);
        harmonic_sum -= F::from_usize(number_of_padding_registers);

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
            for step in 0..<u64 as RegisterWord<B>>::NUMBER_OF_REGISTERS {
                if number_of_registers == P::NUMBER_OF_REGISTERS {
                    break;
                }
                let register = (word_copy >> (step * B::NUMBER_OF_BITS))
                    & <u64 as RegisterWord<B>>::LOWER_REGISTER_MASK;
                let new_register = f(register as u32) as u64;
                debug_assert!(new_register < 1 << B::NUMBER_OF_BITS);
                tmp_word |= new_register << (step * B::NUMBER_OF_BITS);
                number_of_registers += 1;
            }
            *word = tmp_word;
        }
    }

    #[inline(always)]
    fn set_greater(&mut self, index: usize, new_register: u32) -> (u32, u32) {
        debug_assert!(index < P::NUMBER_OF_REGISTERS);
        debug_assert!(new_register < 1 << B::NUMBER_OF_BITS);

        // Calculate the position of the register in the internal buffer array.
        let word_position = index / <u64 as RegisterWord<B>>::NUMBER_OF_REGISTERS;
        let register_position =
            index - word_position * <u64 as RegisterWord<B>>::NUMBER_OF_REGISTERS;

        // Extract the current value of the register at `index`.
        let register_value: u64 = (self[word_position] >> (register_position * B::NUMBER_OF_BITS))
            & <u64 as RegisterWord<B>>::LOWER_REGISTER_MASK;

        if register_value as u32 >= new_register {
            return (register_value as u32, register_value as u32);
        }

        self[word_position] &= !(<u64 as RegisterWord<B>>::LOWER_REGISTER_MASK
            << (register_position * B::NUMBER_OF_BITS));
        self[word_position] |= (new_register as u64) << (register_position * B::NUMBER_OF_BITS);

        (register_value as u32, new_register)
    }

    fn get_register(&self, index: usize) -> u32 {
        // Calculate the position of the register in the internal buffer array.
        let word_position = index / <u64 as RegisterWord<B>>::NUMBER_OF_REGISTERS;
        let register_position =
            index - word_position * <u64 as RegisterWord<B>>::NUMBER_OF_REGISTERS;

        // Extract the current value of the register at `index`.
        let register = (self[word_position] >> (register_position * B::NUMBER_OF_BITS))
            & <u64 as RegisterWord<B>>::LOWER_REGISTER_MASK;
        register as u32
    }

    fn clear(&mut self) {
        for word in self.iter_mut() {
            *word = 0;
        }
    }
}
