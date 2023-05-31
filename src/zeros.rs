//! This module provides a trait to define the ZERO const for all unsigned
//! types and the method is_zero(). This is not a trait in the core library,
//! but we are aware that it is available in other crates - we do not intend
//! to use them as dependencies, as we want to keep the dependencies to the
//! very bare minimum.

pub trait Zero {
    /// The zero value for this type.
    const ZERO: Self;
    /// Whether the value is zero.
    fn is_zero(&self) -> bool;
}

macro_rules! impl_zero {
    ($($t:ty)*) => ($(
        impl Zero for $t {
            const ZERO: Self = 0;
            #[inline(always)]
            fn is_zero(&self) -> bool { *self == 0 }
        }
    )*)
}

impl_zero! { u8 u16 u32 u64 u128 usize }