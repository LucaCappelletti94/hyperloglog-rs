//! Traits regarding numbers.
use crate::utils::{One, Zero};
use core::fmt::{Debug, Display};
use core::hash::Hash;
use core::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Div, Mul, Neg, Not, Shl, Shr, Sub,
    SubAssign,
};

/// A trait for numbers.
pub trait Number:
    Copy
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Div<Self, Output = Self>
    + Mul<Self, Output = Self>
    + AddAssign
    + SubAssign
    + Debug
    + Display
    + Zero
    + One
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
    Number
    + Eq
    + Into<u64>
    + From<u8>
    + BitAnd<Output = Self>
    + BitAndAssign
    + BitOr
    + BitOrAssign
    + Ord
    + Shl<u8, Output = Self>
    + Shr<u8, Output = Self>
    + Hash
    + Not<Output = Self>
{
    /// The error type for the `try_from_u64` method.
    type TryFromU64Error: Debug;

    /// Converts a `u64` to a positive integer number.
    ///
    /// # Errors
    /// * If the value is too large to be converted to a positive integer number.
    fn try_from_u64(value: u64) -> Result<Self, Self::TryFromU64Error>;

    #[allow(unsafe_code)]
    /// Converts a `u64` to a positive integer number without checking the value.
    unsafe fn unchecked_from_u64(value: u64) -> Self;

    /// Converts the positive integer number to a `usize`.
    fn to_usize(self) -> usize;
}

/// A trait for floating point numbers.
pub(crate) trait FloatOps: Number + Neg<Output = Self> {
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

                #[inline(always)]
                fn try_from_u64(value: u64) -> Result<Self, Self::TryFromU64Error> {
                    <$t as core::convert::TryFrom<u64>>::try_from(value)
                }

                #[inline(always)]
                #[allow(unsafe_code)]
                unsafe fn unchecked_from_u64(value: u64) -> Self {
                    value as Self
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

impl_positive_integer_number!(u8, u16, u32, u64);

impl FloatOps for f64 {
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
