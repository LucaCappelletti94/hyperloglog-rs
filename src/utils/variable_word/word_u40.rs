//! Submodule providing a virtual word with 40 bits.

use crate::utils::{Number, One, PositiveInteger, Zero};
use core::fmt::{Display, Formatter};
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign, BitAndAssign, BitAnd, BitOr, BitOrAssign,
};

use super::VariableWord;
use core::fmt;

/// Virtual word with 40 bits.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct u40 {
    /// The value of the word.
    value: u64,
}

impl Display for u40 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Add for u40 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: (self.value + rhs.value) & Self::MASK,
        }
    }
}

impl AddAssign for u40 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Mul for u40 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            value: (self.value * rhs.value) & Self::MASK,
        }
    }
}

impl MulAssign for u40 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Sub for u40 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            value: (self.value - rhs.value),
        }
    }
}

impl SubAssign for u40 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Div for u40 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value / rhs.value,
        }
    }
}

impl DivAssign for u40 {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl<A> Shl<A> for u40
where
    u64: Shl<A, Output = u64>,
{
    type Output = Self;
    fn shl(self, rhs: A) -> Self {
        Self {
            value: self.value.shl(rhs) & Self::MASK,
        }
    }
}

impl<A> ShlAssign<A> for u40
where
    u64: ShlAssign<A>,
{
    fn shl_assign(&mut self, rhs: A) {
        self.value <<= rhs;
        self.value &= Self::MASK;
    }
}

impl<A> Shr<A> for u40
where
    u64: Shr<A, Output = u64>,
{
    type Output = Self;
    fn shr(self, rhs: A) -> Self {
        Self {
            value: self.value.shr(rhs),
        }
    }
}

impl<A> ShrAssign<A> for u40
where
    u64: ShrAssign<A>,
{
    fn shr_assign(&mut self, rhs: A) {
        self.value >>= rhs;
    }
}

impl BitAnd for u40 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value & rhs.value,
        }
    }
}

impl BitAndAssign for u40 {
    fn bitand_assign(&mut self, rhs: Self) {
        self.value &= rhs.value;
    }
}

impl BitOr for u40 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value | rhs.value,
        }
    }
}

impl BitOrAssign for u40 {
    fn bitor_assign(&mut self, rhs: Self) {
        self.value |= rhs.value;
    }
}

impl From<u8> for u40 {
    fn from(value: u8) -> Self {
        Self {
            value: u64::from(value),
        }
    }
}

impl From<u16> for u40 {
    fn from(value: u16) -> Self {
        Self {
            value: u64::from(value),
        }
    }
}

impl From<u32> for u40 {
    fn from(value: u32) -> Self {
        Self {
            value: u64::from(value),
        }
    }
}

impl TryFrom<u64> for u40 {
    type Error = &'static str;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > 0xFFFF_FFFF_FFFF {
            Err("Value is too large for u40")
        } else {
            Ok(Self { value })
        }
    }
}

impl TryFrom<usize> for u40 {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value > 0xFFFF_FFFF_FFFF {
            Err("Value is too large for u40")
        } else {
            Ok(Self { value: u64::try_from(value).unwrap() })
        }
    }
}

impl TryInto<u8> for u40 {
    type Error = &'static str;

    fn try_into(self) -> Result<u8, Self::Error> {
        if self.value > 0xFF {
            Err("Value is too large for u8")
        } else {
            Ok(self.value as u8)
        }
    }
}

impl TryInto<u16> for u40 {
    type Error = &'static str;

    fn try_into(self) -> Result<u16, Self::Error> {
        if self.value > 0xFFFF {
            Err("Value is too large for u16")
        } else {
            Ok(self.value as u16)
        }
    }
}

impl TryInto<u32> for u40 {
    type Error = &'static str;

    fn try_into(self) -> Result<u32, Self::Error> {
        if self.value > 0xFFFF_FFFF {
            Err("Value is too large for u32")
        } else {
            Ok(self.value as u32)
        }
    }
}

impl Into<u64> for u40 {
    fn into(self) -> u64 {
        self.value
    }
}

impl Zero for u40 {
    const ZERO: Self = Self { value: 0 };

    fn is_zero(&self) -> bool {
        self.value == 0
    }
}

impl One for u40 {
    const ONE: Self = Self { value: 1 };

    fn is_one(&self) -> bool {
        self.value == 1
    }
}

impl Number for u40 {
    fn saturating_zero_sub(self, other: Self) -> Self {
        if self.value < other.value {
            Self { value: 0 }
        } else {
            Self {
                value: self.value - other.value,
            }
        }
    }
}

impl PositiveInteger for u40 {
    type TryFromU64Error = &'static str;

    fn try_from_u64(value: u64) -> Result<Self, Self::TryFromU64Error> {
        Self::try_from(value)
    }

    fn to_usize(self) -> usize {
        self.value as usize
    }
}