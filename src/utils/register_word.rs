//! Submodule providing the implementation of the lower register mask for several word types.
use crate::prelude::*;

pub trait RegisterWord<B: Bits> {
    const LOWER_REGISTER_MASK: Self;
    const NUMBER_OF_BITS: usize;
    const NUMBER_OF_REGISTERS: usize = Self::NUMBER_OF_BITS / B::NUMBER_OF_BITS;
}

macro_rules! impl_lower_register_mask {
    ($($t:ty),*) => {
        $(
            impl<B: Bits> RegisterWord<B> for $t {
                const LOWER_REGISTER_MASK: Self = (1 << B::NUMBER_OF_BITS) - 1;
                const NUMBER_OF_BITS: usize = core::mem::size_of::<Self>() * 8;
            }
        )*
    };
}

impl_lower_register_mask!(u8, u16, u32, u64, u128);
