//! Submodule providing the trait WordLike.

use super::Number;

pub trait WordLike:
    Copy
    + Ord
    + Number
    + core::ops::Shl<usize, Output = Self>
    + core::ops::Shr<usize, Output = Self>
    + core::ops::BitAnd<Self, Output = Self>
    + core::ops::BitOr<Self, Output = Self>
{
}

macro_rules! impl_word_like {
    ($($t:ty),*) => {
        $(
            impl WordLike for $t {}
        )*
    };
}

impl_word_like!(u8, u16, u32, u64, u128);
