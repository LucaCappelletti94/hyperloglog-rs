//! In this document, we define the precisions as a trait and we implement it for structs
//! ranging from Precision4 to Precision16. This is necessary so that the compiler can
//! know the size necessary to store the number of zeros, and allows us the save when using
//! a number of registers equal of inferior to 256 a Byte, compared to what is possible when
//! using a number of registers equal or inferior to 65536, which would make us waste another byte.

use core::f64;
use core::fmt::Debug;

#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

use crate::utils::{FloatOps, Number, One, PositiveInteger, VariableWord};

#[cfg(feature = "plusplus")]
use crate::utils::Two;

#[cfg(all(feature = "beta", not(feature = "precomputed_beta")))]
use crate::utils::Zero;

include!(concat!(env!("OUT_DIR"), "/alpha_values.rs"));
include!(concat!(env!("OUT_DIR"), "/number_of_registers.rs"));

#[cfg(feature = "plusplus")]
include!(concat!(env!("OUT_DIR"), "/weights.rs"));

#[cfg(feature = "zero_count_correction")]
include!(concat!(env!("OUT_DIR"), "/linear_count_zeros.rs"));

#[cfg(all(
    not(feature = "std_ln"),
    any(
        all(feature = "beta", not(feature = "precomputed_beta")),
        feature = "plusplus"
    )
))]
include!(concat!(env!("OUT_DIR"), "/ln_values.rs"));

#[cfg(all(feature = "beta", not(feature = "precomputed_beta")))]
include!(concat!(env!("OUT_DIR"), "/beta.rs"));

#[cfg(feature = "precomputed_beta")]
include!(concat!(env!("OUT_DIR"), "/beta_horner.rs"));

#[cfg(feature = "plusplus_kmeans")]
fn kmeans_bias<V: PartialOrd + Number + Two, W: Number>(
    estimates: &'static [V],
    biases: &'static [W],
    estimate: V,
) -> f64
where
    f64: From<W>,
{
    let index = estimates
        .partition_point(|estimate_centroid| estimate_centroid <= &estimate)
        .max(1)
        - 1;

    let mut min = if index > 6 { index - 6 } else { 0 };
    let mut max = core::cmp::min(index + 6, estimates.len());

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
/// Computes the bias correction factor for the estimate using either
/// the k-means algorithm or the simpler linear interpolation.
fn bias<V: PartialOrd + Two + Number, W: Number>(
    estimates: &'static [V],
    biases: &'static [W],
    estimate: V,
) -> f64
where
    f64: From<V> + From<W>,
{
    #[cfg(feature = "plusplus_kmeans")]
    return kmeans_bias(estimates, biases, estimate);

    #[cfg(not(feature = "plusplus_kmeans"))]
    return {
        let index = estimates.partition_point(|estimate_centroid| estimate_centroid <= &estimate);

        if index == 0 {
            return f64::from(biases[0]);
        }

        if index == estimates.len() {
            return f64::from(biases[estimates.len() - 1]);
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
    #[cfg(feature = "mem_dbg")]
    type NumberOfRegisters: PositiveInteger
        + VariableWord<Word = <Self as Precision>::NumberOfRegisters>
        + MemSize
        + Into<u32>
        + TryFrom<u64>
        + MemDbg;
    #[cfg(not(feature = "mem_dbg"))]
    /// Se documentation above.
    type NumberOfRegisters: PositiveInteger
        + TryFrom<u64>
        + Into<u32>
        + VariableWord<Word = <Self as Precision>::NumberOfRegisters>;
    /// The exponent of the number of registers, meaning the number of registers
    /// that will be used is 2^EXPONENT. This is the p parameter in the [`HyperLogLog`].
    const EXPONENT: u8;
    /// The number of registers that will be used.
    const NUMBER_OF_REGISTERS: Self::NumberOfRegisters;

    #[must_use]
    #[inline]
    /// The theoretical error rate of the precision.
    fn error_rate() -> f64 {
        if Self::EXPONENT % 2 == 0 {
            // When p is even, we can compute the integer p_half = p/2.
            let half_exponent = Self::EXPONENT / 2;
            1.04 * f64::integer_exp2_minus(half_exponent)
        } else {
            // 2^-(p/2) = 2^-(1/2)*2^-((p-1)/2).
            // When p is odd, we can compute the integer p_half = (p-1)/2.
            let p_minus_one_half = (Self::EXPONENT - 1) / 2;
            // Since 2^-(1/2) = sqrt(2)^-1, we can compute the error rate as:
            // 1.04 * 2^-((p-1)/2) / sqrt(2)
            1.04 * f64::integer_exp2_minus(p_minus_one_half) / f64::consts::SQRT_2
        }
    }

    /// The alpha constant for the precision, used in the estimation of the cardinality.
    const ALPHA: f64;
    #[cfg(all(feature = "beta", not(feature = "precomputed_beta")))]
    /// Beta constants for the LogLog-Beta bias correction.
    const BETA: [f64; 8];

    #[cfg(feature = "precomputed_beta")]
    /// Returns the precomputed beta value for the given number of zero registers.
    fn const_beta_horner(number_of_zero_registers: u32) -> f64;

    #[cfg(feature = "zero_count_correction")]
    /// The number of zero registers over which the counter should switch to the linear counting.
    const LINEAR_COUNT_ZEROS: u32;
    #[cfg(all(feature = "plusplus", not(feature = "integer_plusplus")))]
    /// The estimate centroids for the [`PlusPlus`] bias correction.
    const ESTIMATES: &'static [f64];
    #[cfg(all(feature = "plusplus", not(feature = "integer_plusplus")))]
    /// The bias values for the [`PlusPlus`] bias correction.
    const BIASES: &'static [f64];
    #[cfg(all(feature = "plusplus", feature = "integer_plusplus"))]
    /// The estimate centroids for the [`PlusPlus`] bias correction.
    const ESTIMATES: &'static [u32];
    #[cfg(all(feature = "plusplus", feature = "integer_plusplus"))]
    /// The bias values for the [`PlusPlus`] bias correction.
    const BIASES: &'static [i32];

    #[cfg(feature = "beta")]
    #[inline]
    #[must_use]
    /// Computes LogLog-Beta estimate bias correction using Horner's method.
    ///
    /// Paper: <https://arxiv.org/pdf/1612.02284.pdf>
    /// Wikipedia: <https://en.wikipedia.org/wiki/Horner%27s_method>
    fn beta_estimate(harmonic_sum: f64, number_of_zero_registers: u32) -> f64 {
        #[cfg(feature = "zero_count_correction")]
        if number_of_zero_registers >= Self::LINEAR_COUNT_ZEROS {
            return Self::small_correction(number_of_zero_registers);
        }
        #[cfg(not(feature = "precomputed_beta"))]
        let beta_horner = {
            #[cfg(not(feature = "std_ln"))]
            let number_of_zero_registers_ln = LN_VALUES[1 + number_of_zero_registers.to_usize()];
            #[cfg(feature = "std_ln")]
            let number_of_zero_registers_ln = f64::ln_1p(f64::from(number_of_zero_registers));
            let mut res = f64::ZERO;
            for i in (1..8).rev() {
                res = res * number_of_zero_registers_ln + Self::BETA[i];
            }
            res * number_of_zero_registers_ln + Self::BETA[0] * f64::from(number_of_zero_registers)
        };

        #[cfg(feature = "precomputed_beta")]
        let beta_horner = Self::const_beta_horner(number_of_zero_registers);

        Self::ALPHA
            * f64::integer_exp2(Self::EXPONENT)
            * (f64::integer_exp2(Self::EXPONENT) - f64::from(number_of_zero_registers))
            / (harmonic_sum + beta_horner)
            + 0.5
    }

    /// Computes the small correction factor for the estimate.
    #[inline]
    #[must_use]
    #[cfg(feature = "zero_count_correction")]
    fn small_correction(number_of_zero_registers: u32) -> f64 {
        #[cfg(not(feature = "std_ln"))]
        return f64::integer_exp2(Self::EXPONENT)
            * (f64::from(Self::EXPONENT) * core::f64::consts::LN_2
                - LN_VALUES[number_of_zero_registers.to_usize()]);
        #[cfg(feature = "std_ln")]
        return f64::integer_exp2(Self::EXPONENT)
            * f64::ln_1p(
                (f64::from((1 << Self::EXPONENT) - number_of_zero_registers))
                    / f64::from(number_of_zero_registers),
            );
    }

    #[must_use]
    #[inline]
    #[cfg(feature = "plusplus")]
    #[cfg_attr(
        feature = "integer_plusplus",
        expect(clippy::cast_sign_loss, reason = "Cardinality is always positive.")
    )]
    #[cfg_attr(
        feature = "integer_plusplus",
        expect(
            clippy::cast_possible_truncation,
            reason = "Bias is only applied to values smaller than 2**21."
        )
    )]
    /// Computes the bias correction factor for the estimate using the [`PlusPlus`] algorithm.
    fn bias(estimate: f64) -> f64 {
        #[cfg(not(feature = "integer_plusplus"))]
        return bias(Self::ESTIMATES, Self::BIASES, estimate);
        #[cfg(feature = "integer_plusplus")]
        return bias(Self::ESTIMATES, Self::BIASES, estimate as u32);
    }

    #[cfg(feature = "plusplus")]
    #[inline]
    #[must_use]
    /// Computes the estimate of the cardinality using the [`PlusPlus`] algorithm.
    fn plusplus_estimate(harmonic_sum: f64, number_of_zeros: u32) -> f64 {
        #[cfg(feature = "zero_count_correction")]
        if number_of_zeros >= Self::LINEAR_COUNT_ZEROS {
            return Self::small_correction(number_of_zeros);
        }

        let estimate =
            Self::ALPHA * f64::integer_exp2(Self::EXPONENT + Self::EXPONENT) / harmonic_sum;

        // Apply the small range correction factor if the raw estimate is below the threshold
        // and there are zero registers in the counter.
        if estimate <= 5.0_f64 * f64::integer_exp2(Self::EXPONENT) {
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
            #[non_exhaustive]
            #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            /// The precision of the HyperLogLog counter.
            pub struct [<Precision $exponent>];

            #[cfg(all(feature = "precision_" $exponent, feature = "std"))]
            impl crate::prelude::Named for [<Precision $exponent>] {
                #[inline]
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
                #[cfg(all(feature = "beta", not(feature = "precomputed_beta")))]
                const BETA: [f64; 8] = [<BETA_ $exponent>];
                #[cfg(feature = "precomputed_beta")]
                fn const_beta_horner(number_of_zero_registers: u32) -> f64 {
                    [<BETA_HORNER_ $exponent>][number_of_zero_registers.to_usize()]
                }

                #[cfg(feature = "zero_count_correction")]
                const LINEAR_COUNT_ZEROS: u32 = [<LINEAR_COUNT_ZEROS_ $exponent>];

                #[cfg(all(feature = "plusplus", not(feature = "integer_plusplus")))]
                const ESTIMATES: &'static [f64] = &[<ESTIMATES_ $exponent>];
                #[cfg(all(feature = "plusplus", not(feature = "integer_plusplus")))]
                const BIASES: &'static [f64] = &[<BIAS_ $exponent>];
                #[cfg(all(feature = "plusplus", feature = "integer_plusplus"))]
                const ESTIMATES: &'static [u32] = &[<ESTIMATES_ $exponent>];
                #[cfg(all(feature = "plusplus", feature = "integer_plusplus"))]
                const BIASES: &'static [i32] = &[<BIAS_ $exponent>];
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

    /// Macro rule to generate test to verify that the estimates are sorted in ascending order
    /// for a given precision.
    macro_rules! test_estimates_sorted {
        ($($exponent:expr),*) => {
            $(
                paste::paste! {
                    #[test]
                    #[cfg(feature = "precision_" $exponent)]
                    fn [<test_error_rate_simmetry_ $exponent>]() {
                        test_error_rate_simmetry::<[<Precision $exponent>]>();
                    }

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
