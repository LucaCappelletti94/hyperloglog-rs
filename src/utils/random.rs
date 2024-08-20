//! Random number generators.

use super::{One, Zero, PositiveInteger, VariableWord};

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
#[allow(unsafe_code)]
/// Returns an iterator over random values.
///
/// # Arguments
/// * `minimal_size` - The minimal size of the iterator.
/// * `maximal_size` - The maximal size of the iterator.
/// * `maximal_value` - The maximal value of the iterator, by default `V::MASK` (the maximal value of the variable).
/// * `random_state` - The random state.
///
/// # Panics
/// * Panics if the maximal value provided is zero.
///
/// # Safety
/// We employ unchecked conversion from `u64` to `V::Word` to avoid the overhead of checking
/// if the value is within the bounds of the variable. This is safe because we ensure that
/// the value is within the bounds of the variable by using the `V::MASK` to mask the value.
pub fn iter_var_len_random_values<V: VariableWord>(
    minimal_size: u64,
    maximal_size: u64,
    maximal_value: Option<V::Word>,
    random_state: Option<u64>,
) -> impl Iterator<Item = V::Word> {
    assert!(minimal_size <= maximal_size, "The minimal size ({minimal_size}) must be less than or equal to the maximal size ({maximal_size}).");
    assert!(
        maximal_value.as_ref().map_or(true, |mv| !mv.is_zero()),
        "The maximal value must be provided if the variable mask is zero."
    );

    let delta = maximal_size - minimal_size;

    let mut state = random_state.unwrap_or(12_834_791_235_231_473_875_u64);

    state = splitmix64(state);

    let size = minimal_size + if delta > 0 { state % delta } else { 0 };

    state = splitmix64(state);
    let actual_maximal_value: V::Word = maximal_value.map_or(
        unsafe { V::Word::unchecked_from_u64(V::MASK) },
        |mv| unsafe {
            V::Word::ONE + V::Word::unchecked_from_u64(xorshift64(state) & V::MASK) % mv
        },
    );
    state = splitmix64(state);

    (0..size).map(move |_| {
        state = xorshift64(state);
        unsafe { V::Word::unchecked_from_u64(state & V::MASK) % actual_maximal_value }
    })
}

#[inline]
/// Returns an iterator over random values.
///
/// # Arguments
/// * `size` - The size of the iterator.
/// * `maximal_value` - The maximal value of the iterator.
/// * `random_state` - The random state. If None, a strong random state is used.
///
/// # Panics
/// Panics if the maximal size is greater than `u64::MAX`.
/// Panics if the minimal size is greater than the maximal size.
pub fn iter_random_values<V: VariableWord>(
    size: u64,
    maximal_value: Option<V::Word>,
    random_state: Option<u64>,
) -> impl Iterator<Item = V::Word> {
    iter_var_len_random_values::<V>(size, size, maximal_value, random_state)
}
