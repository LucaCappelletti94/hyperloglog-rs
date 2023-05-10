use crate::prelude::*;
use core::hash::{Hash, Hasher};
use core::ops::{BitOr, BitOrAssign};
use std::collections::hash_map::DefaultHasher;

#[derive(Clone, Debug, Eq, PartialEq)]
/// A probabilistic algorithm for estimating the number of distinct elements in a set.
///
/// HyperLogLog is a probabilistic algorithm designed to estimate the number
/// of distinct elements in a set. It does so by taking advantage of the fact
/// that the representation of an element can be transformed into a uniform
/// distribution in a space with a fixed range.
///
/// HyperLogLog works by maintaining a fixed-sized register array,
/// where each register holds a counter. The algorithm splits the input set into subsets,
/// applies a hash function to each element in the subset, and then updates
/// the corresponding counter in the register array.
///
/// HyperLogLog uses a trick called "probabilistic counting" to estimate
/// the number of distinct elements in the set. Each register counter is converted
/// to a binary string, and the algorithm counts the number of leading zeros in
/// each binary string. The maximum number of leading zeros over all counters
/// is used to estimate the number of distinct elements in the set.
///
/// HyperLogLog has a tunable parameter called precision that determines
/// the accuracy of the algorithm. Higher precision leads to better accuracy,
/// but requires more memory. The error rate of the algorithm is guaranteed
/// to be within a certain bound, depending on the chosen precision.
///
/// # Examples
///
/// ```
/// use hyperloglog_rs::prelude::*;
///
/// let mut hll = HyperLogLog::<10, 6>::new();
/// hll.insert(&"apple");
/// hll.insert(&"banana");
/// hll.insert(&"cherry");
///
/// let estimated_cardinality = hll.estimate_cardinality();
/// assert!(estimated_cardinality >= 3.0_f32 * 0.9 &&
///         estimated_cardinality <= 3.0_f32 * 1.1);
/// ```
pub struct HyperLogLog<const PRECISION: usize, const BITS: usize>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    words: [u32; ceil(1 << PRECISION, 32 / BITS)],
    number_of_zero_register: usize,
}

impl<const PRECISION: usize, const BITS: usize, T: Hash> From<T> for HyperLogLog<PRECISION, BITS>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
    [(); 1 << PRECISION]:,
    [(); 1 << BITS]:,
{
    fn from(value: T) -> Self {
        let mut hll = Self::new();
        hll.insert(value);
        hll
    }
}

impl<const PRECISION: usize, const BITS: usize> HyperLogLog<PRECISION, BITS>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
    [(); 1 << PRECISION]:,
    [(); 1 << BITS]:,
{
    pub const NUMBER_OF_REGISTERS: usize = 1 << PRECISION;
    pub const NUMBER_OF_REGISTERS_SQUARED: f32 =
        (Self::NUMBER_OF_REGISTERS * Self::NUMBER_OF_REGISTERS) as f32;
    pub const SMALL_RANGE_CORRECTION_THRESHOLD: f32 = 2.5_f32 * (Self::NUMBER_OF_REGISTERS as f32);
    pub const TWO_32: f32 = (1u64 << 32) as f32;
    pub const INTERMEDIATE_RANGE_CORRECTION_THRESHOLD: f32 = Self::TWO_32 / 30.0_f32;
    pub const ALPHA: f32 = get_alpha(1 << PRECISION);
    pub const LOWER_REGISTER_MASK: u32 = (1 << BITS) - 1;
    pub const NUMBER_OF_REGISTERS_IN_WORD: usize = 32 / BITS;

    pub const PRECOMPUTED_RECIPROCALS: [f32; 1 << BITS] = precompute_reciprocals::<BITS>();
    pub const SMALL_CORRECTIONS: [f32; 1 << PRECISION] =
        precompute_small_corrections::<{ 1 << PRECISION }>();

    /// Create a new HyperLogLog counter.
    pub fn new() -> Self {
        assert!(PRECISION >= 4);
        assert!(PRECISION <= 16);
        Self {
            words: [0; ceil(1 << PRECISION, 32 / BITS)],
            number_of_zero_register: 1_usize << PRECISION,
        }
    }

    /// Create a new HyperLogLog counter.
    pub fn from_registers(registers: [u32; 1 << PRECISION]) -> Self {
        let mut words = [0; ceil(1 << PRECISION, 32 / BITS)];
        let number_of_zero_register = words
            .iter_mut()
            .zip(registers.chunks(Self::NUMBER_OF_REGISTERS_IN_WORD))
            .fold(0, |mut number_of_zero_register, (word, word_registers)| {
                number_of_zero_register += word_registers
                    .iter()
                    .filter(|&&register| register == 0)
                    .count();
                *word = to_word::<BITS>(&word_registers);
                number_of_zero_register
            });
        Self {
            words,
            number_of_zero_register,
        }
    }

    #[inline(always)]
    /// Estimates the cardinality of the set based on the HLL counter data.
    ///
    /// # Example
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    /// const PRECISION: usize = 8;
    /// const BITS: usize = 5;
    /// let mut hll = HyperLogLog::<PRECISION, BITS>::new();
    /// let elements = vec![1, 2, 3, 4, 5];
    /// for element in &elements {
    ///     hll.insert(element);
    /// }
    /// let estimated_cardinality = hll.estimate_cardinality();
    /// assert!(estimated_cardinality >= elements.len() as f32 * 0.9 &&
    ///         estimated_cardinality <= elements.len() as f32 * 1.1);
    /// ```
    ///
    /// # Returns
    /// * `f32` - The estimated cardinality of the set.
    pub fn estimate_cardinality(&self) -> f32 {
        // Dispatch specialized count for 32 / BITS registers per word.
        let mut raw_estimate: f32 = self
            .iter()
            .map(|register| Self::PRECOMPUTED_RECIPROCALS[register as usize])
            .sum();

        // Apply the final scaling factor to obtain the estimate of the cardinality
        raw_estimate = Self::ALPHA * Self::NUMBER_OF_REGISTERS_SQUARED / raw_estimate;

        // Apply the small range correction factor if the raw estimate is below the threshold
        // and there are zero registers in the counter.
        if raw_estimate <= Self::SMALL_RANGE_CORRECTION_THRESHOLD
            && self.number_of_zero_register > 0
        {
            Self::SMALL_CORRECTIONS[self.number_of_zero_register - 1]
        // Apply the intermediate range correction factor if the raw estimate is above the threshold.
        } else if raw_estimate >= Self::INTERMEDIATE_RANGE_CORRECTION_THRESHOLD {
            -Self::TWO_32 * (-raw_estimate / Self::TWO_32).ln_1p()
        } else {
            raw_estimate
        }
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        debug_assert_eq!(
            self.words.len(),
            ceil(1 << PRECISION, Self::NUMBER_OF_REGISTERS_IN_WORD)
        );

        self.words
            .iter()
            .copied()
            .flat_map(|six_registers| {
                (0..Self::NUMBER_OF_REGISTERS_IN_WORD)
                    .map(move |i| six_registers >> i * BITS & Self::LOWER_REGISTER_MASK)
            })
            .take(Self::NUMBER_OF_REGISTERS)
    }

    #[inline(always)]
    /// Returns the number of registers in the HLL counter.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// // Create a new HLL counter with 128 registers
    /// let mut hll = HyperLogLog::<12, 8>::new();
    /// assert_eq!(hll.len(), 4096);
    ///
    /// // Insert some elements into the HLL counter
    /// hll.insert(&1);
    /// hll.insert(&2);
    /// hll.insert(&3);
    /// assert_eq!(hll.len(), 1 << 12);
    ///
    /// // Merge another HLL counter with 128 registers
    /// let mut hll2 = HyperLogLog::<12, 8>::new();
    /// hll2.insert(&4);
    /// hll2.insert(&5);
    /// hll |= hll2;
    /// assert_eq!(hll.len(), 1 << 12);
    /// ```
    pub fn len(&self) -> usize {
        debug_assert_eq!(Self::NUMBER_OF_REGISTERS, self.iter().count());
        Self::NUMBER_OF_REGISTERS
    }

    #[inline(always)]
    /// Returns the number of bits used to represent each register in the HyperLogLog counter.
    ///
    /// # Returns
    ///
    /// An unsigned integer value representing the number of bits used to represent each register
    /// in the HyperLogLog counter.
    ///
    /// # Example
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll = HyperLogLog::<13, 6>::new();
    /// assert_eq!(hll.get_number_of_bits(), 6);
    /// ```
    pub fn get_number_of_bits(&self) -> usize {
        BITS
    }

    #[inline(always)]
    /// Returns the number of extra registers that are not actually used.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// // Create a HyperLogLog counter with precision 10 and 6-bit registers
    /// let mut hll = HyperLogLog::<10, 6>::new();
    ///
    /// // Since the number of registers is not a multiple of the number of registers in a word,
    /// // there are padding registers that are not actually used.
    /// assert_eq!(hll.get_number_of_padding_registers(), 1);
    ///
    /// // Insert some elements into the counter
    /// hll.insert(&1);
    /// hll.insert(&2);
    ///
    /// // The number of padding registers is still the same
    /// assert_eq!(hll.get_number_of_padding_registers(), 1);
    /// ```
    pub fn get_number_of_padding_registers(&self) -> usize {
        self.words.len() * Self::NUMBER_OF_REGISTERS_IN_WORD - Self::NUMBER_OF_REGISTERS
    }

    #[inline(always)]
    /// Returns the number of registers with zero values. This value is used for computing a small
    /// correction when estimating the cardinality of a small set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// // Create a new HyperLogLog counter with precision 14 and 5 bits per register.
    /// let mut hll = HyperLogLog::<14, 5>::new();
    ///
    /// // Add some elements to the counter.
    /// hll.insert(&1);
    /// hll.insert(&2);
    /// hll.insert(&3);
    ///
    /// // Get the number of zero registers.
    /// let number_of_zero_registers = hll.get_number_of_zero_registers();
    ///
    /// assert_eq!(number_of_zero_registers, 16381);
    /// ```
    pub fn get_number_of_zero_registers(&self) -> usize {
        self.number_of_zero_register
    }

    #[inline(always)]
    pub fn get_number_of_non_zero_registers(&self) -> usize {
        // Calculates the number of registers that have a non-zero value by
        // subtracting the number of registers with a zero value from the total number of registers
        self.len() - self.get_number_of_zero_registers()
    }

    #[inline(always)]
    pub fn get_registers(&self) -> [u32; 1 << PRECISION] {
        let mut array = [0; (1 << PRECISION)];
        self.iter()
            .zip(array.iter_mut())
            .for_each(|(value, target)| {
                *target = value;
            });
        array
    }

    #[inline(always)]
    /// Adds an element to the HyperLogLog counter.
    ///
    /// # Arguments
    /// * `rhs` - The element to add.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// const PRECISION: usize = 10;
    ///
    /// let mut hll = HyperLogLog::<PRECISION, 6>::new();
    ///
    /// hll.insert("Hello");
    /// hll.insert("World");
    ///
    /// assert!(hll.estimate_cardinality() >= 2.0);
    /// ```
    ///
    /// # Performance
    ///
    /// The performance of this function depends on the size of the HyperLogLog counter (`N`), the number
    /// of distinct elements in the input, and the hash function used to hash elements. For a given value of `N`,
    /// the function has an average time complexity of O(1) and a worst-case time complexity of O(log N).
    /// However, the actual time complexity may vary depending on the distribution of the hashed elements.
    ///
    /// # Errors
    ///
    /// This function does not return any errors.
    pub fn insert<T: Hash>(&mut self, rhs: T) {
        // Create a new hasher.
        let mut hasher = DefaultHasher::new();
        // Calculate the hash.
        rhs.hash(&mut hasher);
        // Drops the higher 32 bits.
        let mut hash: u32 = hasher.finish() as u32;

        // Calculate the register's index.
        let index: usize = (hash >> (32 - PRECISION)) as usize;
        debug_assert!(
            index < Self::NUMBER_OF_REGISTERS,
            "The index {} must be less than the number of registers {}.",
            index,
            Self::NUMBER_OF_REGISTERS
        );

        // Shift left the bits of the index.
        hash = (hash << PRECISION) | (1 << (PRECISION - 1));

        // Count leading zeros.
        let number_of_zeros: u32 = 1 + hash.leading_zeros();

        // Calculate the position of the register in the internal buffer array.
        let register_position_in_array = index / Self::NUMBER_OF_REGISTERS_IN_WORD;

        debug_assert!(
            register_position_in_array < self.words.len(),
            concat!(
                "The register_position_in_array {} must be less than the number of words {}. ",
                "You have obtained this values starting from the index {} and the word size {}."
            ),
            register_position_in_array,
            self.words.len(),
            index,
            Self::NUMBER_OF_REGISTERS_IN_WORD
        );

        // Calculate the position of the register within the 32-bit word containing it.
        let register_position_in_u32 = index % Self::NUMBER_OF_REGISTERS_IN_WORD;

        // Extract the current value of the register at `index`.
        let register_value: u32 = (self.words[register_position_in_array]
            >> (register_position_in_u32 * BITS))
            & Self::LOWER_REGISTER_MASK;

        // If `number_of_zeros` is greater than the current number_of_zeros, update the register.
        if number_of_zeros > register_value {
            let shifted_zeros = number_of_zeros << (register_position_in_u32 * BITS);
            if register_value == 0 {
                self.number_of_zero_register -= 1;
                // If the current number_of_zeros is zero, decrement `zeros` and set the register to `number_of_zeros`.
                self.words[register_position_in_array] |= shifted_zeros;
            } else {
                // Otherwise, update the register using a bit mask.
                let mask = Self::LOWER_REGISTER_MASK << (register_position_in_u32 * BITS);
                self.words[register_position_in_array] =
                    (self.words[register_position_in_array] & !mask) | shifted_zeros;
            }
        }
    }
}

impl<const PRECISION: usize, const BITS: usize> BitOrAssign for HyperLogLog<PRECISION, BITS>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
    [(); 1 << BITS]:,
{
    #[inline(always)]
    /// Computes union between HLL counters.
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    /// # use core::ops::BitOrAssign;
    ///
    /// let mut hll = HyperLogLog::<8, 6>::new();
    /// hll.insert(1u8);
    ///
    /// let mut hll2 = HyperLogLog::<8, 6>::new();
    /// hll2.insert(2u8);
    ///
    /// hll.bitor_assign(hll2);
    ///
    /// assert!(hll.estimate_cardinality() > 2.0 - 0.1, "The cardinality is {}, we were expecting 2.", hll.estimate_cardinality());
    /// assert!(hll.estimate_cardinality() < 2.0 + 0.1, "The cardinality is {}, we were expecting 2.", hll.estimate_cardinality());
    ///
    /// let mut hll = HyperLogLog::<8, 6>::new();
    /// hll.insert(1u8);
    ///
    /// let mut hll2 = HyperLogLog::<8, 6>::new();
    /// hll2.insert(1u8);
    ///
    /// hll.bitor_assign(hll2);
    ///
    /// assert!(hll.estimate_cardinality() > 1.0 - 0.1, "The cardinality is {}, we were expecting 1.", hll.estimate_cardinality());
    /// assert!(hll.estimate_cardinality() < 1.0 + 0.1, "The cardinality is {}, we were expecting 1.", hll.estimate_cardinality());
    ///
    /// let mut hll3 = HyperLogLog::<16, 6>::new();
    /// hll3.insert(3u8);
    /// hll3.insert(4u8);
    ///
    /// let mut hll4 = HyperLogLog::<16, 6>::new();
    /// hll4.insert(5u8);
    /// hll4.insert(6u8);
    ///
    /// hll3.bitor_assign(hll4);
    ///
    /// assert!(hll3.estimate_cardinality() > 4.0 - 0.1, "Expected a value equal to around 4, got {}", hll3.estimate_cardinality());
    /// assert!(hll3.estimate_cardinality() < 4.0 + 0.1, "Expected a value equal to around 4, got {}", hll3.estimate_cardinality());
    /// ```
    fn bitor_assign(&mut self, rhs: Self) {
        let mut new_number_of_zeros = 0;
        for (left_word, right_word) in self.words.iter_mut().zip(rhs.words.iter().copied()) {
            let mut left_registers = split_registers::<{ 32 / BITS }>(*left_word);
            let right_registers = split_registers::<{ 32 / BITS }>(right_word);

            left_registers
                .iter_mut()
                .zip(right_registers.into_iter())
                .for_each(|(left, right)| {
                    *left = (*left).max(right);
                    if *left == 0 {
                        new_number_of_zeros += 1;
                    }
                });

            *left_word = to_word::<BITS>(&left_registers)
        }
        self.number_of_zero_register = new_number_of_zeros - self.get_number_of_padding_registers();
    }
}

impl<const PRECISION: usize, const BITS: usize> BitOr for HyperLogLog<PRECISION, BITS>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
    [(); 1 << BITS]:,
{
    type Output = Self;

    #[inline(always)]
    /// Computes union between HLL counters.
    fn bitor(mut self, rhs: Self) -> Self {
        self.bitor_assign(rhs);
        self
    }
}
