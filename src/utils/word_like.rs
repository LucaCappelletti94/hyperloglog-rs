//! Submodule providing the trait WordLike.

pub trait WordLike:
    Copy
    + Ord
    + core::ops::Shl<usize, Output = Self>
    + core::ops::Shr<usize, Output = Self>
    + core::ops::BitAnd<Self, Output = Self>
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
