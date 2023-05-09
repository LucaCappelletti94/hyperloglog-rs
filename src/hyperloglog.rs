use crate::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub const NUMBER_OF_REGISTERS_IN_WORD: usize = 5;
pub const NUMBER_OF_BITS_PER_REGISTER: usize = 32 / NUMBER_OF_REGISTERS_IN_WORD; // 32 / 5 = 6.4 -> 6
                                                                                 // A mask to get the lower register (from LSB).
pub const LOWER_REGISTER_MASK: u32 = (1 << NUMBER_OF_BITS_PER_REGISTER) - 1; // These need to be {NUMBER_OF_BITS_PER_REGISTER} bits.

#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
/// HyperLogLog is a probabilistic algorithm for estimating the number of distinct elements in a set.
/// It uses a small amount of memory to produce an approximate count with a guaranteed error rate.
pub struct HyperLogLog<const PRECISION: usize>
where
    [(); ceil(1 << PRECISION, NUMBER_OF_REGISTERS_IN_WORD)]:,
{
    registers: [u32; ceil(1 << PRECISION, NUMBER_OF_REGISTERS_IN_WORD)],
}

impl<const PRECISION: usize, T: Hash> From<T> for HyperLogLog<PRECISION>
where
    [(); ceil(1 << PRECISION, NUMBER_OF_REGISTERS_IN_WORD)]:,
    [(); 1 << PRECISION]:,
{
    fn from(value: T) -> Self {
        let mut hll = Self::new();
        hll.insert(value);
        hll
    }
}

impl<const PRECISION: usize> HyperLogLog<PRECISION>
where
    [(); ceil(1 << PRECISION, NUMBER_OF_REGISTERS_IN_WORD)]:,
    [(); 1 << PRECISION]:,
{
    pub const NUMBER_OF_REGISTERS: usize = 1 << PRECISION;
    pub const NUMBER_OF_REGISTERS_SQUARED: f32 =
        (Self::NUMBER_OF_REGISTERS * Self::NUMBER_OF_REGISTERS) as f32;
    pub const SMALL_RANGE_CORRECTION_THRESHOLD: f32 = 2.5_f32 * (Self::NUMBER_OF_REGISTERS as f32);
    pub const TWO_32: f32 = (1u64 << 32) as f32;
    pub const INTERMEDIATE_RANGE_CORRECTION_THRESHOLD: f32 = Self::TWO_32 / 30.0_f32;
    pub const ALPHA: f32 = get_alpha(1 << PRECISION);

    /// Create a new HyperLogLog counter.
    pub fn new() -> Self {
        assert!(PRECISION >= 4);
        assert!(PRECISION <= 16);
        Self {
            registers: [0; ceil(1 << PRECISION, NUMBER_OF_REGISTERS_IN_WORD)],
        }
    }

    #[inline(always)]
    /// Computes the estimate of the cardinality of the set represented by the HyperLogLog data structure.
    pub fn count(&self) -> f32 {
        // Initialize the total estimate to 0.0
        let mut raw_estimate = 0.0;
        let mut number_of_zero_registers: usize = 0;

        // Iterate over the registers in the data structure
        for register in self.iter() {
            number_of_zero_registers += (register == 0) as usize;
            raw_estimate += 1.0 / (1u64 << register) as f32;
        }

        // Apply the final scaling factor to obtain the estimate of the cardinality
        raw_estimate = Self::ALPHA * Self::NUMBER_OF_REGISTERS_SQUARED / raw_estimate;

        if raw_estimate <= Self::SMALL_RANGE_CORRECTION_THRESHOLD && number_of_zero_registers > 0 {
            get_small_correction_lookup_table::<{ 1 << PRECISION }>(number_of_zero_registers)
        } else if raw_estimate >= Self::INTERMEDIATE_RANGE_CORRECTION_THRESHOLD {
            -Self::TWO_32 * (-raw_estimate / Self::TWO_32).ln_1p()
        } else {
            raw_estimate
        }
    }

    pub fn count_dispatched(&self) -> f32 {
        let (number_of_zero_registers, mut raw_estimate) = dispatch_specialized_count::<
            { ceil(1 << PRECISION, NUMBER_OF_REGISTERS_IN_WORD) },
        >(&self.registers);

        // Apply the final scaling factor to obtain the estimate of the cardinality
        raw_estimate = Self::ALPHA * Self::NUMBER_OF_REGISTERS_SQUARED / raw_estimate;

        if raw_estimate <= Self::SMALL_RANGE_CORRECTION_THRESHOLD && number_of_zero_registers > 0 {
            get_small_correction_lookup_table::<{ 1 << PRECISION }>(number_of_zero_registers)
        } else if raw_estimate >= Self::INTERMEDIATE_RANGE_CORRECTION_THRESHOLD {
            -Self::TWO_32 * (-raw_estimate / Self::TWO_32).ln_1p()
        } else {
            raw_estimate
        }
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = u8> + '_ {
        debug_assert_eq!(
            self.registers.len(),
            ceil(1 << PRECISION, NUMBER_OF_REGISTERS_IN_WORD)
        );

        self.registers
            .iter()
            .copied()
            .flat_map(|six_registers| {
                (0..NUMBER_OF_REGISTERS_IN_WORD).map(move |i| {
                    (six_registers >> i * NUMBER_OF_BITS_PER_REGISTER & LOWER_REGISTER_MASK) as u8
                })
            })
            .take(Self::NUMBER_OF_REGISTERS)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        debug_assert_eq!(Self::NUMBER_OF_REGISTERS, self.iter().count());
        Self::NUMBER_OF_REGISTERS
    }

    #[inline(always)]
    pub fn get_number_of_bits(&self) -> u8 {
        PRECISION as u8
    }

    #[inline(always)]
    pub fn get_number_of_zero_registers(&self) -> usize {
        self.iter()
            .filter(|&register_value| register_value == 0)
            .count()
    }

    #[inline(always)]
    pub fn get_number_of_non_zero_registers(&self) -> usize {
        self.iter()
            .filter(|&register_value| register_value > 0)
            .count()
    }

    #[inline(always)]
    pub fn get_registers(&self) -> [u8; 1 << PRECISION] {
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
    /// let mut hll = HyperLogLog::<PRECISION>::new();
    ///
    /// hll.insert("Hello");
    /// hll.insert("World");
    ///
    /// assert!(hll.count() >= 2.0);
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
        let register_position_in_array = index / NUMBER_OF_REGISTERS_IN_WORD;

        debug_assert!(
            register_position_in_array < self.registers.len(),
            concat!(
                "The register_position_in_array {} must be less than the number of words {}. ",
                "You have obtained this values starting from the index {} and the word size {}."
            ),
            register_position_in_array,
            self.registers.len(),
            index,
            NUMBER_OF_REGISTERS_IN_WORD
        );

        // Calculate the position of the register within the 32-bit word containing it.
        let register_position_in_u32 = index % NUMBER_OF_REGISTERS_IN_WORD;

        // Extract the current value of the register at `index`.
        let register_value = (self.registers[register_position_in_array]
            >> (register_position_in_u32 * NUMBER_OF_BITS_PER_REGISTER))
            & LOWER_REGISTER_MASK;

        // If `number_of_zeros` is greater than the current number_of_zeros, update the register.
        if number_of_zeros > register_value {
            let shifted_zeros =
                number_of_zeros << (register_position_in_u32 * NUMBER_OF_BITS_PER_REGISTER);
            if register_value == 0 {
                // If the current number_of_zeros is zero, decrement `zeros` and set the register to `number_of_zeros`.
                self.registers[register_position_in_array] |= shifted_zeros;
            } else {
                // Otherwise, update the register using a bit mask.
                let mask =
                    LOWER_REGISTER_MASK << (register_position_in_u32 * NUMBER_OF_BITS_PER_REGISTER);
                self.registers[register_position_in_array] =
                    (self.registers[register_position_in_array] & !mask) | shifted_zeros;
            }
        }
    }

    #[inline(always)]
    /// Computes union between HLL counters.
    pub fn union(&mut self, rhs: &Self) {
        for (left_word, right_word) in self.registers.iter_mut().zip(rhs.registers.iter().copied())
        {
            let left_word_copy: u32 = *left_word;
            let left_register_0 = left_word_copy & LOWER_REGISTER_MASK;
            let left_register_1 =
                (left_word_copy >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
            let left_register_2 =
                (left_word_copy >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
            let left_register_3 =
                (left_word_copy >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
            let left_register_4 =
                (left_word_copy >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;

            let right_register_0 = right_word & LOWER_REGISTER_MASK;
            let right_register_1 =
                (right_word >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
            let right_register_2 =
                (right_word >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
            let right_register_3 =
                (right_word >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
            let right_register_4 =
                (right_word >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;

            *left_word = left_register_0.max(right_register_0)
                | (left_register_1.max(right_register_1) << NUMBER_OF_BITS_PER_REGISTER)
                | (left_register_2.max(right_register_2) << 2 * NUMBER_OF_BITS_PER_REGISTER)
                | (left_register_3.max(right_register_3) << 3 * NUMBER_OF_BITS_PER_REGISTER)
                | (left_register_4.max(right_register_4) << 4 * NUMBER_OF_BITS_PER_REGISTER);
        }
    }
}
