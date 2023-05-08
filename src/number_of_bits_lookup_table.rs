/// Returns the number of bits needed to represent each register for a given number of registers N,
/// according to the HyperLogLog algorithm.
///
/// # Arguments
/// * `N` - The number of registers.
///
/// # Examples
///
/// ```
/// use hyperloglog::prelude::*;
///
/// const N: usize = 16;
///
/// let number_of_bits = HyperLogLog::<N>::NUMBER_OF_BITS;
///
/// assert_eq!(number_of_bits, 6);
/// ```
pub const fn get_number_of_bits<const N: usize>() -> usize {
    // These are precomputed (((6 * N) as f32).ln() / 2.0_f32.ln()) as usize
    match N {
        1 => 2,
        2 => 3,
        3 => 4,
        4 => 4,
        5 => 4,
        6 => 5,
        7 => 5,
        8 => 5,
        9 => 5,
        10 => 5,
        11 => 6,
        12 => 6,
        13 => 6,
        14 => 6,
        15 => 6,
        16 => 6,
        17 => 6,
        18 => 6,
        19 => 6,
        20 => 6,
        21 => 6,
        22 => 7,
        23 => 7,
        24 => 7,
        25 => 7,
        26 => 7,
        27 => 7,
        28 => 7,
        29 => 7,
        30 => 7,
        31 => 7,
        // "The case for N > 31 is not implemented. Please open an issue on GitHub."
        _ => unimplemented!(),
    }
}
