//! Traits regarding numbers.
use crate::utils::{One, Zero};
use core::fmt::{Debug, Display};
use core::hash::Hash;
use core::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Div, Mul, Neg, Not, Rem, Shl, Shr,
    Sub, SubAssign,
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
    + From<bool>
    + PartialOrd
    + Send
    + Sync
{
    #[must_use]
    /// A method to subtract the second number from the first number, returning zero if the result is negative.
    fn saturating_zero_sub(self, other: Self) -> Self;
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
    + Rem<Output = Self>
    + Shl<u8, Output = Self>
    + Shr<u8, Output = Self>
    + Hash
    + Not<Output = Self>
{
}

/// A trait for floating point numbers.
pub(crate) trait FloatOps: Number + Neg<Output = Self> {
    /// Returns the value of 2^(-register), with strict positivite register.
    fn integer_exp2_minus(register: u8) -> Self;

    /// Returns the value of 2^(-register), including negative registers.
    fn integer_exp2_minus_signed(register: i16) -> Self;

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
                #[inline]
                fn saturating_zero_sub(self, other: Self) -> Self {
                    debug_assert!(self >= Self::ZERO, "The first number must be positive, got: {}", self);
                    debug_assert!(other >= Self::ZERO, "The second number must be positive, got: {}", other);
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
    fn integer_exp2_minus_signed(register: i16) -> Self {
        debug_assert!(
            register > -1024,
            "The register must be greater than -1024, got: {register}",
        );
        f64::from_le_bytes((u64::try_from(1023_i16 - register).unwrap() << 52).to_le_bytes())
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

    #[test]
    fn test_integer_exp2_minus() {
        // At the most, we create registers with 6 bits, which
        // means that the maximum values is 2^7 - 1 = 127.
        for bits in 1..=8 {
            for register_value in 0..(1 << bits) {
                assert_eq!(
                    2.0_f64.powf(-(register_value as f64)),
                    f64::integer_exp2_minus(register_value as u8),
                    "Expected: 2^(-{}), Got: {}",
                    register_value,
                    f64::integer_exp2_minus(register_value as u8)
                );
                assert_eq!(
                    f64::from_bits(
                        u64::max_value().wrapping_sub(u64::from(register_value as u64)) << 54 >> 2
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
