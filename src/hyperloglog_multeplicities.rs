use crate::array_default::{ArrayDefault, ArrayIter};
use crate::precisions::{Precision, WordType};
use crate::prelude::*;
use siphasher::sip::SipHasher13;
use std::{hash::Hash, marker::PhantomData};

pub struct HyperLogLogWithMulteplicities<
    PRECISION: Precision + WordType<BITS>,
    const BITS: usize,
    M: HasherMethod = SipHasher13,
> {
    pub(crate) words: PRECISION::Words,
    pub(crate) multeplicities: PRECISION::RegisterMultiplicities,
    pub(crate) _phantom: PhantomData<M>,
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize, M: HasherMethod>
    From<HyperLogLog<PRECISION, BITS, M>> for HyperLogLogWithMulteplicities<PRECISION, BITS, M>
{
    fn from(hll: HyperLogLog<PRECISION, BITS, M>) -> Self {
        Self::from_words(hll.get_words())
    }
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize, M: HasherMethod>
    HyperLogLogTrait<PRECISION, BITS, M> for HyperLogLogWithMulteplicities<PRECISION, BITS, M>
{
    fn new() -> Self {
        let mut multeplicities = PRECISION::RegisterMultiplicities::default_array();

        multeplicities[0] = PRECISION::NumberOfZeros::reverse(PRECISION::NUMBER_OF_REGISTERS);

        Self {
            words: PRECISION::Words::default_array(),
            multeplicities: PRECISION::RegisterMultiplicities::default_array(),
            _phantom: PhantomData,
        }
    }

    /// Create a new HyperLogLog counter from an array of registers.
    ///
    /// # Arguments
    ///
    /// * `registers` - An array of u32 registers to use for the HyperLogLog counter.
    ///
    /// # Returns
    ///
    /// A new HyperLogLog counter initialized with the given registers.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let registers = [0_u32; 1 << 4];
    /// let hll = HyperLogLog::<Precision4, 6>::from_registers(&registers);
    /// assert_eq!(hll.len(), 1 << 4);
    /// ```
    fn from_registers(registers: &[u32]) -> Self {
        debug_assert!(
            registers.len() == PRECISION::NUMBER_OF_REGISTERS,
            "We expect {} registers, but got {}",
            PRECISION::NUMBER_OF_REGISTERS,
            registers.len()
        );
        let mut words = PRECISION::Words::default_array();
        let mut multeplicities = PRECISION::RegisterMultiplicities::default_array();
        words
            .iter_elements_mut()
            .zip(registers.chunks(Self::NUMBER_OF_REGISTERS_IN_WORD))
            .for_each(|(word, word_registers)| {
                for (i, register) in word_registers.iter().copied().enumerate() {
                    debug_assert!(
                        register <= Self::LOWER_REGISTER_MASK,
                        "Register value {} is too large for the given number of bits {}",
                        register,
                        BITS
                    );
                    multeplicities[register as usize] += PRECISION::NumberOfZeros::ONE;
                    *word |= register << (i * BITS);
                }
            });
        Self {
            words,
            multeplicities,
            _phantom: PhantomData,
        }
    }

    /// Create a new HyperLogLog counter from an array of words.
    ///
    /// # Arguments
    /// * `words` - An array of u64 words to use for the HyperLogLog counter.
    ///
    /// # Returns
    /// A new HyperLogLog counter initialized with the given words.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let words = [0_u32; 4];
    /// let hll = HyperLogLog::<Precision4, 6>::from_words(&words);
    /// assert_eq!(hll.len(), 16);
    /// ```
    fn from_words(words: &PRECISION::Words) -> Self {
        let mut multeplicities = PRECISION::RegisterMultiplicities::default_array();

        words.iter_elements().for_each(|word| {
            (0..Self::NUMBER_OF_REGISTERS_IN_WORD).for_each(|i| {
                let register = (word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                multeplicities[register as usize] += PRECISION::NumberOfZeros::ONE;
            });
        });

        Self {
            words: *words,
            multeplicities,
            _phantom: PhantomData,
        }
    }

    #[inline(always)]
    fn estimate_cardinality(&self) -> f32 {
        if self.get_number_of_zero_registers() > 0 {
            let low_range_correction =
                PRECISION::SMALL_CORRECTIONS[self.get_number_of_zero_registers() - 1];
            if low_range_correction <= Self::LINEAR_COUNT_THRESHOLD {
                return low_range_correction;
            }
        }

        let mut raw_estimate = 0.0;

        for (current_register, multeplicity) in self.multeplicities.iter_elements().enumerate() {
            let two_to_minus_register: i32 = (127 - current_register as i32) << 23;
            raw_estimate += (multeplicity.convert() as f32)
                * f32::from_le_bytes(two_to_minus_register.to_le_bytes());
        }

        self.adjust_estimate(raw_estimate)
    }

    /// Returns a reference to the words vector.
    fn get_words(&self) -> &PRECISION::Words {
        &self.words
    }

    #[inline(always)]
    /// Returns the number of bits used to represent the hashed value of an element.
    fn get_words_mut(&mut self) -> &mut PRECISION::Words {
        &mut self.words
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
    /// let mut hll = HyperLogLog::<Precision14, 5>::new();
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
    fn get_number_of_zero_registers(&self) -> usize {
        self.multeplicities[0].convert()
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
    /// let mut hll = HyperLogLog::<Precision10, 6>::new();
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
    fn insert<T: Hash>(&mut self, rhs: T) {
        let (mut hash, index) = self.get_hash_and_index::<T>(&rhs);

        // We need to add ones to the hash to make sure that the
        // the number of zeros we obtain afterwards is never higher
        // than the maximal value that may be represented in a register
        // with BITS bits.
        if BITS < 6 {
            hash |= 1 << (64 - ((1 << BITS) - 1));
        } else {
            hash |= 1 << (PRECISION::EXPONENT - 1);
        }

        // Count leading zeros.
        let number_of_zeros: u32 = 1 + hash.leading_zeros();

        debug_assert!(
            number_of_zeros < (1 << BITS),
            concat!(
                "The number of leading zeros {} must be less than the number of bits {}. ",
                "You have obtained this values starting from the hash {:064b} and the precision {}."
            ),
            number_of_zeros,
            1 << BITS,
            hash,
            PRECISION::EXPONENT
        );

        // Calculate the position of the register in the internal buffer array.
        let word_position = index / Self::NUMBER_OF_REGISTERS_IN_WORD;
        let register_position_in_u32 = index - word_position * Self::NUMBER_OF_REGISTERS_IN_WORD;

        debug_assert!(
            word_position < self.words.len(),
            concat!(
                "The word_position {} must be less than the number of words {}. ",
                "You have obtained this values starting from the index {} and the number of registers in word {}. ",
                "We currently have {} registers. Currently using precision {} and number of bits {}."
            ),
            word_position,
            self.words.len(),
            index,
            Self::NUMBER_OF_REGISTERS_IN_WORD,
            PRECISION::NUMBER_OF_REGISTERS,
            PRECISION::EXPONENT,
            BITS
        );

        // Extract the current value of the register at `index`.
        let register_value: u32 = (self.words[word_position] >> (register_position_in_u32 * BITS))
            & Self::LOWER_REGISTER_MASK;

        // Otherwise, update the register using a bit mask.
        if number_of_zeros > register_value {
            self.multeplicities[register_value as usize] -= PRECISION::NumberOfZeros::ONE;
            self.multeplicities[number_of_zeros as usize] += PRECISION::NumberOfZeros::ONE;

            self.words[word_position] &=
                !(Self::LOWER_REGISTER_MASK << (register_position_in_u32 * BITS));
            self.words[word_position] |= number_of_zeros << (register_position_in_u32 * BITS);

            // We check that the word we have edited maintains that the padding bits are all zeros
            // and have not been manipulated in any way. If these bits were manipulated, it would mean
            // that we have a bug in the code.
            debug_assert!(
                self.words[word_position] & Self::PADDING_BITS_MASK == 0,
                concat!(
                    "The padding bits of the word {} must be all zeros. ",
                    "We have obtained {} instead."
                ),
                self.words[word_position],
                self.words[word_position] & Self::PADDING_BITS_MASK
            );
        }
    }
}
