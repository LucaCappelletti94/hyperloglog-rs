use crate::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign};

#[derive(Clone, Debug)]
/// HyperLogLog is a probabilistic algorithm for estimating the number of distinct elements in a set.
/// It uses a small amount of memory to produce an approximate count with a guaranteed error rate.
pub struct HyperLogLog<const N: usize> {
    registers: [u32; N],
}

impl<const N: usize> HyperLogLog<N> {
    pub const NUMBER_OF_REGISTERS: usize = N * 6;
    pub const NUMBER_OF_REGISTERS_SQUARED: f32 =
        (Self::NUMBER_OF_REGISTERS * Self::NUMBER_OF_REGISTERS) as f32;
    pub const SMALL_RANGE_CORRECTION_THRESHOLD: f32 = 2.5_f32 * (Self::NUMBER_OF_REGISTERS as f32);
    pub const MAX_U32_FLOAT: f32 = u32::MAX as f32;
    pub const INTERMEDIATE_RANGE_CORRECTION_THRESHOLD: f32 = Self::MAX_U32_FLOAT / 30.0_f32;

    // A mask to get the lower register (from LSB).
    const MASK: u32 = 0b111111;
    pub const NUMBER_OF_BITS: usize = get_number_of_bits::<N>();
    pub const NUMBER_OF_REGISTERS_IN_WORD: usize = 5;
    pub const NUMBER_OF_BITS_PER_REGISTER: usize = 6;
    pub const ALPHA_32: f32 = 0.697_f32;

    /// Create a new HyperLogLog counter.
    pub fn new() -> Self {
        Self { registers: [0; N] }
    }

    #[inline(always)]
    /// Computes the estimate of the cardinality of the set represented by the HyperLogLog data structure.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog::prelude::*;
    ///
    /// const N: usize = 20;
    /// const NUMBER_OF_ELEMENTS: usize = 1_000_000;
    ///
    /// let mut hll = HyperLogLog::<N>::new();
    ///
    /// for i in 0..NUMBER_OF_ELEMENTS {
    ///     hll += i;
    /// }
    ///
    /// assert!(
    ///     hll.count() >= NUMBER_OF_ELEMENTS as f32,
    ///     "Expected >= {}, got {}. The registers are: {:?}.",
    ///     NUMBER_OF_ELEMENTS,
    ///     hll.count(),
    ///     hll.get_registers()
    /// );
    ///
    pub fn count(&self) -> f32 {
        // Initialize the total estimate to 0.0
        let mut total = 0.0;
        let mut number_of_zero_registers: usize = 0;

        // Iterate over the registers in the data structure
        for mut six_registers in self.registers.iter().copied() {
            // Extract the register value and update the total estimate
            for _local_register_number in 0..Self::NUMBER_OF_REGISTERS_IN_WORD {
                let register_value: u32 = six_registers & Self::MASK;
                number_of_zero_registers += (register_value == 0) as usize;
                total += 2.0_f32.powf(-(register_value as f32));
                six_registers >>= Self::NUMBER_OF_BITS_PER_REGISTER;
            }
        }

        // Apply the final scaling factor to obtain the estimate of the cardinality
        total = Self::ALPHA_32 * Self::NUMBER_OF_REGISTERS_SQUARED * total.recip();

        if total <= Self::SMALL_RANGE_CORRECTION_THRESHOLD {
            if number_of_zero_registers > 0 {
                get_small_correction_lookup_table::<N>(number_of_zero_registers)
            } else {
                total
            }
        } else if total <= Self::INTERMEDIATE_RANGE_CORRECTION_THRESHOLD {
            total
        } else {
            -Self::MAX_U32_FLOAT * (-total / Self::MAX_U32_FLOAT).ln_1p()
        }
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = u8> + '_ {
        self.registers
            .iter()
            .copied()
            .flat_map(|mut six_registers| {
                (0..Self::NUMBER_OF_REGISTERS_IN_WORD).map(move |_| {
                    let register_value: u8 = (six_registers & Self::MASK) as u8;
                    six_registers >>= Self::NUMBER_OF_BITS_PER_REGISTER;
                    register_value
                })
            })
    }

    #[inline(always)]
    pub fn get_registers(&self) -> [u8; 6 * N] {
        let mut array = [0; 6 * N];
        self.iter()
            .zip(array.iter_mut())
            .for_each(|(value, target)| {
                *target = value;
            });
        array
    }
}

impl<const N: usize, T> Add<T> for HyperLogLog<N>
where
    T: Hash,
{
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        let mut copy = self.clone();
        copy += rhs;
        copy
    }
}

impl<const N: usize, T> AddAssign<T> for HyperLogLog<N>
where
    T: Hash,
{
    #[inline(always)]
    /// Adds an element to the HyperLogLog counter.
    ///
    /// # Arguments
    /// * `rhs` - The element to add.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog::prelude::*;
    ///
    /// const N: usize = 16;
    ///
    /// let mut hll = HyperLogLog::<N>::new();
    ///
    /// hll += "Hello";
    /// hll += "World";
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
    fn add_assign(&mut self, rhs: T) {
        // Create a new hasher.
        let mut hasher = DefaultHasher::new();
        // Calculate the hash.
        rhs.hash(&mut hasher);
        // Drops the higher 32 bits.
        let mut hash: u32 = hasher.finish() as u32;

        // Calculate the register's index.
        let index: usize = (hash >> (32 - Self::NUMBER_OF_BITS)) as usize;
        debug_assert!(
            index < Self::NUMBER_OF_REGISTERS,
            "The index {} must be less than the number of registers {}.",
            index,
            Self::NUMBER_OF_REGISTERS
        );

        // Shift left the bits of the index.
        hash = (hash << Self::NUMBER_OF_BITS) | (1 << (Self::NUMBER_OF_BITS - 1));

        // Count leading zeros.
        let number_of_zeros: u32 = 1 + hash.leading_zeros();

        // Calculate the position of the register in the internal buffer array.
        let register_position_in_array = index / Self::NUMBER_OF_REGISTERS_IN_WORD;
        // Calculate the position of the register within the 32-bit word containing it.
        let register_position_in_u32 = index % Self::NUMBER_OF_REGISTERS_IN_WORD;

        // Extract the current value of the register at `index`.
        let register_value = (self.registers[register_position_in_array]
            >> (register_position_in_u32 * Self::NUMBER_OF_BITS_PER_REGISTER))
            & Self::MASK;

        // If `number_of_zeros` is greater than the current number_of_zeros, update the register.
        if number_of_zeros > register_value {
            let shifted_zeros =
                number_of_zeros << (register_position_in_u32 * Self::NUMBER_OF_BITS_PER_REGISTER);
            if register_value == 0 {
                // If the current number_of_zeros is zero, decrement `zeros` and set the register to `number_of_zeros`.
                self.registers[register_position_in_array] |= shifted_zeros;
            } else {
                // Otherwise, update the register using a bit mask.
                let mask =
                    Self::MASK << (register_position_in_u32 * Self::NUMBER_OF_BITS_PER_REGISTER);
                self.registers[register_position_in_array] =
                    (self.registers[register_position_in_array] & !mask) | shifted_zeros;
            }
        }
    }
}
