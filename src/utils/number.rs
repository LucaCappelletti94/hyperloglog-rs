//! Traits regarding numbers.
use crate::utils::*;

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
    + core::iter::Sum
    + PartialOrd
{
    fn saturating_zero_sub(self, other: Self) -> Self;
}

pub trait PositiveIntegerNumber: Number + Eq + Ord + TryInto<usize> + TryFrom<usize> {}

pub trait FloatNumber:
    Number + Half + crate::utils::OneThousand + core::ops::Neg<Output = Self>
{
    const INFINITY: Self;
    const EPSILON: Self;

    fn inverse_register(register: u32) -> Self;

    fn inverse_register_with_scalar(register: u32, scalar: u32) -> Self;

    fn saturating_one_div(self, other: Self) -> Self {
        debug_assert!(self >= Self::ZERO);
        debug_assert!(other >= Self::ZERO);
        if self >= other {
            Self::ONE
        } else {
            self / other
        }
    }

    fn from_usize(value: usize) -> Self;

    fn to_usize(self) -> usize;

    #[cfg(feature = "std")]
    fn abs(self) -> Self;

    #[cfg(feature = "std")]
    fn ln(self) -> Self;

    #[cfg(feature = "std")]
    fn ln_1p(self) -> Self;

    #[cfg(feature = "std")]
    fn log2(self) -> Self;

    #[cfg(feature = "std")]
    fn exp(self) -> Self;

    #[cfg(feature = "std")]
    fn sqrt(self) -> Self;

    #[cfg(feature = "std")]
    fn powi(self, n: i32) -> Self;

    #[cfg(feature = "std")]
    fn floor(self) -> Self;

    #[cfg(feature = "std")]
    fn exp_m1(self) -> Self;
}

macro_rules! impl_number {
    ($($t:ty),*) => {
        $(
            impl Number for $t {
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
    const EPSILON: Self = f32::EPSILON;

    fn inverse_register(register: u32) -> Self {
        f32::from_le_bytes(((127 - register) << 23).to_le_bytes())
    }

    fn inverse_register_with_scalar(register: u32, scalar: u32) -> Self {
        scalar as f32 * f32::inverse_register(register)
    }

    fn from_usize(value: usize) -> Self {
        value as Self
    }

    fn to_usize(self) -> usize {
        self as usize
    }

    #[cfg(feature = "std")]
    fn abs(self) -> Self {
        self.abs()
    }

    #[cfg(feature = "std")]
    fn ln(self) -> Self {
        self.ln()
    }

    #[cfg(feature = "std")]
    fn ln_1p(self) -> Self {
        self.ln_1p()
    }

    #[cfg(feature = "std")]
    fn log2(self) -> Self {
        self.log2()
    }

    #[cfg(feature = "std")]
    fn exp(self) -> Self {
        self.exp()
    }

    #[cfg(feature = "std")]
    fn sqrt(self) -> Self {
        self.sqrt()
    }

    #[cfg(feature = "std")]
    fn powi(self, n: i32) -> Self {
        self.powi(n)
    }

    #[cfg(feature = "std")]
    fn floor(self) -> Self {
        self.floor()
    }

    #[cfg(feature = "std")]
    fn exp_m1(self) -> Self {
        self.exp_m1()
    }
}
impl FloatNumber for f64 {
    const INFINITY: Self = f64::INFINITY;
    const EPSILON: Self = f64::EPSILON;

    fn inverse_register(register: u32) -> Self {
        f64::from_le_bytes(((1023 - register as i64) << 52).to_le_bytes())
    }

    fn inverse_register_with_scalar(register: u32, scalar: u32) -> Self {
        scalar as f64 * f64::inverse_register(register)
    }

    fn from_usize(value: usize) -> Self {
        value as Self
    }

    fn to_usize(self) -> usize {
        self as usize
    }

    #[cfg(feature = "std")]
    fn abs(self) -> Self {
        self.abs()
    }

    #[cfg(feature = "std")]
    fn ln(self) -> Self {
        self.ln()
    }

    #[cfg(feature = "std")]
    fn ln_1p(self) -> Self {
        self.ln_1p()
    }

    #[cfg(feature = "std")]
    fn log2(self) -> Self {
        self.log2()
    }

    #[cfg(feature = "std")]
    fn exp(self) -> Self {
        self.exp()
    }

    #[cfg(feature = "std")]
    fn sqrt(self) -> Self {
        self.sqrt()
    }

    #[cfg(feature = "std")]
    fn powi(self, n: i32) -> Self {
        self.powi(n)
    }

    #[cfg(feature = "std")]
    fn floor(self) -> Self {
        self.floor()
    }

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
                        f32::inverse_register(register_value as u32),
                        "Expected: 2^(-{}), Got: {}",
                        register_value,
                        f32::inverse_register(register_value as u32)
                    );
                    assert_eq!(
                        2.0_f64.powf(-(register_value as f64)),
                        f64::inverse_register(register_value as u32),
                        "Expected: 2^(-{}), Got: {}",
                        register_value,
                        f64::inverse_register(register_value as u32)
                    );
                    assert_eq!(
                        f64::from_bits(
                            u64::max_value().wrapping_sub(u64::from(register_value as u64)) << 54
                                >> 2
                        ),
                        f64::inverse_register(register_value as u32),
                        "Expected: 2^(-{}), Got: {}",
                        register_value,
                        f64::inverse_register(register_value as u32)
                    );
                }
            }
        }
    }

    #[test]
    fn test_inverse_register_with_scalar() {
        // At the most, we create registers with 6 bits, which
        // means that the maximum values is 2^7 - 1 = 127.
        // For scrupulousness, we test for all values from 0 to 256.
        for precision in 4..=16 {
            for bits in 1..=6 {
                for register_value in 0..=maximal_multeplicity(precision, bits) {
                    for scalar_value in 0..=maximal_multeplicity(precision, bits) {
                        assert_eq!(
                            scalar_value as f32 * 2.0_f32.powf(-(register_value as f32)),
                            f32::inverse_register_with_scalar(
                                register_value as u32,
                                scalar_value as u32
                            )
                        );
                        assert_eq!(
                            scalar_value as f64 * 2.0_f64.powf(-(register_value as f64)),
                            f64::inverse_register_with_scalar(
                                register_value as u32,
                                scalar_value as u32
                            )
                        );
                    }
                }
            }
        }
    }
}
