//! # Utils
//!
//! This module provides utility functions used by the HyperLogLog algorithm implementation.
//!
//! The functions provided are:
//!
//! - `ceil(numerator: usize, denominator: usize) -> usize`: Calculates the integer ceil of the division
//! of `numerator` by `denominator`.
//!
//! - `word_from_registers<const NUMBER_OF_BITS_PER_REGISTER: usize>(registers: &[u32]) -> u32`: Converts an array
//! of HLL registers into a single 32-bit word.
//!
//!

include!(concat!(env!("OUT_DIR"), "/alpha_values.rs"));

#[inline]
/// Calculates the integer ceil of the division of `numerator` by `denominator`.
pub(crate) const fn ceil(numerator: usize, denominator: usize) -> usize {
    (numerator + denominator - 1) / denominator
}

/// Computes the alpha constant for the given number of registers.
///
/// The alpha constant is used to scale the raw HyperLogLog estimate into an
/// estimate of the true cardinality of the set.
///
/// # Arguments
/// * `NUMBER_OF_REGISTERS`: The number of registers in the HyperLogLog
/// data structure.
///
/// # Returns
/// The alpha constant for the given number of registers.
///
/// # Examples
///
/// ```
///
/// ```
#[inline(always)]
pub(crate) const fn get_alpha(precision: usize) -> f32 {
    ALPHA_VALUES[precision - 4]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_alpha() {
        let alpha_16 = get_alpha(4);
        let alpha_32 = get_alpha(5);
        let alpha_64 = get_alpha(6);

        assert_eq!(alpha_16, 0.673);
        assert_eq!(alpha_32, 0.697);
        assert_eq!(alpha_64, 0.709);

        let alpha_4096 = get_alpha(12);

        assert_eq!(alpha_4096, 0.7213 / (1.0 + 1.079 / 4096.0));
    }
}

#[inline]
/// Returns an empirically determined threshold to decide on
/// the use of linear counting.
///
/// # Arguments
/// * `precision`: The precision of the HyperLogLog algorithm.
///
/// # References
/// This data is made available by the authors of the paper
/// in [this Google Docs document](https://docs.google.com/document/d/1gyjfMHy43U9OWBXxfaeG-3MjGzejW1dlpyMwEYAAWEI/view?fullscreen).
///
/// # Examples
///
/// ```rust
/// # use hyperloglog_rs::utils::linear_counting_threshold;
///
/// assert_eq!(linear_counting_threshold(4), 10.0);
/// assert_eq!(linear_counting_threshold(5), 20.0);
/// assert_eq!(linear_counting_threshold(6), 40.0);
/// assert_eq!(linear_counting_threshold(7), 80.0);
/// ```
pub const fn linear_counting_threshold(precision: usize) -> f32 {
    match precision {
        4 => 10.0,
        5 => 20.0,
        6 => 40.0,
        7 => 80.0,
        8 => 220.0,
        9 => 400.0,
        10 => 900.0,
        11 => 1800.0,
        12 => 3100.0,
        13 => 6500.0,
        14 => 11500.0,
        15 => 20000.0,
        16 => 50000.0,
        17 => 120000.0,
        18 => 350000.0,
        // The documentation for the HyperLogLog algorithm only provides empirically determined thresholds for precisions from 4 to 18.
        _ => unreachable!(),
    }
}
