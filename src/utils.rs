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
/// * registers - A 32-bit word containing the registers to be split.
///
/// # Returns
/// An array of bytes where each element corresponds to a register of NUMBER_OF_BITS_PER_REGISTER
/// bits. The length of the returned array is equal to NUMBER_OF_REGISTERS_IN_WORD.
///
pub fn split_registers<
    const NUMBER_OF_REGISTERS_IN_WORD: usize,
    const NUMBER_OF_BITS_PER_REGISTER: usize,
>(
    registers: u32,
) -> [u32; NUMBER_OF_REGISTERS_IN_WORD] {
    let mut result = [0; NUMBER_OF_REGISTERS_IN_WORD];
    for i in 0..NUMBER_OF_REGISTERS_IN_WORD {
        result[i] = registers >> (i * NUMBER_OF_BITS_PER_REGISTER);
    }
    result
}
