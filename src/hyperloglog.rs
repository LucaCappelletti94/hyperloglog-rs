use crate::array_default::{ArrayDefault, ArrayIter};
use crate::precisions::{Precision, WordType};
use crate::prelude::HyperLogLogTrait;
use crate::primitive::Primitive;
use core::hash::Hash;

#[derive(Clone, Debug, Copy)]
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
/// let mut hll = HyperLogLog::<Precision12, 6>::default();
/// hll.insert(&"apple");
/// hll.insert(&"banana");
/// hll.insert(&"cherry");
///
/// let estimated_cardinality = hll.estimate_cardinality();
/// assert!(estimated_cardinality >= 3.0_f32 * 0.9 && estimated_cardinality <= 3.0_f32 * 1.1);
/// ```
///
/// # Citations
///
/// This implementation is based on the following papers:
///
/// * Flajolet, Philippe, et al. "HyperLogLog: the analysis of a near-optimal cardinality estimation algorithm." DMTCS Proceedings 1 (2007): 127-146.
/// * Heule, Stefan, Marc Nunkesser, and Alexander Hall. "HyperLogLog in practice: algorithmic engineering of a state of the art cardinality estimation algorithm." Proceedings of the 16th International Conference on Extending Database Technology. 2013.
pub struct HyperLogLog<P: Precision + WordType<BITS>, const BITS: usize> {
    pub(crate) words: P::Words,
    pub(crate) number_of_zero_registers: P::NumberOfZeros,
}

impl<P: Precision + WordType<BITS>, const BITS: usize> HyperLogLog<P, BITS> {
    /// Create a new HyperLogLog counter.
    fn new() -> Self {
        Self {
            words: P::Words::default_array(),
            number_of_zero_registers: P::NumberOfZeros::reverse(P::NUMBER_OF_REGISTERS),
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
    pub fn from_words(words: &P::Words) -> Self {
        let number_of_zero_registers =
            P::NumberOfZeros::reverse(words.iter_elements().fold(
                0,
                |number_of_zero_registers, word| {
                    // We check that in all words the PADDING_BITS_MASK
                    // is all zeros.
                    debug_assert!(
                        word & Self::PADDING_BITS_MASK == 0,
                        concat!(
                            "The padding bits of the word {} must be all zeros. ",
                            "We have obtained {} instead."
                        ),
                        word,
                        word & Self::PADDING_BITS_MASK
                    );

                    (0..Self::NUMBER_OF_REGISTERS_IN_WORD).fold(
                        number_of_zero_registers,
                        |number_of_zero_registers, i| {
                            let register = (word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                            number_of_zero_registers + (register == 0) as usize
                        },
                    )
                },
            )) - P::NumberOfZeros::reverse(Self::get_number_of_padding_registers());

        // We check that the values in the last word are masked
        // according to the LAST_WORD_PADDING_BITS_MASK.
        debug_assert!(
            words.last().unwrap() & Self::LAST_WORD_PADDING_BITS_MASK == 0,
            concat!(
                "The padding bits of the last word {} must be all zeros. ",
                "We have obtained {} instead. The last word padding bits mask is, ",
                "when represented in binary, {:#034b}."
            ),
            words.last().unwrap(),
            words.last().unwrap() & Self::LAST_WORD_PADDING_BITS_MASK,
            Self::LAST_WORD_PADDING_BITS_MASK
        );

        Self {
            words: *words,
            number_of_zero_registers,
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
    pub fn from_registers(registers: &[u32]) -> Self {
        debug_assert!(
            registers.len() == P::NUMBER_OF_REGISTERS,
            "We expect {} registers, but got {}",
            P::NUMBER_OF_REGISTERS,
            registers.len()
        );
        let mut words = P::Words::default_array();
        let number_of_zero_registers = P::NumberOfZeros::reverse(
            words
                .iter_elements_mut()
                .zip(registers.chunks(Self::NUMBER_OF_REGISTERS_IN_WORD))
                .fold(0, |number_of_zero_registers, (word, word_registers)| {
                    word_registers.iter().copied().enumerate().fold(
                        number_of_zero_registers,
                        |number_of_zero_registers, (i, register)| {
                            debug_assert!(
                                register <= Self::LOWER_REGISTER_MASK,
                                "Register value {} is too large for the given number of bits {}",
                                register,
                                BITS
                            );
                            *word |= register << (i * BITS);
                            number_of_zero_registers + (register == 0) as usize
                        },
                    )
                }),
        );

        Self {
            words,
            number_of_zero_registers,
        }
    }

    #[inline(always)]
    /// Adds an element to the HyperLogLog counter, and returns whether the counter has changed.
    ///
    /// # Arguments
    /// * `rhs` - The element to add.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll = HyperLogLog::<Precision10, 6>::default();
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
            // In the case of BITS = 5, we have registers of
            // 5 bits each. The maximal value that can be represented
            // in a register is 31. As such, we need to add 1 << (65 - 32)
            // to the hash. Similarly, in the case of BITS = 4, we have
            // registers of 4 bits each. The maximal value that can be
            // represented in a register is 15.
            hash |= 1 << (65 - (1 << BITS))
        } // else {
          //     // Otherwise, with registers from 6 bits upwards we can
          //     // represent a value of 64 or larger. Since we are using
          //     // an hash function of 64 bits, the maximal number of
          //     //
          //     // 1 << (P::EXPONENT - 1)
          // };

        // Count leading zeros.
        let number_of_zeros: u32 = 1 + hash.leading_zeros();

        // We add a debug assertion to make sure that the number of zeros
        // we have obtained is not larger than the maximal value that can
        // be represented in a register with BITS bits.
        debug_assert!(
            number_of_zeros < 1 << BITS,
            concat!("The number of zeros {} must be less than or equal to {}. ",),
            number_of_zeros,
            1 << BITS
        );

        // Calculate the position of the register in the internal buffer array.
        let word_position = index / Self::NUMBER_OF_REGISTERS_IN_WORD;
        let register_position = index - word_position * Self::NUMBER_OF_REGISTERS_IN_WORD;

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
        let register_value: u32 =
            (self.words[word_position] >> (register_position * BITS)) & Self::LOWER_REGISTER_MASK;

        // Otherwise, update the register using a bit mask.
        if number_of_zeros > register_value {
            self.words[word_position] &= !(Self::LOWER_REGISTER_MASK << (register_position * BITS));
            self.words[word_position] |= number_of_zeros << (register_position * BITS);
            self.number_of_zero_registers -=
                P::NumberOfZeros::reverse((register_value == 0) as usize);

            // We check that the value we have written to the register is correct.
            debug_assert!(
                self.words[word_position] >> (register_position * BITS) & Self::LOWER_REGISTER_MASK
                    == number_of_zeros,
                concat!(
                    "The value of the register at position {} must be {}. ",
                    "We have obtained {} instead. ",
                    "The current value of the word is {}."
                ),
                index,
                number_of_zeros,
                self.words[word_position] >> (register_position * BITS) & Self::LOWER_REGISTER_MASK,
                self.words[word_position]
            );

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

impl<P: Precision + WordType<BITS>, const BITS: usize> Eq for HyperLogLog<P, BITS> {
    fn assert_receiver_is_total_eq(&self) {
        // This is a no-op because we know that `Self` is `Eq`.
    }
}

/// Implements PartialEq for HyperLogLog.
///
/// # Implementative details
/// Two HyperLogLog counters are considered equal if they have the same words.
///
/// # Examples
///
/// ```
/// # use hyperloglog_rs::prelude::*;
///
/// let mut hll1 = HyperLogLog::<Precision14, 5>::default();
/// hll1.insert(&2);
///
/// let mut hll2 = HyperLogLog::<Precision14, 5>::default();
/// hll2.insert(&2);
/// hll2.insert(&3);
///
/// assert_ne!(hll1, hll2);
///
/// hll1 |= hll2;
///
/// assert_eq!(hll1, hll2);
/// ```
impl<P: Precision + WordType<BITS>, const BITS: usize> PartialEq for HyperLogLog<P, BITS> {
    /// Returns whether the two HyperLogLog counters are equal.
    fn eq(&self, other: &Self) -> bool {
        self.words == other.words
    }
}

/// Implements the Default trait for HyperLogLog.
///
/// HyperLogLog is a probabilistic cardinality estimator that uses a fixed
/// amount of memory to estimate the number of distinct elements in a set.
///
/// # Examples
///
/// ```rust
/// # use hyperloglog_rs::prelude::*;
///
/// let hll: HyperLogLog<Precision10, 6> = Default::default();
/// assert_eq!(hll.len(), 1024);
/// ```
impl<P: Precision + WordType<BITS>, const BITS: usize> Default for HyperLogLog<P, BITS> {
    /// Returns a new HyperLogLog instance with default configuration settings.
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Precision + WordType<BITS>, const BITS: usize, T: Hash> From<T> for HyperLogLog<P, BITS> {
    /// Create a new HyperLogLog counter from a value.
    ///
    /// This method creates a new empty HyperLogLog counter and inserts the hash
    /// of the given value into it. The value can be any type that implements
    /// the `Hash` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll = HyperLogLog::<Precision14, 5>::from("test");
    ///
    /// assert!(!hll.is_empty());
    /// assert!(hll.may_contain(&"test"));
    /// assert!(hll.estimate_cardinality() >= 0.9_f32);
    /// ```
    fn from(value: T) -> Self {
        let mut hll = Self::new();
        hll.insert(value);
        hll
    }
}

impl<P: Precision + WordType<BITS>, const BITS: usize> HyperLogLogTrait<P, BITS>
    for HyperLogLog<P, BITS>
{
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
    /// let mut hll = HyperLogLog::<Precision14, 5>::default();
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
        self.number_of_zero_registers.convert()
    }

    #[inline(always)]
    /// Returns the array of words of the HyperLogLog counter.
    fn get_words(&self) -> &P::Words {
        &self.words
    }
}

impl<P: Precision + WordType<BITS>, const BITS: usize, A: Hash> core::iter::FromIterator<A>
    for HyperLogLog<P, BITS>
{
    #[inline(always)]
    /// Creates a new HyperLogLog counter and adds all elements from an iterator to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    /// let hll: HyperLogLog<Precision12, 5> = data.iter().collect();
    /// assert!(
    ///     hll.estimate_cardinality() > 0.9 * data.len() as f32,
    ///     concat!("The estimate is too low, expected ", "at least {}, got {}",),
    ///     0.9 * data.len() as f32,
    ///     hll.estimate_cardinality()
    /// );
    /// assert!(
    ///     hll.estimate_cardinality() < 1.1 * data.len() as f32,
    ///     concat!("The estimate is too high, expected ", "at most {}, got {}",),
    ///     1.1 * data.len() as f32,
    ///     hll.estimate_cardinality()
    /// );
    /// ```
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut hll = Self::new();
        for item in iter {
            hll.insert(item);
        }
        hll
    }
}
