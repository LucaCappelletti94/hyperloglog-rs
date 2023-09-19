//! This module defines a trait for the primitive conversion of unsigned integer values
//! between one-another. While this is not a trait in the core library, we are aware that
//! it is available in other crates - we do not intend to use them as dependencies, as we want to keep
//! the dependencies to the very bare minimum.

use core::{
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Sub, MulAssign, SubAssign, DivAssign},
};

use crate::max_min::MaxMin;

pub trait Primitive<U>:
    Sized
    + Copy
    + Send
    + Sync
    + Sum
    + Default
    + Sub<Self, Output = Self>
    + Add<Self, Output = Self>
    + AddAssign
    + MulAssign
    + SubAssign
    + DivAssign
    + PartialOrd
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + MaxMin
{
    fn convert(self) -> U;
    fn reverse(other: U) -> Self;
}

impl Primitive<u8> for u16 {
    #[inline(always)]
    fn convert(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    fn reverse(other: u8) -> Self {
        other as u16
    }
}

impl Primitive<u8> for u32 {
    #[inline(always)]
    fn convert(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    fn reverse(other: u8) -> Self {
        other as u32
    }
}

impl Primitive<u8> for u64 {
    #[inline(always)]
    fn convert(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    fn reverse(other: u8) -> Self {
        other as u64
    }
}

impl Primitive<u8> for u128 {
    #[inline(always)]
    fn convert(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    fn reverse(other: u8) -> Self {
        other as u128
    }
}

impl Primitive<u8> for usize {
    #[inline(always)]
    fn convert(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    fn reverse(other: u8) -> Self {
        other as usize
    }
}

impl Primitive<u16> for u8 {
    #[inline(always)]
    fn convert(self) -> u16 {
        self as u16
    }

    #[inline(always)]
    fn reverse(other: u16) -> Self {
        other as u8
    }
}

impl Primitive<u16> for u32 {
    #[inline(always)]
    fn convert(self) -> u16 {
        self as u16
    }

    #[inline(always)]
    fn reverse(other: u16) -> Self {
        other as u32
    }
}

impl Primitive<u16> for u64 {
    #[inline(always)]
    fn convert(self) -> u16 {
        self as u16
    }

    #[inline(always)]
    fn reverse(other: u16) -> Self {
        other as u64
    }
}

impl Primitive<u16> for u128 {
    #[inline(always)]
    fn convert(self) -> u16 {
        self as u16
    }

    #[inline(always)]
    fn reverse(other: u16) -> Self {
        other as u128
    }
}

impl Primitive<u16> for usize {
    #[inline(always)]
    fn convert(self) -> u16 {
        self as u16
    }

    #[inline(always)]
    fn reverse(other: u16) -> Self {
        other as usize
    }
}

impl Primitive<u32> for u8 {
    #[inline(always)]
    fn convert(self) -> u32 {
        self as u32
    }

    #[inline(always)]
    fn reverse(other: u32) -> Self {
        other as u8
    }
}

impl Primitive<u32> for u16 {
    #[inline(always)]
    fn convert(self) -> u32 {
        self as u32
    }

    #[inline(always)]
    fn reverse(other: u32) -> Self {
        other as u16
    }
}

impl Primitive<u32> for u64 {
    #[inline(always)]
    fn convert(self) -> u32 {
        self as u32
    }

    #[inline(always)]
    fn reverse(other: u32) -> Self {
        other as u64
    }
}

impl Primitive<u32> for u128 {
    #[inline(always)]
    fn convert(self) -> u32 {
        self as u32
    }

    #[inline(always)]
    fn reverse(other: u32) -> Self {
        other as u128
    }
}

impl Primitive<u32> for usize {
    #[inline(always)]
    fn convert(self) -> u32 {
        self as u32
    }

    #[inline(always)]
    fn reverse(other: u32) -> Self {
        other as usize
    }
}

impl Primitive<u64> for u8 {
    #[inline(always)]
    fn convert(self) -> u64 {
        self as u64
    }

    #[inline(always)]
    fn reverse(other: u64) -> Self {
        other as u8
    }
}

impl Primitive<u64> for u16 {
    #[inline(always)]
    fn convert(self) -> u64 {
        self as u64
    }

    #[inline(always)]
    fn reverse(other: u64) -> Self {
        other as u16
    }
}

impl Primitive<u64> for u32 {
    #[inline(always)]
    fn convert(self) -> u64 {
        self as u64
    }

    #[inline(always)]
    fn reverse(other: u64) -> Self {
        other as u32
    }
}

impl Primitive<u64> for u128 {
    #[inline(always)]
    fn convert(self) -> u64 {
        self as u64
    }

    #[inline(always)]
    fn reverse(other: u64) -> Self {
        other as u128
    }
}

impl Primitive<u64> for usize {
    #[inline(always)]
    fn convert(self) -> u64 {
        self as u64
    }

    #[inline(always)]
    fn reverse(other: u64) -> Self {
        other as usize
    }
}

impl Primitive<u128> for u8 {
    #[inline(always)]
    fn convert(self) -> u128 {
        self as u128
    }

    #[inline(always)]
    fn reverse(other: u128) -> Self {
        other as u8
    }
}

impl Primitive<u128> for u16 {
    #[inline(always)]
    fn convert(self) -> u128 {
        self as u128
    }

    #[inline(always)]
    fn reverse(other: u128) -> Self {
        other as u16
    }
}

impl Primitive<u128> for u32 {
    #[inline(always)]
    fn convert(self) -> u128 {
        self as u128
    }

    #[inline(always)]
    fn reverse(other: u128) -> Self {
        other as u32
    }
}

impl Primitive<u128> for u64 {
    #[inline(always)]
    fn convert(self) -> u128 {
        self as u128
    }

    #[inline(always)]
    fn reverse(other: u128) -> Self {
        other as u64
    }
}

impl Primitive<u128> for usize {
    #[inline(always)]
    fn convert(self) -> u128 {
        self as u128
    }

    #[inline(always)]
    fn reverse(other: u128) -> Self {
        other as usize
    }
}

impl Primitive<usize> for u8 {
    #[inline(always)]
    fn convert(self) -> usize {
        self as usize
    }

    #[inline(always)]
    fn reverse(other: usize) -> Self {
        other as u8
    }
}

impl Primitive<usize> for u16 {
    #[inline(always)]
    fn convert(self) -> usize {
        self as usize
    }

    #[inline(always)]
    fn reverse(other: usize) -> Self {
        other as u16
    }
}

impl Primitive<usize> for u32 {
    #[inline(always)]
    fn convert(self) -> usize {
        self as usize
    }

    #[inline(always)]
    fn reverse(other: usize) -> Self {
        other as u32
    }
}

impl Primitive<usize> for u64 {
    #[inline(always)]
    fn convert(self) -> usize {
        self as usize
    }

    #[inline(always)]
    fn reverse(other: usize) -> Self {
        other as u64
    }
}

impl Primitive<usize> for u128 {
    #[inline(always)]
    fn convert(self) -> usize {
        self as usize
    }

    #[inline(always)]
    fn reverse(other: usize) -> Self {
        other as u128
    }
}

impl Primitive<bool> for u8 {
    #[inline(always)]
    fn convert(self) -> bool {
        self != 0
    }

    #[inline(always)]
    fn reverse(other: bool) -> Self {
        if other {
            1
        } else {
            0
        }
    }
}

impl Primitive<bool> for u16 {
    #[inline(always)]
    fn convert(self) -> bool {
        self != 0
    }

    #[inline(always)]
    fn reverse(other: bool) -> Self {
        if other {
            1
        } else {
            0
        }
    }
}

impl Primitive<bool> for u32 {
    #[inline(always)]
    fn convert(self) -> bool {
        self != 0
    }

    #[inline(always)]
    fn reverse(other: bool) -> Self {
        if other {
            1
        } else {
            0
        }
    }
}

impl Primitive<bool> for u64 {
    #[inline(always)]
    fn convert(self) -> bool {
        self != 0
    }

    #[inline(always)]
    fn reverse(other: bool) -> Self {
        if other {
            1
        } else {
            0
        }
    }
}

impl Primitive<bool> for u128 {
    #[inline(always)]
    fn convert(self) -> bool {
        self != 0
    }

    #[inline(always)]
    fn reverse(other: bool) -> Self {
        if other {
            1
        } else {
            0
        }
    }
}

impl Primitive<bool> for usize {
    #[inline(always)]
    fn convert(self) -> bool {
        self != 0
    }

    #[inline(always)]
    fn reverse(other: bool) -> Self {
        if other {
            1
        } else {
            0
        }
    }
}

impl Primitive<f32> for f32 {
    #[inline(always)]
    fn convert(self) -> f32 {
        self
    }

    #[inline(always)]
    fn reverse(other: f32) -> Self {
        other
    }
}

impl Primitive<f64> for f64 {
    #[inline(always)]
    fn convert(self) -> f64 {
        self
    }

    #[inline(always)]
    fn reverse(other: f64) -> Self {
        other
    }
}

impl Primitive<f32> for f64 {
    #[inline(always)]
    fn convert(self) -> f32 {
        self as f32
    }

    #[inline(always)]
    fn reverse(other: f32) -> Self {
        other as f64
    }
}

impl Primitive<f64> for f32 {
    #[inline(always)]
    fn convert(self) -> f64 {
        self as f64
    }

    #[inline(always)]
    fn reverse(other: f64) -> Self {
        other as f32
    }
}


impl Primitive<u8> for f32 {
    #[inline(always)]
    fn convert(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    fn reverse(other: u8) -> Self {
        other as f32
    }
}

impl Primitive<u8> for f64 {
    #[inline(always)]
    fn convert(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    fn reverse(other: u8) -> Self {
        other as f64
    }
}

impl Primitive<u16> for f32 {
    #[inline(always)]
    fn convert(self) -> u16 {
        self as u16
    }

    #[inline(always)]
    fn reverse(other: u16) -> Self {
        other as f32
    }
}

impl Primitive<u16> for f64 {
    #[inline(always)]
    fn convert(self) -> u16 {
        self as u16
    }

    #[inline(always)]
    fn reverse(other: u16) -> Self {
        other as f64
    }
}

impl Primitive<u32> for f32 {
    #[inline(always)]
    fn convert(self) -> u32 {
        self as u32
    }

    #[inline(always)]
    fn reverse(other: u32) -> Self {
        other as f32
    }
}

impl Primitive<u32> for f64 {
    #[inline(always)]
    fn convert(self) -> u32 {
        self as u32
    }

    #[inline(always)]
    fn reverse(other: u32) -> Self {
        other as f64
    }
}

impl Primitive<u64> for f32 {
    #[inline(always)]
    fn convert(self) -> u64 {
        self as u64
    }

    #[inline(always)]
    fn reverse(other: u64) -> Self {
        other as f32
    }
}

impl Primitive<u64> for f64 {
    #[inline(always)]
    fn convert(self) -> u64 {
        self as u64
    }

    #[inline(always)]
    fn reverse(other: u64) -> Self {
        other as f64
    }
}

impl Primitive<u128> for f32 {
    #[inline(always)]
    fn convert(self) -> u128 {
        self as u128
    }

    #[inline(always)]
    fn reverse(other: u128) -> Self {
        other as f32
    }
}

impl Primitive<u128> for f64 {
    #[inline(always)]
    fn convert(self) -> u128 {
        self as u128
    }

    #[inline(always)]
    fn reverse(other: u128) -> Self {
        other as f64
    }
}

impl Primitive<usize> for f32 {
    #[inline(always)]
    fn convert(self) -> usize {
        self as usize
    }

    #[inline(always)]
    fn reverse(other: usize) -> Self {
        other as f32
    }
}

impl Primitive<usize> for f64 {
    #[inline(always)]
    fn convert(self) -> usize {
        self as usize
    }

    #[inline(always)]
    fn reverse(other: usize) -> Self {
        other as f64
    }
}

impl Primitive<bool> for f32 {
    #[inline(always)]
    fn convert(self) -> bool {
        self != 0.0
    }

    #[inline(always)]
    fn reverse(other: bool) -> Self {
        if other {
            1.0
        } else {
            0.0
        }
    }
}

impl Primitive<bool> for f64 {
    #[inline(always)]
    fn convert(self) -> bool {
        self != 0.0
    }

    #[inline(always)]
    fn reverse(other: bool) -> Self {
        if other {
            1.0
        } else {
            0.0
        }
    }
}

impl Primitive<f32> for u8 {
    #[inline(always)]
    fn convert(self) -> f32 {
        self as f32
    }

    #[inline(always)]
    fn reverse(other: f32) -> Self {
        other as u8
    }
}

impl Primitive<f32> for u16 {
    #[inline(always)]
    fn convert(self) -> f32 {
        self as f32
    }

    #[inline(always)]
    fn reverse(other: f32) -> Self {
        other as u16
    }
}

impl Primitive<f32> for u32 {
    #[inline(always)]
    fn convert(self) -> f32 {
        self as f32
    }

    #[inline(always)]
    fn reverse(other: f32) -> Self {
        other as u32
    }
}

impl Primitive<f32> for u64 {
    #[inline(always)]
    fn convert(self) -> f32 {
        self as f32
    }

    #[inline(always)]
    fn reverse(other: f32) -> Self {
        other as u64
    }
}

impl Primitive<f32> for u128 {
    #[inline(always)]
    fn convert(self) -> f32 {
        self as f32
    }

    #[inline(always)]
    fn reverse(other: f32) -> Self {
        other as u128
    }
}

impl Primitive<f32> for usize {
    #[inline(always)]
    fn convert(self) -> f32 {
        self as f32
    }

    #[inline(always)]
    fn reverse(other: f32) -> Self {
        other as usize
    }
}

impl Primitive<f64> for u8 {
    #[inline(always)]
    fn convert(self) -> f64 {
        self as f64
    }

    #[inline(always)]
    fn reverse(other: f64) -> Self {
        other as u8
    }
}

impl Primitive<f64> for u16 {
    #[inline(always)]
    fn convert(self) -> f64 {
        self as f64
    }

    #[inline(always)]
    fn reverse(other: f64) -> Self {
        other as u16
    }
}

impl Primitive<f64> for u32 {
    #[inline(always)]
    fn convert(self) -> f64 {
        self as f64
    }

    #[inline(always)]
    fn reverse(other: f64) -> Self {
        other as u32
    }
}

impl Primitive<f64> for u64 {
    #[inline(always)]
    fn convert(self) -> f64 {
        self as f64
    }

    #[inline(always)]
    fn reverse(other: f64) -> Self {
        other as u64
    }
}

impl Primitive<f64> for u128 {
    #[inline(always)]
    fn convert(self) -> f64 {
        self as f64
    }

    #[inline(always)]
    fn reverse(other: f64) -> Self {
        other as u128
    }
}

impl Primitive<f64> for usize {
    #[inline(always)]
    fn convert(self) -> f64 {
        self as f64
    }

    #[inline(always)]
    fn reverse(other: f64) -> Self {
        other as usize
    }
}
