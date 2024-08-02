use crate::array_default::{ArrayDefault, ArrayIter};
use crate::precisions::{Precision, WordType};
use crate::prelude::*;
use core::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A HyperLogLog counter with multiplicities.
///
/// # Implementation details
/// This struct differs from the traditional HyperLogLog counter in that it stores the multiplicities
/// of the registers. This allows us to speed up significantly the computation of the cardinality of
/// the counter, as we do not need to compute the harmonic mean of the registers but we can instead
/// use the multiplities instead, reducing by a large amount the sums we need to compute.
///
/// For instance, for a counter with 2^14 registers, we need to compute the harmonic mean of 2^14
/// registers, i.e. 16384 registers. With the multiplicities, we only need to compute the sum of the
/// multiplicities, which is much smaller, and at most equal to 52 when you use 6 bits per register.
///
/// That being said, when memory is an extreme concern, you may want to use the traditional HyperLogLog
/// as this struct contains the multiplicities vector, which in the example case we considered above
/// would be adding u16 * 52 = 104 bytes to the size of the counter.
///
/// Additionally, note that while one may expect to obtain better accuracy by executing less sums,
/// we do not observe any statistically significant difference in the accuracy of the counter when
/// using the multiplicities instead of the registers in our tests.
///
/// Note that this struct DOES NOT provide any other faster operation other than the estimation of the
/// cardinality of the counter. All other operations, such as the union of two counters, are fast as
/// they are implemented using the traditional HyperLogLog counter.
///
pub struct HyperLogLogWithMultiplicities<P: Precision + WordType<BITS>, const BITS: usize> {
    pub(crate) words: P::Words,
    pub(crate) multiplicities: P::RegisterMultiplicities,
}

impl<P: Precision + WordType<BITS>, const BITS: usize> HyperLogLogWithMultiplicities<P, BITS> {
    fn new() -> Self {
        let mut multiplicities = P::RegisterMultiplicities::default_array();

        multiplicities[0] = P::NumberOfZeros::reverse(P::NUMBER_OF_REGISTERS);

        Self {
            words: P::Words::default_array(),
            multiplicities,
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
    /// let hll = HyperLogLogWithMultiplicities::<Precision4, 6>::from_words(&words);
    /// assert_eq!(hll.len(), 16);
    /// ```
    pub fn from_words(words: &P::Words) -> Self {
        let mut multiplicities = P::RegisterMultiplicities::default_array();

        words.iter_elements().for_each(|word| {
            (0..Self::NUMBER_OF_REGISTERS_IN_WORD).for_each(|i| {
                let register = (word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                multiplicities[register as usize] += P::NumberOfZeros::ONE;
            });
        });

        multiplicities[0] -= P::NumberOfZeros::reverse(Self::get_number_of_padding_registers());

        Self {
            words: *words,
            multiplicities,
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
    /// let hll = HyperLogLogWithMultiplicities::<Precision4, 6>::from_registers(&registers);
    /// assert_eq!(hll.len(), 1 << 4);
    /// ```
    pub fn from_registers(registers: &[u32]) -> Self {
        debug_assert!(
            registers.len() == P::NUMBER_OF_REGISTERS,
            "We expect {} registers, but got {}",
            P::NUMBER_OF_REGISTERS,
            registers.len()
        );
        let mut words = P::Words::default_array();
        let mut multiplicities = P::RegisterMultiplicities::default_array();
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
                    multiplicities[register as usize] += P::NumberOfZeros::ONE;
                    *word |= register << (i * BITS);
                }
            });

        Self {
            words,
            multiplicities,
        }
    }

    #[inline(always)]
    /// Adds an element to the HyperLogLog counter , and returns whether the counter has changed.
    ///
    /// # Arguments
    /// * `rhs` - The element to add.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll = HyperLogLogWithMultiplicities::<Precision10, 6>::default();
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
    pub fn insert<T: Hash>(&mut self, rhs: T) -> bool {
        let (mut hash, index) = self.get_hash_and_index::<T>(&rhs);

        // We need to add ones to the hash to make sure that the
        // the number of zeros we obtain afterwards is never higher
        // than the maximal value that may be represented in a register
        // with BITS bits.
        if BITS < 6 {
            hash |= 1 << (64 - ((1 << BITS) - 1));
        } else {
            hash |= 1 << (P::EXPONENT - 1);
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
            P::EXPONENT
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
            P::NUMBER_OF_REGISTERS,
            P::EXPONENT,
            BITS
        );

        // Extract the current value of the register at `index`.
        let register_value: u32 = (self.words[word_position] >> (register_position_in_u32 * BITS))
            & Self::LOWER_REGISTER_MASK;

        // Otherwise, update the register using a bit mask.
        if number_of_zeros > register_value {
            debug_assert!(self.multiplicities[register_value as usize] > P::NumberOfZeros::ZERO,);

            self.multiplicities[register_value as usize] -= P::NumberOfZeros::ONE;
            self.multiplicities[number_of_zeros as usize] += P::NumberOfZeros::ONE;

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

            // We also check that if the word we have edites is the last word, then the padding bits
            // of the word must be all zeros.
            debug_assert!(
                index != P::NUMBER_OF_REGISTERS - 1
                    || self.words[word_position] & Self::LAST_WORD_PADDING_BITS_MASK == 0,
                concat!(
                    "The padding bits of the last word {} must be all zeros. ",
                    "We have obtained {} instead. The last word padding bits mask is, ",
                    "when represented in binary, {:#034b}.\n ",
                    "The word in binary is {:#034b}. ",
                    "The current case is using precision {} and bits {}. As such, ",
                    "we expect to have {} padding registers in the last word."
                ),
                self.words[word_position],
                self.words[word_position] & Self::LAST_WORD_PADDING_BITS_MASK,
                Self::LAST_WORD_PADDING_BITS_MASK,
                self.words[word_position],
                P::EXPONENT,
                BITS,
                Self::get_number_of_padding_registers()
            );

            true
        } else {
            false
        }
    }
}

impl<P: Precision + WordType<BITS>, const BITS: usize> Default
    for HyperLogLogWithMultiplicities<P, BITS>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Precision + WordType<BITS>, const BITS: usize> From<HyperLogLogWithMultiplicities<P, BITS>>
    for HyperLogLog<P, BITS>
{
    fn from(hll: HyperLogLogWithMultiplicities<P, BITS>) -> Self {
        Self::from_words(hll.get_words())
    }
}

impl<P: Precision + WordType<BITS>, const BITS: usize> From<HyperLogLog<P, BITS>>
    for HyperLogLogWithMultiplicities<P, BITS>
{
    fn from(hll: HyperLogLog<P, BITS>) -> Self {
        Self::from_words(hll.get_words())
    }
}

impl<P: Precision + WordType<BITS>, const BITS: usize> HyperLogLogTrait<P, BITS>
    for HyperLogLogWithMultiplicities<P, BITS>
{
    #[inline(always)]
    /// Returns the number of registers in the counter.
    ///
    /// # Implementation details
    /// This function is overriding the estimate_cardinality function of the HyperLogLogTrait trait
    /// as we can compute the cardinality of the counter using the multiplicities instead of the
    /// registers. This is much faster as we do not need to compute the harmonic mean of the registers.
    fn estimate_cardinality(&self) -> f32 {
        Self::estimate_cardinality_from_multiplicities(&self.multiplicities)
    }

    /// Returns a reference to the words vector.
    fn get_words(&self) -> &P::Words {
        &self.words
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
    /// let mut hll = HyperLogLogWithMultiplicities::<Precision14, 5>::default();
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
        self.multiplicities[0].convert()
    }
}

impl<P: Precision + WordType<BITS>, const BITS: usize, A: Hash> core::iter::FromIterator<A>
    for HyperLogLogWithMultiplicities<P, BITS>
{
    #[inline(always)]
    /// Creates a new HyperLogLogWithMultiplicities counter and adds all elements from an iterator to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    /// let hll: HyperLogLogWithMultiplicities<Precision12, 5> = data.iter().collect();
    /// assert!(
    ///     hll.estimate_cardinality() > 0.9 * data.len() as f32,
    ///     concat!(
    ///         "The estimate is too low, expected ",
    ///         "at least {}, got {}",
    ///     ),
    ///     0.9 * data.len() as f32,
    ///     hll.estimate_cardinality()
    /// );
    /// assert!(
    ///     hll.estimate_cardinality() < 1.1 * data.len() as f32,
    ///     concat!(
    ///     "The estimate is too high, expected ",
    ///     "at most {}, got {}",
    ///    ),
    ///     1.1 * data.len() as f32,
    ///     hll.estimate_cardinality()
    /// );
    /// ```
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut hll = Self::default();
        for item in iter {
            hll.insert(item);
        }
        hll
    }
}
