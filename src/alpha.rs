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
/// # use hyperloglog_rs::prelude::*;
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
/// // Clamping the value
/// let value = 0.7213 / (1.0 + 1.079 / 4096.0);
/// let value: usize = (value * 1000.0) as usize;
/// let value : f32 = (value as f32) / 1000.0;
/// 
/// assert_eq!(alpha_4096, value);
/// ```
#[inline(always)]
pub const fn get_alpha(number_of_registers: usize) -> f32 {
    // Match the number of registers to the known alpha values
    match number_of_registers {
        16 => 0.673,
        32 => 0.697,
        64 => 0.709,
        128 => 0.715,
        256 => 0.718,
        512 => 0.720,
        1024 => 0.721,
        2048 => 0.721,
        4096 => 0.721,
        8192 => 0.721,
        16384 => 0.721,
        32768 => 0.721,
        65536 => 0.721,
        // The precision only allows values from 2^4 to 2^16.
        _ => unreachable!(),
    }
}
