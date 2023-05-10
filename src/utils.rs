use crate::prelude::*;

#[inline]
/// Calculates the integer ceil of the division of `numerator` by `denominator`.
pub const fn ceil(numerator: usize, denominator: usize) -> usize {
    (numerator + denominator - 1) / denominator
}

#[inline]
/// Split the given registers into an array of bytes where each element corresponds
/// to a register of NUMBER_OF_BITS_PER_REGISTER bits. The number of registers in a single
/// 32-bit registers word is specified by NUMBER_OF_REGISTERS_IN_WORD.
///
/// # Arguments
/// * word - A 32-bit word containing the registers to be split.
///
/// # Returns
/// An array of bytes where each element corresponds to a register of NUMBER_OF_BITS_PER_REGISTER
/// bits. The length of the returned array is equal to NUMBER_OF_REGISTERS_IN_WORD.
///
/// # Examples
/// Split a 32-bit word into 4 8-bit registers
/// ```rust
/// # use hyperloglog_rs::prelude::*;
///
/// let word = 0b0110_1001_1010_1111_0110_1001_1010_1111;
/// let registers = split_registers::<4>(word);
/// let expected = [0b1010_1111, 0b0110_1001, 0b1010_1111, 0b0110_1001];
/// assert_eq!(registers, expected, "Example 1, Expected: {:?}, got: {:?}", expected, registers);
/// ```
///
/// Split a 32-bit word into 2 16-bit registers
/// ```rust
/// # use hyperloglog_rs::prelude::*;
///
/// let word = 0b1111_0000_1111_0000_1010_1010_0101_0101;
/// let registers = split_registers::<2>(word);
/// let expected = [0b1010_1010_0101_0101, 0b1111_0000_1111_0000];
/// assert_eq!(registers, expected, "Example 2, Expected: {:?}, got: {:?}", expected, registers);
/// ```
///
/// Split a 32-bit word into 8 4-bit registers
///
/// ```rust
/// # use hyperloglog_rs::prelude::*;
///
/// let word = 0b1010_0101_1111_0000_1111_1111_0101_1010;
/// let registers = split_registers::<8>(word);
/// let expected = [0b1010, 0b0101, 0b1111, 0b1111, 0b0000, 0b1111, 0b0101, 0b1010];
/// assert_eq!(registers, expected, "Example 3, Expected: {:?}, got: {:?}", expected, registers);
/// ```
///
/// Split a 32-bit word into 1 32-bit register
///
/// ```rust
/// # use hyperloglog_rs::prelude::*;
///
/// let word = 0b1111_1111_0000_0000_1111_0000_1111_1111;
/// let registers = split_registers::<1>(word);
/// let expected = [0b1111_1111_0000_0000_1111_0000_1111_1111];
/// assert_eq!(registers, expected, "Example 4, Expected: {:?}, got: {:?}", expected, registers);
/// ```
pub fn split_registers<const NUMBER_OF_REGISTERS_IN_WORD: usize>(
    word: u32,
) -> [u32; NUMBER_OF_REGISTERS_IN_WORD] {
    let mask = if NUMBER_OF_REGISTERS_IN_WORD == 1 {
        u32::MAX
    } else {
        (1 << (32 / NUMBER_OF_REGISTERS_IN_WORD)) - 1
    };
    let mut result = [0_u32; NUMBER_OF_REGISTERS_IN_WORD];
    result.iter_mut().enumerate().for_each(|(i, res)| {
        *res = (word >> i * (32 / NUMBER_OF_REGISTERS_IN_WORD)) & mask;
    });
    result
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
/// # use hyperloglog_rs::prelude::*;
///
/// let registers = [0b1111, 0b0101, 0b0011, 0b1010, 0b1001];
/// let word = to_word::<4>(&registers);
/// let expected = 0b0000_0000_0000_1001_1010_0011_0101_1111;
/// assert_eq!(word, expected, "Example 1, expected {:b}, got {:b}", expected, word);
/// ```
///
/// Convert an array of 3 registers with 6 bits per register to a single 32-bit word:
///
/// ```rust
/// # use hyperloglog_rs::prelude::*;
/// let registers = [0b111111, 0b001001, 0b101010];
/// let word = to_word::<6>(&registers);
/// let expected = 0b00_000000_000000_000000_101010_001001_111111;
/// assert_eq!(word, expected, "Example 2, expected {:b}, got {:b}", expected, word);
/// ```
///
/// If the number of registers in the input array is less than NUMBER_OF_REGISTERS_IN_WORD,
/// the remaining bits in the output word will be set to 0. In the following example, we convert
/// an array of 3 registers with 4 bits per register to a single 32-bit word:
///
/// ```rust
/// # use hyperloglog_rs::prelude::*;
/// let registers = [0b1111, 0b0101, 0b0011];
/// let word = to_word::<5>(&registers);
/// let expected = 0b00_00000_00000_00000_00011_00101_01111;
/// assert_eq!(word, expected, "Example 3, expected {:b}, got {:b}", expected, word);
/// ```
///
/// If the number of registers in the input array is greater than number of registers that fits,
/// in a single 32-bit word, which is NUMBER_OF_REGISTERS_IN_WORD, the extra registers are ignored.
///
/// ```rust
/// # use hyperloglog_rs::prelude::*;
/// let registers = [0b111, 0b010, 0b001, 0b101, 0b100];
/// let word = to_word::<8>(&registers);
/// let expected = 0b00000101_00000001_00000010_00000111;
/// assert_eq!(word, expected, "Example 4, expected {:b}, got {:b}", expected, word);
/// ```
///
pub fn to_word<const NUMBER_OF_BITS_PER_REGISTER: usize>(registers: &[u32]) -> u32 {
    registers.iter().rev().fold(0, |mut word, &register| {
        word <<= NUMBER_OF_BITS_PER_REGISTER;
        word |= register;
        word
    })
}

/// Precomputes an array of reciprocal powers of two and returns it.
///
/// This function generates an array of reciprocal powers of two, which is used as a lookup table
/// for the HyperLogLog algorithm. The array is of length 2^BITS and contains the reciprocal
/// value of each power of two up to 2^(BITS-1).
///
/// # Example
///
/// ```
/// use hyperloglog_rs::prelude::*;
///
/// const BITS: usize = 5;
///
/// let reciprocals = precompute_reciprocals::<BITS>();
///
/// assert_eq!(reciprocals[0], 1.0_f32);
/// assert_eq!(reciprocals[1], 0.5_f32);
/// assert_eq!(reciprocals[2], 0.25_f32);
/// assert_eq!(reciprocals[3], 0.125_f32);
/// assert_eq!(reciprocals[4], 0.0625_f32);
/// assert_eq!(reciprocals[5], 0.03125_f32);
/// assert_eq!(reciprocals[6], 0.015625_f32);
/// assert_eq!(reciprocals[7], 0.0078125_f32);
/// assert_eq!(reciprocals[8], 0.00390625_f32);
/// ```
pub const fn precompute_reciprocals<const BITS: usize>() -> [f32; 1 << BITS] {
    let mut reciprocals = [0_f32; 1 << BITS];
    let number_of_possible_registers = 1 << BITS;
    let mut i = 0;
    let mut current_power_of_two: f32 = 1.0_f32;
    while i < number_of_possible_registers {
        reciprocals[i] = 1.0_f32 / current_power_of_two;
        current_power_of_two *= 2.0;
        i += 1;
    }
    reciprocals
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
/// use hyperloglog_rs::prelude::*;
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
