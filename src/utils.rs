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

mod composite_hash;
mod constants;
mod hasher_type;
mod matrix;
mod number;
mod random;
mod variable_word;
mod variable_words;

pub use composite_hash::CompositeHash;
pub use constants::*;
pub use hasher_type::HasherType;
pub use matrix::Matrix;
pub(crate) use number::{FloatOps, Number, PositiveInteger};
pub use random::*;
pub use variable_word::{VariableWord, u40, u48, u56};
pub use variable_words::VariableWords;

#[cfg(feature = "std")]
/// Trait for an object with a name.
pub trait Named {
    /// Returns the name of the object.
    fn name(&self) -> String;
}

#[cfg(feature = "std")]
impl Named for u8 {
    fn name(&self) -> String {
        format!("u8")
    }
}

#[cfg(feature = "std")]
impl Named for u16 {
    fn name(&self) -> String {
        format!("u16")
    }
}

#[cfg(feature = "std")]
impl Named for u32 {
    fn name(&self) -> String {
        format!("u32")
    }
}

#[cfg(feature = "std")]
impl Named for u64 {
    fn name(&self) -> String {
        format!("u64")
    }
}

#[inline]
/// Calculates the integer ceil of the division of `numerator` by `denominator`.
///
/// # Arguments
/// * `numerator` - The numerator of the division.
/// * `denominator` - The denominator of the division.
pub(crate) const fn ceil(numerator: usize, denominator: usize) -> usize {
    (numerator + denominator - 1) / denominator
}

#[inline]
/// Returns the maximal multiplicity of the registers given the precision and the number of bits.
pub(crate) const fn maximal_multeplicity(precision: u8, bits: u8) -> u8 {
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
