//! # Utils
//!
//! This module provides utility functions used by the HyperLogLog algorithm implementation.
//!
//! The functions provided are:
//!
//! - `ceil(numerator: usize, denominator: usize) -> usize`: Calculates the integer ceil of the division
//!   of `numerator` by `denominator`.
//!
//! - `word_from_registers<const NUMBER_OF_BITS_PER_REGISTER: usize>(registers: &[u32]) -> u32`: Converts an array
//!   of HLL registers into a single 32-bit word.

mod constants;
mod hasher_type;
mod number;
mod random;
mod register_word;
mod word_like;
mod words;

pub use constants::*;
pub use hasher_type::HasherType;
pub use number::{FloatNumber, Number, PositiveIntegerNumber};
pub use random::*;
pub(crate) use register_word::RegisterWord;
pub use word_like::WordLike;
pub use words::Words;

use crate::{bits::Bits, prelude::Precision};

#[inline]
/// Calculates the integer ceil of the division of `numerator` by `denominator`.
pub(crate) const fn ceil(numerator: usize, denominator: usize) -> usize {
    (numerator + denominator - 1) / denominator
}

#[inline]
pub(crate) const fn maximal_multeplicity(precision: usize, bits: usize) -> usize {
    debug_assert!(
        precision >= 4,
        "The precision must be greater than or equal to 4."
    );
    debug_assert!(
        precision <= 18,
        "The precision must be less than or equal to 16."
    );

    if bits < 6 {
        1 << bits
    } else {
        64 - precision
    }
}

#[inline]
pub(crate) fn miminal_harmonic_sum<F: FloatNumber, P: Precision, B: Bits>() -> F {
    F::inverse_register(
        maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS) as i32 - P::EXPONENT as i32 - 1,
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_miminal_harmonic_sum() {
        assert_eq!(miminal_harmonic_sum::<f32, Precision4, Bits1>(), 8.0);
        assert_eq!(miminal_harmonic_sum::<f32, Precision10, Bits4>(), 0.03125);
        assert_eq!(miminal_harmonic_sum::<f64, Precision4, Bits1>(), 8.0);
        assert_eq!(miminal_harmonic_sum::<f64, Precision10, Bits4>(), 0.03125);
    }

    macro_rules! test_minimal_harmonic_sum_by_precision_and_bits {
        ($precision: expr, $($bits: expr),*) => {
            $(
                paste::item! {
                    #[cfg(feature = "precision_" $precision)]
                    #[test]
                    fn [<test_miminal_harmonic_sum_ $precision _ $bits _against_baseline>]() {
                        let maximal_register_value = maximal_multeplicity([<Precision $precision>]::EXPONENT, [<Bits $bits>]::NUMBER_OF_BITS) - 1;
                        let expected = [<Precision $precision>]::NUMBER_OF_REGISTERS as f64 * (-(maximal_register_value as f64)).exp2();
                        let actual = miminal_harmonic_sum::<f64, [<Precision $precision>], [<Bits $bits>]>();
                        assert!(
                            (expected - actual).abs() < f64::EPSILON,
                            "The minimal harmonic sum ({}) is different from the expected value ({}) for precision {} and bits {}.",
                            actual,
                            expected,
                            [<Precision $precision>]::EXPONENT,
                            [<Bits $bits>]::NUMBER_OF_BITS,
                        );
                    }
                }
            )*
        };
    }

    macro_rules! test_minimal_harmonic_sum_by_precisions {
        ($($precision: expr),*) => {
            $(
                test_minimal_harmonic_sum_by_precision_and_bits!($precision, 1, 2, 3, 4);
            )*
        };
    }

    test_minimal_harmonic_sum_by_precisions!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

    #[test]
    fn test_ceil() {
        assert_eq!(ceil(5, 2), 3);
        assert_eq!(ceil(4, 2), 2);
        assert_eq!(ceil(3, 2), 2);
        assert_eq!(ceil(2, 2), 1);
        assert_eq!(ceil(1, 2), 1);

        assert_eq!(ceil(5, 3), 2);
        assert_eq!(ceil(4, 3), 2);
        assert_eq!(ceil(3, 3), 1);
        assert_eq!(ceil(2, 3), 1);
        assert_eq!(ceil(1, 3), 1);
        assert_eq!(ceil(0, 3), 0);
    }
}
