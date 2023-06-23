//! In this document, we define the precisions as a trait and we implement it for structs
//! ranging from Precision4 to Precision16. This is necessary so that the compiler can
//! know the size necessary to store the number of zeros, and allows us the save when using
//! a number of registers equal of inferior to 256 a Byte, compared to what is possible when
//! using a number of registers equal or inferior to 65536, which would make us waste another byte.
//!

use core::{fmt::Debug, ops::SubAssign};
use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};

use crate::{
    array_default::ArrayDefault, array_default::ArrayIter, atomic_alias::AtomicAlias, prelude::precompute_linear_counting,
    primitive::Primitive, zeros::Zero,
};

pub trait Precision<const BITS: usize>:
    Default + Copy + Eq + Serialize + Debug + Send + Sync
{
    /// The data type to use for the number of zeros registers counter.
    /// This should be the smallest possinle data type that allows us to count
    /// all the registers without overflowing. We can tollerate a one-off error
    /// when counting the number of zeros, as it will be corrected when computing
    /// the cardinality as it is known before hand whether this can happen at all.
    type NumberOfZeros: Copy
        + Debug
        + AtomicAlias
        + Eq
        + PartialEq
        + Primitive<usize>
        + Send
        + Sync
        + Zero
        + Ord
        + PartialOrd
        + SubAssign;
    /// The exponent of the number of registers, meaning the number of registers
    /// that will be used is 2^EXPONENT.
    const EXPONENT: usize;
    /// The number of registers that will be used.
    const NUMBER_OF_REGISTERS: usize = 1 << Self::EXPONENT;
    /// The maximal number that can be represented with the selected NumberOfZeros.
    const MAXIMAL: usize;
    /// Type for small corrections:
    type SmallCorrrections: Index<usize, Output = f32> + Copy;
    /// The precomputed small corrections used in the HyperLogLog algorithm for better performance.
    const SMALL_CORRECTIONS: Self::SmallCorrrections;
    /// The type to use for the associated vector of words.
    /// The type of Words is always an array of u32, as this is the smallest
    /// type that can be used to store the number of registers.
    ///
    /// The length of the array is: ceil(PRECISION::NUMBER_OF_REGISTERS, 32 / BITS)
    ///
    /// We cannot use the above expression directly, as it would force the library
    /// user to propagate some very ugly constraints around.
    ///
    type Words: Copy
        + Debug
        + AtomicAlias
        + IndexMut<usize, Output = u32>
        + Index<usize, Output = u32>
        + Send
        + Sync
        + Eq
        + PartialEq
        + ArrayIter<u32>
        + ArrayDefault<u32>; // = [u32; ceil(PRECISION::NUMBER_OF_REGISTERS, 32 / BITS)]
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision4;

impl Precision<4> for Precision4 {
    type NumberOfZeros = u8;
    const EXPONENT: usize = 4;
    const MAXIMAL: usize = u8::MAX as usize;
    type SmallCorrrections = [f32; 16];
    const SMALL_CORRECTIONS: [f32; 16] = precompute_linear_counting::<16>();
    type Words = [u32; 2];
}

impl Precision<5> for Precision4 {
    type NumberOfZeros = u8;
    const EXPONENT: usize = 4;
    const MAXIMAL: usize = u8::MAX as usize;
    type SmallCorrrections = [f32; 16];
    const SMALL_CORRECTIONS: [f32; 16] = precompute_linear_counting::<16>();
    type Words = [u32; 3];
}

impl Precision<6> for Precision4 {
    type NumberOfZeros = u8;
    const EXPONENT: usize = 4;
    const MAXIMAL: usize = u8::MAX as usize;
    type SmallCorrrections = [f32; 16];
    const SMALL_CORRECTIONS: [f32; 16] = precompute_linear_counting::<16>();
    type Words = [u32; 4];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision5;

impl Precision<4> for Precision5 {
    type NumberOfZeros = u8;
    const EXPONENT: usize = 5;
    const MAXIMAL: usize = u8::MAX as usize;
    type SmallCorrrections = [f32; 32];
    const SMALL_CORRECTIONS: [f32; 32] = precompute_linear_counting::<32>();
    type Words = [u32; 4];
}

impl Precision<5> for Precision5 {
    type NumberOfZeros = u8;
    const EXPONENT: usize = 5;
    const MAXIMAL: usize = u8::MAX as usize;
    type SmallCorrrections = [f32; 32];
    const SMALL_CORRECTIONS: [f32; 32] = precompute_linear_counting::<32>();
    type Words = [u32; 6];
}

impl Precision<6> for Precision5 {
    type NumberOfZeros = u8;
    const EXPONENT: usize = 5;
    const MAXIMAL: usize = u8::MAX as usize;
    type SmallCorrrections = [f32; 32];
    const SMALL_CORRECTIONS: [f32; 32] = precompute_linear_counting::<32>();
    type Words = [u32; 7];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision6;

impl Precision<4> for Precision6 {
    type NumberOfZeros = u8;
    const EXPONENT: usize = 6;
    const MAXIMAL: usize = u8::MAX as usize;
    type SmallCorrrections = [f32; 64];
    const SMALL_CORRECTIONS: [f32; 64] = precompute_linear_counting::<64>();
    type Words = [u32; 8];
}

impl Precision<5> for Precision6 {
    type NumberOfZeros = u8;
    const EXPONENT: usize = 6;
    const MAXIMAL: usize = u8::MAX as usize;
    type SmallCorrrections = [f32; 64];
    const SMALL_CORRECTIONS: [f32; 64] = precompute_linear_counting::<64>();
    type Words = [u32; 11];
}

impl Precision<6> for Precision6 {
    type NumberOfZeros = u8;
    const EXPONENT: usize = 6;
    const MAXIMAL: usize = u8::MAX as usize;
    type SmallCorrrections = [f32; 64];
    const SMALL_CORRECTIONS: [f32; 64] = precompute_linear_counting::<64>();
    type Words = [u32; 13];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision7;

impl Precision<4> for Precision7 {
    type NumberOfZeros = u8;
    const EXPONENT: usize = 7;
    const MAXIMAL: usize = u8::MAX as usize;
    type SmallCorrrections = [f32; 128];
    const SMALL_CORRECTIONS: [f32; 128] = precompute_linear_counting::<128>();
    type Words = [u32; 16];
}

impl Precision<5> for Precision7 {
    type NumberOfZeros = u8;
    const EXPONENT: usize = 7;
    const MAXIMAL: usize = u8::MAX as usize;
    type SmallCorrrections = [f32; 128];
    const SMALL_CORRECTIONS: [f32; 128] = precompute_linear_counting::<128>();
    type Words = [u32; 22];
}

impl Precision<6> for Precision7 {
    type NumberOfZeros = u8;
    const EXPONENT: usize = 7;
    const MAXIMAL: usize = u8::MAX as usize;
    type SmallCorrrections = [f32; 128];
    const SMALL_CORRECTIONS: [f32; 128] = precompute_linear_counting::<128>();
    type Words = [u32; 26];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision8;

impl Precision<4> for Precision8 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 8;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 256];
    const SMALL_CORRECTIONS: [f32; 256] = precompute_linear_counting::<256>();
    type Words = [u32; 32];
}

impl Precision<5> for Precision8 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 8;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 256];
    const SMALL_CORRECTIONS: [f32; 256] = precompute_linear_counting::<256>();
    type Words = [u32; 43];
}

impl Precision<6> for Precision8 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 8;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 256];
    const SMALL_CORRECTIONS: [f32; 256] = precompute_linear_counting::<256>();
    type Words = [u32; 52];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision9;

impl Precision<4> for Precision9 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 9;
    const MAXIMAL: usize = 65536;
    type SmallCorrrections = [f32; 512];
    const SMALL_CORRECTIONS: [f32; 512] = precompute_linear_counting::<512>();
    type Words = [u32; 64];
}

impl Precision<5> for Precision9 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 9;
    const MAXIMAL: usize = 65536;
    type SmallCorrrections = [f32; 512];
    const SMALL_CORRECTIONS: [f32; 512] = precompute_linear_counting::<512>();
    type Words = [u32; 86];
}

impl Precision<6> for Precision9 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 9;
    const MAXIMAL: usize = 65536;
    type SmallCorrrections = [f32; 512];
    const SMALL_CORRECTIONS: [f32; 512] = precompute_linear_counting::<512>();
    type Words = [u32; 103];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision10;

impl Precision<4> for Precision10 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 10;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 1024];
    const SMALL_CORRECTIONS: [f32; 1024] = precompute_linear_counting::<1024>();
    type Words = [u32; 128];
}

impl Precision<5> for Precision10 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 10;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 1024];
    const SMALL_CORRECTIONS: [f32; 1024] = precompute_linear_counting::<1024>();
    type Words = [u32; 171];
}

impl Precision<6> for Precision10 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 10;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 1024];
    const SMALL_CORRECTIONS: [f32; 1024] = precompute_linear_counting::<1024>();
    type Words = [u32; 205];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision11;

impl Precision<4> for Precision11 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 11;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 2048];
    const SMALL_CORRECTIONS: [f32; 2048] = precompute_linear_counting::<2048>();
    type Words = [u32; 256];
}

impl Precision<5> for Precision11 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 11;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 2048];
    const SMALL_CORRECTIONS: [f32; 2048] = precompute_linear_counting::<2048>();
    type Words = [u32; 342];
}

impl Precision<6> for Precision11 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 11;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 2048];
    const SMALL_CORRECTIONS: [f32; 2048] = precompute_linear_counting::<2048>();
    type Words = [u32; 410];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision12;

impl Precision<4> for Precision12 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 12;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 4096];
    const SMALL_CORRECTIONS: [f32; 4096] = precompute_linear_counting::<4096>();
    type Words = [u32; 512];
}

impl Precision<5> for Precision12 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 12;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 4096];
    const SMALL_CORRECTIONS: [f32; 4096] = precompute_linear_counting::<4096>();
    type Words = [u32; 683];
}

impl Precision<6> for Precision12 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 12;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 4096];
    const SMALL_CORRECTIONS: [f32; 4096] = precompute_linear_counting::<4096>();
    type Words = [u32; 820];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision13;

impl Precision<4> for Precision13 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 13;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 8192];
    const SMALL_CORRECTIONS: [f32; 8192] = precompute_linear_counting::<8192>();
    type Words = [u32; 1024];
}

impl Precision<5> for Precision13 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 13;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 8192];
    const SMALL_CORRECTIONS: [f32; 8192] = precompute_linear_counting::<8192>();
    type Words = [u32; 1366];
}

impl Precision<6> for Precision13 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 13;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 8192];
    const SMALL_CORRECTIONS: [f32; 8192] = precompute_linear_counting::<8192>();
    type Words = [u32; 1639];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision14;

impl Precision<4> for Precision14 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 14;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 16384];
    const SMALL_CORRECTIONS: [f32; 16384] = precompute_linear_counting::<16384>();
    type Words = [u32; 2048];
}

impl Precision<5> for Precision14 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 14;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 16384];
    const SMALL_CORRECTIONS: [f32; 16384] = precompute_linear_counting::<16384>();
    type Words = [u32; 2731];
}

impl Precision<6> for Precision14 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 14;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 16384];
    const SMALL_CORRECTIONS: [f32; 16384] = precompute_linear_counting::<16384>();
    type Words = [u32; 3277];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision15;

impl Precision<4> for Precision15 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 15;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 32768];
    const SMALL_CORRECTIONS: [f32; 32768] = precompute_linear_counting::<32768>();
    type Words = [u32; 4096];
}

impl Precision<5> for Precision15 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 15;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 32768];
    const SMALL_CORRECTIONS: [f32; 32768] = precompute_linear_counting::<32768>();
    type Words = [u32; 5462];
}

impl Precision<6> for Precision15 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 15;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrrections = [f32; 32768];
    const SMALL_CORRECTIONS: [f32; 32768] = precompute_linear_counting::<32768>();
    type Words = [u32; 6554];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision16;

impl Precision<4> for Precision16 {
    // For Precision 16, we are still able to use a u16, but we
    // will loose the ability to count one of the registers.
    type NumberOfZeros = u32;
    const EXPONENT: usize = 16;
    const MAXIMAL: usize = u32::MAX as usize;
    type SmallCorrrections = [f32; 65536];
    const SMALL_CORRECTIONS: [f32; 65536] = precompute_linear_counting::<65536>();
    type Words = [u32; 8192];
}

impl Precision<5> for Precision16 {
    type NumberOfZeros = u32;
    const EXPONENT: usize = 16;
    const MAXIMAL: usize = u32::MAX as usize;
    type SmallCorrrections = [f32; 65536];
    const SMALL_CORRECTIONS: [f32; 65536] = precompute_linear_counting::<65536>();
    type Words = [u32; 10923];
}

impl Precision<6> for Precision16 {
    type NumberOfZeros = u32;
    const EXPONENT: usize = 16;
    const MAXIMAL: usize = u32::MAX as usize;
    type SmallCorrrections = [f32; 65536];
    const SMALL_CORRECTIONS: [f32; 65536] = precompute_linear_counting::<65536>();
    type Words = [u32; 13108];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision17;

impl Precision<4> for Precision17 {
    type NumberOfZeros = u32;
    const EXPONENT: usize = 17;
    const MAXIMAL: usize = u32::MAX as usize;
    type SmallCorrrections = [f32; 131072];
    const SMALL_CORRECTIONS: [f32; 131072] = precompute_linear_counting::<131072>();
    type Words = [u32; 16384];
}

impl Precision<5> for Precision17 {
    type NumberOfZeros = u32;
    const EXPONENT: usize = 17;
    const MAXIMAL: usize = u32::MAX as usize;
    type SmallCorrrections = [f32; 131072];
    const SMALL_CORRECTIONS: [f32; 131072] = precompute_linear_counting::<131072>();
    type Words = [u32; 21846];
}

impl Precision<6> for Precision17 {
    type NumberOfZeros = u32;
    const EXPONENT: usize = 17;
    const MAXIMAL: usize = u32::MAX as usize;
    type SmallCorrrections = [f32; 131072];
    const SMALL_CORRECTIONS: [f32; 131072] = precompute_linear_counting::<131072>();
    type Words = [u32; 26215];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision18;

impl Precision<4> for Precision18 {
    type NumberOfZeros = u32;
    const EXPONENT: usize = 18;
    const MAXIMAL: usize = u32::MAX as usize;
    type SmallCorrrections = [f32; 262144];
    const SMALL_CORRECTIONS: [f32; 262144] = precompute_linear_counting::<262144>();
    type Words = [u32; 32768];
}

impl Precision<5> for Precision18 {
    type NumberOfZeros = u32;
    const EXPONENT: usize = 18;
    const MAXIMAL: usize = u32::MAX as usize;
    type SmallCorrrections = [f32; 262144];
    const SMALL_CORRECTIONS: [f32; 262144] = precompute_linear_counting::<262144>();
    type Words = [u32; 43691];
}

impl Precision<6> for Precision18 {
    type NumberOfZeros = u32;
    const EXPONENT: usize = 18;
    const MAXIMAL: usize = u32::MAX as usize;
    type SmallCorrrections = [f32; 262144];
    const SMALL_CORRECTIONS: [f32; 262144] = precompute_linear_counting::<262144>();
    type Words = [u32; 52429];
}
