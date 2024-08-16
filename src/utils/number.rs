//! Traits regarding numbers.
use core::ops::{Shl, Shr, Add, Sub, Div, Mul, AddAssign, SubAssign, MulAssign, Neg};
use core::fmt::{Debug, Display};
use core::iter::Sum;
use crate::utils::{Half, Zero, One, Two, Three, Five, Six, Seven, Eight, Nine, Ten, OneHundred, OneThousand};

/// A trait for numbers.
pub trait Number:
    Copy
    + Default
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Div<Self, Output = Self>
    + Mul<Self, Output = Self>
    + MulAssign
    + AddAssign
    + SubAssign
    + Debug
    + Display
    + Zero
    + One
    + Two
    + Three
    + Five
    + Six
    + Seven
    + Eight
    + Nine
    + Ten
    + OneHundred
    + From<u8>
    + Sum
    + PartialOrd
    + Send
    + Sync
{
    #[must_use]
    /// A method to subtract the second number from the first number, returning zero if the result is negative.
    fn saturating_zero_sub(self, other: Self) -> Self;

    #[must_use]
    #[inline]
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
    type TryFromU64Error: Debug;
    /// The error type for the `try_from_usize` method.
    type TryFromUsizeError: Debug;

    /// Converts a `u64` to a positive integer number.
    /// 
    /// # Errors
    /// * If the value is too large to be converted to a positive integer number.
    fn try_from_u64(value: u64) -> Result<Self, Self::TryFromU64Error>;

    /// Converts a `usize` to a positive integer number.
    /// 
    /// # Errors
    /// * If the value is too large to be converted to a positive integer number.
    fn try_from_usize(value: usize) -> Result<Self, Self::TryFromUsizeError>;

    /// Converts the positive integer number to a `usize`.
    fn to_usize(self) -> usize;
}

/// A trait for floating point numbers.
pub trait Float: Number + Half + OneThousand + Neg<Output = Self> {
    /// The value of positive infinity.
    const INFINITY: Self;
    /// The value of negative infinity.
    const NEG_INFINITY: Self;
    /// The value of epsilon.
    const EPSILON: Self;

    /// Returns the value of 2^(-register), with strict positivite register.
    fn integer_exp2_minus(register: u8) -> Self;

    /// Returns the value of 2^(-register), including negative registers.
    fn integer_exp2_minus_signed(register: i8) -> Self;

    /// Returns the value of 2^(register)
    fn integer_exp2(register: u8) -> Self;

    #[must_use]
    #[inline]
    /// Computes the saturating division of two numbers that are expected to be positive.
    /// and at most equal to one.
    fn saturating_one_div(self, other: Self) -> Self {
        debug_assert!(self >= Self::ZERO, "The dividend must be positive.");
        debug_assert!(other >= Self::ZERO, "The divisor must be positive.");
        if self >= other {
            Self::ONE
        } else {
            self / other
        }
    }

    /// Converts the provided usize to a floating point number
    /// with the same value, checking that the conversion is lossless.
    /// 
    /// # Errors
    /// * If the value is too large to be losslessly converted to a floating point number.
    fn from_usize_checked(value: usize) -> Result<Self, &'static str>;

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
    /// Returns the exponential of the floating point number minus one.
    fn exp_m1(self) -> Self;
}

/// A trait for numbers.
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
            }
        )*
    };
}

impl_number!(u8, u16, u32, u64, usize);
impl_number!(i32);
impl_number!(f64);

/// A trait for signed numbers.
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
                    usize::try_from(self).unwrap()
                }
            }
        )*
    };
}

impl_positive_integer_number!(u8, u16, u32);

impl Float for f64 {
    const INFINITY: Self = f64::INFINITY;
    const NEG_INFINITY: Self = f64::NEG_INFINITY;
    const EPSILON: Self = f64::EPSILON;

    #[must_use]
    #[inline]
    fn integer_exp2_minus(register: u8) -> Self {
        f64::from_le_bytes((u64::from(1023_u16 - u16::from(register)) << 52).to_le_bytes())
    }

    #[must_use]
    #[inline]
    fn integer_exp2_minus_signed(register: i8) -> Self {
        f64::from_le_bytes(
            (u64::try_from(1023_i16 - i16::from(register)).unwrap() << 52).to_le_bytes(),
        )
    }

    #[must_use]
    #[inline]
    fn integer_exp2(register: u8) -> Self {
        f64::from_le_bytes((u64::from(1023_u16 + u16::from(register)) << 52).to_le_bytes())
    }

    #[inline]
    fn from_usize_checked(value: usize) -> Result<Self, &'static str> {
        let max_lossless_integer = 2_usize.pow(53) - 1;

        if value > max_lossless_integer {
            Err("The value is too large to be losslessly converted to a f64.")
        } else {
            Ok(value as Self)
        }
    }

    #[must_use]
    #[inline]
    fn is_finite(self) -> bool {
        self.is_finite()
    }

    #[must_use]
    #[inline]
    #[cfg(feature = "std")]
    fn abs(self) -> Self {
        self.abs()
    }

    #[must_use]
    #[inline]
    #[cfg(feature = "std")]
    fn ln(self) -> Self {
        self.ln()
    }

    #[must_use]
    #[inline]
    #[cfg(feature = "std")]
    fn ln_1p(self) -> Self {
        self.ln_1p()
    }

    #[must_use]
    #[inline]
    #[cfg(feature = "std")]
    fn log2(self) -> Self {
        self.log2()
    }

    #[must_use]
    #[inline]
    #[cfg(feature = "std")]
    fn exp(self) -> Self {
        self.exp()
    }

    #[must_use]
    #[inline]
    #[cfg(feature = "std")]
    fn sqrt(self) -> Self {
        self.sqrt()
    }

    #[must_use]
    #[inline]
    #[cfg(feature = "std")]
    fn powi(self, n: i32) -> Self {
        self.powi(n)
    }

    #[must_use]
    #[inline]
    #[cfg(feature = "std")]
    fn exp_m1(self) -> Self {
        self.exp_m1()
    }
}

#[cfg(test)]
mod test_integer_exp2_minus {
    use super::*;
    use crate::prelude::maximal_multeplicity;

    #[test]
    fn test_integer_exp2_minus() {
        // At the most, we create registers with 6 bits, which
        // means that the maximum values is 2^7 - 1 = 127.
        for precision in 4..=16 {
            for bits in 1..=6 {
                for register_value in 0..=maximal_multeplicity(precision, bits) {
                    assert_eq!(
                        2.0_f64.powf(-(register_value as f64)),
                        f64::integer_exp2_minus(register_value as u8),
                        "Expected: 2^(-{}), Got: {}",
                        register_value,
                        f64::integer_exp2_minus(register_value as u8)
                    );
                    assert_eq!(
                        f64::from_bits(
                            u64::max_value().wrapping_sub(u64::from(register_value as u64)) << 54
                                >> 2
                        ),
                        f64::integer_exp2_minus(register_value as u8),
                        "Expected: 2^(-{}), Got: {}",
                        register_value,
                        f64::integer_exp2_minus(register_value as u8)
                    );
                }
            }
        }
    }
}
