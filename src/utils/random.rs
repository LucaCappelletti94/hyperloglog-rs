//! Random number generators.

use core::usize;

/// SplitMix64 is a pseudorandom number generator that is very fast and has a good quality of randomness.
pub fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

/// Xorshift64 is a pseudorandom number generator that is very fast and has a good quality of randomness.
pub fn xorshift64(mut x: u64) -> u64 {
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

/// Returns an iterator over random values.
pub fn iter_random_values(
    maximal_size: usize,
    maximal_value: Option<usize>,
    mut random_state: u64,
) -> impl Iterator<Item = u64> {
    random_state = splitmix64(random_state);
    let maximal_size = 1 + xorshift64(random_state) as usize % maximal_size;
    random_state = splitmix64(random_state);

    let maximal_value = maximal_value.map_or(u64::MAX, |maximal_value| {
        (1 + xorshift64(random_state) as usize % maximal_value) as u64
    });
    (0..maximal_size).map(move |_| {
        random_state = splitmix64(random_state);
        random_state = xorshift64(random_state);
        random_state as u64 % maximal_value
    })
}
