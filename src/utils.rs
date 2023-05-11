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

#[inline]
/// Converts an array of HLL registers into a single 32-bit word.
///
/// # Arguments
///
/// * `registers` - A fixed-size array of `u32` registers.
///
/// # Type parameters
/// * `NUMBER_OF_BITS_PER_REGISTER` - The number of bits used to represent each HLL register.
///
/// # Returns
/// A 32-bit word that represents the input `registers` array, with each HLL register occupying
/// `NUMBER_OF_BITS_PER_REGISTER` bits of the word.
///
/// # Examples
///
/// Convert an array of 5 registers with 4 bits per register to a single 32-bit word:
///
/// ```rust
/// # use hyperloglog_rs::utils::word_from_registers;
///
/// let registers = [0b1111, 0b0101, 0b0011, 0b1010, 0b1001];
/// let word = word_from_registers::<4>(&registers);
/// let expected = 0b0000_0000_0000_1001_1010_0011_0101_1111;
/// assert_eq!(word, expected, "Example 1, expected {:b}, got {:b}", expected, word);
/// ```
///
/// Convert an array of 3 registers with 6 bits per register to a single 32-bit word:
///
/// ```rust
/// # use hyperloglog_rs::utils::word_from_registers;
/// let registers = [0b111111, 0b001001, 0b101010];
/// let word = word_from_registers::<6>(&registers);
/// let expected = 0b00_000000_000000_000000_101010_001001_111111;
/// assert_eq!(word, expected, "Example 2, expected {:b}, got {:b}", expected, word);
/// ```
///
/// If the number of registers in the input array is less than NUMBER_OF_REGISTERS_IN_WORD,
/// the remaining bits in the output word will be set to 0. In the following example, we convert
/// an array of 3 registers with 4 bits per register to a single 32-bit word:
///
/// ```rust
/// # use hyperloglog_rs::utils::word_from_registers;
/// let registers = [0b1111, 0b0101, 0b0011];
/// let word = word_from_registers::<5>(&registers);
/// let expected = 0b00_00000_00000_00000_00011_00101_01111;
/// assert_eq!(word, expected, "Example 3, expected {:b}, got {:b}", expected, word);
/// ```
///
/// If the number of registers in the input array is greater than number of registers that fits,
/// in a single 32-bit word, which is NUMBER_OF_REGISTERS_IN_WORD, the extra registers are ignored.
///
/// ```rust
/// # use hyperloglog_rs::utils::word_from_registers;
/// let registers = [0b111, 0b010, 0b001, 0b101, 0b100];
/// let word = word_from_registers::<8>(&registers);
/// let expected = 0b00000101_00000001_00000010_00000111;
/// assert_eq!(word, expected, "Example 4, expected {:b}, got {:b}", expected, word);
/// ```
///
pub fn word_from_registers<const NUMBER_OF_BITS_PER_REGISTER: usize>(registers: &[u32]) -> u32 {
    registers.iter().rev().fold(0, |mut word, &register| {
        word <<= NUMBER_OF_BITS_PER_REGISTER;
        word |= register;
        word
    })
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
    let number_of_possible_registers = NUMBER_OF_REGISTERS;
    let mut i = 0;
    while i < number_of_possible_registers {
        small_corrections[i] =
            NUMBER_OF_REGISTERS as f32 * log(NUMBER_OF_REGISTERS as f64 / (i + 1) as f64) as f32;
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
