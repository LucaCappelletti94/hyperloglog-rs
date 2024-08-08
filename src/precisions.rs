//! In this document, we define the precisions as a trait and we implement it for structs
//! ranging from Precision4 to Precision16. This is necessary so that the compiler can
//! know the size necessary to store the number of zeros, and allows us the save when using
//! a number of registers equal of inferior to 256 a Byte, compared to what is possible when
//! using a number of registers equal or inferior to 65536, which would make us waste another byte.

use core::fmt::Debug;
use core::ops::Index;

use crate::{array_default::ArrayIter, utils::*};

include!("bias.rs");
include!(concat!(env!("OUT_DIR"), "/log_values.rs"));
include!(concat!(env!("OUT_DIR"), "/alpha_values.rs"));
include!(concat!(env!("OUT_DIR"), "/linear_count_zeros.rs"));

pub trait Precision: Default + Copy + Eq + Debug + Send + Sync {
    /// The data type to use for the number of zeros registers counter.
    /// This should be the smallest possinle data type that allows us to count
    /// all the registers without overflowing. We can tollerate a one-off error
    /// when counting the number of zeros, as it will be corrected when computing
    /// the cardinality as it is known before hand whether this can happen at all.
    type NumberOfZeros: PositiveIntegerNumber;
    /// The exponent of the number of registers, meaning the number of registers
    /// that will be used is 2^EXPONENT. This is the p parameter in the HyperLogLog.
    const EXPONENT: usize;
    /// The number of registers that will be used.
    const NUMBER_OF_REGISTERS: usize = 1 << Self::EXPONENT;

    /// Type for small corrections:
    type SmallCorrections: Index<usize, Output = f64> + Copy;
    type Registers: Copy + ArrayIter<u32>;

    fn error_rate() -> f64 {
        let exponent = (Self::EXPONENT as f64) / 2.0;
        1.04 / 2f64.powf(exponent)
    }
}

pub trait PrecisionConstants<F: FloatNumber>: Precision {
    const ALPHA: F;
    const LINEAR_COUNT_ZEROS: Self::NumberOfZeros;
    const NUMBER_OF_REGISTERS_FLOAT: F;
    /// Estimates vector associated to the precision.
    type EstimatesType: ArrayIter<F> + Copy;
    const ESTIMATES: Self::EstimatesType;
    /// Bias vector associated to the precision.
    type BiasType: ArrayIter<F> + Copy;
    const BIAS: Self::BiasType;

    fn small_correction(number_of_zero_registers: Self::NumberOfZeros) -> F;

    #[inline(always)]
    fn bias(harmonic_sum: F) -> F {
        // Raw estimate is first/last in estimates. Return the first/last bias.
        if harmonic_sum <= Self::ESTIMATES[0] {
            Self::BIAS[0]
        } else if Self::ESTIMATES[Self::ESTIMATES.len() - 1] <= harmonic_sum {
            Self::BIAS[Self::BIAS.len() - 1]
        } else {
            // Raw estimate is somewhere in between estimates.
            // Binary search for the calculated raw estimate.
            //
            // Here we unwrap because neither the values in `estimates`
            // nor `raw` are going to be NaN.
            let partition_index = Self::ESTIMATES.partition_point(|est| *est <= harmonic_sum);

            // Return linear interpolation between raw's neighboring points.
            let ratio = (harmonic_sum - Self::ESTIMATES[partition_index - 1])
                / (Self::ESTIMATES[partition_index] - Self::ESTIMATES[partition_index - 1]);

            // Calculate bias.
            Self::BIAS[partition_index]
                + ratio * (Self::BIAS[partition_index] - Self::BIAS[partition_index - 1])
        }
    }

    #[inline(always)]
    fn adjust_estimate(mut harmonic_sum: F) -> F {
        // Apply the final scaling factor to obtain the estimate of the cardinality
        harmonic_sum =
            Self::ALPHA * Self::NUMBER_OF_REGISTERS_FLOAT * Self::NUMBER_OF_REGISTERS_FLOAT
                / harmonic_sum;

        // Apply the small range correction factor if the raw estimate is below the threshold
        // and there are zero registers in the counter.
        if harmonic_sum <= F::FIVE * Self::NUMBER_OF_REGISTERS_FLOAT {
            harmonic_sum - Self::bias(harmonic_sum)
        } else {
            harmonic_sum
        }
    }
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

/// Macro to implement the Precision trait for a given precision.
macro_rules! impl_precision {
    ($exponent:expr) => {
        paste::paste! {
            #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            #[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
            pub struct [<Precision $exponent>];

            impl PrecisionConstants<f32> for [<Precision $exponent>] {
                const ALPHA: f32 = ALPHA_VALUES[Self::EXPONENT - 4] as f32;
                const LINEAR_COUNT_ZEROS: Self::NumberOfZeros = LINEAR_COUNT_ZEROS[Self::EXPONENT - 4] as Self::NumberOfZeros;
                const NUMBER_OF_REGISTERS_FLOAT: f32 = Self::NUMBER_OF_REGISTERS as f32;
                type BiasType = [f32; bias_size_from_precision!($exponent)];
                const BIAS: Self::BiasType = bias_from_precision!($exponent);
                type EstimatesType = [f32; bias_size_from_precision!($exponent)];
                const ESTIMATES: Self::EstimatesType = estimates_from_precision!($exponent);

                #[inline(always)]
                fn small_correction(number_of_zero_registers: Self::NumberOfZeros) -> f32 {
                    <Self as PrecisionConstants<f64>>::small_correction(number_of_zero_registers) as f32
                }
            }

            impl PrecisionConstants<f64> for [<Precision $exponent>] {
                const ALPHA: f64 = ALPHA_VALUES[Self::EXPONENT - 4];
                const LINEAR_COUNT_ZEROS: Self::NumberOfZeros = LINEAR_COUNT_ZEROS[Self::EXPONENT - 4] as Self::NumberOfZeros;
                const NUMBER_OF_REGISTERS_FLOAT: f64 = Self::NUMBER_OF_REGISTERS as f64;
                type BiasType = [f64; bias_size_from_precision!($exponent)];
                const BIAS: Self::BiasType = bias_from_precision!($exponent);
                type EstimatesType = [f64; bias_size_from_precision!($exponent)];
                const ESTIMATES: Self::EstimatesType = estimates_from_precision!($exponent);

                #[inline(always)]
                fn small_correction(number_of_zero_registers: Self::NumberOfZeros) -> f64 {
                    Self::NUMBER_OF_REGISTERS as f64
                        * (LOG_VALUES[Self::NUMBER_OF_REGISTERS] - LOG_VALUES[<Self::NumberOfZeros as TryInto<usize>>::try_into(number_of_zero_registers).unwrap()])
                }
            }

            impl Precision for [<Precision $exponent>] {
                type NumberOfZeros = impl_number_of_zeros!($exponent);
                const EXPONENT: usize = $exponent;
                type SmallCorrections = [f64; usize::pow(2, $exponent)];
                type Registers = [u32; usize::pow(2, $exponent)];
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_error_rate_simmetry<P: Precision>() {
        let error_rate = P::error_rate();
        let exponent = (f64::log2(1.04 / error_rate) * 2.0).ceil();
        assert_eq!(exponent as usize, P::EXPONENT);
    }

    macro_rules! test_error_rate_simmetry {
        ($($exponent:expr),*) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_error_rate_simmetry_ $exponent>]() {
                        test_error_rate_simmetry::<[<Precision $exponent>]>();
                    }
                }
            )*
        };
    }

    test_error_rate_simmetry!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
}
