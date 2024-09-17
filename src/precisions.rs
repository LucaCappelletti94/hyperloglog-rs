//! In this document, we define the precisions as a trait and we implement it for structs
//! ranging from Precision4 to Precision16. This is necessary so that the compiler can
//! know the size necessary to store the number of zeros, and allows us the save when using
//! a number of registers equal of inferior to 256 a Byte, compared to what is possible when
//! using a number of registers equal or inferior to 65536, which would make us waste another byte.

use core::f64;
use core::fmt::Debug;

use crate::utils::FloatOps;

/// Macro defining the alpha constant for a given precision.
macro_rules! alpha {
    (4 ) => {
        0.673
    };
    (5 ) => {
        0.697
    };
    (6 ) => {
        0.709
    };
    ($exponent:expr) => {
        0.7213 / (1.0 + 1.079 / (1 << $exponent) as f64)
    };
}

/// The precision of the [`HyperLogLog`] counter.
pub trait Precision: Default + Copy + Eq + Debug + Send + Sync {
    /// The exponent of the number of registers, meaning the number of registers
    /// that will be used is 2^EXPONENT. This is the p parameter in the [`HyperLogLog`].
    const EXPONENT: u8;

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

            impl Precision for [<Precision $exponent>] {
                const EXPONENT: u8 = $exponent;
                const ALPHA: f64 = alpha!($exponent);
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
                    fn [<test_error_rate_simmetry_ $exponent>]() {
                        test_error_rate_simmetry::<[<Precision $exponent>]>();
                    }
                }
            )*
        };
    }

    test_estimates_sorted!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);
}
