//! # Utils
//!
//! This module provides utility functions used by the [`HyperLogLog`] algorithm implementation.
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
mod matrix;
mod number;
mod random;
mod variable_word;

pub use constants::*;
pub use hasher_type::HasherType;
pub use matrix::Matrix;
pub(crate) use number::{FloatOps, Number, PositiveInteger};
pub use random::*;
pub(crate) use variable_word::VariableWord;

#[cfg(all(
    not(feature = "std_ln"),
    any(
        all(feature = "beta", not(feature = "precomputed_beta")),
        feature = "plusplus"
    )
))]
include!(concat!(env!("OUT_DIR"), "/ln_values.rs"));

#[cfg(feature = "std")]
/// Trait for an object with a name.
pub trait Named {
    /// Returns the name of the object.
    fn name(&self) -> String;
}

#[cfg(feature = "std")]
impl Named for u8 {
    fn name(&self) -> String {
        "u8".to_owned()
    }
}

#[cfg(feature = "std")]
impl Named for u16 {
    fn name(&self) -> String {
        "u16".to_owned()
    }
}

#[cfg(feature = "std")]
impl Named for u32 {
    fn name(&self) -> String {
        "u32".to_owned()
    }
}

#[cfg(feature = "std")]
impl Named for u64 {
    fn name(&self) -> String {
        "u64".to_owned()
    }
}

#[inline]
#[must_use]
/// Calculates the integer ceil of the division of `numerator` by `denominator`.
///
/// # Arguments
/// * `numerator` - The numerator of the division.
/// * `denominator` - The denominator of the division.
pub const fn ceil(numerator: usize, denominator: usize) -> usize {
    (numerator + denominator - 1) / denominator
}

#[inline]
/// Applies a correction to the provided union cardinality estimate.
pub(crate) fn correct_union_estimate(
    left_cardinality: f64,
    right_cardinality: f64,
    union_cardinality: f64,
) -> f64 {
    union_cardinality
        .min(right_cardinality + left_cardinality)
        .max(right_cardinality.max(left_cardinality))
}

#[inline]
/// Returns the linear counting correction.
pub(crate) fn linear_counting_correction(exponent: u8, number_of_zero_registers: u32) -> f64 {
    #[cfg(not(feature = "std_ln"))]
    return f64::integer_exp2(exponent)
        * (LN_VALUES[1 << exponent] - LN_VALUES[number_of_zero_registers.to_usize()]);
    #[cfg(feature = "std_ln")]
    return f64::integer_exp2(exponent)
        * f64::ln(f64::integer_exp2(exponent) / f64::from(number_of_zero_registers));
}

#[cfg(test)]
mod test {
    use super::*;

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
