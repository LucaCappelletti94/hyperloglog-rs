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
mod number;
mod register_word;
mod word_like;
mod words;

pub use constants::*;
pub use number::{FloatNumber, Number, PositiveIntegerNumber};
pub(crate) use register_word::RegisterWord;
pub(crate) use word_like::WordLike;
pub(crate) use words::Words;

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
        precision <= 16,
        "The precision must be less than or equal to 16."
    );

    if bits < 6 {
        1 << bits
    } else {
        64 - precision
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;
    use crate::sip::hash_and_index;

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

    fn test_maximal_multeplicity<P: Precision, B: Bits>() {
        assert_eq!(
            0_u64.leading_zeros() as usize,
            64,
            "The number of zeros in the leading position of a 64-bit integer must be 64, but it is {}.",
            0_u64.leading_zeros() as usize,
        );
        assert_eq!(
            u64::MAX.leading_zeros() as usize,
            0,
            "The number of zeros in the leading position of a 64-bit MAX integer must be 0, but it is {}.",
            u64::MAX.leading_zeros() as usize,
        );

        let maximal = maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS) - 1;
        let mut maximal_encountered = 0;

        for i in 0..1_000_000_u64 {
            let hash = hash_and_index::<u64, P, B>(&i).0;
            let number_of_zeros = hash.leading_zeros() as usize + 1;
            assert!(
                number_of_zeros <= maximal,
                "The number of zeros ({}) must be less than or equal to the maximal multiplicity ({}).",
                number_of_zeros,
                maximal,
            );
            maximal_encountered = maximal_encountered.max(number_of_zeros);
        }
        assert_eq!(
            maximal_encountered,
            maximal,
            "At least one hash should have the maximal multiplicity ({}) for precision {} and bits {}, but at most we found {}.",
            maximal,
            P::EXPONENT,
            B::NUMBER_OF_BITS,
            maximal_encountered,
        );
    }

    macro_rules! test_maximal_multeplicity_by_precision_and_bits {
        ($precision: expr, $($bits: expr),*) => {
            $(
                paste::item! {
                    #[test]
                    fn [<test_maximal_multeplicity_ $precision _ $bits>]() {
                        test_maximal_multeplicity::<[<Precision $precision>], [<Bits $bits>]>();
                    }
                }
            )*
        };
    }

    macro_rules! test_maximal_multeplicity_by_precisions {
        ($($precision: expr),*) => {
            $(
                test_maximal_multeplicity_by_precision_and_bits!($precision, 1, 2, 3, 4);
            )*
        };
    }

    test_maximal_multeplicity_by_precisions!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
}
