//! Constants for common values.

/// The zero value for this type.
pub trait Zero {
    /// The zero value for this type.
    const ZERO: Self;
    /// Whether the value is zero.
    fn is_zero(&self) -> bool;
}

/// The one value for this type.
pub trait One {
    /// The one value for this type.
    const ONE: Self;
    /// Whether the value is one.
    fn is_one(&self) -> bool;
}

/// The two value for this type.
pub trait Two {
    /// The two value for this type.
    const TWO: Self;
}

/// The five value for this type.
pub trait Five {
    /// The five value for this type.
    const FIVE: Self;
}

/// The ten value for this type.
pub trait Ten {
    /// The ten value for this type.
    const TEN: Self;
}

/// Macro implementing several constants for integers.
macro_rules! impl_constants {
    ($($t:ty)*) => ($(
        impl One for $t {
            const ONE: Self = 1;
            #[inline(always)]
            fn is_one(&self) -> bool { *self == 1 }
        }
        impl Zero for $t {
            const ZERO: Self = 0;
            #[inline(always)]
            fn is_zero(&self) -> bool { *self == 0 }
        }
        impl Two for $t {
            const TWO: Self = 2;
        }
        impl Five for $t {
            const FIVE: Self = 5;
        }
        impl Ten for $t {
            const TEN: Self = 10;
        }
    )*)
}

impl_constants! { u8 u16 u32 u64 u128 usize }
impl_constants! { i8 i16 i32 i64 i128 isize }


impl One for f64 {
    const ONE: Self = 1.0;
    #[inline]
    fn is_one(&self) -> bool { (*self - 1.0).abs() < Self::EPSILON }
}
impl Zero for f64 {
    const ZERO: Self = 0.0;
    #[inline]
    fn is_zero(&self) -> bool { *self == 0.0 }
}
impl Two for f64 {
    const TWO: Self = 2.0;
}
impl Five for f64 {
    const FIVE: Self = 5.0;
}
impl Ten for f64 {
    const TEN: Self = 10.0;
}