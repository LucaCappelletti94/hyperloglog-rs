//! Submodule providing the implementation of the lower register mask for several word types.
use crate::prelude::*;

pub trait RegisterWord<B: Bits> {
    const LOWER_REGISTER_MASK: Self;
    const NUMBER_OF_BITS: u8;
    const NUMBER_OF_REGISTERS_IN_WORD: u8 = Self::NUMBER_OF_BITS / B::NUMBER_OF_BITS;
}

impl<B: Bits> RegisterWord<B> for u64 {
    const LOWER_REGISTER_MASK: Self = (1 << B::NUMBER_OF_BITS) - 1;
    const NUMBER_OF_BITS: u8 = 64;
}