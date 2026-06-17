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

    /// Returns the natural logarithm of a strictly positive value, computed without `std` (the core
    /// crate is `no_std` and transcendental-free outside the `mle` feature).
    fn natural_log(self) -> Self;

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
    #[inline]
    fn integer_exp2_minus(register: u8) -> Self {
        f64::from_le_bytes((u64::from(1023_u16 - u16::from(register)) << 52).to_le_bytes())
    }

    #[inline]
    fn integer_exp2_minus_signed(register: i16) -> Self {
        debug_assert!(
            register > -1024,
            "The register must be greater than -1024, got: {register}",
        );
        f64::from_le_bytes((u64::try_from(1023_i16 - register).unwrap() << 52).to_le_bytes())
    }

    #[inline]
    fn integer_exp2(register: u8) -> Self {
        f64::from_le_bytes((u64::from(1023_u16 + u16::from(register)) << 52).to_le_bytes())
    }

    #[inline]
    fn natural_log(self) -> Self {
        debug_assert!(
            self > 0.0,
            "natural_log requires a strictly positive argument, got {self}"
        );
        // Decompose self = mantissa * 2^exponent with mantissa in [1, 2).
        let bits = self.to_bits();
        let mut exponent = ((bits >> 52) & 0x7ff) as i64 - 1023;
        let mut mantissa = f64::from_bits((bits & 0x000f_ffff_ffff_ffff) | 0x3ff0_0000_0000_0000);
        // Center the mantissa around 1 (into [1/sqrt(2), sqrt(2))) so the series converges fast.
        if mantissa > core::f64::consts::SQRT_2 {
            mantissa *= 0.5;
            exponent += 1;
        }
        // ln(mantissa) via the atanh series with s = (m - 1) / (m + 1):
        // ln(m) = 2 (s + s^3/3 + s^5/5 + ...). |s| <= 0.172 here, so the odd reciprocals up to
        // 1/15 (Horner over s^2) reach full f64 accuracy across our [1, 2^18] argument range.
        let s = (mantissa - 1.0) / (mantissa + 1.0);
        let s2 = s * s;
        let mut series = 1.0 / 15.0;
        series = series * s2 + 1.0 / 13.0;
        series = series * s2 + 1.0 / 11.0;
        series = series * s2 + 1.0 / 9.0;
        series = series * s2 + 1.0 / 7.0;
        series = series * s2 + 1.0 / 5.0;
        series = series * s2 + 1.0 / 3.0;
        series = series * s2 + 1.0;
        exponent as f64 * core::f64::consts::LN_2 + 2.0 * s * series
    }
}

#[cfg(test)]
mod test_natural_log {
    use super::*;

    #[test]
    fn test_natural_log_matches_std() {
        // The no_std atanh-series implementation must match std's ln across the range of arguments
        // linear counting produces, m / zeros for m up to 2^18 and zeros in [1, m], i.e. [1, 2^18].
        let mut value = 1.0_f64;
        while value <= (1u64 << 18) as f64 {
            let approx = value.natural_log();
            let exact = value.ln();
            assert!(
                (approx - exact).abs() <= 1e-9 * exact.abs().max(1.0),
                "natural_log({value}) = {approx}, std ln = {exact}"
            );
            value *= 1.0009765625; // 1 + 2^-10, a fine geometric sweep
        }
        // Exact landmarks.
        assert!((1.0_f64.natural_log()).abs() < 1e-12);
        assert!((core::f64::consts::E.natural_log() - 1.0).abs() < 1e-12);
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
