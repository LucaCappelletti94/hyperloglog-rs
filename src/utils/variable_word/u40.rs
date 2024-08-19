//! Submodule providing a virtual word with 40 bits.

use crate::utils::{Number, One, PositiveInteger, Zero};
use core::fmt::{Display, Formatter};
use core::ops::{Add, Sub, Mul, Div, SubAssign, AddAssign, MulAssign, DivAssign, Shl, Shr, ShlAssign, ShrAssign};

use super::VariableWord;

/// Virtual word with 40 bits.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct U40 {
    /// The value of the word.
    value: u64,
}

impl Display for U40 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Add for U40 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: (self.value + rhs.value) & Self::MASK,
        }
    }
}

impl AddAssign for U40 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Mul for U40 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            value: (self.value * rhs.value) & Self::MASK,
        }
    }
}

impl MulAssign for U40 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Sub for U40 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            value: (self.value - rhs.value),
        }
    }
}

impl SubAssign for U40 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Div for U40 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value / rhs.value,
        }
    }
}

impl DivAssign for U40 {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl<A, B> Shl<A, Output=B> for U40
where
    A: Into<u32>,
    B: From<u64>,
{
    fn shl(self, rhs: A) -> B {
        B::from(self.value << rhs.into())
    }
}

impl From<u8> for U40 {
    fn from(value: u8) -> Self {
        Self {
            value: u64::from(value),
        }
    }
}

impl From<u16> for U40 {
    fn from(value: u16) -> Self {
        Self {
            value: u64::from(value),
        }
    }
}

impl From<u32> for U40 {
    fn from(value: u32) -> Self {
        Self {
            value: u64::from(value),
        }
    }
}

impl TryFrom<u64> for U40 {
    type Error = &'static str;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > 0xFFFF_FFFF_FFFF {
            Err("Value is too large for U40")
        } else {
            Ok(Self { value })
        }
    }
}

impl Zero for U40 {
    const ZERO: Self = Self { value: 0 };

    fn is_zero(&self) -> bool {
        self.value == 0
    }
}

impl One for U40 {
    const ONE: Self = Self { value: 1 };

    fn is_one(&self) -> bool {
        self.value == 1
    }
}

impl Number for U40 {
    fn saturating_zero_sub(self, other: Self) -> Self {
        if self.value < other.value {
            Self { value: 0 }
        } else {
            Self { value: self.value - other.value }
        }
    }
}

impl PositiveInteger for U40 {
    type TryFromU64Error = &'static str;
    
    fn try_from_u64(value: u64) -> Result<Self, Self::TryFromU64Error> {
        Self::try_from(value)
    }

    fn to_usize(self) -> usize {
        self.value as usize
    }
}

impl VariableWord for U40 {
    const NUMBER_OF_BITS: u8 = 40;
    type Word = u64;
    const MASK: u64 = 0xFFFF_FFFF_FFFF;
}
