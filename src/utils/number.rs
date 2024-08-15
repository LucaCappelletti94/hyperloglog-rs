//! Traits regarding numbers.
use crate::utils::*;

/// A trait for numbers.
pub trait Number:
    Copy
    + Default
    + core::ops::Add<Self, Output = Self>
    + core::ops::Sub<Self, Output = Self>
    + core::ops::Div<Self, Output = Self>
    + core::ops::Mul<Self, Output = Self>
    + core::ops::MulAssign
    + core::ops::AddAssign
    + core::ops::SubAssign
    + core::fmt::Debug
    + core::fmt::Display
    + crate::utils::Zero
    + crate::utils::One
    + crate::utils::Two
    + crate::utils::Three
    + crate::utils::Five
    + crate::utils::Six
    + crate::utils::Seven
    + crate::utils::Eight
    + crate::utils::Nine
    + crate::utils::Ten
    + crate::utils::OneHundred
    + core::iter::Sum
    + PartialOrd
    + Send
    + Sync
{
    /// A method to subtract the second number from the first number, returning zero if the result is negative.
    fn saturating_zero_sub(self, other: Self) -> Self;

    /// Converts the number to a 32-bit float.
    fn to_f32(self) -> f32;

    /// Converts the number to a 64-bit float.
    fn to_f64(self) -> f64;

    #[inline(always)]
    #[must_use]
    /// Converts a boolean value to a number.
    fn from_bool(value: bool) -> Self {
        if value {
            Self::ONE
        } else {
            Self::ZERO
        }
    }
}

/// A trait for positive integer numbers.
pub trait PositiveIntegerNumber: Number + Eq + Ord + TryInto<usize> + TryFrom<usize> {}

/// A trait for floating point numbers.
pub trait FloatNumber:
    Number + Half + crate::utils::OneThousand + core::ops::Neg<Output = Self>
{
    /// The value of positive infinity.
    const INFINITY: Self;
    /// The value of negative infinity.
    const NEG_INFINITY: Self;
    /// The value of epsilon.
    const EPSILON: Self;

    /// Returns the value of 2^(-register).
    fn inverse_register(register: i32) -> Self;

    #[inline(always)]
    #[must_use]
    /// Computes the saturating division of two numbers that are expected to be positive.
    /// and at most equal to one.
    fn saturating_one_div(self, other: Self) -> Self {
        debug_assert!(self >= Self::ZERO);
        debug_assert!(other >= Self::ZERO);
        if self >= other {
            Self::ONE
        } else {
            self / other
        }
    }

    /// Converts an `i32` to a floating point number.
    fn from_i32(value: i32) -> Self;

    /// Converts a `usize` to a floating point number.
    fn from_usize(value: usize) -> Self;

    /// Converts a `f64` to a floating point number.
    fn from_f64(value: f64) -> Self;

    /// Converts a `f32` to a floating point number.
    fn from_f32(value: f32) -> Self;

    /// Converts the floating point number to a `usize`.
    fn to_u32(self) -> u32;

    /// Converts the floating point number to a `usize`.
    fn to_usize(self) -> usize;

    /// Checks if the floating point number is finite.
    fn is_finite(self) -> bool;

    #[cfg(feature = "std")]
    /// Returns the absolute value of the floating point number.
    fn abs(self) -> Self;

    #[cfg(feature = "std")]
    /// Returns the natural logarithm of the floating point number.
    fn ln(self) -> Self;

    #[cfg(feature = "std")]
    /// Returns the natural logarithm of 1 plus the floating point number.
    fn ln_1p(self) -> Self;

    #[cfg(feature = "std")]
    /// Returns the base 2 logarithm of the floating point number.
    fn log2(self) -> Self;

    #[cfg(feature = "std")]
    /// Returns the exponential of the floating point number.
    fn exp(self) -> Self;

    #[cfg(feature = "std")]
    /// Returns the square root of the floating point number.
    fn sqrt(self) -> Self;

    #[cfg(feature = "std")]
    /// Returns the floating point number raised to the power of an integer.
    fn powi(self, n: i32) -> Self;

    #[cfg(feature = "std")]
    /// Returns the floor of the floating point number.
    fn floor(self) -> Self;

    #[cfg(feature = "std")]
    /// Returns the exponential of the floating point number minus one.
    fn exp_m1(self) -> Self;
}

macro_rules! impl_number {
    ($($t:ty),*) => {
        $(
            impl Number for $t {
                #[inline(always)]
                fn saturating_zero_sub(self, other: Self) -> Self {
                    debug_assert!(self >= Self::ZERO);
                    debug_assert!(other >= Self::ZERO);
                    if self < other {
                        Self::ZERO
                    } else {
                        self - other
                    }
                }

                #[inline(always)]
                fn to_f32(self) -> f32 {
                    self as f32
                }

                #[inline(always)]
                fn to_f64(self) -> f64 {
                    self as f64
                }
            }
        )*
    };
}

impl_number!(u8, u16, u32, u64, u128, usize);
impl_number!(i8, i16, i32, i64, i128, isize);
impl_number!(f32, f64);

macro_rules! impl_positive_integer_number {
    ($($t:ty),*) => {
        $(
            impl PositiveIntegerNumber for $t {}
        )*
    };
}

impl_positive_integer_number!(u8, u16, u32, u64, u128, usize);

impl FloatNumber for f32 {
    const INFINITY: Self = f32::INFINITY;
    const NEG_INFINITY: Self = f32::NEG_INFINITY;
    const EPSILON: Self = f32::EPSILON;

    #[inline(always)]
    #[must_use]
    fn inverse_register(register: i32) -> Self {
        f32::from_le_bytes(((127 - register) << 23).to_le_bytes())
    }

    #[inline(always)]
    #[must_use]
    fn from_i32(value: i32) -> Self {
        value as Self
    }

    #[inline(always)]
    #[must_use]
    fn to_u32(self) -> u32 {
        self as u32
    }

    #[inline(always)]
    #[must_use]
    fn is_finite(self) -> bool {
        self.is_finite()
    }

    #[inline(always)]
    #[must_use]
    fn from_usize(value: usize) -> Self {
        value as Self
    }

    #[inline(always)]
    #[must_use]
    fn from_f64(value: f64) -> Self {
        value as Self
    }

    #[inline(always)]
    #[must_use]
    fn from_f32(value: f32) -> Self {
        value as Self
    }

    #[inline(always)]
    #[must_use]
    fn to_usize(self) -> usize {
        self as usize
    }

    #[cfg(feature = "std")]
    #[must_use]
    fn abs(self) -> Self {
        self.abs()
    }

    #[cfg(feature = "std")]
    #[must_use]
    fn ln(self) -> Self {
        self.ln()
    }

    #[cfg(feature = "std")]
    #[must_use]
    fn ln_1p(self) -> Self {
        self.ln_1p()
    }

    #[cfg(feature = "std")]
    #[must_use]
    fn log2(self) -> Self {
        self.log2()
    }

    #[cfg(feature = "std")]
    #[must_use]
    fn exp(self) -> Self {
        self.exp()
    }

    #[cfg(feature = "std")]
    #[must_use]
    fn sqrt(self) -> Self {
        self.sqrt()
    }

    #[cfg(feature = "std")]
    #[must_use]
    fn powi(self, n: i32) -> Self {
        self.powi(n)
    }

    #[cfg(feature = "std")]
    #[must_use]
    fn floor(self) -> Self {
        self.floor()
    }

    #[cfg(feature = "std")]
    #[must_use]
    fn exp_m1(self) -> Self {
        self.exp_m1()
    }
}
impl FloatNumber for f64 {
    const INFINITY: Self = f64::INFINITY;
    const NEG_INFINITY: Self = f64::NEG_INFINITY;
    const EPSILON: Self = f64::EPSILON;

    #[inline(always)]
    fn inverse_register(register: i32) -> Self {
        f64::from_le_bytes(((1023 - register as i64) << 52).to_le_bytes())
    }

    #[inline(always)]
    fn from_i32(value: i32) -> Self {
        value as Self
    }

    #[inline(always)]
    fn to_u32(self) -> u32 {
        self as u32
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }

    #[inline(always)]
    fn is_finite(self) -> bool {
        self.is_finite()
    }

    #[inline(always)]
    fn from_f64(value: f64) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_f32(value: f32) -> Self {
        value as Self
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }

    #[inline(always)]
    #[must_use]
    #[cfg(feature = "std")]
    fn abs(self) -> Self {
        self.abs()
    }

    #[inline(always)]
    #[must_use]
    #[cfg(feature = "std")]
    fn ln(self) -> Self {
        self.ln()
    }

    #[inline(always)]
    #[must_use]
    #[cfg(feature = "std")]
    fn ln_1p(self) -> Self {
        self.ln_1p()
    }

    #[inline(always)]
    #[must_use]
    #[cfg(feature = "std")]
    fn log2(self) -> Self {
        self.log2()
    }

    #[inline(always)]
    #[must_use]
    #[cfg(feature = "std")]
    fn exp(self) -> Self {
        self.exp()
    }

    #[inline(always)]
    #[must_use]
    #[cfg(feature = "std")]
    fn sqrt(self) -> Self {
        self.sqrt()
    }

    #[inline(always)]
    #[must_use]
    #[cfg(feature = "std")]
    fn powi(self, n: i32) -> Self {
        self.powi(n)
    }

    #[inline(always)]
    #[must_use]
    #[cfg(feature = "std")]
    fn floor(self) -> Self {
        self.floor()
    }

    #[inline(always)]
    #[must_use]
    #[cfg(feature = "std")]
    fn exp_m1(self) -> Self {
        self.exp_m1()
    }
}

#[cfg(test)]
mod test_inverse_register {
    use super::*;

    #[test]
    fn test_inverse_register() {
        // At the most, we create registers with 6 bits, which
        // means that the maximum values is 2^7 - 1 = 127.
        for precision in 4..=16 {
            for bits in 1..=6 {
                for register_value in 0..=maximal_multeplicity(precision, bits) {
                    assert_eq!(
                        2.0_f32.powf(-(register_value as f32)),
                        f32::inverse_register(register_value as i32),
                        "Expected: 2^(-{}), Got: {}",
                        register_value,
                        f32::inverse_register(register_value as i32)
                    );
                    assert_eq!(
                        2.0_f64.powf(-(register_value as f64)),
                        f64::inverse_register(register_value as i32),
                        "Expected: 2^(-{}), Got: {}",
                        register_value,
                        f64::inverse_register(register_value as i32)
                    );
                    assert_eq!(
                        f64::from_bits(
                            u64::max_value().wrapping_sub(u64::from(register_value as u64)) << 54
                                >> 2
                        ),
                        f64::inverse_register(register_value as i32),
                        "Expected: 2^(-{}), Got: {}",
                        register_value,
                        f64::inverse_register(register_value as i32)
                    );
                }
            }
        }
    }
}
