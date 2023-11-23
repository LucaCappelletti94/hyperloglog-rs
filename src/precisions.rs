//! In this document, we define the precisions as a trait and we implement it for structs
//! ranging from Precision4 to Precision16. This is necessary so that the compiler can
//! know the size necessary to store the number of zeros, and allows us the save when using
//! a number of registers equal of inferior to 256 a Byte, compared to what is possible when
//! using a number of registers equal or inferior to 65536, which would make us waste another byte.
//!

use core::ops::{Index, IndexMut};
use core::{fmt::Debug, ops::SubAssign};

use serde::{Deserialize, Serialize};

use crate::{
    array_default::ArrayDefault,
    array_default::ArrayIterArgmin,
    array_default::PrimitiveArray,
    array_default::{ArrayIter, ArrayIterArgmax},
    prelude::{precompute_linear_counting, One},
    primitive::Primitive,
    zeros::Zero,
};

pub trait WordType<const BITS: usize>: Precision {
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
        + IndexMut<usize, Output = u32>
        + Index<usize, Output = u32>
        + Send
        + Sync
        + Eq
        + PartialEq
        + ArrayIter<u32>
        + ArrayDefault<u32>; // = [u32; ceil(PRECISION::NUMBER_OF_REGISTERS, 32 / BITS)]
    /// The register multiplicities is an array with the length of the largest possible
    /// value that can appear in a register. The value at index i is the number of registers
    /// that have a value equal to i, meaning the largest value that any register can have
    /// is the number of registers, i.e. 2^EXPONENT. This is the m parameter in the HyperLogLog.
    /// The lenth of the multiplicities array varies depending on the exponent and the number of bits,
    /// as the length of the array is equal to the maximal value that can be stored in a register.
    /// The value being stored in the register is the number of leading zeros in the hash of the
    /// value that is being inserted.
    /// When the register is of 1 bit, the maximal value is 1, and the length of the array is 2.
    /// When the register is of 2 bits, the maximal value is 3, and the length of the array is 4.
    /// When the register is of 3 bits, the maximal value is 7, and the length of the array is 8.
    /// When the register is of 4 bits, the maximal value is 15, and the length of the array is 16.
    /// When the register is of 5 bits, the maximal value is 31, and the length of the array is 32.
    /// Things start to get interesting for the cases of register at least 6 bits, as the maximal
    /// value representable in 6 bits is 63, but here we have to take into account the fact that
    /// the precision will be the limiting factor. In fact, the higher the precision, the larger
    /// the number of registers, and therefore more bits of the hash will be used to sample the
    /// register associated to a given value. For this reason, in those cases, the maximal value
    /// that we may encounter stored in the register is not 63, but 64 - (PRECISION - 1) + 1.
    /// So, when the register is of 6 bits, we need to consider the precision as follows:
    /// When the precision is 4, the maximal value is 61, and the length of the array is 62.
    /// When the precision is 5, the length of the array is 61.
    /// When the precision is 6, the length of the array is 60.
    /// The decrease in length continues linearly until the maximal precision for which we
    /// have experimental parameters, which is 18, where the length of the array is 48.
    type RegisterMultiplicities: Index<usize, Output = Self::NumberOfZeros>
        + IndexMut<usize, Output = Self::NumberOfZeros>
        + Copy
        + Debug
        + Eq
        + ArrayIterArgmin<Self::NumberOfZeros>
        + ArrayIterArgmax<Self::NumberOfZeros>
        + ArrayDefault<Self::NumberOfZeros>
        + PrimitiveArray<f32, Array=Self::FloatMultiplicities>
        + ArrayIter<Self::NumberOfZeros>;

    type FloatMultiplicities: Index<usize, Output = f32>;
}

pub trait Precision: Default + Copy + Eq + Serialize + Debug + Send + Sync {
    /// The data type to use for the number of zeros registers counter.
    /// This should be the smallest possinle data type that allows us to count
    /// all the registers without overflowing. We can tollerate a one-off error
    /// when counting the number of zeros, as it will be corrected when computing
    /// the cardinality as it is known before hand whether this can happen at all.
    type NumberOfZeros: Copy
        + Debug
        + Eq
        + PartialEq
        + Primitive<usize>
        + Primitive<f32>
        + Send
        + Copy
        + Sync
        + Zero
        + One
        + Ord
        + PartialOrd
        + SubAssign;
    /// The exponent of the number of registers, meaning the number of registers
    /// that will be used is 2^EXPONENT. This is the p parameter in the HyperLogLog.
    const EXPONENT: usize;
    /// The number of registers that will be used.
    const NUMBER_OF_REGISTERS: usize = 1 << Self::EXPONENT;
    /// The maximal number that can be represented with the selected NumberOfZeros.
    const MAXIMAL: usize;
    /// Type for small corrections:
    type SmallCorrections: Index<usize, Output = f32> + Copy;
    type Registers: Index<usize, Output = u32> + Copy + ArrayDefault<u32> + ArrayIter<u32>;
    /// The precomputed small corrections used in the HyperLogLog algorithm for better performance.
    const SMALL_CORRECTIONS: Self::SmallCorrections;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision4;

impl Precision for Precision4 {
    type NumberOfZeros = u8;
    const EXPONENT: usize = 4;
    const MAXIMAL: usize = u8::MAX as usize;
    type SmallCorrections = [f32; 16];
    type Registers = [u32; 16];
    const SMALL_CORRECTIONS: Self::SmallCorrections =
        precompute_linear_counting::<{ Self::NUMBER_OF_REGISTERS }>();
}

impl WordType<1> for Precision4 {
    type Words = [u32; 1];
    type RegisterMultiplicities = [Self::NumberOfZeros; 2];
    type FloatMultiplicities = [f32; 2];
}

impl WordType<2> for Precision4 {
    type Words = [u32; 1];
    type RegisterMultiplicities = [Self::NumberOfZeros; 4];
    type FloatMultiplicities = [f32; 4];
}

impl WordType<3> for Precision4 {
    type Words = [u32; 2];
    type RegisterMultiplicities = [Self::NumberOfZeros; 8];
    type FloatMultiplicities = [f32; 8];
}

impl WordType<4> for Precision4 {
    type Words = [u32; 2];
    type RegisterMultiplicities = [Self::NumberOfZeros; 16];
    type FloatMultiplicities = [f32; 16];
}

impl WordType<5> for Precision4 {
    type Words = [u32; 3];
    type RegisterMultiplicities = [Self::NumberOfZeros; 32];
    type FloatMultiplicities = [f32; 32];
}

impl WordType<6> for Precision4 {
    type Words = [u32; 4];
    type RegisterMultiplicities = [Self::NumberOfZeros; 62];
    type FloatMultiplicities = [f32; 62];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision5;

impl Precision for Precision5 {
    type NumberOfZeros = u8;
    const EXPONENT: usize = 5;
    const MAXIMAL: usize = u8::MAX as usize;
    type SmallCorrections = [f32; 32];
    type Registers = [u32; 32];
    const SMALL_CORRECTIONS: Self::SmallCorrections =
        precompute_linear_counting::<{ Self::NUMBER_OF_REGISTERS }>();
}

impl WordType<1> for Precision5 {
    type Words = [u32; 1];
    type RegisterMultiplicities = [Self::NumberOfZeros; 2];
    type FloatMultiplicities = [f32; 2];
}

impl WordType<2> for Precision5 {
    type Words = [u32; 2];
    type RegisterMultiplicities = [Self::NumberOfZeros; 4];
    type FloatMultiplicities = [f32; 4];
}

impl WordType<3> for Precision5 {
    type Words = [u32; 4];
    type RegisterMultiplicities = [Self::NumberOfZeros; 8];
    type FloatMultiplicities = [f32; 8];
}

impl WordType<4> for Precision5 {
    type Words = [u32; 4];
    type RegisterMultiplicities = [Self::NumberOfZeros; 16];
    type FloatMultiplicities = [f32; 16];
}

impl WordType<5> for Precision5 {
    type Words = [u32; 6];
    type RegisterMultiplicities = [Self::NumberOfZeros; 32];
    type FloatMultiplicities = [f32; 32];
}

impl WordType<6> for Precision5 {
    type Words = [u32; 7];
    type RegisterMultiplicities = [Self::NumberOfZeros; 61];
    type FloatMultiplicities = [f32; 61];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision6;

impl Precision for Precision6 {
    type NumberOfZeros = u8;
    const EXPONENT: usize = 6;
    const MAXIMAL: usize = u8::MAX as usize;
    type SmallCorrections = [f32; 64];
    type Registers = [u32; 64];
    const SMALL_CORRECTIONS: Self::SmallCorrections =
        precompute_linear_counting::<{ Self::NUMBER_OF_REGISTERS }>();
}

impl WordType<1> for Precision6 {
    type Words = [u32; 2];
    type RegisterMultiplicities = [Self::NumberOfZeros; 2];
    type FloatMultiplicities = [f32; 2];
}

impl WordType<2> for Precision6 {
    type Words = [u32; 4];
    type RegisterMultiplicities = [Self::NumberOfZeros; 4];
    type FloatMultiplicities = [f32; 4];
}

impl WordType<3> for Precision6 {
    type Words = [u32; 7];
    type RegisterMultiplicities = [Self::NumberOfZeros; 8];
    type FloatMultiplicities = [f32; 8];
}

impl WordType<4> for Precision6 {
    type Words = [u32; 8];
    type RegisterMultiplicities = [Self::NumberOfZeros; 16];
    type FloatMultiplicities = [f32; 16];
}

impl WordType<5> for Precision6 {
    type Words = [u32; 11];
    type RegisterMultiplicities = [Self::NumberOfZeros; 32];
    type FloatMultiplicities = [f32; 32];
}

impl WordType<6> for Precision6 {
    type Words = [u32; 13];
    type RegisterMultiplicities = [Self::NumberOfZeros; 60];
    type FloatMultiplicities = [f32; 60];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision7;

impl Precision for Precision7 {
    type NumberOfZeros = u8;
    const EXPONENT: usize = 7;
    const MAXIMAL: usize = u8::MAX as usize;
    type SmallCorrections = [f32; 128];
    type Registers = [u32; 128];
    const SMALL_CORRECTIONS: Self::SmallCorrections =
        precompute_linear_counting::<{ Self::NUMBER_OF_REGISTERS }>();
}

impl WordType<1> for Precision7 {
    type Words = [u32; 4];
    type RegisterMultiplicities = [Self::NumberOfZeros; 2];
    type FloatMultiplicities = [f32; 2];
}

impl WordType<2> for Precision7 {
    type Words = [u32; 8];
    type RegisterMultiplicities = [Self::NumberOfZeros; 4];
    type FloatMultiplicities = [f32; 4];
}

impl WordType<3> for Precision7 {
    type Words = [u32; 13];
    type RegisterMultiplicities = [Self::NumberOfZeros; 8];
    type FloatMultiplicities = [f32; 8];
}

impl WordType<4> for Precision7 {
    type Words = [u32; 16];
    type RegisterMultiplicities = [Self::NumberOfZeros; 16];
    type FloatMultiplicities = [f32; 16];
}

impl WordType<5> for Precision7 {
    type Words = [u32; 22];
    type RegisterMultiplicities = [Self::NumberOfZeros; 32];
    type FloatMultiplicities = [f32; 32];
}

impl WordType<6> for Precision7 {
    type Words = [u32; 26];
    type RegisterMultiplicities = [Self::NumberOfZeros; 59];
    type FloatMultiplicities = [f32; 59];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision8;

impl Precision for Precision8 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 8;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrections = [f32; 256];
    type Registers = [u32; 256];
    const SMALL_CORRECTIONS: Self::SmallCorrections =
        precompute_linear_counting::<{ Self::NUMBER_OF_REGISTERS }>();
}
impl WordType<1> for Precision8 {
    type Words = [u32; 8];
    type RegisterMultiplicities = [Self::NumberOfZeros; 2];
    type FloatMultiplicities = [f32; 2];
}

impl WordType<2> for Precision8 {
    type Words = [u32; 16];
    type RegisterMultiplicities = [Self::NumberOfZeros; 4];
    type FloatMultiplicities = [f32; 4];
}

impl WordType<3> for Precision8 {
    type Words = [u32; 26];
    type RegisterMultiplicities = [Self::NumberOfZeros; 8];
    type FloatMultiplicities = [f32; 8];
}

impl WordType<4> for Precision8 {
    type Words = [u32; 32];
    type RegisterMultiplicities = [Self::NumberOfZeros; 16];
    type FloatMultiplicities = [f32; 16];
}

impl WordType<5> for Precision8 {
    type Words = [u32; 43];
    type RegisterMultiplicities = [Self::NumberOfZeros; 32];
    type FloatMultiplicities = [f32; 32];
}

impl WordType<6> for Precision8 {
    type Words = [u32; 52];
    type RegisterMultiplicities = [Self::NumberOfZeros; 58];
    type FloatMultiplicities = [f32; 58];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision9;

impl Precision for Precision9 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 9;
    const MAXIMAL: usize = 65536;
    type SmallCorrections = [f32; 512];
    type Registers = [u32; 512];
    const SMALL_CORRECTIONS: Self::SmallCorrections =
        precompute_linear_counting::<{ Self::NUMBER_OF_REGISTERS }>();
}
impl WordType<1> for Precision9 {
    type Words = [u32; 16];
    type RegisterMultiplicities = [Self::NumberOfZeros; 2];
    type FloatMultiplicities = [f32; 2];
}

impl WordType<2> for Precision9 {
    type Words = [u32; 32];
    type RegisterMultiplicities = [Self::NumberOfZeros; 4];
    type FloatMultiplicities = [f32; 4];
}

impl WordType<3> for Precision9 {
    type Words = [u32; 52];
    type RegisterMultiplicities = [Self::NumberOfZeros; 8];
    type FloatMultiplicities = [f32; 8];
}

impl WordType<4> for Precision9 {
    type Words = [u32; 64];
    type RegisterMultiplicities = [Self::NumberOfZeros; 16];
    type FloatMultiplicities = [f32; 16];
}

impl WordType<5> for Precision9 {
    type Words = [u32; 86];
    type RegisterMultiplicities = [Self::NumberOfZeros; 32];
    type FloatMultiplicities = [f32; 32];
}

impl WordType<6> for Precision9 {
    type Words = [u32; 103];
    type RegisterMultiplicities = [Self::NumberOfZeros; 57];
    type FloatMultiplicities = [f32; 57];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision10;

impl Precision for Precision10 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 10;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrections = [f32; 1024];
    type Registers = [u32; 1024];
    const SMALL_CORRECTIONS: Self::SmallCorrections =
        precompute_linear_counting::<{ Self::NUMBER_OF_REGISTERS }>();
}
impl WordType<1> for Precision10 {
    type Words = [u32; 32];
    type RegisterMultiplicities = [Self::NumberOfZeros; 2];
    type FloatMultiplicities = [f32; 2];
}

impl WordType<2> for Precision10 {
    type Words = [u32; 64];
    type RegisterMultiplicities = [Self::NumberOfZeros; 4];
    type FloatMultiplicities = [f32; 4];
}

impl WordType<3> for Precision10 {
    type Words = [u32; 103];
    type RegisterMultiplicities = [Self::NumberOfZeros; 8];
    type FloatMultiplicities = [f32; 8];
}

impl WordType<4> for Precision10 {
    type Words = [u32; 128];
    type RegisterMultiplicities = [Self::NumberOfZeros; 16];
    type FloatMultiplicities = [f32; 16];
}

impl WordType<5> for Precision10 {
    type Words = [u32; 171];
    type RegisterMultiplicities = [Self::NumberOfZeros; 32];
    type FloatMultiplicities = [f32; 32];
}

impl WordType<6> for Precision10 {
    type Words = [u32; 205];
    type RegisterMultiplicities = [Self::NumberOfZeros; 56];
    type FloatMultiplicities = [f32; 56];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision11;

impl Precision for Precision11 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 11;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrections = [f32; 2048];
    type Registers = [u32; 2048];
    const SMALL_CORRECTIONS: Self::SmallCorrections =
        precompute_linear_counting::<{ Self::NUMBER_OF_REGISTERS }>();
}
impl WordType<1> for Precision11 {
    type Words = [u32; 64];
    type RegisterMultiplicities = [Self::NumberOfZeros; 2];
    type FloatMultiplicities = [f32; 2];
}

impl WordType<2> for Precision11 {
    type Words = [u32; 128];
    type RegisterMultiplicities = [Self::NumberOfZeros; 4];
    type FloatMultiplicities = [f32; 4];
}

impl WordType<3> for Precision11 {
    type Words = [u32; 205];
    type RegisterMultiplicities = [Self::NumberOfZeros; 8];
    type FloatMultiplicities = [f32; 8];
}

impl WordType<4> for Precision11 {
    type Words = [u32; 256];
    type RegisterMultiplicities = [Self::NumberOfZeros; 16];
    type FloatMultiplicities = [f32; 16];
}

impl WordType<5> for Precision11 {
    type Words = [u32; 342];
    type RegisterMultiplicities = [Self::NumberOfZeros; 32];
    type FloatMultiplicities = [f32; 32];
}

impl WordType<6> for Precision11 {
    type Words = [u32; 410];
    type RegisterMultiplicities = [Self::NumberOfZeros; 55];
    type FloatMultiplicities = [f32; 55];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision12;

impl Precision for Precision12 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 12;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrections = [f32; 4096];
    type Registers = [u32; 4096];
    const SMALL_CORRECTIONS: Self::SmallCorrections =
        precompute_linear_counting::<{ Self::NUMBER_OF_REGISTERS }>();
}
impl WordType<1> for Precision12 {
    type Words = [u32; 128];
    type RegisterMultiplicities = [Self::NumberOfZeros; 2];
    type FloatMultiplicities = [f32; 2];
}

impl WordType<2> for Precision12 {
    type Words = [u32; 256];
    type RegisterMultiplicities = [Self::NumberOfZeros; 4];
    type FloatMultiplicities = [f32; 4];
}

impl WordType<3> for Precision12 {
    type Words = [u32; 410];
    type RegisterMultiplicities = [Self::NumberOfZeros; 8];
    type FloatMultiplicities = [f32; 8];
}

impl WordType<4> for Precision12 {
    type Words = [u32; 512];
    type RegisterMultiplicities = [Self::NumberOfZeros; 16];
    type FloatMultiplicities = [f32; 16];
}

impl WordType<5> for Precision12 {
    type Words = [u32; 683];
    type RegisterMultiplicities = [Self::NumberOfZeros; 32];
    type FloatMultiplicities = [f32; 32];
}

impl WordType<6> for Precision12 {
    type Words = [u32; 820];
    type RegisterMultiplicities = [Self::NumberOfZeros; 54];
    type FloatMultiplicities = [f32; 54];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision13;

impl Precision for Precision13 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 13;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrections = [f32; 8192];
    type Registers = [u32; 8192];
    const SMALL_CORRECTIONS: Self::SmallCorrections =
        precompute_linear_counting::<{ Self::NUMBER_OF_REGISTERS }>();
}
impl WordType<1> for Precision13 {
    type Words = [u32; 256];
    type RegisterMultiplicities = [Self::NumberOfZeros; 2];
    type FloatMultiplicities = [f32; 2];
}

impl WordType<2> for Precision13 {
    type Words = [u32; 512];
    type RegisterMultiplicities = [Self::NumberOfZeros; 4];
    type FloatMultiplicities = [f32; 4];
}

impl WordType<3> for Precision13 {
    type Words = [u32; 820];
    type RegisterMultiplicities = [Self::NumberOfZeros; 8];
    type FloatMultiplicities = [f32; 8];
}

impl WordType<4> for Precision13 {
    type Words = [u32; 1024];
    type RegisterMultiplicities = [Self::NumberOfZeros; 16];
    type FloatMultiplicities = [f32; 16];
}

impl WordType<5> for Precision13 {
    type Words = [u32; 1366];
    type RegisterMultiplicities = [Self::NumberOfZeros; 32];
    type FloatMultiplicities = [f32; 32];
}

impl WordType<6> for Precision13 {
    type Words = [u32; 1639];
    type RegisterMultiplicities = [Self::NumberOfZeros; 53];
    type FloatMultiplicities = [f32; 53];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision14;

impl Precision for Precision14 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 14;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrections = [f32; 16384];
    type Registers = [u32; 16384];
    const SMALL_CORRECTIONS: Self::SmallCorrections =
        precompute_linear_counting::<{ Self::NUMBER_OF_REGISTERS }>();
}
impl WordType<1> for Precision14 {
    type Words = [u32; 512];
    type RegisterMultiplicities = [Self::NumberOfZeros; 2];
    type FloatMultiplicities = [f32; 2];
}

impl WordType<2> for Precision14 {
    type Words = [u32; 1024];
    type RegisterMultiplicities = [Self::NumberOfZeros; 4];
    type FloatMultiplicities = [f32; 4];
}

impl WordType<3> for Precision14 {
    type Words = [u32; 1639];
    type RegisterMultiplicities = [Self::NumberOfZeros; 8];
    type FloatMultiplicities = [f32; 8];
}

impl WordType<4> for Precision14 {
    type Words = [u32; 2048];
    type RegisterMultiplicities = [Self::NumberOfZeros; 16];
    type FloatMultiplicities = [f32; 16];
}

impl WordType<5> for Precision14 {
    type Words = [u32; 2731];
    type RegisterMultiplicities = [Self::NumberOfZeros; 32];
    type FloatMultiplicities = [f32; 32];
}

impl WordType<6> for Precision14 {
    type Words = [u32; 3277];
    type RegisterMultiplicities = [Self::NumberOfZeros; 52];
    type FloatMultiplicities = [f32; 52];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision15;

impl Precision for Precision15 {
    type NumberOfZeros = u16;
    const EXPONENT: usize = 15;
    const MAXIMAL: usize = u16::MAX as usize;
    type SmallCorrections = [f32; 32768];
    type Registers = [u32; 32768];
    const SMALL_CORRECTIONS: Self::SmallCorrections =
        precompute_linear_counting::<{ Self::NUMBER_OF_REGISTERS }>();
}
impl WordType<1> for Precision15 {
    type Words = [u32; 1024];
    type RegisterMultiplicities = [Self::NumberOfZeros; 2];
    type FloatMultiplicities = [f32; 2];
}

impl WordType<2> for Precision15 {
    type Words = [u32; 2048];
    type RegisterMultiplicities = [Self::NumberOfZeros; 4];
    type FloatMultiplicities = [f32; 4];
}

impl WordType<3> for Precision15 {
    type Words = [u32; 3277];
    type RegisterMultiplicities = [Self::NumberOfZeros; 8];
    type FloatMultiplicities = [f32; 8];
}

impl WordType<4> for Precision15 {
    type Words = [u32; 4096];
    type RegisterMultiplicities = [Self::NumberOfZeros; 16];
    type FloatMultiplicities = [f32; 16];
}

impl WordType<5> for Precision15 {
    type Words = [u32; 5462];
    type RegisterMultiplicities = [Self::NumberOfZeros; 32];
    type FloatMultiplicities = [f32; 32];
}

impl WordType<6> for Precision15 {
    type Words = [u32; 6554];
    type RegisterMultiplicities = [Self::NumberOfZeros; 51];
    type FloatMultiplicities = [f32; 51];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision16;

impl Precision for Precision16 {
    // For Precision 16, we are still able to use a u16, but we
    // will loose the ability to count one of the registers.
    type NumberOfZeros = u32;
    const EXPONENT: usize = 16;
    const MAXIMAL: usize = u32::MAX as usize;
    type SmallCorrections = [f32; 65536];
    type Registers = [u32; 65536];
    const SMALL_CORRECTIONS: Self::SmallCorrections =
        precompute_linear_counting::<{ Self::NUMBER_OF_REGISTERS }>();
}
impl WordType<1> for Precision16 {
    type Words = [u32; 2048];
    type RegisterMultiplicities = [Self::NumberOfZeros; 2];
    type FloatMultiplicities = [f32; 2];
}

impl WordType<2> for Precision16 {
    type Words = [u32; 4096];
    type RegisterMultiplicities = [Self::NumberOfZeros; 4];
    type FloatMultiplicities = [f32; 4];
}

impl WordType<3> for Precision16 {
    type Words = [u32; 6554];
    type RegisterMultiplicities = [Self::NumberOfZeros; 8];
    type FloatMultiplicities = [f32; 8];
}

impl WordType<4> for Precision16 {
    type Words = [u32; 8192];
    type RegisterMultiplicities = [Self::NumberOfZeros; 16];
    type FloatMultiplicities = [f32; 16];
}

impl WordType<5> for Precision16 {
    type Words = [u32; 10923];
    type RegisterMultiplicities = [Self::NumberOfZeros; 32];
    type FloatMultiplicities = [f32; 32];
}

impl WordType<6> for Precision16 {
    type Words = [u32; 13108];
    type RegisterMultiplicities = [Self::NumberOfZeros; 50];
    type FloatMultiplicities = [f32; 50];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision17;

impl Precision for Precision17 {
    type NumberOfZeros = u32;
    const EXPONENT: usize = 17;
    const MAXIMAL: usize = u32::MAX as usize;
    type SmallCorrections = [f32; 131072];
    type Registers = [u32; 131072];
    const SMALL_CORRECTIONS: Self::SmallCorrections =
        precompute_linear_counting::<{ Self::NUMBER_OF_REGISTERS }>();
}
impl WordType<1> for Precision17 {
    type Words = [u32; 4096];
    type RegisterMultiplicities = [Self::NumberOfZeros; 2];
    type FloatMultiplicities = [f32; 2];
}

impl WordType<2> for Precision17 {
    type Words = [u32; 8192];
    type RegisterMultiplicities = [Self::NumberOfZeros; 4];
    type FloatMultiplicities = [f32; 4];
}

impl WordType<3> for Precision17 {
    type Words = [u32; 13108];
    type RegisterMultiplicities = [Self::NumberOfZeros; 8];
    type FloatMultiplicities = [f32; 8];
}

impl WordType<4> for Precision17 {
    type Words = [u32; 16384];
    type RegisterMultiplicities = [Self::NumberOfZeros; 16];
    type FloatMultiplicities = [f32; 16];
}

impl WordType<5> for Precision17 {
    type Words = [u32; 21846];
    type RegisterMultiplicities = [Self::NumberOfZeros; 32];
    type FloatMultiplicities = [f32; 32];
}

impl WordType<6> for Precision17 {
    type Words = [u32; 26215];
    type RegisterMultiplicities = [Self::NumberOfZeros; 49];
    type FloatMultiplicities = [f32; 49];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Precision18;

impl Precision for Precision18 {
    type NumberOfZeros = u32;
    const EXPONENT: usize = 18;
    const MAXIMAL: usize = u32::MAX as usize;
    type SmallCorrections = [f32; 262144];
    type Registers = [u32; 262144];
    const SMALL_CORRECTIONS: Self::SmallCorrections =
        precompute_linear_counting::<{ Self::NUMBER_OF_REGISTERS }>();
}
impl WordType<1> for Precision18 {
    type Words = [u32; 8192];
    type RegisterMultiplicities = [Self::NumberOfZeros; 2];
    type FloatMultiplicities = [f32; 2];
}

impl WordType<2> for Precision18 {
    type Words = [u32; 16384];
    type RegisterMultiplicities = [Self::NumberOfZeros; 4];
    type FloatMultiplicities = [f32; 4];
}

impl WordType<3> for Precision18 {
    type Words = [u32; 26215];
    type RegisterMultiplicities = [Self::NumberOfZeros; 8];
    type FloatMultiplicities = [f32; 8];
}

impl WordType<4> for Precision18 {
    type Words = [u32; 32768];
    type RegisterMultiplicities = [Self::NumberOfZeros; 16];
    type FloatMultiplicities = [f32; 16];
}

impl WordType<5> for Precision18 {
    type Words = [u32; 43691];
    type RegisterMultiplicities = [Self::NumberOfZeros; 32];
    type FloatMultiplicities = [f32; 32];
}

impl WordType<6> for Precision18 {
    type Words = [u32; 52429];
    type RegisterMultiplicities = [Self::NumberOfZeros; 48];
    type FloatMultiplicities = [f32; 48];
}
