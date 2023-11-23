//! This module provides a trait to define the ONE const for all unsigned
//! types and the method is_one(). This is not a trait in the core library,
//! but we are aware that it is available in other crates - we do not intend
//! to use them as dependencies, as we want to keep the dependencies to the
//! very bare minimum.

pub trait One {
    /// The one value for this type.
    const ONE: Self;
    /// Whether the value is one.
    fn is_one(&self) -> bool;
}

macro_rules! impl_one {
    ($($t:ty)*) => ($(
        impl One for $t {
            const ONE: Self = 1;
            #[inline(always)]
            fn is_one(&self) -> bool { *self == 1 }
        }
    )*)
}

impl_one! { u8 u16 u32 u64 u128 usize }

macro_rules! impl_one_float {
    ($($t:ty)*) => ($(
        impl One for $t {
            const ONE: Self = 1.0;
            #[inline(always)]
            fn is_one(&self) -> bool { *self == 1.0 }
        }
    )*)
}

impl_one_float! { f32 f64 }
