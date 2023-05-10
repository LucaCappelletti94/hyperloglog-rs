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
