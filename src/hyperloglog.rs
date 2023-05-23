use crate::utils::{ceil, get_alpha, precompute_small_corrections};
use core::hash::{Hash, Hasher};
use siphasher::sip::SipHasher13;

/// A struct for more readable code.
pub struct EstimatedUnionCardinalities {
    /// The estimated cardinality of the left set.
    left_cardinality: f32,
    /// The estimated cardinality of the right set.
    right_cardinality: f32,
    /// The estimated cardinality of the union of the two sets.
    union_cardinality: f32,
}

impl From<(f32, f32, f32)> for EstimatedUnionCardinalities {
    fn from(value: (f32, f32, f32)) -> Self {
        Self {
            left_cardinality: value.0,
            right_cardinality: value.1,
            union_cardinality: value.2,
        }
    }
}

impl EstimatedUnionCardinalities {
    #[inline(always)]
    /// Returns the estimated cardinality of the left set.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::EstimatedUnionCardinalities;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let left_cardinality = estimated_union_cardinalities.get_left_cardinality();
    ///
    /// assert_eq!(left_cardinality, 2.0);
    ///
    /// ```
    pub fn get_left_cardinality(&self) -> f32 {
        self.left_cardinality
    }

    #[inline(always)]
    /// Returns the estimated cardinality of the right set.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::EstimatedUnionCardinalities;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let right_cardinality = estimated_union_cardinalities.get_right_cardinality();
    ///
    /// assert_eq!(right_cardinality, 3.0);
    ///
    /// ```
    pub fn get_right_cardinality(&self) -> f32 {
        self.right_cardinality
    }

    #[inline(always)]
    /// Returns the estimated cardinality of the union of the two sets.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::EstimatedUnionCardinalities;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let union_cardinality = estimated_union_cardinalities.get_union_cardinality();
    ///
    /// assert_eq!(union_cardinality, 4.0);
    ///
    /// ```
    pub fn get_union_cardinality(&self) -> f32 {
        self.union_cardinality
    }

    #[inline(always)]
    /// Returns the estimated cardinality of the intersection of the two sets.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::EstimatedUnionCardinalities;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let intersection_cardinality = estimated_union_cardinalities.get_intersection_cardinality();
    ///
    /// assert_eq!(intersection_cardinality, 1.0);
    ///
    /// ```
    pub fn get_intersection_cardinality(&self) -> f32 {
        self.left_cardinality + self.right_cardinality - self.union_cardinality
    }

    #[inline(always)]
    /// Returns the estimated Jaccard index of the two sets.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::EstimatedUnionCardinalities;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let jaccard_index = estimated_union_cardinalities.get_jaccard_index();
    ///
    /// assert_eq!(jaccard_index, 1.0 / 4.0, "Example 1: Expected 1.0 / 4.0, got {}", jaccard_index);
    ///
    /// ```
    ///
    pub fn get_jaccard_index(&self) -> f32 {
        unsafe { self.get_jaccard_index_with_offset(0.0) }
    }

    #[inline(always)]
    /// Returns the estimated Jaccard index of the two sets with an offset.
    ///
    /// # Arguments
    /// * `offset`: The offset to apply to the union cardinality.
    ///
    /// # Safety
    /// The offset, if negative, must be smaller than the union intersection.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::EstimatedUnionCardinalities;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let jaccard_index = unsafe{estimated_union_cardinalities.get_jaccard_index_with_offset(1.0)};
    ///
    /// assert_eq!(jaccard_index, 1.0 / 5.0, "Example 1: Expected 1.0 / 5.0, got {}", jaccard_index);
    ///
    /// ```
    ///
    /// Similarly, an example with a negative offset that can be useful
    /// when one has to remove self-loops from the union, for instance:
    ///
    /// ```
    /// use hyperloglog_rs::EstimatedUnionCardinalities;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 4.0, 4.0));
    ///
    /// let jaccard_index = unsafe{estimated_union_cardinalities.get_jaccard_index_with_offset(-2.0)};
    /// let expected = (2.0 + 4.0 - 2.0 - (4.0 - 2.0)) / (4.0 - 2.0);
    ///
    /// assert_eq!(jaccard_index, expected, "Example 2: Expected {}, got {}", expected, jaccard_index);
    ///
    /// ```
    pub unsafe fn get_jaccard_index_with_offset(&self, offset: f32) -> f32 {
        ((self.left_cardinality + self.right_cardinality - self.union_cardinality)
            / (self.union_cardinality + offset).max(f32::EPSILON))
        .max(0.0)
        .min(1.0)
    }
}

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
/// use hyperloglog_rs::HyperLogLog;
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
///
/// # Citations
///
/// This implementation is based on the following papers:
///
/// * Flajolet, Philippe, et al. "HyperLogLog: the analysis of a near-optimal cardinality estimation algorithm." DMTCS Proceedings 1 (2007): 127-146.
/// * Heule, Stefan, Marc Nunkesser, and Alexander Hall. "HyperLogLog in practice: algorithmic engineering of a state of the art cardinality estimation algorithm." Proceedings of the 16th International Conference on Extending Database Technology. 2013.
///
pub struct HyperLogLog<const PRECISION: usize, const BITS: usize>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    pub(crate) words: [u32; ceil(1 << PRECISION, 32 / BITS)],
    pub(crate) number_of_zero_register: usize,
}

impl<const PRECISION: usize, const BITS: usize, T: Hash> From<T> for HyperLogLog<PRECISION, BITS>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
    [(); 1 << PRECISION]:,
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
    /// # use hyperloglog_rs::HyperLogLog;
    ///
    /// let hll = HyperLogLog::<14, 5>::from("test");
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
/// # use hyperloglog_rs::HyperLogLog;
///
/// let hll: HyperLogLog<10, 6> = Default::default();
/// assert_eq!(hll.len(), 1024);
/// assert_eq!(hll.get_number_of_bits(), 6);
/// ```
impl<const PRECISION: usize, const BITS: usize> Default for HyperLogLog<PRECISION, BITS>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
    [(); 1 << PRECISION]:,
{
    /// Returns a new HyperLogLog instance with default configuration settings.
    fn default() -> Self {
        Self::new()
    }
}

impl<const PRECISION: usize, const BITS: usize> HyperLogLog<PRECISION, BITS>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
    [(); 1 << PRECISION]:,
{
    /// The number of registers used by the HyperLogLog algorithm, which depends on its precision.
    pub const NUMBER_OF_REGISTERS: usize = 1 << PRECISION;

    /// The threshold value used in the small range correction of the HyperLogLog algorithm.
    pub const SMALL_RANGE_CORRECTION_THRESHOLD: f32 = 2.5_f32 * (Self::NUMBER_OF_REGISTERS as f32);

    /// The float value of 2^32, used in the intermediate range correction of the HyperLogLog algorithm.
    pub const TWO_32: f32 = (1u64 << 32) as f32;

    /// The threshold value used in the intermediate range correction of the HyperLogLog algorithm.
    pub const INTERMEDIATE_RANGE_CORRECTION_THRESHOLD: f32 = Self::TWO_32 / 30.0_f32;

    /// The mask used to obtain the lower register bits in the HyperLogLog algorithm.
    pub const LOWER_REGISTER_MASK: u32 = (1 << BITS) - 1;

    /// The number of registers that can fit in a single 32-bit word in the HyperLogLog algorithm.
    pub const NUMBER_OF_REGISTERS_IN_WORD: usize = 32 / BITS;

    /// The precomputed small corrections used in the HyperLogLog algorithm for better performance.
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
    /// use hyperloglog_rs::HyperLogLog;
    ///
    /// let registers = [0_u32; 1 << 4];
    /// let hll = HyperLogLog::<4, 6>::from_registers(&registers);
    /// assert_eq!(hll.len(), 1 << 4);
    /// ```
    pub fn from_registers(registers: &[u32]) -> Self {
        assert!(
            registers.len() == Self::NUMBER_OF_REGISTERS,
            "We expect {} registers, but got {}",
            Self::NUMBER_OF_REGISTERS,
            registers.len()
        );
        let mut words = [0; ceil(1 << PRECISION, 32 / BITS)];
        let number_of_zero_register = words
            .iter_mut()
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
            number_of_zero_register,
        }
    }

    fn adjust_estimate(&self, mut raw_estimate: f32, number_of_zeros: usize) -> f32 {
        debug_assert!(!raw_estimate.is_nan(), "Raw estimate is NaN");
        // Apply the final scaling factor to obtain the estimate of the cardinality
        raw_estimate = get_alpha(1 << PRECISION)
            * (Self::NUMBER_OF_REGISTERS * Self::NUMBER_OF_REGISTERS) as f32
            / raw_estimate;

        debug_assert!(!raw_estimate.is_nan(), "Updated raw estimate is NaN");

        // Apply the small range correction factor if the raw estimate is below the threshold
        // and there are zero registers in the counter.
        if raw_estimate <= Self::SMALL_RANGE_CORRECTION_THRESHOLD && number_of_zeros > 0 {
            raw_estimate = Self::SMALL_CORRECTIONS[number_of_zeros - 1];
            debug_assert!(
                !raw_estimate.is_nan(),
                "Small range correction factor is NaN"
            )
        // Apply the intermediate range correction factor if the raw estimate is above the threshold.
        } else if raw_estimate >= Self::INTERMEDIATE_RANGE_CORRECTION_THRESHOLD {
            let corrected_raw_estimate =
                -Self::TWO_32 * (-raw_estimate.min(Self::TWO_32) / Self::TWO_32).ln_1p();
            debug_assert!(
                !corrected_raw_estimate.is_nan(),
                "Intermediate range correction factor is NaN, starting raw estimate was {}",
                raw_estimate
            );
            raw_estimate = corrected_raw_estimate;
        }
        raw_estimate
    }

    #[inline(always)]
    /// Estimates the cardinality of the set based on the HLL counter data.
    ///
    /// # Example
    ///
    /// ```
    /// # use hyperloglog_rs::HyperLogLog;
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
        let mut raw_estimate = 0.0;

        for word in self.words {
            let mut partial: f32 = 0.0;
            for i in 0..Self::NUMBER_OF_REGISTERS_IN_WORD {
                let register = (word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                let two_to_minus_register = (127 - register) << 23;
                partial += f32::from_le_bytes(two_to_minus_register.to_le_bytes());
            }
            raw_estimate += partial;
        }

        debug_assert!(!raw_estimate.is_nan(), "Raw estimate is NaN");

        raw_estimate -= self.get_number_of_padding_registers() as f32;

        self.adjust_estimate(raw_estimate, self.get_number_of_zero_registers())
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
    /// use hyperloglog_rs::HyperLogLog;
    ///
    /// let mut hll1 = HyperLogLog::<12, 6>::new();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    ///
    /// let mut hll2 = HyperLogLog::<12, 6>::new();
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

        let mut union_zeros = 0;
        for (left_word, right_word) in self.words.iter().copied().zip(other.words.iter().copied()) {
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

        self.adjust_estimate(raw_union_estimate, union_zeros)
    }

    #[inline(always)]
    /// Returns an estimate of the cardinality of the two HLL counters union.
    pub fn estimate_union_and_sets_cardinality(&self, other: &Self) -> EstimatedUnionCardinalities {
        let mut raw_union_estimate = 0.0;
        let mut raw_left_estimate = 0.0;
        let mut raw_right_estimate = 0.0;

        let mut union_zeros = 0;
        for (left_word, right_word) in self.words.iter().copied().zip(other.words.iter().copied()) {
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

        let union_estimate = self.adjust_estimate(raw_union_estimate, union_zeros);
        let left_estimate =
            self.adjust_estimate(raw_left_estimate, self.get_number_of_zero_registers());
        let right_estimate =
            self.adjust_estimate(raw_right_estimate, other.get_number_of_zero_registers());

        EstimatedUnionCardinalities {
            left_cardinality: left_estimate,
            right_cardinality: right_estimate,
            union_cardinality: union_estimate,
        }
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
    /// use hyperloglog_rs::HyperLogLog;
    ///
    /// let mut hll1 = HyperLogLog::<12, 6>::new();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    ///
    /// let mut hll2 = HyperLogLog::<12, 6>::new();
    /// hll2.insert(&2);
    /// hll2.insert(&3);
    ///
    /// let intersection_cardinality = hll1.estimate_intersection_cardinality(&hll2);
    ///
    /// assert!(intersection_cardinality >= 1.0 * 0.9 &&
    ///         intersection_cardinality <= 1.0 * 1.1);
    /// ```
    pub fn estimate_intersection_cardinality(&self, other: &Self) -> f32 {
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
    /// use hyperloglog_rs::HyperLogLog;
    ///
    /// let mut hll1 = HyperLogLog::<12, 6>::new();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    /// hll1.insert(&3);
    /// hll1.insert(&4);
    ///
    /// let mut hll2 = HyperLogLog::<12, 6>::new();
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
    /// use hyperloglog_rs::HyperLogLog;
    /// const PRECISION: usize = 8;
    /// const BITS: usize = 5;
    /// const HYPERLOGLOG_SIZE: usize = 1 << PRECISION;
    ///
    /// let mut hll = HyperLogLog::<PRECISION, BITS>::new();
    /// assert_eq!(hll.iter().count(), HYPERLOGLOG_SIZE);
    ///
    /// hll.insert(&"foo");
    /// hll.insert(&"bar");
    ///
    /// let mut hll2 = HyperLogLog::<PRECISION, BITS>::new();
    /// hll2|= hll;
    ///
    /// assert_eq!(hll2.iter().count(), HYPERLOGLOG_SIZE);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        debug_assert_eq!(
            self.words.len(),
            ceil(1 << PRECISION, Self::NUMBER_OF_REGISTERS_IN_WORD)
        );

        self.words
            .iter()
            .flat_map(|word| {
                (0..Self::NUMBER_OF_REGISTERS_IN_WORD)
                    .map(move |i| (word >> (i * BITS)) & Self::LOWER_REGISTER_MASK)
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
    /// # use hyperloglog_rs::HyperLogLog;
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
    /// Returns whether no element was yet added to the HLL counter.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::HyperLogLog;
    ///
    /// let mut hll: HyperLogLog<8, 8> = HyperLogLog::new();
    ///
    /// assert!(hll.is_empty());
    ///
    /// hll.insert(&1);
    ///
    /// assert!(!hll.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.number_of_zero_register == self.len()
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
    /// use hyperloglog_rs::HyperLogLog;
    ///
    /// let hll = HyperLogLog::<13, 6>::new();
    /// assert_eq!(hll.get_number_of_bits(), 6);
    /// ```
    pub const fn get_number_of_bits(&self) -> usize {
        BITS
    }

    #[inline(always)]
    /// Returns the number of extra registers that are not actually used.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hyperloglog_rs::HyperLogLog;
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
    pub const fn get_number_of_padding_registers(&self) -> usize {
        ceil(1 << PRECISION, 32 / BITS) * Self::NUMBER_OF_REGISTERS_IN_WORD
            - Self::NUMBER_OF_REGISTERS
    }

    #[inline(always)]
    /// Returns the number of registers with zero values. This value is used for computing a small
    /// correction when estimating the cardinality of a small set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hyperloglog_rs::HyperLogLog;
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
    /// Returns an array of registers of the HyperLogLog counter.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hyperloglog_rs::HyperLogLog;
    ///
    /// let mut hll = HyperLogLog::<10, 6>::new();
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
    /// # use hyperloglog_rs::HyperLogLog;
    ///
    /// let expected = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 11, 11, 0];
    /// let mut hll = HyperLogLog::<4, 6>::from_registers(&expected);
    /// assert_eq!(hll.get_registers(), expected, "Expected {:?}, got {:?}", expected, hll.get_registers());
    /// ```
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
    /// Returns the array of words of the HyperLogLog counter.
    pub fn get_words(&self) -> [u32; ceil(1 << PRECISION, 32 / BITS)] {
        self.words
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
    /// use hyperloglog_rs::HyperLogLog;
    ///
    /// let mut hll: HyperLogLog<8, 6> = HyperLogLog::new();
    /// let value = 42;
    /// let (hash, index) = hll.get_hash_and_index(&value);
    ///
    /// assert_eq!(index, 213, "Expected index {}, got {}.", 213, index);
    /// assert_eq!(hash, 15387811073369036852, "Expected hash {}, got {}.", 15387811073369036852, hash);
    /// ```
    pub fn get_hash_and_index<T: Hash>(&self, value: &T) -> (u64, usize) {
        // Create a new hasher.
        let mut hasher = SipHasher13::new();
        // Calculate the hash.
        value.hash(&mut hasher);
        let hash: u64 = hasher.finish();

        // Calculate the register's index.
        let index: usize = (hash >> (64 - PRECISION)) as usize;
        debug_assert!(
            index < Self::NUMBER_OF_REGISTERS,
            "The index {} must be less than the number of registers {}.",
            index,
            Self::NUMBER_OF_REGISTERS
        );

        (hash, index)
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
    /// # use hyperloglog_rs::HyperLogLog;
    ///
    /// let mut hll: HyperLogLog<8, 6> = HyperLogLog::new();
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
    /// Adds an element to the HyperLogLog counter.
    ///
    /// # Arguments
    /// * `rhs` - The element to add.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::HyperLogLog;
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
        let (mut hash, index) = self.get_hash_and_index(&rhs);

        // Shift left the bits of the index.
        hash = (hash << PRECISION) | (1 << (PRECISION - 1));

        // Count leading zeros.
        let number_of_zeros: u32 = 1 + hash.leading_zeros();

        // Calculate the position of the register in the internal buffer array.
        let word_position = index / Self::NUMBER_OF_REGISTERS_IN_WORD;
        let register_position_in_u32 = index - word_position * Self::NUMBER_OF_REGISTERS_IN_WORD;

        debug_assert!(
            word_position < self.words.len(),
            concat!(
                "The word_position {} must be less than the number of words {}. ",
                "You have obtained this values starting from the index {} and the word size {}."
            ),
            word_position,
            self.words.len(),
            index,
            Self::NUMBER_OF_REGISTERS_IN_WORD
        );

        // Extract the current value of the register at `index`.
        let register_value: u32 = (self.words[word_position] >> (register_position_in_u32 * BITS))
            & Self::LOWER_REGISTER_MASK;

        // Otherwise, update the register using a bit mask.
        if number_of_zeros > register_value {
            self.number_of_zero_register -= (register_value == 0) as usize;
            self.words[word_position] &=
                !(Self::LOWER_REGISTER_MASK << (register_position_in_u32 * BITS));
            self.words[word_position] |= number_of_zeros << (register_position_in_u32 * BITS);
        }
    }
}

impl<const PRECISION: usize, const BITS: usize, A: Hash> core::iter::FromIterator<A>
    for HyperLogLog<PRECISION, BITS>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Creates a new HyperLogLog counter and adds all elements from an iterator to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::HyperLogLog;
    ///
    /// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    /// let hll: HyperLogLog<12, 5> = data.iter().collect();
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
