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
use crate::log::log;

#[inline]
/// Calculates the integer ceil of the division of `numerator` by `denominator`.
///
/// # Examples
///
/// Test with evenly divisible values:
/// ```rust
///  # use hyperloglog_rs::utils::ceil;
///  assert_eq!(ceil(10, 5), 2);
///  assert_eq!(ceil(25, 5), 5);
///  assert_eq!(ceil(100, 10), 10);
///  ```
///
/// Test with values that require rounding up:
/// ```
/// # use hyperloglog_rs::utils::ceil;
/// assert_eq!(ceil(3, 2), 2);
/// assert_eq!(ceil(7, 3), 3);
/// assert_eq!(ceil(11, 4), 3);
/// assert_eq!(ceil(100, 7), 15);
/// ```
///
pub const fn ceil(numerator: usize, denominator: usize) -> usize {
    (numerator + denominator - 1) / denominator
}

/// Precomputes small corrections for HyperLogLog algorithm.
///
/// This function calculates a correction factor for each register that helps to improve the
/// accuracy of the HyperLogLog algorithm. The corrections are stored in an array and can be
/// accessed for later use by other methods of the HyperLogLog struct.
///
/// # Arguments
/// * `NUMBER_OF_REGISTERS` - The number of registers used in the HyperLogLog algorithm.
///
/// # Examples
/// ```
/// use hyperloglog_rs::utils::precompute_small_corrections;
/// use hyperloglog_rs::log::log;
///
/// const NUMBER_OF_REGISTERS: usize = 16;
/// let small_corrections = precompute_small_corrections::<NUMBER_OF_REGISTERS>();
/// assert_eq!(small_corrections.len(), NUMBER_OF_REGISTERS);
/// assert_eq!(small_corrections[0], NUMBER_OF_REGISTERS as f32 * log(NUMBER_OF_REGISTERS as f64) as f32);
/// assert_eq!(small_corrections[1], NUMBER_OF_REGISTERS as f32 * log(NUMBER_OF_REGISTERS as f64 / 2.0_f64) as f32);
/// ```
pub const fn precompute_small_corrections<const NUMBER_OF_REGISTERS: usize>(
) -> [f32; NUMBER_OF_REGISTERS] {
    let mut small_corrections = [0_f32; NUMBER_OF_REGISTERS];
    let mut i = 0;
    // We can skip the last value in the small range correction array, because it is always 0.
    while i < NUMBER_OF_REGISTERS - 1 {
        small_corrections[i] =
            (NUMBER_OF_REGISTERS as f64 * log(NUMBER_OF_REGISTERS as f64 / (i + 1) as f64)) as f32;
        i += 1;
    }
    small_corrections
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
/// # use hyperloglog_rs::utils::get_alpha;
///
/// let alpha_16 = get_alpha(16);
/// let alpha_32 = get_alpha(32);
/// let alpha_64 = get_alpha(64);
///
/// assert_eq!(alpha_16, 0.673);
/// assert_eq!(alpha_32, 0.697);
/// assert_eq!(alpha_64, 0.709);
///
/// let alpha_4096 = get_alpha(4096);
///
/// assert_eq!(alpha_4096, 0.7213 / (1.0 + 1.079 / 4096.0));
/// ```
#[inline(always)]
pub const fn get_alpha(number_of_registers: usize) -> f32 {
    // Match the number of registers to the known alpha values
    match number_of_registers {
        16 => 0.673,
        32 => 0.697,
        64 => 0.709,
        _ => 0.7213 / (1.0 + 1.079 / number_of_registers as f32),
    }
}
