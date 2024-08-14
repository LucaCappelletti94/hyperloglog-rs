//! In this document, we define the precisions as a trait and we implement it for structs
//! ranging from Precision4 to Precision16. This is necessary so that the compiler can
//! know the size necessary to store the number of zeros, and allows us the save when using
//! a number of registers equal of inferior to 256 a Byte, compared to what is possible when
//! using a number of registers equal or inferior to 65536, which would make us waste another byte.

use core::fmt::Debug;

use crate::utils::*;

include!(concat!(env!("OUT_DIR"), "/alpha_values.rs"));
include!(concat!(env!("OUT_DIR"), "/number_of_zeros.rs"));

#[cfg(feature = "plusplus")]
include!(concat!(env!("OUT_DIR"), "/weights.rs"));

#[cfg(feature = "plusplus")]
include!(concat!(env!("OUT_DIR"), "/linear_count_zeros.rs"));

#[cfg(any(
    all(feature = "beta", not(feature = "precomputed_beta")),
    feature = "plusplus"
))]
include!(concat!(env!("OUT_DIR"), "/ln_values.rs"));

#[cfg(all(feature = "beta", not(feature = "precomputed_beta")))]
include!(concat!(env!("OUT_DIR"), "/beta.rs"));

#[cfg(feature = "precomputed_beta")]
include!(concat!(env!("OUT_DIR"), "/beta_horner.rs"));

#[cfg(feature = "plusplus_kmeans")]
fn kmeans_bias<const N: usize, V: PartialOrd + Number, W: Number>(
    estimates: &'static [V; N],
    biases: &'static [W; N],
    estimate: V,
) -> f32 {
    let index = estimates.partition_point(|a| a <= &estimate).max(1) -1;

    let mut min = if index > 6 { index - 6 } else { 0 };
    let mut max = core::cmp::min(index + 6, N);

    while max - min != 6 {
        let (min_val, max_val) = unsafe {
            (
                *estimates.get_unchecked(min),
                *estimates.get_unchecked(max - 1),
            )
        };
        if V::TWO * estimate - min_val > max_val {
            min += 1;
        } else {
            max -= 1;
        }
    }
    biases[min..max].iter().map(|bias| bias.to_f32()).sum::<f32>() / 6.0_f32
}

#[cfg(all(feature = "plusplus", not(feature = "plusplus_kmeans")))]
fn interpolated_bias<const N: usize, V: PartialOrd + Number, W: Number>(
    estimates: &'static [V; N],
    biases: &'static [W; N],
    estimate: V,
) -> f32 {
    let index = estimates.partition_point(|a| a <= &estimate);

    if index == 0 {
        return biases[0].to_f32();
    }

    if index == N {
        return biases[N - 1].to_f32();
    }

    let x0 = unsafe { estimates.get_unchecked(index - 1).to_f32() };
    let x1 = unsafe { estimates.get_unchecked(index).to_f32() };

    let y0 = unsafe { biases.get_unchecked(index - 1).to_f32() };
    let y1 = unsafe { biases.get_unchecked(index).to_f32() };

    y0 + (y1 - y0) * (estimate.to_f32() - x0) / (x1 - x0)
}

#[cfg(feature = "plusplus")]
fn bias<const N: usize, V: PartialOrd + Number, W: Number, F: FloatNumber>(
    estimates: &'static [V; N],
    biases: &'static [W; N],
    estimate: V,
) -> F {
    #[cfg(feature = "plusplus_kmeans")]
    return F::from_f32(kmeans_bias(estimates, biases, estimate));

    #[cfg(not(feature = "plusplus_kmeans"))]
    return F::from_f32(interpolated_bias(estimates, biases, estimate));
}

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

    fn error_rate() -> f64 {
        let exponent = (Self::EXPONENT as f64) / 2.0;
        1.04 / 2f64.powf(exponent)
    }
}

pub trait PrecisionConstants<F: FloatNumber>: Precision {
    const NUMBER_OF_REGISTERS_FLOAT: F;
    const ALPHA: F;

    #[cfg(feature = "plusplus")]
    const LINEAR_COUNT_ZEROS: Self::NumberOfZeros;

    /// Computes LogLog-Beta estimate bias correction using Horner's method.
    ///
    /// Paper: https://arxiv.org/pdf/1612.02284.pdf
    /// Wikipedia: https://en.wikipedia.org/wiki/Horner%27s_method
    #[cfg(feature = "beta")]
    fn beta_estimate(harmonic_sum: F, number_of_zero_registers: Self::NumberOfZeros) -> F;

    #[cfg(feature = "plusplus")]
    fn small_correction(number_of_zero_registers: Self::NumberOfZeros) -> F;

    fn bias(estimate: F) -> F;

    #[inline(always)]
    #[cfg(feature = "plusplus")]
    fn plusplus_estimate(harmonic_sum: F, number_of_zeros: Self::NumberOfZeros) -> F {
        if number_of_zeros >= Self::LINEAR_COUNT_ZEROS {
            return Self::small_correction(number_of_zeros);
        }

        let estimate =
            Self::ALPHA * Self::NUMBER_OF_REGISTERS_FLOAT * Self::NUMBER_OF_REGISTERS_FLOAT
                / harmonic_sum;

        // Apply the small range correction factor if the raw estimate is below the threshold
        // and there are zero registers in the counter.
        if estimate <= F::FIVE * Self::NUMBER_OF_REGISTERS_FLOAT {
            estimate - Self::bias(estimate)
        } else {
            estimate
        }
    }
}

/// Macro to implement the Precision trait for a given precision.
macro_rules! impl_precision {
    ($exponent:expr) => {
        paste::paste! {
            #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            #[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
            pub struct [<Precision $exponent>];

            #[cfg(feature = "precision_" $exponent)]
            impl PrecisionConstants<f32> for [<Precision $exponent>] {
                const NUMBER_OF_REGISTERS_FLOAT: f32 = Self::NUMBER_OF_REGISTERS as f32;
                const ALPHA: f32 = [<ALPHA_ $exponent>] as f32;

                #[cfg(feature = "plusplus")]
                const LINEAR_COUNT_ZEROS: Self::NumberOfZeros = [<LINEAR_COUNT_ZEROS_ $exponent>];

                #[inline]
                #[cfg(feature = "plusplus")]
                fn bias(estimate: f32) -> f32 {
                    #[cfg(not(feature = "integer_plusplus"))]
                    return bias(
                        &[<ESTIMATES_ $exponent>],
                        &[<BIAS_ $exponent>],
                        estimate
                    );
                    #[cfg(feature = "integer_plusplus")]
                    return bias(
                        &[<ESTIMATES_ $exponent>],
                        &[<BIAS_ $exponent>],
                        estimate.to_u32()
                    );
                }

                /// Computes LogLog-Beta estimate bias correction using Horner's method.
                ///
                /// Paper: https://arxiv.org/pdf/1612.02284.pdf
                /// Wikipedia: https://en.wikipedia.org/wiki/Horner%27s_method
                #[inline]
                #[cfg(feature = "beta")]
                fn beta_estimate(harmonic_sum: f32, number_of_zero_registers: Self::NumberOfZeros) -> f32 {
                    <Self as PrecisionConstants<f64>>::beta_estimate(harmonic_sum as f64, number_of_zero_registers)
                        as f32
                }

                #[inline(always)]
                #[cfg(feature = "plusplus")]
                fn small_correction(number_of_zero_registers: Self::NumberOfZeros) -> f32 {
                    <Self as PrecisionConstants<f64>>::small_correction(number_of_zero_registers) as f32
                }
            }

            #[cfg(all(feature = "precision_" $exponent))]
            impl PrecisionConstants<f64> for [<Precision $exponent>] {
                const NUMBER_OF_REGISTERS_FLOAT: f64 = (1 << $exponent) as f64;
                const ALPHA: f64 = [<ALPHA_ $exponent>];

                #[cfg(feature = "plusplus")]
                const LINEAR_COUNT_ZEROS: Self::NumberOfZeros = [<LINEAR_COUNT_ZEROS_ $exponent>];

                #[inline]
                #[cfg(feature = "plusplus")]
                fn bias(estimate: f64) -> f64 {
                    <Self as PrecisionConstants<f32>>::bias(estimate as f32) as f64
                }

                /// Computes LogLog-Beta estimate bias correction using Horner's method.
                ///
                /// Paper: https://arxiv.org/pdf/1612.02284.pdf
                /// Wikipedia: https://en.wikipedia.org/wiki/Horner%27s_method
                #[inline]
                #[cfg(feature = "beta")]
                fn beta_estimate(harmonic_sum: f64, number_of_zero_registers: Self::NumberOfZeros) -> f64 {
                    if number_of_zero_registers >= [<LINEAR_COUNT_ZEROS_ $exponent>] {
                        return Self::small_correction(number_of_zero_registers);
                    }
                    #[cfg(not(feature = "precomputed_beta"))]
                    let beta_horner = {
                        let number_of_zero_registers_ln = LN_VALUES[1 + number_of_zero_registers as usize];
                        let mut res = 0.0;
                        for i in (1..8).rev() {
                            res = res * number_of_zero_registers_ln + [<BETA_ $exponent>][i];
                        }
                        res * number_of_zero_registers_ln + [<BETA_ $exponent>][0] * number_of_zero_registers as f64
                    };

                    #[cfg(feature = "precomputed_beta")]
                    let beta_horner = [<BETA_HORNER_ $exponent>][number_of_zero_registers as usize];

                    [<ALPHA_ $exponent>]
                        * Self::NUMBER_OF_REGISTERS as f64
                        * (Self::NUMBER_OF_REGISTERS - number_of_zero_registers as usize) as f64
                        / (harmonic_sum + beta_horner)
                        + 0.5
                }

                #[inline(always)]
                #[cfg(feature = "plusplus")]
                fn small_correction(number_of_zero_registers: Self::NumberOfZeros) -> f64 {
                    Self::NUMBER_OF_REGISTERS as f64
                        * (Self::EXPONENT as f64 * core::f64::consts::LN_2 - LN_VALUES[number_of_zero_registers as usize])
                }
            }

            #[cfg(feature = "precision_" $exponent)]
            impl Precision for [<Precision $exponent>] {
                type NumberOfZeros = [<NumberOfZeros $exponent>];
                const EXPONENT: usize = $exponent;
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

impl_precisions!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);

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
                    #[cfg(feature = "precision_" $exponent)]
                    fn [<test_error_rate_simmetry_ $exponent>]() {
                        test_error_rate_simmetry::<[<Precision $exponent>]>();
                    }

                    #[test]
                    #[cfg(feature = "precision_" $exponent)]
                    fn [<test_harmonic_sum_resolution_at_precision_ $exponent >]() {
                        // The smallest possible harmonic sum is determined by
                        // the number of registers, which is 2^EXPONENT, times
                        // the reciprocal of two to the the largest register value:
                        // 2^EXPONENT * 2^(-BITS) = 2^(EXPONENT - BITS).
                        // In this test, we check that the harmonic sum is able to
                        // store accurately the value 2^(EXPONENT - BITS).
                        for number_of_bits in [1, 2, 3, 4, 5, 6 , 7 , 8] {
                            let harmonic_sum = 2f64.powi([<Precision $exponent>]::EXPONENT as i32 - number_of_bits);
                            let harmonic_sum_plus_one = 2f64.powi([<Precision $exponent>]::EXPONENT as i32 - number_of_bits) + 1.0;
                            let harmonic_sum_minus_one = harmonic_sum_plus_one - 1.0;
                            assert_eq!(harmonic_sum, harmonic_sum_minus_one);

                            let harmonic_sum = 2f32.powi([<Precision $exponent>]::EXPONENT as i32 - number_of_bits);
                            let harmonic_sum_plus_one = 2f32.powi([<Precision $exponent>]::EXPONENT as i32 - number_of_bits) + 1.0;
                            let harmonic_sum_minus_one = harmonic_sum_plus_one - 1.0;
                            assert_eq!(harmonic_sum, harmonic_sum_minus_one);
                        }
                    }
                }
            )*
        };
    }

    test_error_rate_simmetry!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);

    /// Macro rule to generate test to verify that the estimates are sorted in ascending order
    /// for a given precision.
    macro_rules! test_estimates_sorted {
        ($($exponent:expr),*) => {
            $(
                paste::paste! {
                    #[test]
                    #[cfg(all(feature = "precision_" $exponent, feature="plusplus"))]
                    fn [<test_estimates_sorted_ $exponent>]() {
                        let mut last = [<ESTIMATES_ $exponent>][0];
                        for estimate in [<ESTIMATES_ $exponent>].iter() {
                            assert!(*estimate >= last, "Estimate: {}, Last: {}", *estimate, last);
                            last = *estimate;
                        }
                    }
                }
            )*
        };
    }

    test_estimates_sorted!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);
}
