//! # Utils
//!
//! This module provides utility functions used by the [`HyperLogLog`] algorithm implementation.
//!
//! The functions provided are:
//!
//! - `word_from_registers<const NUMBER_OF_BITS_PER_REGISTER: usize>(registers: &[u32]) -> u32`: Converts an array
//!   of HLL registers into a single 32-bit word.

mod constants;
mod hasher_type;
mod matrix;
mod number;
mod random;
mod variable_word;
mod intersection_from_sorted_iterators;

pub use constants::*;
pub use hasher_type::HasherType;
pub use matrix::Matrix;
pub(crate) use number::{FloatOps, Number, PositiveInteger};
pub use random::*;
pub use variable_word::VariableWord;
pub(crate) use intersection_from_sorted_iterators::intersection_from_sorted_iterators;

#[inline]
#[must_use]
/// Calculates the integer floor of the division of `numerator` by `denominator`.
/// 
/// # Arguments
/// * `numerator` - The numerator of the division.
/// * `denominator` - The denominator of the division.
pub const fn floor(numerator: usize, denominator: usize) -> usize {
    numerator / denominator
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
