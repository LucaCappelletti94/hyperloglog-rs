#[inline]
/// Calculates the integer ceil of the division of `numerator` by `denominator`.
pub const fn ceil(numerator: usize, denominator: usize) -> usize {
    (numerator + denominator - 1) / denominator
}
