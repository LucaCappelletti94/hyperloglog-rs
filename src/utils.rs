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
//!

include!(concat!(env!("OUT_DIR"), "/alpha_values.rs"));

#[inline]
/// Calculates the integer ceil of the division of `numerator` by `denominator`.
pub(crate) const fn ceil(numerator: usize, denominator: usize) -> usize {
    (numerator + denominator - 1) / denominator
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

/// Computes the alpha constant for the given number of registers.
///
/// The alpha constant is used to scale the raw HyperLogLog estimate into an
/// estimate of the true cardinality of the set.
///
/// # Arguments
/// * `NUMBER_OF_REGISTERS`: The number of registers in the HyperLogLog
///   data structure.
///
/// # Returns
/// The alpha constant for the given number of registers.
///
/// # Examples
///
/// ```
/// ```
#[inline(always)]
pub(crate) fn get_alpha(precision: usize) -> f32 {
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
