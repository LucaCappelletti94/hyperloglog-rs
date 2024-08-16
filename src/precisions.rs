//! In this document, we define the precisions as a trait and we implement it for structs
//! ranging from Precision4 to Precision16. This is necessary so that the compiler can
//! know the size necessary to store the number of zeros, and allows us the save when using
//! a number of registers equal of inferior to 256 a Byte, compared to what is possible when
//! using a number of registers equal or inferior to 65536, which would make us waste another byte.

use core::fmt::Debug;

use crate::utils::{Five, Float, Number, One, PositiveInteger};

include!(concat!(env!("OUT_DIR"), "/alpha_values.rs"));
include!(concat!(env!("OUT_DIR"), "/number_of_registers.rs"));

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
) -> f64
where
    f64: From<W>,
{
    let index = estimates.partition_point(|a| a <= &estimate).max(1) - 1;

    let mut min = if index > 6 { index - 6 } else { 0 };
    let mut max = core::cmp::min(index + 6, N);

    while max - min != 6 {
        let (min_val, max_val) = (estimates[min], estimates[max - 1]);
        if V::TWO * estimate - min_val > max_val {
            min += 1;
        } else {
            max -= 1;
        }
    }
    biases[min..max].iter().map(|b| f64::from(*b)).sum::<f64>() / 6.0
}

#[cfg(feature = "plusplus")]
fn bias<const N: usize, V: PartialOrd + Number, W: Number>(
    estimates: &'static [V; N],
    biases: &'static [W; N],
    estimate: V,
) -> f64
where
    f64: From<V> + From<W>,
{
    #[cfg(feature = "plusplus_kmeans")]
    return kmeans_bias(estimates, biases, estimate);

    #[cfg(not(feature = "plusplus_kmeans"))]
    return {
        let index = estimates.partition_point(|a| a <= &estimate);

        if index == 0 {
            return f64::from(biases[0]);
        }

        if index == N {
            return f64::from(biases[N - 1]);
        }

        let x0 = f64::from(estimates[index - 1]);
        let x1 = f64::from(estimates[index]);

        let y0 = f64::from(biases[index - 1]);
        let y1 = f64::from(biases[index]);

        y0 + (y1 - y0) * (f64::from(estimate) - x0) / (x1 - x0)
    };
}

/// The precision of the [`HyperLogLog`] counter.
pub trait Precision: Default + Copy + Eq + Debug + Send + Sync {
    /// The data type to use for the number of zeros registers counter.
    /// This should be the smallest possinle data type that allows us to count
    /// all the registers without overflowing. We can tollerate a one-off error
    /// when counting the number of zeros, as it will be corrected when computing
    /// the cardinality as it is known before hand whether this can happen at all.
    type NumberOfRegisters: PositiveInteger;
    /// The exponent of the number of registers, meaning the number of registers
    /// that will be used is 2^EXPONENT. This is the p parameter in the [`HyperLogLog`].
    const EXPONENT: u8;
    /// The number of registers that will be used.
    const NUMBER_OF_REGISTERS: Self::NumberOfRegisters;

    #[must_use]
    /// The theoretical error rate of the precision.
    fn error_rate() -> f64 {
        1.04 / 2f64.powf(f64::from(Self::EXPONENT) / 2.0)
    }

    /// The alpha constant for the precision, used in the estimation of the cardinality.
    const ALPHA: f64;

    #[cfg(feature = "plusplus")]
    /// The number of zero registers over which the counter should switch to the linear counting.
    const LINEAR_COUNT_ZEROS: Self::NumberOfRegisters;

    #[cfg(feature = "beta")]
    /// Computes LogLog-Beta estimate bias correction using Horner's method.
    ///
    /// Paper: <https://arxiv.org/pdf/1612.02284.pdf>
    /// Wikipedia: <https://en.wikipedia.org/wiki/Horner%27s_method>
    fn beta_estimate(harmonic_sum: f64, number_of_zero_registers: Self::NumberOfRegisters) -> f64;

    #[cfg(feature = "plusplus")]
    /// Computes the small correction factor for the estimate.
    fn small_correction(number_of_zero_registers: Self::NumberOfRegisters) -> f64;

    /// Computes the bias correction factor for the estimate.
    fn bias(estimate: f64) -> f64;

    #[cfg(feature = "plusplus")]
    /// Computes the estimate of the cardinality using the `LogLog++` algorithm.
    fn plusplus_estimate(harmonic_sum: f64, number_of_zeros: Self::NumberOfRegisters) -> f64 {
        if number_of_zeros >= Self::LINEAR_COUNT_ZEROS {
            return Self::small_correction(number_of_zeros);
        }

        let estimate =
            Self::ALPHA * f64::integer_exp2(Self::EXPONENT + Self::EXPONENT) / harmonic_sum;

        // Apply the small range correction factor if the raw estimate is below the threshold
        // and there are zero registers in the counter.
        if estimate <= f64::FIVE * f64::integer_exp2(Self::EXPONENT) {
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
            /// The precision of the HyperLogLog counter.
            pub struct [<Precision $exponent>];

            #[cfg(all(feature = "precision_" $exponent, feature = "std"))]
            impl crate::prelude::Named for [<Precision $exponent>] {
                fn name(&self) -> String {
                    format!("P{}", $exponent)
                }
            }

            #[cfg(feature = "precision_" $exponent)]
            impl Precision for [<Precision $exponent>] {
                type NumberOfRegisters = [<NumberOfRegisters $exponent>];
                const EXPONENT: u8 = $exponent;
                const NUMBER_OF_REGISTERS: Self::NumberOfRegisters = [<NumberOfRegisters $exponent>]::ONE << $exponent;
                const ALPHA: f64 = [<ALPHA_ $exponent>];

                #[cfg(feature = "plusplus")]
                const LINEAR_COUNT_ZEROS: Self::NumberOfRegisters = [<LINEAR_COUNT_ZEROS_ $exponent>];

                #[inline]
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                #[cfg(feature = "plusplus")]
                fn bias(estimate: f64) -> f64 {
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
                        estimate as u32
                    );
                }

                /// Computes LogLog-Beta estimate bias correction using Horner's method.
                ///
                /// Paper: https://arxiv.org/pdf/1612.02284.pdf
                /// Wikipedia: https://en.wikipedia.org/wiki/Horner%27s_method
                #[inline]
                #[cfg(feature = "beta")]
                fn beta_estimate(harmonic_sum: f64, number_of_zero_registers: Self::NumberOfRegisters) -> f64 {
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
                        res * number_of_zero_registers_ln + [<BETA_ $exponent>][0] *  f64::from(number_of_zero_registers)
                    };

                    #[cfg(feature = "precomputed_beta")]
                    let beta_horner = [<BETA_HORNER_ $exponent>][number_of_zero_registers as usize];

                    [<ALPHA_ $exponent>]
                        * f64::integer_exp2(Self::EXPONENT)
                        * (f64::integer_exp2(Self::EXPONENT) - f64::from(number_of_zero_registers))
                        / (harmonic_sum + beta_horner)
                        + 0.5
                }

                #[inline(always)]
                #[cfg(feature = "plusplus")]
                fn small_correction(number_of_zero_registers: Self::NumberOfRegisters) -> f64 {
                    f64::integer_exp2(Self::EXPONENT)
                        * (f64::from(Self::EXPONENT) * core::f64::consts::LN_2 - LN_VALUES[number_of_zero_registers as usize])
                    // f64::integer_exp2(Self::EXPONENT)
                    //     * (f64::integer_exp2(Self::EXPONENT) / f64::from(number_of_zero_registers)).ln()
                }
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
        assert_eq!(exponent as u8, P::EXPONENT);
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
