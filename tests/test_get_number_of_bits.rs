#[cfg(test)]
mod tests {
    use hyperloglog::prelude::*;

    #[test]
    /// number of bits for a given number of registers `N`.
    ///
    /// The function `get_number_of_bits` returns the number of bits `b` to be used in
    /// the HyperLogLog algorithm for a given number of registers `N`. This test ensures
    /// that the function returns the correct values for the precomputed cases, which
    /// have been computed from the formula (((6 * N) as f32).ln() / 2.0_f32.ln()) as usize.
    ///
    /// This test iterates over each precomputed case and compares the value returned by
    /// the function to the expected value. If the function returns the correct value
    fn test_get_number_of_bits() {
        assert_eq!(HyperLogLog::<1>::NUMBER_OF_BITS, 2);
        assert_eq!(HyperLogLog::<2>::NUMBER_OF_BITS, 3);
        assert_eq!(HyperLogLog::<3>::NUMBER_OF_BITS, 4);
        assert_eq!(HyperLogLog::<4>::NUMBER_OF_BITS, 4);
        assert_eq!(HyperLogLog::<5>::NUMBER_OF_BITS, 4);
        assert_eq!(HyperLogLog::<6>::NUMBER_OF_BITS, 5);
        assert_eq!(HyperLogLog::<7>::NUMBER_OF_BITS, 5);
        assert_eq!(HyperLogLog::<8>::NUMBER_OF_BITS, 5);
        assert_eq!(HyperLogLog::<9>::NUMBER_OF_BITS, 5);
        assert_eq!(HyperLogLog::<10>::NUMBER_OF_BITS, 5);
        assert_eq!(HyperLogLog::<11>::NUMBER_OF_BITS, 6);
        assert_eq!(HyperLogLog::<12>::NUMBER_OF_BITS, 6);
        assert_eq!(HyperLogLog::<13>::NUMBER_OF_BITS, 6);
        assert_eq!(HyperLogLog::<14>::NUMBER_OF_BITS, 6);
        assert_eq!(HyperLogLog::<15>::NUMBER_OF_BITS, 6);
        assert_eq!(HyperLogLog::<16>::NUMBER_OF_BITS, 6);
        assert_eq!(HyperLogLog::<17>::NUMBER_OF_BITS, 6);
        assert_eq!(HyperLogLog::<18>::NUMBER_OF_BITS, 6);
        assert_eq!(HyperLogLog::<19>::NUMBER_OF_BITS, 6);
    }
}
