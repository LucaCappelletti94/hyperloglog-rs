//! In this document, we define the precisions as a trait and we implement it for structs
//! ranging from Precision4 to Precision16. This is necessary so that the compiler can
//! know the size necessary to store the number of zeros, and allows us the save when using
//! a number of registers equal of inferior to 256 a Byte, compared to what is possible when
//! using a number of registers equal or inferior to 65536, which would make us waste another byte.
//!

include!(concat!(env!("OUT_DIR"), "/linear_counting_corrections.rs"));

use core::ops::{Index, IndexMut};
use core::{fmt::Debug, ops::SubAssign};

use crate::{
    array_default::ArrayDefault,
    array_default::ArrayIterArgmin,
    array_default::PrimitiveArray,
    array_default::{ArrayIter, ArrayIterArgmax},
    prelude::One,
    primitive::Primitive,
    zeros::Zero,
};

pub trait WordType<const BITS: usize>: Precision {
    /// The type to use for the associated vector of words.
    /// The type of Words is always an array of u32, as this is the smallest
    /// type that can be used to store the number of registers.
    ///
    /// The length of the array is: ceil(P::NUMBER_OF_REGISTERS, 32 / BITS)
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
        + ArrayDefault<u32>; // = [u32; ceil(P::NUMBER_OF_REGISTERS, 32 / BITS)]
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
        + PrimitiveArray<f32, Array = Self::FloatMultiplicities>
        + ArrayIter<Self::NumberOfZeros>;

    type FloatMultiplicities: Index<usize, Output = f32>;
}

pub trait Precision: Default + Copy + Eq + Debug + Send + Sync {
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
    /// Type for small corrections:
    type SmallCorrections: Index<usize, Output = f32> + Copy;
    type Registers: Index<usize, Output = u32> + Copy + ArrayDefault<u32> + ArrayIter<u32>;
    /// The precomputed small corrections used in the HyperLogLog algorithm for better performance.
    const SMALL_CORRECTIONS: Self::SmallCorrections;
}

/// Macro to map a given precision exponent to the adequate number of zeros data type to use.
macro_rules! impl_number_of_zeros {
    (4) => {
        u8
    };
    (5) => {
        u8
    };
    (6) => {
        u8
    };
    (7) => {
        u8
    };
    (8) => {
        u16
    };
    (9) => {
        u16
    };
    (10) => {
        u16
    };
    (11) => {
        u16
    };
    (12) => {
        u16
    };
    (13) => {
        u16
    };
    (14) => {
        u16
    };
    (15) => {
        u16
    };
    (16) => {
        u32
    };
    // Add more mappings as needed
    ($n:expr) => {
        compile_error!(concat!(
            "No type mapping defined for number: ",
            stringify!($n)
        ));
    };
}

/// Macro to map a given number of bits and precision exponent to the maximal multiplicity value.
macro_rules! impl_maximal_multeplicity {
    (1, $precision: expr) => {
        2
    };
    (2, $precision: expr) => {
        4
    };
    (3, $precision: expr) => {
        8
    };
    (4, $precision: expr) => {
        16
    };
    (5, $precision: expr) => {
        32
    };
    (6, $precision: expr) => {
        64 - $precision
    };
    ($bits: expr, $precision: expr) => {
        compile_error!(concat!(
            "No maximal multiplicity defined for bits: ",
            stringify!($bits),
            " and precision: ",
            stringify!($precision)
        ));
    };
}

/// Macro to implement WordType for a given precision and number of bits.
macro_rules! impl_word_type {
    ($exponent:expr, $bits:expr) => {
        paste::paste! {
            impl WordType<$bits> for [<Precision $exponent>] {
                type Words = [u32; crate::utils::ceil(Self::NUMBER_OF_REGISTERS, 32 / $bits)];
                type RegisterMultiplicities = [Self::NumberOfZeros; impl_maximal_multeplicity!($bits, $exponent)];
                type FloatMultiplicities = [f32; impl_maximal_multeplicity!($bits, $exponent)];
            }
        }
    };
}

/// Macro to implement the Precision trait for a given precision.
macro_rules! impl_precision {
    ($exponent:expr) => {
        paste::paste! {
            #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub struct [<Precision $exponent>];

            impl Precision for [<Precision $exponent>] {
                type NumberOfZeros = impl_number_of_zeros!($exponent);
                const EXPONENT: usize = $exponent;
                type SmallCorrections = [f32; usize::pow(2, $exponent)];
                type Registers = [u32; usize::pow(2, $exponent)];
                const SMALL_CORRECTIONS: Self::SmallCorrections = [<LINEAR_COUNTING_CORRECTIONS_ $exponent>];
            }

            impl_word_type!($exponent, 1);
            impl_word_type!($exponent, 2);
            impl_word_type!($exponent, 3);
            impl_word_type!($exponent, 4);
            impl_word_type!($exponent, 5);
            impl_word_type!($exponent, 6);
        }
    };
}

/// Macro to implement the Precision trait for a list of precisions.
macro_rules! impl_precisions {
    ($($exponent:expr),*) => {
        $(
            impl_precision!($exponent);
        )*
    };
}

impl_precisions!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
