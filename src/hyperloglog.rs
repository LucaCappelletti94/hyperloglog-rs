use siphasher::sip::SipHasher13;

use crate::array_default::{ArrayDefault, ArrayIter};
use crate::hasher_method::HasherMethod;
use crate::precisions::{Precision, WordType};
use crate::prelude::HyperLogLogTrait;
use crate::prelude::*;
use crate::primitive::Primitive;
use core::hash::Hash;
use std::marker::PhantomData;

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
/// let mut hll = HyperLogLog::<Precision12, 6>::new();
/// hll.insert(&"apple");
/// hll.insert(&"banana");
/// hll.insert(&"cherry");
///
/// let estimated_cardinality = hll.estimate_cardinality();
/// assert!(estimated_cardinality >= 3.0_f32 * 0.9 &&
///         estimated_cardinality <= 3.0_f32 * 1.1);
/// ```
///
/// # Citations
///
/// This implementation is based on the following papers:
///
/// * Flajolet, Philippe, et al. "HyperLogLog: the analysis of a near-optimal cardinality estimation algorithm." DMTCS Proceedings 1 (2007): 127-146.
/// * Heule, Stefan, Marc Nunkesser, and Alexander Hall. "HyperLogLog in practice: algorithmic engineering of a state of the art cardinality estimation algorithm." Proceedings of the 16th International Conference on Extending Database Technology. 2013.
///
pub struct HyperLogLog<
    PRECISION: Precision + WordType<BITS>,
    const BITS: usize,
    M: HasherMethod = SipHasher13,
> {
    pub(crate) words: PRECISION::Words,
    pub(crate) number_of_zero_registers: PRECISION::NumberOfZeros,
    pub(crate) upper_bound: usize,
    pub(crate) _phantom: PhantomData<M>,
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize, M: HasherMethod>
    From<HyperLogLogWithMulteplicities<PRECISION, BITS, M>> for HyperLogLog<PRECISION, BITS, M>
{
    fn from(hll: HyperLogLogWithMulteplicities<PRECISION, BITS, M>) -> Self {
        Self::from_words(hll.get_words())
    }
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize, M: HasherMethod> Eq
    for HyperLogLog<PRECISION, BITS, M>
{
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
/// let mut hll1 = HyperLogLog::<Precision14, 5>::new();
/// hll1.insert(&2);
///
/// let mut hll2 = HyperLogLog::<Precision14, 5>::new();
/// hll2.insert(&2);
/// hll2.insert(&3);
///
/// assert_ne!(hll1, hll2);
///
/// hll1 |= hll2;
///
/// assert_eq!(hll1, hll2);
/// ```
impl<PRECISION: Precision + WordType<BITS>, const BITS: usize, M: HasherMethod> PartialEq
    for HyperLogLog<PRECISION, BITS, M>
{
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
impl<PRECISION: Precision + WordType<BITS>, const BITS: usize, M: HasherMethod> Default
    for HyperLogLog<PRECISION, BITS, M>
{
    /// Returns a new HyperLogLog instance with default configuration settings.
    fn default() -> Self {
        Self::new()
    }
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize, T: Hash, M: HasherMethod> From<T>
    for HyperLogLog<PRECISION, BITS, M>
{
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
    /// let hll = HyperLogLog::<Precision14, 5>::from("test");
    ///
    /// assert!(hll.estimate_cardinality() >=  1.0_f32);
    /// assert!(!hll.is_empty());
    /// assert!(hll.may_contain(&"test"));
    /// ```
    fn from(value: T) -> Self {
        let mut hll = Self::new();
        hll.insert(value);
        hll
    }
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize, M: HasherMethod>
    HyperLogLogTrait<PRECISION, BITS, M> for HyperLogLog<PRECISION, BITS, M>
{
    /// Create a new HyperLogLog counter.
    fn new() -> Self {
        Self {
            words: PRECISION::Words::default_array(),
            number_of_zero_registers: PRECISION::NumberOfZeros::reverse(
                PRECISION::NUMBER_OF_REGISTERS,
            ),
            upper_bound: 0,
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
        let number_of_zero_registers = PRECISION::NumberOfZeros::reverse(
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
        let mut hll = Self {
            words,
            number_of_zero_registers,
            upper_bound: usize::MAX,
            _phantom: PhantomData,
        };
        hll.upper_bound = hll.estimate_cardinality() as usize;
        hll
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
        let number_of_zero_registers = PRECISION::NumberOfZeros::reverse(
            words
                .iter_elements()
                .fold(0, |number_of_zero_registers, word| {
                    (0..Self::NUMBER_OF_REGISTERS_IN_WORD).fold(
                        number_of_zero_registers,
                        |number_of_zero_registers, i| {
                            let register = (word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                            number_of_zero_registers + (register == 0) as usize
                        },
                    )
                }),
        );

        let mut hll = Self {
            words: *words,
            number_of_zero_registers,
            upper_bound: usize::MAX,
            _phantom: PhantomData,
        };
        hll.upper_bound = hll.estimate_cardinality() as usize;
        hll
    }

    fn get_upper_bound(&self) -> f32 {
        self.upper_bound as f32
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
        self.number_of_zero_registers.convert()
    }

    #[inline(always)]
    /// Returns the array of words of the HyperLogLog counter.
    fn get_words(&self) -> &PRECISION::Words {
        &self.words
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
            word_position < self.get_words().len(),
            concat!(
                "The word_position {} must be less than the number of words {}. ",
                "You have obtained this values starting from the index {} and the number of registers in word {}. ",
                "We currently have {} registers. Currently using precision {} and number of bits {}."
            ),
            word_position,
            self.get_words().len(),
            index,
            Self::NUMBER_OF_REGISTERS_IN_WORD,
            PRECISION::NUMBER_OF_REGISTERS,
            PRECISION::EXPONENT,
            BITS
        );

        // Extract the current value of the register at `index`.
        let register_value: u32 = (self.get_words()[word_position]
            >> (register_position_in_u32 * BITS))
            & Self::LOWER_REGISTER_MASK;

        self.upper_bound += 1;

        // Otherwise, update the register using a bit mask.
        if number_of_zeros > register_value {
            self.words[word_position] &=
                !(Self::LOWER_REGISTER_MASK << (register_position_in_u32 * BITS));
            self.words[word_position] |=
                number_of_zeros << (register_position_in_u32 * BITS);
            self.number_of_zero_registers -=
                PRECISION::NumberOfZeros::reverse((register_value == 0) as usize);

            // We check that the word we have edited maintains that the padding bits are all zeros
            // and have not been manipulated in any way. If these bits were manipulated, it would mean
            // that we have a bug in the code.
            debug_assert!(
                self.get_words()[word_position] & Self::PADDING_BITS_MASK == 0,
                concat!(
                    "The padding bits of the word {} must be all zeros. ",
                    "We have obtained {} instead."
                ),
                self.get_words()[word_position],
                self.get_words()[word_position] & Self::PADDING_BITS_MASK
            );
        }
    }
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize, A: Hash, M: HasherMethod>
    core::iter::FromIterator<A> for HyperLogLog<PRECISION, BITS, M>
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
        let mut hll = Self::new();
        for item in iter {
            hll.insert(item);
        }
        hll
    }
}
