//! Traits regarding numbers.
use core::ops::{Shl, Shr};

use crate::utils::{Half, Zero};

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
    + From<u8>
    + core::iter::Sum
    + PartialOrd
    + Send
    + Sync
{
    #[must_use]
    /// A method to subtract the second number from the first number, returning zero if the result is negative.
    fn saturating_zero_sub(self, other: Self) -> Self;

    /// Converts the number to a 32-bit float.
    fn to_f32(self) -> f32;

    /// Converts the number to a 64-bit float.
    fn to_f64(self) -> f64;

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
pub trait PositiveInteger:
    Number + Eq + Into<u64> + Ord + Shl<u8, Output = Self> + Shr<u8, Output = Self>
{
    /// The error type for the `try_from_u64` method.
    type TryFromU64Error: core::fmt::Debug;
    /// The error type for the `try_from_usize` method.
    type TryFromUsizeError: core::fmt::Debug;

    /// Converts a `u64` to a positive integer number.
    fn try_from_u64(value: u64) -> Result<Self, Self::TryFromU64Error>;

    /// Converts a `usize` to a positive integer number.
    fn try_from_usize(value: usize) -> Result<Self, Self::TryFromUsizeError>;

    /// Converts the positive integer number to a `usize`.
    fn to_usize(self) -> usize;
}

/// A trait for floating point numbers.
pub trait Float: Number + Half + crate::utils::OneThousand + core::ops::Neg<Output = Self> {
    /// The value of positive infinity.
    const INFINITY: Self;
    /// The value of negative infinity.
    const NEG_INFINITY: Self;
    /// The value of epsilon.
    const EPSILON: Self;

    /// Returns the value of 2^(-register), with strict positivite register.
    fn exp2_minus(register: u8) -> Self;

    /// Returns the value of 2^(-register), including negative registers.
    fn exp2_minus_signed(register: i8) -> Self;

    /// Returns the value of 2^(register)
    fn exp2(register: u8) -> Self;

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

    /// Converts a `f64` to a floating point number.
    fn from_f64(value: f64) -> Self;

    /// Converts a `f32` to a floating point number.
    fn from_f32(value: f32) -> Self;

    /// Converts the floating point number to a `usize`.
    fn to_u32(self) -> u32;

    /// Converts the provided usize to a floating point number
    /// with the same value, checking that the conversion is lossless.
    fn from_usize_checked(value: usize) -> Result<Self, String>;

    /// Checks if the floating point number is finite.
    fn is_finite(self) -> bool;

    #[cfg(feature = "std")]
    #[must_use]
    /// Returns the absolute value of the floating point number.
    fn abs(self) -> Self;

    #[cfg(feature = "std")]
    #[must_use]
    /// Returns the natural logarithm of the floating point number.
    fn ln(self) -> Self;

    #[cfg(feature = "std")]
    #[must_use]
    /// Returns the natural logarithm of 1 plus the floating point number.
    fn ln_1p(self) -> Self;

    #[cfg(feature = "std")]
    #[must_use]
    /// Returns the base 2 logarithm of the floating point number.
    fn log2(self) -> Self;

    #[cfg(feature = "std")]
    #[must_use]
    /// Returns the exponential of the floating point number.
    fn exp(self) -> Self;

    #[cfg(feature = "std")]
    #[must_use]
    /// Returns the square root of the floating point number.
    fn sqrt(self) -> Self;

    #[cfg(feature = "std")]
    #[must_use]
    /// Returns the floating point number raised to the power of an integer.
    fn powi(self, n: i32) -> Self;

    #[cfg(feature = "std")]
    #[must_use]
    /// Returns the floor of the floating point number.
    fn floor(self) -> Self;

    #[cfg(feature = "std")]
    #[must_use]
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
                #[must_use]
                fn to_f32(self) -> f32 {
                    self as f32
                }

                #[inline(always)]
                #[must_use]
                fn to_f64(self) -> f64 {
                    self as f64
                }

            }
        )*
    };
}

impl_number!(u8, u16, u32, u64, usize);
impl_number!(i32);
impl_number!(f32, f64);

macro_rules! impl_positive_integer_number {
    ($($t:ty),*) => {
        $(
            impl PositiveInteger for $t {
                type TryFromU64Error = <$t as core::convert::TryFrom<u64>>::Error;
                type TryFromUsizeError = <$t as core::convert::TryFrom<usize>>::Error;

                #[inline(always)]
                fn try_from_u64(value: u64) -> Result<Self, Self::TryFromU64Error> {
                    <$t as core::convert::TryFrom<u64>>::try_from(value)
                }

                #[inline(always)]
                fn try_from_usize(value: usize) -> Result<Self, Self::TryFromUsizeError> {
                    <$t as core::convert::TryFrom<usize>>::try_from(value)
                }

                #[inline(always)]
                #[must_use]
                fn to_usize(self) -> usize {
                    self as usize
                }
            }
        )*
    };
}

impl_positive_integer_number!(u8, u16, u32);

impl Float for f32 {
    const INFINITY: Self = f32::INFINITY;
    const NEG_INFINITY: Self = f32::NEG_INFINITY;
    const EPSILON: Self = f32::EPSILON;

    #[must_use]
    fn exp2_minus(register: u8) -> Self {
        f32::from_bits(u32::from(127 - register) << 23)
    }

    #[must_use]
    fn exp2_minus_signed(register: i8) -> Self {
        f32::from_bits(u32::try_from(127 - register).unwrap() << 23)
    }

    #[must_use]
    fn exp2(register: u8) -> Self {
        f32::from_bits(u32::from(127 + register) << 23)
    }

    fn from_usize_checked(value: usize) -> Result<Self, String> {
        let max_lossless_integer = 2_usize.pow(24) - 1;

        if value > max_lossless_integer {
            Err(format!(
                "The value {} is too large to be losslessly converted to a f32.",
                value
            ))
        } else {
            Ok(value as Self)
        }
    }

    #[must_use]
    fn from_i32(value: i32) -> Self {
        value as Self
    }

    #[must_use]
    fn to_u32(self) -> u32 {
        self as u32
    }

    #[must_use]
    fn is_finite(self) -> bool {
        self.is_finite()
    }

    #[must_use]
    fn from_f64(value: f64) -> Self {
        value as Self
    }

    #[must_use]
    fn from_f32(value: f32) -> Self {
        value as Self
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
impl Float for f64 {
    const INFINITY: Self = f64::INFINITY;
    const NEG_INFINITY: Self = f64::NEG_INFINITY;
    const EPSILON: Self = f64::EPSILON;

    #[must_use]
    fn exp2_minus(register: u8) -> Self {
        f64::from_le_bytes((u64::from(1023_u16 - u16::from(register)) << 52).to_le_bytes())
    }

    #[must_use]
    fn exp2_minus_signed(register: i8) -> Self {
        f64::from_le_bytes(
            (u64::try_from(1023_i16 - i16::from(register)).unwrap() << 52).to_le_bytes(),
        )
    }

    #[must_use]
    fn exp2(register: u8) -> Self {
        f64::from_le_bytes((u64::from(1023_u16 + u16::from(register)) << 52).to_le_bytes())
    }

    #[must_use]
    fn from_usize_checked(value: usize) -> Result<Self, String> {
        let max_lossless_integer = 2_usize.pow(53) - 1;

        if value > max_lossless_integer {
            Err(format!(
                "The value {} is too large to be losslessly converted to a f64.",
                value
            ))
        } else {
            Ok(value as Self)
        }
    }

    #[must_use]
    fn from_i32(value: i32) -> Self {
        value as Self
    }

    #[must_use]
    fn to_u32(self) -> u32 {
        self as u32
    }

    #[must_use]
    fn is_finite(self) -> bool {
        self.is_finite()
    }

    #[must_use]
    fn from_f64(value: f64) -> Self {
        value as Self
    }

    #[must_use]
    fn from_f32(value: f32) -> Self {
        value as Self
    }

    #[must_use]
    #[cfg(feature = "std")]
    fn abs(self) -> Self {
        self.abs()
    }

    #[must_use]
    #[cfg(feature = "std")]
    fn ln(self) -> Self {
        self.ln()
    }

    #[must_use]
    #[cfg(feature = "std")]
    fn ln_1p(self) -> Self {
        self.ln_1p()
    }

    #[must_use]
    #[cfg(feature = "std")]
    fn log2(self) -> Self {
        self.log2()
    }

    #[must_use]
    #[cfg(feature = "std")]
    fn exp(self) -> Self {
        self.exp()
    }

    #[must_use]
    #[cfg(feature = "std")]
    fn sqrt(self) -> Self {
        self.sqrt()
    }

    #[must_use]
    #[cfg(feature = "std")]
    fn powi(self, n: i32) -> Self {
        self.powi(n)
    }

    #[must_use]
    #[cfg(feature = "std")]
    fn floor(self) -> Self {
        self.floor()
    }

    #[must_use]
    #[cfg(feature = "std")]
    fn exp_m1(self) -> Self {
        self.exp_m1()
    }
}

#[cfg(test)]
mod test_exp2_minus {
    use super::*;
    use crate::prelude::maximal_multeplicity;

    #[test]
    fn test_exp2_minus() {
        // At the most, we create registers with 6 bits, which
        // means that the maximum values is 2^7 - 1 = 127.
        for precision in 4..=16 {
            for bits in 1..=6 {
                for register_value in 0..=maximal_multeplicity(precision, bits) {
                    assert_eq!(
                        2.0_f32.powf(-(register_value as f32)),
                        f32::exp2_minus(register_value as u8),
                        "Expected: 2^(-{}), Got: {}",
                        register_value,
                        f32::exp2_minus(register_value as u8)
                    );
                    assert_eq!(
                        2.0_f64.powf(-(register_value as f64)),
                        f64::exp2_minus(register_value as u8),
                        "Expected: 2^(-{}), Got: {}",
                        register_value,
                        f64::exp2_minus(register_value as u8)
                    );
                    assert_eq!(
                        f64::from_bits(
                            u64::max_value().wrapping_sub(u64::from(register_value as u64)) << 54
                                >> 2
                        ),
                        f64::exp2_minus(register_value as u8),
                        "Expected: 2^(-{}), Got: {}",
                        register_value,
                        f64::exp2_minus(register_value as u8)
                    );
                }
            }
        }
    }
}
