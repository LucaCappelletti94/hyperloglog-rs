use crate::array_default::{ArrayDefault, ArrayIter};
use crate::bias::BIAS_DATA;
use crate::estimated_union_cardinalities::EstimatedUnionCardinalities;
use crate::precisions::{Precision, WordType};
use crate::prelude::linear_counting_threshold;
use crate::primitive::Primitive;
use crate::raw_estimate_data::RAW_ESTIMATE_DATA;
use crate::utils::{ceil, get_alpha};
use crate::zeros::Zero;
use core::hash::{Hash, Hasher};
use siphasher::sip::SipHasher13;

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
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
pub struct HyperLogLog<PRECISION: Precision + WordType<BITS>, const BITS: usize> {
    pub(crate) words: PRECISION::Words,
    pub(crate) number_of_zero_register: PRECISION::NumberOfZeros,
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize, T: Hash> From<T>
    for HyperLogLog<PRECISION, BITS>
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
/// assert_eq!(hll.get_number_of_bits(), 6);
/// ```
impl<PRECISION: Precision + WordType<BITS>, const BITS: usize> Default for HyperLogLog<PRECISION, BITS> {
    /// Returns a new HyperLogLog instance with default configuration settings.
    fn default() -> Self {
        Self::new()
    }
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize> HyperLogLog<PRECISION, BITS> {
    /// The threshold value used in the small range correction of the HyperLogLog algorithm.
    pub const INTERMEDIATE_RANGE_CORRECTION_THRESHOLD: f32 =
        5.0_f32 * (PRECISION::NUMBER_OF_REGISTERS as f32);

    pub const LINEAR_COUNT_THRESHOLD: f32 = linear_counting_threshold(PRECISION::EXPONENT);

    /// The mask used to obtain the lower register bits in the HyperLogLog algorithm.
    pub const LOWER_REGISTER_MASK: u32 = (1 << BITS) - 1;

    /// The mask used to obtain the lower precision bits in the HyperLogLog algorithm.
    pub const LOWER_PRECISION_MASK: usize = PRECISION::NUMBER_OF_REGISTERS - 1;
    pub const NOT_LOWER_PRECISION_MASK: u64 = !Self::LOWER_PRECISION_MASK as u64;

    /// The mask representing the bits that are never used in the u32 word in the cases
    /// where the number of bits is not a divisor of 32, such as 5 or 6.
    /// We set the LEADING bits as the padding bits, the unused one, so the leftmost bits.
    pub const PADDING_BITS_MASK: u32 =
        !((1_u64 << (BITS * Self::NUMBER_OF_REGISTERS_IN_WORD)) - 1_u64) as u32;

    /// The mask used to obtain the upper precision bits in the HyperLogLog algorithm.
    pub const UPPER_PRECISION_MASK: usize =
        Self::LOWER_PRECISION_MASK << (64 - PRECISION::EXPONENT);

    /// The number of registers that can fit in a single 32-bit word in the HyperLogLog algorithm.
    pub const NUMBER_OF_REGISTERS_IN_WORD: usize = 32 / BITS;

    /// Create a new HyperLogLog counter.
    pub fn new() -> Self {
        let number_of_zero_register =
            PRECISION::NumberOfZeros::reverse(PRECISION::NUMBER_OF_REGISTERS);

        Self {
            words: PRECISION::Words::default_array(),
            number_of_zero_register,
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
        assert!(
            registers.len() == PRECISION::NUMBER_OF_REGISTERS,
            "We expect {} registers, but got {}",
            PRECISION::NUMBER_OF_REGISTERS,
            registers.len()
        );
        let mut words = PRECISION::Words::default_array();
        let number_of_zero_register = words
            .iter_elements_mut()
            .zip(registers.chunks(Self::NUMBER_OF_REGISTERS_IN_WORD))
            .fold(0, |mut number_of_zero_register, (word, word_registers)| {
                for (i, register) in word_registers.iter().copied().enumerate() {
                    assert!(
                        register <= Self::LOWER_REGISTER_MASK,
                        "Register value {} is too large for the given number of bits {}",
                        register,
                        BITS
                    );
                    number_of_zero_register += (register == 0) as usize;
                    *word |= register << (i * BITS);
                }
                number_of_zero_register
            });
        Self {
            words,
            number_of_zero_register: PRECISION::NumberOfZeros::reverse(number_of_zero_register),
        }
    }

    fn adjust_estimate(&self, mut raw_estimate: f32) -> f32 {
        // Apply the final scaling factor to obtain the estimate of the cardinality
        raw_estimate = get_alpha(PRECISION::NUMBER_OF_REGISTERS)
            * (PRECISION::NUMBER_OF_REGISTERS * PRECISION::NUMBER_OF_REGISTERS) as f32
            / raw_estimate;

        // Apply the small range correction factor if the raw estimate is below the threshold
        // and there are zero registers in the counter.
        if raw_estimate <= Self::INTERMEDIATE_RANGE_CORRECTION_THRESHOLD {
            // Get a reference to raw estimates/biases for precision.
            let biases = BIAS_DATA[PRECISION::EXPONENT - 4];
            let estimates = RAW_ESTIMATE_DATA[PRECISION::EXPONENT - 4];

            // Raw estimate is first/last in estimates. Return the first/last bias.
            if raw_estimate <= estimates[0] {
                return raw_estimate - biases[0];
            }

            if estimates[estimates.len() - 1] <= raw_estimate {
                return raw_estimate - biases[biases.len() - 1];
            }

            // Raw estimate is somewhere in between estimates.
            // Binary search for the calculated raw estimate.
            //
            // Here we unwrap because neither the values in `estimates`
            // nor `raw` are going to be NaN.
            let partition_index = estimates.partition_point(|est| *est <= raw_estimate);

            // Return linear interpolation between raw's neighboring points.
            let ratio = (raw_estimate - estimates[partition_index - 1])
                / (estimates[partition_index] - estimates[partition_index - 1]);

            // Calculate bias.
            raw_estimate
                - (biases[partition_index - 1]
                    + ratio * (biases[partition_index] - biases[partition_index - 1]))
        } else {
            raw_estimate
        }
    }

    fn adjust_estimate_with_zeros(&self, raw_estimate: f32, number_of_zeros: usize) -> f32 {
        if number_of_zeros > 0 {
            let low_range_correction = PRECISION::SMALL_CORRECTIONS[number_of_zeros - 1];
            if low_range_correction <= Self::LINEAR_COUNT_THRESHOLD {
                return low_range_correction;
            }
        }
        self.adjust_estimate(raw_estimate)
    }

    /// Returns whether the cardinality of this HLL will be computed using the small-range correction.
    ///
    /// # Implementation details
    /// The small-range correction is used when the cardinality of the set is small enough that the
    /// linear counting algorithm can be used to estimate the cardinality. The threshold for using
    /// the linear counting algorithm is determined by the number of registers in the HLL counter.
    pub fn use_small_range_correction(&self) -> bool {
        self.number_of_zero_register > PRECISION::NumberOfZeros::ZERO
            && PRECISION::SMALL_CORRECTIONS[self.number_of_zero_register.convert() - 1]
                <= Self::LINEAR_COUNT_THRESHOLD
    }

    #[inline(always)]
    /// Estimates the cardinality of the set based on the HLL counter data.
    ///
    /// # Example
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    /// let mut hll = HyperLogLog::<Precision9, 5>::new();
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
        if self.number_of_zero_register > PRECISION::NumberOfZeros::ZERO {
            let low_range_correction =
                PRECISION::SMALL_CORRECTIONS[self.number_of_zero_register.convert() - 1];
            if low_range_correction <= Self::LINEAR_COUNT_THRESHOLD {
                return low_range_correction;
            }
        }

        let mut raw_estimate = 0.0;

        for &word in self.words.iter_elements() {
            let mut partial: f32 = 0.0;
            for i in 0..Self::NUMBER_OF_REGISTERS_IN_WORD {
                let register = (word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                let two_to_minus_register = (127 - register) << 23;
                partial += f32::from_le_bytes(two_to_minus_register.to_le_bytes());
            }
            raw_estimate += partial;
        }

        raw_estimate -= self.get_number_of_padding_registers() as f32;

        self.adjust_estimate(raw_estimate)
    }

    #[inline(always)]
    /// Returns an estimate of the cardinality of the union of two HyperLogLog counters.
    ///
    /// This method calculates an estimate of the cardinality of the union of two HyperLogLog counters
    /// using the raw estimation values of each counter. It combines the estimation values by iterating
    /// over the register words of both counters and performing necessary calculations.
    ///
    /// # Arguments
    /// * `other`: A reference to the other HyperLogLog counter.
    ///
    /// # Returns
    /// An estimation of the cardinality of the union of the two HyperLogLog counters.
    ///
    /// # Example
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1 = HyperLogLog::<Precision12, 6>::new();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    ///
    /// let mut hll2 = HyperLogLog::<Precision12, 6>::new();
    /// hll2.insert(&2);
    /// hll2.insert(&3);
    ///
    /// let union_cardinality = hll1.estimate_union_cardinality(&hll2);
    ///
    /// assert!(union_cardinality >= 3.0 * 0.9 &&
    ///         union_cardinality <= 3.0 * 1.1);
    /// ```
    pub fn estimate_union_cardinality(&self, other: &Self) -> f32 {
        let mut raw_union_estimate = 0.0;

        let mut union_zeros: usize = 0;
        for (left_word, right_word) in self
            .words
            .iter_elements()
            .copied()
            .zip(other.words.iter_elements().copied())
        {
            let mut partial: f32 = 0.0;
            for i in 0..Self::NUMBER_OF_REGISTERS_IN_WORD {
                let left_register = (left_word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                let right_register = (right_word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                let maximal_register = (left_register).max(right_register);
                let two_to_minus_register = (127 - maximal_register) << 23;
                partial += f32::from_le_bytes(two_to_minus_register.to_le_bytes());
                union_zeros += (maximal_register == 0) as usize;
            }
            raw_union_estimate += partial;
        }

        union_zeros -= self.get_number_of_padding_registers();
        raw_union_estimate -= self.get_number_of_padding_registers() as f32;

        self.adjust_estimate_with_zeros(raw_union_estimate, union_zeros)
    }

    #[inline(always)]
    /// Returns an estimate of the cardinality of the two HLL counters union.
    pub fn estimate_union_and_sets_cardinality<F: Primitive<f32>>(
        &self,
        other: &Self,
    ) -> EstimatedUnionCardinalities<F> {
        let mut raw_union_estimate = 0.0;
        let mut raw_left_estimate = 0.0;
        let mut raw_right_estimate = 0.0;

        let mut union_zeros = 0;
        for (left_word, right_word) in self
            .words
            .iter_elements()
            .copied()
            .zip(other.words.iter_elements().copied())
        {
            let mut union_partial: f32 = 0.0;
            let mut left_partial: f32 = 0.0;
            let mut right_partial: f32 = 0.0;
            for i in 0..Self::NUMBER_OF_REGISTERS_IN_WORD {
                let left_register = (left_word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                let right_register = (right_word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                let maximal_register = (left_register).max(right_register);
                union_partial += f32::from_le_bytes(((127 - maximal_register) << 23).to_le_bytes());
                left_partial += f32::from_le_bytes(((127 - left_register) << 23).to_le_bytes());
                right_partial += f32::from_le_bytes(((127 - right_register) << 23).to_le_bytes());
                union_zeros += (maximal_register == 0) as usize;
            }
            raw_union_estimate += union_partial;
            raw_left_estimate += left_partial;
            raw_right_estimate += right_partial;
        }

        union_zeros -= self.get_number_of_padding_registers();
        raw_union_estimate -= self.get_number_of_padding_registers() as f32;
        raw_left_estimate -= self.get_number_of_padding_registers() as f32;
        raw_right_estimate -= self.get_number_of_padding_registers() as f32;

        let union_estimate =
            F::reverse(self.adjust_estimate_with_zeros(raw_union_estimate, union_zeros));
        let left_estimate =
            F::reverse(self.adjust_estimate_with_zeros(
                raw_left_estimate,
                self.number_of_zero_register.convert(),
            ));
        let right_estimate = F::reverse(self.adjust_estimate_with_zeros(
            raw_right_estimate,
            other.number_of_zero_register.convert(),
        ));

        EstimatedUnionCardinalities::from((left_estimate, right_estimate, union_estimate))
    }

    #[inline(always)]
    /// Returns an estimate of the cardinality of the intersection of two HyperLogLog counters.
    ///
    /// This method calculates an estimate of the cardinality of the intersection of two HyperLogLog
    /// counters using the raw estimation values of each counter. It combines the estimation values by
    /// iterating over the register words of both counters and performing necessary calculations.
    ///
    /// # Arguments
    /// * `other`: A reference to the other HyperLogLog counter.
    ///
    /// # Returns
    /// An estimation of the cardinality of the intersection of the two HyperLogLog counters.
    ///
    /// # Example
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1 = HyperLogLog::<Precision12, 6>::new();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    ///
    /// let mut hll2 = HyperLogLog::<Precision12, 6>::new();
    /// hll2.insert(&2);
    /// hll2.insert(&3);
    ///
    /// let intersection_cardinality: f32 = hll1.estimate_intersection_cardinality(&hll2);
    ///
    /// assert!(intersection_cardinality >= 1.0 * 0.9 &&
    ///         intersection_cardinality <= 1.0 * 1.1);
    /// ```
    pub fn estimate_intersection_cardinality<F: Primitive<f32>>(&self, other: &Self) -> F {
        self.estimate_union_and_sets_cardinality(other)
            .get_intersection_cardinality()
    }

    #[inline(always)]
    /// Returns an estimate of the Jaccard index between two HyperLogLog counters.
    ///
    /// The Jaccard index is a measure of similarity between two sets. In the context of HyperLogLog
    /// counters, it represents the ratio of the size of the intersection of the sets represented by
    /// the counters to the size of their union. This method estimates the Jaccard index by utilizing
    /// the cardinality estimation values of the intersection, left set, and right set.
    ///
    /// # Arguments
    /// * `other`: A reference to the other HyperLogLog counter.
    ///
    /// # Returns
    /// An estimation of the Jaccard index between the two HyperLogLog counters.
    ///
    /// # Example
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1 = HyperLogLog::<Precision12, 6>::new();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    /// hll1.insert(&3);
    /// hll1.insert(&4);
    ///
    /// let mut hll2 = HyperLogLog::<Precision12, 6>::new();
    /// hll2.insert(&2);
    /// hll2.insert(&3);
    /// hll2.insert(&5);
    /// hll2.insert(&6);
    ///
    /// let jaccard_index = hll1.estimate_jaccard_cardinality(&hll2);
    ///
    /// let expected = 2.0 / 6.0;
    ///
    /// assert!(jaccard_index >= expected * 0.9 &&
    ///         jaccard_index <= expected * 1.1);
    /// ```
    pub fn estimate_jaccard_cardinality(&self, other: &Self) -> f32 {
        self.estimate_union_and_sets_cardinality(other)
            .get_jaccard_index()
    }

    #[inline(always)]
    /// Returns an estimate of the cardinality of the current HyperLogLog counter minus the provided one.
    ///
    /// # Arguments
    /// * `other`: A reference to the other HyperLogLog counter.
    ///
    /// # Example
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1 = HyperLogLog::<Precision12, 6>::new();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    /// hll1.insert(&3);
    /// hll1.insert(&4);
    ///     
    /// let mut hll2 = HyperLogLog::<Precision12, 6>::new();
    /// hll2.insert(&2);
    /// hll2.insert(&3);
    /// hll2.insert(&5);
    /// hll2.insert(&6);
    ///
    /// let difference_cardinality: f32 = hll1.estimate_difference_cardinality(&hll2);
    ///
    /// assert!(difference_cardinality >= 2.0 * 0.9 &&
    ///        difference_cardinality <= 2.0 * 1.1);
    /// ```
    pub fn estimate_difference_cardinality<F: Primitive<f32>>(&self, other: &Self) -> F {
        self.estimate_union_and_sets_cardinality(other)
            .get_left_difference_cardinality()
    }

    #[inline(always)]
    /// Returns an iterator over the register values of the HyperLogLog instance.
    ///
    /// The register values are extracted from the words array, where each word contains multiple
    /// register values. This method first checks that the size of the words array matches the expected
    /// number of registers per word, which is determined by the number of bits per register. It then
    /// iterates over each word in the array and extracts the register values using bit shifting and
    /// masking operations. Finally, it takes only the expected number of register values and returns
    /// an iterator over them.
    ///
    /// # Returns
    ///
    /// An iterator over the register values of the HyperLogLog instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll = HyperLogLog::<Precision6, 5>::new();
    /// assert_eq!(hll.iter().count(), 1<<6);
    ///
    /// hll.insert(&"foo");
    /// hll.insert(&"bar");
    ///
    /// let mut hll2 = HyperLogLog::<Precision6, 5>::new();
    /// hll2|= hll;
    ///
    /// assert_eq!(hll2.iter().count(), 1<<6);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        debug_assert_eq!(
            self.words.len(),
            ceil(
                PRECISION::NUMBER_OF_REGISTERS,
                Self::NUMBER_OF_REGISTERS_IN_WORD
            )
        );

        self.words
            .iter_elements()
            .flat_map(|word| {
                (0..Self::NUMBER_OF_REGISTERS_IN_WORD)
                    .map(move |i| (word >> (i * BITS)) & Self::LOWER_REGISTER_MASK)
            })
            .take(PRECISION::NUMBER_OF_REGISTERS)
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
    /// let mut hll = HyperLogLog::<Precision12, 4>::new();
    /// assert_eq!(hll.len(), 4096);
    ///
    /// // Insert some elements into the HLL counter
    /// hll.insert(&1);
    /// hll.insert(&2);
    /// hll.insert(&3);
    /// assert_eq!(hll.len(), 1 << 12);
    ///
    /// // Merge another HLL counter with 128 registers
    /// let mut hll2 = HyperLogLog::<Precision12, 4>::new();
    /// hll2.insert(&4);
    /// hll2.insert(&5);
    /// hll |= hll2;
    /// assert_eq!(hll.len(), 1 << 12);
    /// ```
    pub fn len(&self) -> usize {
        debug_assert_eq!(PRECISION::NUMBER_OF_REGISTERS, self.iter().count());
        PRECISION::NUMBER_OF_REGISTERS
    }

    #[inline(always)]
    /// Returns whether no element was yet added to the HLL counter.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll: HyperLogLog<Precision8, 4> = HyperLogLog::new();
    ///
    /// assert!(hll.is_empty());
    ///
    /// hll.insert(&1);
    ///
    /// assert!(!hll.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == self.number_of_zero_register.convert()
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
    /// let hll = HyperLogLog::<Precision13, 6>::new();
    /// assert_eq!(hll.get_number_of_bits(), 6);
    /// ```
    pub const fn get_number_of_bits(&self) -> usize {
        BITS
    }

    #[inline(always)]
    /// Returns the number of registers in the HyperLogLog counter.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let hll = HyperLogLog::<Precision10, 6>::new();
    ///
    /// assert_eq!(hll.get_number_of_registers(), 1024);
    ///
    /// ```
    ///
    pub const fn get_number_of_registers(&self) -> usize {
        PRECISION::NUMBER_OF_REGISTERS
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
    /// let mut hll = HyperLogLog::<Precision10, 6>::new();
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
    ///
    /// For instance, in the case using the bare minimum bits per registers (4)
    /// and the minimal precision (4), for a total of 16 registers, we expect
    /// to not have any padding registers.
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let hll = HyperLogLog::<Precision4, 4>::new();
    /// assert_eq!(hll.get_number_of_padding_registers(), 0);
    ///
    /// ```
    ///
    pub const fn get_number_of_padding_registers(&self) -> usize {
        ceil(PRECISION::NUMBER_OF_REGISTERS, 32 / BITS) * Self::NUMBER_OF_REGISTERS_IN_WORD
            - PRECISION::NUMBER_OF_REGISTERS
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
    pub fn get_number_of_zero_registers(&self) -> usize {
        // Here we must be aware that when the precision is 8 or 16,
        // in order to save space, we cannot store really a number
        // equal to the number of registers, but at most number of
        // registers minus one.
        self.number_of_zero_register.convert()
    }

    #[inline(always)]
    pub fn get_number_of_non_zero_registers(&self) -> usize {
        // Calculates the number of registers that have a non-zero value by
        // subtracting the number of registers with a zero value from the total number of registers
        self.len() - self.get_number_of_zero_registers()
    }

    #[inline(always)]
    /// Returns an array of registers of the HyperLogLog counter.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll = HyperLogLog::<Precision10, 6>::new();
    /// hll.insert(&4);
    /// hll.insert(&5);
    /// hll.insert(&6);
    /// let registers = hll.get_registers();
    ///
    /// assert_eq!(registers.len(), 1024);
    /// assert!(registers.iter().any(|&x| x > 0));
    /// ```
    ///
    /// We can also create an HLL from registers, and then check
    /// whether the registers are what we expect:
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let expected = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 11, 11, 0];
    /// let mut hll = HyperLogLog::<Precision4, 6>::from_registers(&expected);
    /// assert_eq!(hll.get_registers(), expected, "Expected {:?}, got {:?}", expected, hll.get_registers());
    /// ```
    pub fn get_registers(&self) -> Vec<u32> {
        self.iter().collect()
    }

    #[inline(always)]
    /// Returns the array of words of the HyperLogLog counter.
    pub fn get_words(&self) -> PRECISION::Words {
        self.words
    }

    #[inline(always)]
    /// Returns `true` if the HyperLogLog counter may contain the given element.
    ///
    /// # Arguments
    /// * `rhs` - The element to check.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll: HyperLogLog<Precision8, 6> = HyperLogLog::new();
    /// assert_eq!(hll.may_contain(&42), false);
    ///
    /// hll.insert(&42);
    /// assert_eq!(hll.may_contain(&42), true);
    /// ```
    pub fn may_contain<T: Hash>(&self, rhs: &T) -> bool {
        let (_hash, index) = self.get_hash_and_index(&rhs);

        // Calculate the position of the register in the internal buffer array.
        let word_position = index / Self::NUMBER_OF_REGISTERS_IN_WORD;

        // Calculate the position of the register within the 32-bit word containing it.
        let register_position_in_u32 = index % Self::NUMBER_OF_REGISTERS_IN_WORD;

        // Extract the current value of the register at `index`.
        let register_value: u32 = (self.words[word_position] >> (register_position_in_u32 * BITS))
            & Self::LOWER_REGISTER_MASK;

        register_value > 0
    }

    #[inline(always)]
    /// Returns whether the provided HyperLogLog counter may be fully contained in the current HyperLogLog counter.
    ///
    /// # Arguments
    /// * `rhs` - The HyperLogLog counter to check.
    ///
    /// # Implementative details
    /// We define a counter that fully contains another counter when all of the registers
    /// of the first counter are greater than or equal to the corresponding registers of the second counter.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1: HyperLogLog<Precision8, 6> = HyperLogLog::new();
    /// let mut hll2: HyperLogLog<Precision8, 6> = HyperLogLog::new();
    ///
    /// hll1.insert(&42);
    /// hll1.insert(&43);
    /// hll1.insert(&44);
    ///
    /// hll2.insert(&42);
    /// hll2.insert(&43);
    ///
    /// assert_eq!(hll1.may_contain_all(&hll2), true);
    /// assert_eq!(hll2.may_contain_all(&hll1), false);
    ///
    /// hll2.insert(&44);
    ///
    /// assert_eq!(hll1.may_contain_all(&hll2), true);
    /// assert_eq!(hll2.may_contain_all(&hll1), true);
    /// ```
    pub fn may_contain_all(&self, rhs: &Self) -> bool {
        for (left_word, right_word) in self
            .words
            .iter_elements()
            .copied()
            .zip(rhs.words.iter_elements().copied())
        {
            for i in 0..Self::NUMBER_OF_REGISTERS_IN_WORD {
                let left_register = (left_word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                let right_register = (right_word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                if left_register < right_register {
                    return false;
                }
            }
        }
        true
    }

    /// Returns estimated overlapping and differences cardinality matrices of the provided HyperLogLog counters.
    ///
    /// # Arguments
    /// * `left` - Array of `L` HyperLogLog counters describing increasingly large surroundings of a first element.
    /// * `right` - Array of `R` HyperLogLog counters describing increasingly large surroundings of a second element.
    ///
    /// # Returns
    /// * `overlap_cardinality_matrix` - Matrix of estimated overlapping cardinalities between the elements of the left and right arrays.
    /// * `left_difference_cardinality_vector` - Vector of estimated difference cardinalities between the elements of the left array and the last element of the right array.
    /// * `right_difference_cardinality_vector` - Vector of estimated difference cardinalities between the elements of the right array and the last element of the left array.
    ///
    /// # Implementative details
    /// We expect the elements of the left and right arrays to be increasingly contained in the next one.
    ///
    /// # Examples
    /// In the following illustration, we show that for two vectors left and right of three elements,
    /// we expect to compute the exclusively overlap matrix $A_{ij}$ and the exclusively differences vectors $B_i$.    
    ///
    /// ![Illustration of overlaps](https://github.com/LucaCappelletti94/hyperloglog-rs/blob/main/triple_overlap.png?raw=true)
    ///
    /// Very similarly, for the case of vectors of two elements:
    ///
    /// ![Illustration of overlaps](https://github.com/LucaCappelletti94/hyperloglog-rs/blob/main/tuple_overlap.png?raw=true)
    ///
    /// In this crate, we make available the singular method to compute the estimated overlapping
    /// and differences cardinality matrices and vectors. In some instances, it is necessary to
    /// compute both of these matrices and vectors at once, and as they have a lot of common
    /// computations, it is more efficient to compute them at once. We expect that the two methods
    /// provide the same results, and we test this in the following example.
    ///
    /// In this example, we populate randomly two arrays of two HLL each, and we compute the
    /// estimated overlapping and differences cardinality matrices and vectors using the method
    /// computing the values at once, and we compare them to the values computed using the methods
    /// which compute the values separately.
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// for k in 0..1000 {
    ///     let mut left = [HyperLogLog::<Precision12, 6>::new(), HyperLogLog::<Precision12, 6>::new()];
    ///     let mut right = [HyperLogLog::<Precision12, 6>::new(), HyperLogLog::<Precision12, 6>::new()];
    ///
    ///     for i in 0..2 {
    ///          for j in 0..100 {
    ///             left[i].insert((k + 1) * (i + 1) * (j + 1) % 50);
    ///             right[i].insert((k + 1) * (i + 1) * (j + 1) % 30);
    ///         }
    ///     }
    ///
    ///     let left_tmp = left[0] | left[1];
    ///     left[1] = left_tmp;
    ///     let right_tmp = right[0] | right[1];
    ///     right[1] = right_tmp;
    ///
    ///     let (overlap_cardinality_matrix, left_difference_cardinality_vector, right_difference_cardinality_vector) =
    ///         HyperLogLog::<Precision12, 6>::estimated_overlap_and_differences_cardinality_matrices::<f32, 2, 2>(&left, &right);
    ///     let overlap_cardinality_matrix_singular: [[f32; 2]; 2] = HyperLogLog::estimated_overlap_cardinality_matrix(&left, &right);
    ///     let left_difference_cardinality_vector_singular: [f32; 2] = HyperLogLog::estimated_difference_cardinality_vector(&left, &right[1]);
    ///     let right_difference_cardinality_vector_singular: [f32; 2] = HyperLogLog::estimated_difference_cardinality_vector(&right, &left[1]);
    ///
    ///    for i in 0..2 {
    ///        for j in 0..2 {
    ///             assert!(
    ///                 overlap_cardinality_matrix[i][j] >= overlap_cardinality_matrix_singular[i][j] * 0.9 &&
    ///                 overlap_cardinality_matrix[i][j] <= overlap_cardinality_matrix_singular[i][j] * 1.1,
    ///                 "The value of the overlap cardinality matrix at position ({}, {}) is not the same when computed at once and separately.",
    ///                 i,
    ///                 j
    ///             );
    ///         }
    ///     }
    ///
    /// for i in 0..2 {
    ///     assert!(
    ///         left_difference_cardinality_vector[i] >= left_difference_cardinality_vector_singular[i] * 0.9 - 0.1 &&
    ///         left_difference_cardinality_vector[i] <= left_difference_cardinality_vector_singular[i] * 1.1 + 0.1,
    ///         "The value of the left difference cardinality vector at position {} is not the same when computed at once and separately.",
    ///         i
    ///     );
    ///
    ///     assert!(
    ///         right_difference_cardinality_vector[i] >= right_difference_cardinality_vector_singular[i] * 0.9 - 0.1 &&
    ///         right_difference_cardinality_vector[i] <= right_difference_cardinality_vector_singular[i] * 1.1 + 0.1,
    ///         concat!(
    ///             "The value of the right difference cardinality vector at position {} ",
    ///             "is not the same when computed at once and separately.",
    ///             "We expected {} but got {}."
    ///         ),
    ///         i,
    ///         right_difference_cardinality_vector_singular[i],
    ///         right_difference_cardinality_vector[i]
    ///     );
    /// }
    ///
    /// }
    ///
    /// ```
    ///
    pub fn estimated_overlap_and_differences_cardinality_matrices<
        F: Primitive<f32>,
        const L: usize,
        const R: usize,
    >(
        left: &[Self; L],
        right: &[Self; R],
    ) -> ([[F; R]; L], [F; L], [F; R]) {
        // When we are not in release mode, we check that the HLL are increasing in size.
        #[cfg(debug_assertions)]
        for i in 1..L {
            assert!(
                left[i].may_contain_all(&left[i - 1]),
                concat!(
                    "We expected for all the elements of the left array to be contained in the next one, ",
                    "but this is not the case for the element at position {}."
                ),
                i
            );
        }

        // When we are not in release mode, we check that the HLL are increasing in size.
        #[cfg(debug_assertions)]
        for i in 1..R {
            assert!(
                right[i].may_contain_all(&right[i - 1]),
                concat!(
                    "We expected for all the elements of the right array to be contained in the next one, ",
                    "but this is not the case for the element at position {}."
                ),
                i
            );
        }

        let mut overlap_cardinality_matrix = [[F::reverse(0.0); R]; L];
        let mut left_difference_cardinality_vector = [F::reverse(0.0); L];
        let mut right_difference_cardinality_vector = [F::reverse(0.0); R];
        let mut left_comulative_estimated_cardinality = F::reverse(0.0);
        let mut right_comulative_estimated_cardinality = F::reverse(0.0);

        for i in 0..(L - 1) {
            for j in 0..(R - 1) {
                overlap_cardinality_matrix[i][j] = (left[i]
                    .estimate_intersection_cardinality::<F>(&right[j])
                    // Since we need to compute the exclusive overlap cardinality, i.e. we exclude the elements
                    // contained in the smaller HLLs, we need to subtract all of the partial cardinality of elements
                    // with a smaller index than the current one.
                    - overlap_cardinality_matrix[0..(i+1)]
                        .iter()
                        .map(|row| row[0..(j+1)].iter().copied().sum::<F>())
                        .sum::<F>())
                .get_max(F::reverse(0.0));
            }

            // We handle separately the case for j = R - 1, since we need to also compute
            // the difference cardinality of the left HLL.
            let estimate = left[i].estimate_union_and_sets_cardinality::<F>(&right[R - 1]);
            left_difference_cardinality_vector[i] = (estimate.get_left_difference_cardinality()
                - left_comulative_estimated_cardinality)
                .get_max(F::reverse(0.0));
            left_comulative_estimated_cardinality += left_difference_cardinality_vector[i];
            overlap_cardinality_matrix[i][R - 1] = (estimate.get_intersection_cardinality()
                // Since we need to compute the exclusive overlap cardinality, i.e. we exclude the elements
                // contained in the smaller HLLs, we need to subtract all of the partial cardinality of elements
                // with a smaller index than the current one.
                - overlap_cardinality_matrix[0..(i+1)]
                    .iter()
                    .map(|row| row[0..R].iter().copied().sum::<F>())
                    .sum::<F>())
            .get_max(F::reverse(0.0));
        }

        // We handle separately the case for i = L - 1, since we need to also compute
        // the difference cardinality of the left and right HLL. We still have to compute
        // all difference cardinalities between the last element of left (the L-1 one) and
        // all the elements of right.

        for j in 0..(R - 1) {
            let estimate = left[L - 1].estimate_union_and_sets_cardinality::<F>(&right[j]);
            right_difference_cardinality_vector[j] = (estimate.get_right_difference_cardinality()
                - right_comulative_estimated_cardinality)
                .get_max(F::reverse(0.0));
            right_comulative_estimated_cardinality += right_difference_cardinality_vector[j];
            overlap_cardinality_matrix[L - 1][j] = (estimate.get_intersection_cardinality()
                // Since we need to compute the exclusive overlap cardinality, i.e. we exclude the elements
                // contained in the smaller HLLs, we need to subtract all of the partial cardinality of elements
                // with a smaller index than the current one.
                - overlap_cardinality_matrix[0..L]
                    .iter()
                    .map(|row| row[0..(j+1)].iter().copied().sum::<F>())
                    .sum::<F>())
            .get_max(F::reverse(0.0));
        }

        // We handle separately the case for i = L - 1 and j = R - 1, since we need to also compute
        // the difference cardinality of the left and right HLL.
        let estimate = left[L - 1].estimate_union_and_sets_cardinality::<F>(&right[R - 1]);
        left_difference_cardinality_vector[L - 1] = (estimate.get_left_difference_cardinality()
            - left_comulative_estimated_cardinality)
            .get_max(F::reverse(0.0));
        right_difference_cardinality_vector[R - 1] = (estimate.get_right_difference_cardinality()
            - right_comulative_estimated_cardinality)
            .get_max(F::reverse(0.0));
        overlap_cardinality_matrix[L - 1][R - 1] = (estimate.get_intersection_cardinality()
                // Since we need to compute the exclusive overlap cardinality, i.e. we exclude the elements
                // contained in the smaller HLLs, we need to subtract all of the partial cardinality of elements
                // with a smaller index than the current one.
                - overlap_cardinality_matrix[0..L]
                    .iter()
                    .map(|row| row[0..R].iter().copied().sum::<F>())
                    .sum::<F>())
        .get_max(F::reverse(0.0));

        (
            overlap_cardinality_matrix,
            left_difference_cardinality_vector,
            right_difference_cardinality_vector,
        )
    }

    /// Returns estimated overlapping cardinality matrices of the provided HyperLogLog counters.
    ///
    /// # Arguments
    /// * `left` - Array of `L` HyperLogLog counters describing increasingly large surroundings of a first element.
    /// * `right` - Array of `R` HyperLogLog counters describing increasingly large surroundings of a second element.
    ///
    /// # Implementation details
    /// Both arrays are expected to contain HyperLogLog counters increasing in size, i.e. the first element of `left`
    /// should be contained in the second element of `left`, which should be contained in the third element of `left`,
    /// and so on. The same applies to `right`.
    ///
    /// # Examples
    ///
    /// We start with a trivial example with solely two counters.
    /// In this case, the result will have to contain a single element
    /// which is the estimated intersection cardinality of the two counters.
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1: HyperLogLog<Precision9, 6> = HyperLogLog::new();
    /// let mut hll2: HyperLogLog<Precision9, 6> = HyperLogLog::new();
    ///
    /// hll1.insert(&42);
    /// hll1.insert(&43);
    /// hll1.insert(&44);
    ///
    /// hll2.insert(&42);
    /// hll2.insert(&43);
    ///
    /// let result = HyperLogLog::estimated_overlap_cardinality_matrix::<f32, 1, 1>(&[hll1,], &[hll2,]);
    ///
    /// assert!(
    ///     result[0][0] < 2.0 * 1.1 &&
    ///     result[0][0] > 2.0 * 0.9,
    ///     "The estimated intersection cardinality should be around 2, but it is {}.",
    ///     result[0][0]
    /// );
    /// ```
    ///
    /// Now we consider a more complex example with two arrays of counters.
    /// We start with two arrays of two elements each. This means that in the end we will have a 2x2 matrix.
    /// The value in position (0,0) of the matrix will be the estimated intersection cardinality of the first element
    /// of the first array and the first element of the second array. The value of the subsequent positions
    /// are less trivial, as we will have to take into account the difference of the elements present in the
    /// smaller sets which we do not want to count multiple times.
    ///
    /// It follows that, for the value in position (0, 1), we will need to subtract the value in position (0,0).
    /// For the value in position (1, 0), we will need to subtract the value in position (0,0). And finally, for
    /// the value in position (1, 1), we will need to subtract the values in positions (0,0), (0,1) and (1,0).
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1: HyperLogLog<Precision9, 6> = HyperLogLog::new();
    /// let mut hll2: HyperLogLog<Precision9, 6> = HyperLogLog::new();
    ///
    /// hll1.insert(&42);
    /// hll1.insert(&43);
    /// hll1.insert(&44);
    ///    
    /// hll2.insert(&42);
    /// hll2.insert(&43);
    ///
    /// let mut hll3: HyperLogLog<Precision9, 6> = HyperLogLog::new();
    /// let mut hll4: HyperLogLog<Precision9, 6> = HyperLogLog::new();
    ///
    /// hll3.insert(&42);
    /// hll3.insert(&43);
    /// hll3.insert(&44);
    /// hll3.insert(&45);
    ///
    /// hll4.insert(&42);
    /// hll4.insert(&43);
    /// hll4.insert(&44);
    ///
    /// let result = HyperLogLog::estimated_overlap_cardinality_matrix::<f32, 2, 2>(&[hll1, hll3], &[hll2, hll4]);
    ///
    /// assert!(
    ///     result[0][0] < 2.0 * 1.1 &&
    ///     result[0][0] > 2.0 * 0.9,
    ///     "Test 1a: The estimated intersection cardinality should be around 2, but it is {}.",
    ///     result[0][0]
    /// );
    ///
    /// assert!(
    ///     result[0][1] < 1.0 * 1.1 &&
    ///     result[0][1] > 1.0 * 0.9,
    ///     "Test 2a: The estimated intersection cardinality should be around 1, but it is {}.",
    ///     result[0][1]
    /// );
    ///
    /// assert!(
    ///     result[1][0] < 0.1 &&
    ///     result[1][0] > -0.1,
    ///     "Test 3a: The estimated intersection cardinality should be around 0, but it is {}.",
    ///     result[1][0]
    /// );
    ///
    /// assert!(
    ///     result[1][1] < 0.1 &&
    ///     result[1][1] > -0.1,
    ///     "Test 4a: The estimated intersection cardinality should be around 0, but it is {}.",
    ///     result[1][1]
    /// );
    ///
    /// ```
    ///
    /// We now consider a more complex example, with two arrays of three elements each.
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1: HyperLogLog<Precision8, 6> = HyperLogLog::new();
    /// let mut hll2: HyperLogLog<Precision8, 6> = HyperLogLog::new();
    /// let mut hll3: HyperLogLog<Precision8, 6> = HyperLogLog::new();
    ///
    /// let mut hll4: HyperLogLog<Precision8, 6> = HyperLogLog::new();
    /// let mut hll5: HyperLogLog<Precision8, 6> = HyperLogLog::new();
    /// let mut hll6: HyperLogLog<Precision8, 6> = HyperLogLog::new();
    ///
    /// hll1.insert(&42);
    /// hll1.insert(&43);
    /// hll1.insert(&44);
    ///
    /// hll2.insert(&42);
    /// hll2.insert(&43);
    /// hll2.insert(&44);
    ///
    /// hll3.insert(&42);
    /// hll3.insert(&43);
    /// hll3.insert(&44);
    /// hll3.insert(&45);
    ///
    /// hll4.insert(&42);
    /// hll4.insert(&43);
    /// hll4.insert(&44);
    ///
    /// hll5.insert(&42);
    /// hll5.insert(&43);
    /// hll5.insert(&44);
    /// hll5.insert(&45);
    ///
    /// hll6.insert(&42);
    /// hll6.insert(&43);
    /// hll6.insert(&44);
    /// hll6.insert(&45);
    /// hll6.insert(&46);
    ///
    /// let result = HyperLogLog::estimated_overlap_cardinality_matrix::<f32, 3, 3>(&[hll1, hll3, hll6], &[hll2, hll4, hll5]);
    ///
    /// assert!(
    ///     result[0][0] < 3.0 * 1.1 &&
    ///     result[0][0] > 3.0 * 0.9,
    ///     concat!(
    ///         "Test 1b: The estimated intersection cardinality should be around 3, but it is {}. ",
    ///         "This is because the value in cell {:?} is dependent on no previous intersection.",
    ///     ),
    ///     result[0][0],
    ///     (0, 0),
    /// );
    ///
    /// assert!(
    ///     result[0][1] < 0.1 &&
    ///     result[0][1] > -0.1,
    ///     concat!(
    ///         "Test 2b: The estimated intersection cardinality should be around 0, but it is {}. ",
    ///         "This is because the value in cell {:?} is dependent on the previous intersection ",
    ///         "values {:?}, and is not equal to the simple intersection of the HLL counters in ",
    ///         "positions {} and {}, which would have been an estimated cardinality of {}."
    ///     ),
    ///     result[0][1],
    ///     (0, 1),
    ///     vec![result[0][0]],
    ///     0,
    ///     1,
    ///     hll1.estimate_intersection_cardinality::<f32>(&hll4),
    /// );
    ///
    /// assert!(
    ///     result[1][0] < 0.1 &&
    ///     result[1][0] > -0.1,
    ///     concat!(
    ///         "Test 3b: The estimated intersection cardinality should be around 1, but it is {}. ",
    ///         "This is because the value in cell {:?} is dependent on the previous intersection ",
    ///         "values {:?}, and is not equal to the simple intersection of the HLL counters in ",
    ///         "positions {} and {}, which would have been an estimated cardinality of {}."
    ///     ),
    ///     result[1][0],
    ///     (1, 0),
    ///     vec![result[0][0]],
    ///     1,
    ///     0,
    ///     hll3.estimate_intersection_cardinality::<f32>(&hll2),
    /// );
    ///
    /// assert!(
    ///     result[1][1] < 0.1 &&
    ///     result[1][1] > -0.1,
    ///     concat!(
    ///         "Test 4b: The estimated intersection cardinality should be around 2, but it is {}. ",
    ///         "This is because the value in cell {:?} is dependent on the previous intersection ",
    ///         "values {:?}, and is not equal to the simple intersection of the HLL counters in ",
    ///         "positions {} and {}, which would have been an estimated cardinality of {}."
    ///     ),
    ///     result[1][1],
    ///     (1, 1),
    ///     vec![result[0][0], result[1][0], result[0][1]],
    ///     1,
    ///     1,
    ///     hll3.estimate_intersection_cardinality::<f32>(&hll4),
    /// );
    ///
    /// assert!(
    ///     result[1][2] < 1.0 * 1.1 &&
    ///     result[1][2] > 1.0 * 0.9,
    ///     concat!(
    ///         "Test 5b: The estimated intersection cardinality should be around 1, but it is {}. ",
    ///         "This is because the value in cell {:?} is dependent on the previous intersection ",
    ///         "values {:?}, and is not equal to the simple intersection of the HLL counters in ",
    ///         "positions {} and {}, which would have been an estimated cardinality of {}."
    ///     ),
    ///     result[1][2],
    ///     (1, 2),
    ///     vec![result[0][0], result[1][0], result[0][1], result[1][1]],
    ///     1,
    ///     2,
    ///     hll3.estimate_intersection_cardinality::<f32>(&hll5),
    /// );
    ///
    /// assert!(
    ///     result[2][0] < 0.1 &&
    ///     result[2][0] > -0.1,
    ///     concat!(
    ///         "Test 6b: The estimated intersection cardinality should be around 0, but it is {}. ",
    ///         "This is because the value in cell {:?} is dependent on the previous intersection ",
    ///         "values {:?}, and is not equal to the simple intersection of the HLL counters in ",
    ///         "positions {} and {}, which would have been an estimated cardinality of {}."
    ///     ),
    ///     result[2][0],
    ///     (2, 0),
    ///     vec![result[0][0], result[1][0],],
    ///     2,
    ///     0,
    ///     hll6.estimate_intersection_cardinality::<f32>(&hll2),
    /// );
    ///
    /// assert!(
    ///     result[2][1] < 0.1 &&
    ///     result[2][1] > -0.1,
    ///     concat!(
    ///         "Test 7b: The estimated intersection cardinality should be around 0, but it is {}." ,
    ///         "This is because the value in cell {:?} is dependent on the previous intersection ",
    ///         "values {:?}, and is not equal to the simple intersection of the HLL counters in ",
    ///         "positions {} and {}, which would have been an estimated cardinality of {}."
    ///     ),
    ///     result[2][1],
    ///     (2, 1),
    ///     vec![result[0][0], result[1][0], result[2][0]],
    ///     2,
    ///     1,
    ///     hll6.estimate_intersection_cardinality::<f32>(&hll4),
    /// );
    ///
    /// assert!(
    ///     result[2][2] < 0.1 &&
    ///     result[2][2] > -0.1,
    ///     concat!(
    ///         "Test 8b: The estimated intersection cardinality should be around 0, but it is {}. ",
    ///         "This is because the value in cell {:?} is dependent on the previous intersection ",
    ///         "values {:?}, and is not equal to the simple intersection of the HLL counters in ",
    ///         "positions {} and {}, which would have been an estimated cardinality of {}."
    ///     ),
    ///     result[2][2],
    ///     (2, 2),
    ///     vec![result[0][0], result[1][0], result[0][1], result[2][0], result[0][2], result[2][1], result[1][2]],
    ///     2,
    ///     2,
    ///     hll6.estimate_intersection_cardinality::<f32>(&hll5),
    /// );
    ///
    /// ```
    ///
    pub fn estimated_overlap_cardinality_matrix<
        F: Primitive<f32>,
        const L: usize,
        const R: usize,
    >(
        left: &[Self; L],
        right: &[Self; R],
    ) -> [[F; R]; L] {
        // When we are not in release mode, we check that the HLL are increasing in size.
        #[cfg(debug_assertions)]
        for i in 1..L {
            assert!(
                left[i].may_contain_all(&left[i - 1]),
                concat!(
                    "We expected for all the elements of the left array to be contained in the next one, ",
                    "but this is not the case for the element at position {}."
                ),
                i
            );
        }

        // When we are not in release mode, we check that the HLL are increasing in size.
        #[cfg(debug_assertions)]
        for i in 1..R {
            assert!(
                right[i].may_contain_all(&right[i - 1]),
                concat!(
                    "We expected for all the elements of the right array to be contained in the next one, ",
                    "but this is not the case for the element at position {}."
                ),
                i
            );
        }

        let mut overlap_cardinality_matrix = [[F::reverse(0.0); R]; L];

        for i in 0..L {
            for j in 0..R {
                overlap_cardinality_matrix[i][j] = (left[i]
                    .estimate_intersection_cardinality::<F>(&right[j])
                    // Since we need to compute the exclusive overlap cardinality, i.e. we exclude the elements
                    // contained in the smaller HLLs, we need to subtract all of the partial cardinality of elements
                    // with a smaller index than the current one.
                    - overlap_cardinality_matrix[0..(i+1)]
                        .iter()
                        .map(|row| row[0..(j+1)].iter().copied().sum::<F>())
                        .sum::<F>())
                .get_max(F::reverse(0.0));
            }
        }

        overlap_cardinality_matrix
    }

    #[inline(always)]
    /// Returns estimated overlapping cardinality vectors of the provided HyperLogLog counters.
    ///
    /// # Arguments
    /// * `left` - Array of `N` HyperLogLog counters describing increasingly large surroundings of a first element.
    /// * `right` - A single HyperLogLog counter describing, usually, the largest surroundings of a second element.
    ///
    /// # Implementation details
    /// The array of HyperLogLog counters is expected to contain HyperLogLog counters increasing in size, i.e. the first element of `left`
    /// should be contained in the second element of `left`, which should be contained in the third element of `left`,
    /// and so on.
    ///
    /// # Examples
    ///
    /// We start with a trivial example with solely two counters.
    /// In this case, the result will have to contain a single element
    /// which is the estimated left-difference cardinality of the two counters.
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1: HyperLogLog<Precision8, 6> = HyperLogLog::new();
    /// let mut hll2: HyperLogLog<Precision8, 6> = HyperLogLog::new();
    ///
    /// hll1.insert(&42);
    /// hll1.insert(&43);
    /// hll1.insert(&44);
    ///
    /// hll2.insert(&42);
    /// hll2.insert(&43);
    ///
    /// let result: [f32; 1] = HyperLogLog::estimated_difference_cardinality_vector(&[hll1,], &hll2);
    ///
    /// assert!(
    ///    result[0] < 1.0 * 1.1 &&
    ///   result[0] > 1.0 * 0.9,
    ///  "The estimated left-difference cardinality should be around 1, but it is {}.",
    /// result[0]
    /// );
    ///
    /// ```
    ///
    /// Now we consider a more complex example with two arrays of counters.
    /// We start with two arrays of two elements each. This means that in the end we will have a 2x1 vector.
    /// The value in position (0) of the vector will be the estimated left-difference cardinality of the first element
    /// of the first array and the first element of the second array. The value of the subsequent positions
    /// are less trivial, as we will have to take into account the difference of the elements present in the
    /// smaller sets which we do not want to count multiple times.
    ///
    /// It follows that, for the value in position (1), we will need to subtract the value in position (0).
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1: HyperLogLog<Precision8, 6> = HyperLogLog::new();
    /// let mut hll2: HyperLogLog<Precision8, 6> = HyperLogLog::new();
    ///
    /// hll1.insert(&42);
    /// hll1.insert(&43);
    /// hll1.insert(&44);
    ///
    /// hll2.insert(&42);
    /// hll2.insert(&43);
    ///
    /// let mut hll3: HyperLogLog<Precision8, 6> = HyperLogLog::new();
    ///
    /// hll3.insert(&42);
    /// hll3.insert(&43);
    /// hll3.insert(&44);
    /// hll3.insert(&45);
    ///
    /// let result: [f32; 2] = HyperLogLog::estimated_difference_cardinality_vector(&[hll1, hll3], &hll2);
    ///
    /// assert!(
    ///     result[0] < 1.0 * 1.1 &&
    ///     result[0] > 1.0 * 0.9,
    ///     "Test 1a: The estimated left-difference cardinality should be around 1, but it is {}.",
    ///     result[0]
    /// );
    ///
    /// assert!(
    ///     result[1] < 1.1 &&
    ///     result[1] > 0.9,
    ///     "Test 2a: The estimated left-difference cardinality should be around 1, but it is {}.",
    ///     result[1]
    /// );
    ///
    /// ```
    ///
    pub fn estimated_difference_cardinality_vector<F: Primitive<f32>, const N: usize>(
        array: &[Self; N],
        other: &Self,
    ) -> [F; N] {
        // When we are not in release mode, we check that the HLL are increasing in size.
        #[cfg(debug_assertions)]
        for i in 1..N {
            assert!(
                array[i].may_contain_all(&array[i - 1]),
                concat!(
                    "We expected for all the elements of the array to be contained in the next one, ",
                    "but this is not the case for the element at position {}."
                ),
                i
            );
        }

        let mut difference_cardinality_vector = [F::reverse(0.0); N];
        let mut comulative_estimated_cardinality = F::reverse(0.0);

        for i in 0..N {
            difference_cardinality_vector[i] = (array[i]
                .estimate_difference_cardinality::<F>(other)
                // Since we need to compute the exclusive overlap cardinality, i.e. we exclude the elements
                // contained in the smaller HLLs, we need to subtract all of the partial cardinality of elements
                // with a smaller index than the current one.
                - comulative_estimated_cardinality)
                .get_max(F::reverse(0.0));

            comulative_estimated_cardinality += difference_cardinality_vector[i];
        }

        difference_cardinality_vector
    }

    #[inline(always)]
    /// Returns the hash value and the corresponding register's index for a given value.
    ///
    /// # Arguments
    /// * `value` - A reference to the value to be hashed.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll: HyperLogLog<Precision8, 6> = HyperLogLog::new();
    /// let value = 42;
    /// let (hash, index) = hll.get_hash_and_index(&value);
    ///
    /// //assert_eq!(hash, 10123147082338939904, "Expected hash {}, got {}.", 10123147082338939904, hash);
    /// ```
    pub fn get_hash_and_index<T: Hash>(&self, value: &T) -> (u64, usize) {
        // Create a new hasher.
        let mut hasher = SipHasher13::new();
        // Calculate the hash.
        value.hash(&mut hasher);
        let hash: u64 = hasher.finish();

        // Calculate the register's index using the highest bits of the hash.
        let index: usize =
            (hash as usize & Self::UPPER_PRECISION_MASK) >> (64 - PRECISION::EXPONENT);
        // And we delete the used bits from the hash.
        let hash: u64 = hash << PRECISION::EXPONENT;
        debug_assert!(
            index < PRECISION::NUMBER_OF_REGISTERS,
            "The index {} must be less than the number of registers {}.",
            index,
            PRECISION::NUMBER_OF_REGISTERS
        );

        (hash, index)
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
    pub fn insert<T: Hash>(&mut self, rhs: T) {
        let (mut hash, index) = self.get_hash_and_index(&rhs);

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
            self.number_of_zero_register -=
                PRECISION::NumberOfZeros::reverse((register_value == 0) as usize);
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

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize, A: Hash> core::iter::FromIterator<A>
    for HyperLogLog<PRECISION, BITS>
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
