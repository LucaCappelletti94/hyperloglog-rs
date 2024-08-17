//! Random number generators.

#[must_use]
#[inline]
/// `SplitMix64` is a pseudorandom number generator that is very fast and has a good quality of randomness.
pub fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// `Xorshift64` is a pseudorandom number generator that is very fast and has a good quality of randomness.
#[must_use]
#[inline]
pub fn xorshift64(mut x: u64) -> u64 {
    x ^= x << 13_u8;
    x ^= x >> 7_u8;
    x ^= x << 17_u8;
    x
}

#[inline]
/// Returns an iterator over random values.
///
/// # Arguments
/// * `maximal_size` - The maximal size of the iterator.
/// * `maximal_value` - The maximal value of the iterator.
/// * `random_state` - The random state.
///
/// # Panics
/// Panics if the maximal size is greater than `u64::MAX`.
pub fn iter_random_values(
    maximal_size: usize,
    maximal_value: Option<u64>,
    mut random_state: u64,
) -> impl Iterator<Item = u64> {
    random_state = splitmix64(splitmix64(random_state));

    let actual_maximal_size = if maximal_size > 0 {
        random_state % u64::try_from(maximal_size).unwrap()
    } else {
        0
    };

    random_state = splitmix64(splitmix64(random_state));
    let actual_maximal_value = maximal_value.map_or(u64::MAX, |mv| xorshift64(random_state) % mv);
    (0..actual_maximal_size).map(move |_| {
        random_state = splitmix64(splitmix64(random_state));
        random_state = xorshift64(random_state);
        if actual_maximal_value > 0 {
            random_state % actual_maximal_value
        } else {
            0
        }
    })
}
