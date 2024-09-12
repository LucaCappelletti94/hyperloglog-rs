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

/// Macro implementing several constants for integers.
macro_rules! impl_constants {
    ($($t:ty)*) => ($(
        impl One for $t {
            const ONE: Self = 1;
            #[inline]
            fn is_one(&self) -> bool { *self == 1 }
        }
        impl Zero for $t {
            const ZERO: Self = 0;
            #[inline]
            fn is_zero(&self) -> bool { *self == 0 }
        }
    )*)
}

impl_constants! { u8 u16 u32 u64 usize }
impl_constants! { i32 }

impl One for f64 {
    const ONE: Self = 1.0;
    #[inline]
    fn is_one(&self) -> bool {
        let delta = *self - 1.0;
        if delta < 0.0 {
            delta > -f64::EPSILON
        } else {
            delta < f64::EPSILON
        }
    }
}
impl Zero for f64 {
    const ZERO: Self = 0.0;
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0.0
    }
}
