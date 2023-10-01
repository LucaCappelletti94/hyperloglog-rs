use siphasher::sip::SipHasher13;

use crate::array_default::ArrayIter;
use crate::bias::BIAS_DATA;
use crate::estimated_union_cardinalities::EstimatedUnionCardinalities;
use core::hash::Hasher;
use crate::precisions::{Precision, WordType};
use crate::prelude::*;
use crate::prelude::{linear_counting_threshold, MaxMin};
use crate::primitive::Primitive;
use crate::raw_estimate_data::RAW_ESTIMATE_DATA;
use crate::utils::{ceil, get_alpha};
use core::hash::Hash;

pub trait HyperLogLogTrait<
    PRECISION: Precision + WordType<BITS>,
    const BITS: usize,
>: Sized
{
    /// The threshold value used in the small range correction of the HyperLogLog algorithm.
    const INTERMEDIATE_RANGE_CORRECTION_THRESHOLD: f32 =
        5.0_f32 * (PRECISION::NUMBER_OF_REGISTERS as f32);

    const LINEAR_COUNT_THRESHOLD: f32 = linear_counting_threshold(PRECISION::EXPONENT);

    /// The mask used to obtain the lower register bits in the HyperLogLog algorithm.
    const LOWER_REGISTER_MASK: u32 = (1 << BITS) - 1;

    /// The mask used to obtain the lower precision bits in the HyperLogLog algorithm.
    const LOWER_PRECISION_MASK: usize = PRECISION::NUMBER_OF_REGISTERS - 1;
    const NOT_LOWER_PRECISION_MASK: u64 = !Self::LOWER_PRECISION_MASK as u64;

    /// The mask representing the bits that are never used in the u32 word in the cases
    /// where the number of bits is not a divisor of 32, such as 5 or 6.
    /// We set the LEADING bits as the padding bits, the unused one, so the leftmost bits.
    const PADDING_BITS_MASK: u32 =
        !((1_u64 << (BITS * Self::NUMBER_OF_REGISTERS_IN_WORD)) - 1_u64) as u32;

    /// The mask used to obtain the upper precision bits in the HyperLogLog algorithm.
    const UPPER_PRECISION_MASK: usize = Self::LOWER_PRECISION_MASK << (64 - PRECISION::EXPONENT);

    /// The number of registers that can fit in a single 32-bit word in the HyperLogLog algorithm.
    const NUMBER_OF_REGISTERS_IN_WORD: usize = 32 / BITS;

    /// Create a new HyperLogLog counter.
    fn new() -> Self;

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
    fn from_registers(registers: &[u32]) -> Self;

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
    fn from_words(words: &PRECISION::Words) -> Self;

    fn adjust_estimate(mut raw_estimate: f32) -> f32 {
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

    fn adjust_estimate_with_zeros(raw_estimate: f32, number_of_zeros: usize) -> f32 {
        if number_of_zeros > 0 {
            let low_range_correction = PRECISION::SMALL_CORRECTIONS[number_of_zeros - 1];
            if low_range_correction <= Self::LINEAR_COUNT_THRESHOLD {
                return low_range_correction;
            }
        }
        Self::adjust_estimate(raw_estimate)
    }

    /// Returns whether the cardinality of this HLL will be computed using the small-range correction.
    ///
    /// # Implementation details
    /// The small-range correction is used when the cardinality of the set is small enough that the
    /// linear counting algorithm can be used to estimate the cardinality. The threshold for using
    /// the linear counting algorithm is determined by the number of registers in the HLL counter.
    fn use_small_range_correction(&self) -> bool {
        self.get_number_of_zero_registers() > 0
            && PRECISION::SMALL_CORRECTIONS[self.get_number_of_zero_registers() - 1]
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
    fn estimate_cardinality(&self) -> f32 {
        if self.get_number_of_zero_registers() > 0 {
            let low_range_correction =
                PRECISION::SMALL_CORRECTIONS[self.get_number_of_zero_registers() - 1];
            if low_range_correction <= Self::LINEAR_COUNT_THRESHOLD {
                return low_range_correction;
            }
        }

        let mut raw_estimate = 0.0;

        for &word in self.get_words().iter_elements() {
            let mut partial: f32 = 0.0;
            for i in 0..Self::NUMBER_OF_REGISTERS_IN_WORD {
                let register = (word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                let two_to_minus_register = (127 - register) << 23;
                partial += f32::from_le_bytes(two_to_minus_register.to_le_bytes());
            }
            raw_estimate += partial;
        }

        raw_estimate -= Self::get_number_of_padding_registers() as f32;

        Self::adjust_estimate(raw_estimate)
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
    fn estimate_union_cardinality(&self, other: &Self) -> f32 {
        self.estimate_union_and_sets_cardinality(other)
            .get_union_cardinality()
    }

    #[inline(always)]
    /// Returns an estimate of the cardinality of the two HLL counters union.
    fn estimate_union_and_sets_cardinality<F: Primitive<f32> + MaxMin>(
        &self,
        other: &Self,
    ) -> EstimatedUnionCardinalities<F> {
        let mut raw_union_estimate = 0.0;
        let mut raw_left_estimate = 0.0;
        let mut raw_right_estimate = 0.0;

        let mut union_zeros = 0;
        for (left_word, right_word) in self
            .get_words()
            .iter_elements()
            .copied()
            .zip(other.get_words().iter_elements().copied())
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

        union_zeros -= Self::get_number_of_padding_registers();

        // We need to subtract the padding registers from the raw estimates
        // as for each such register we are adding a one.
        raw_union_estimate -= Self::get_number_of_padding_registers() as f32;
        raw_left_estimate -= Self::get_number_of_padding_registers() as f32;
        raw_right_estimate -= Self::get_number_of_padding_registers() as f32;

        let mut union_estimate = F::reverse(Self::adjust_estimate_with_zeros(
            raw_union_estimate,
            union_zeros,
        ));
        let left_estimate = F::reverse(Self::adjust_estimate_with_zeros(
            raw_left_estimate,
            self.get_number_of_zero_registers(),
        ));
        let right_estimate = F::reverse(Self::adjust_estimate_with_zeros(
            raw_right_estimate,
            other.get_number_of_zero_registers(),
        ));

        // The union estimate cannot be higher than the sum of the left and right estimates.
        union_estimate = union_estimate.get_min(left_estimate + right_estimate);

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
    fn estimate_intersection_cardinality<F: Primitive<f32>>(&self, other: &Self) -> F {
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
    /// let jaccard_index = hll1.estimate_jaccard_index(&hll2);
    ///
    /// let expected = 2.0 / 6.0;
    ///
    /// assert!(jaccard_index >= expected * 0.9 &&
    ///         jaccard_index <= expected * 1.1);
    /// ```
    fn estimate_jaccard_index(&self, other: &Self) -> f32 {
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
    fn estimate_difference_cardinality<F: Primitive<f32>>(&self, other: &Self) -> F {
        self.estimate_union_and_sets_cardinality(other)
            .get_left_difference_cardinality()
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
    fn len(&self) -> usize {
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
    fn is_empty(&self) -> bool {
        self.len() == self.get_number_of_zero_registers()
    }

    #[inline(always)]
    /// Returns the number of extra registers that are not actually used.
    ///
    /// # Examples
    /// Since the number of registers is not a multiple of the number of registers in a word,
    /// there are padding registers that are not actually used.
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// assert_eq!(HyperLogLog::<Precision10, 6>::get_number_of_padding_registers(), 1);
    /// ```
    ///
    /// For instance, in the case using the bare minimum bits per registers (4)
    /// and the minimal precision (4), for a total of 16 registers, we expect
    /// to not have any padding registers.
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// assert_eq!(HyperLogLog::<Precision4, 4>::get_number_of_padding_registers(), 0);
    ///
    /// ```
    ///
    fn get_number_of_padding_registers() -> usize {
        ceil(PRECISION::NUMBER_OF_REGISTERS, 32 / BITS) * Self::NUMBER_OF_REGISTERS_IN_WORD
            - PRECISION::NUMBER_OF_REGISTERS
    }

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
    fn get_number_of_zero_registers(&self) -> usize;

    #[inline(always)]
    fn get_number_of_non_zero_registers(&self) -> usize {
        // Calculates the number of registers that have a non-zero value by
        // subtracting the number of registers with a zero value from the total number of registers
        self.len() - self.get_number_of_zero_registers()
    }

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
    fn get_registers(&self) -> PRECISION::Registers {
        let mut registers = PRECISION::Registers::default_array();
        self.get_words()
            .iter_elements()
            .flat_map(|word| {
                (0..Self::NUMBER_OF_REGISTERS_IN_WORD)
                    .map(move |i: usize| (word >> (i * BITS)) & Self::LOWER_REGISTER_MASK)
            })
            .zip(registers.iter_elements_mut())
            .for_each(|(value, cell): (u32, &mut u32)| {
                *cell = value;
            });
        registers
    }

    /// Returns the array of words of the HyperLogLog counter.
    fn get_words(&self) -> &PRECISION::Words;

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
    fn may_contain<T: Hash>(&self, rhs: &T) -> bool {
        let (_hash, index) = self.get_hash_and_index::<T>(rhs);

        // Calculate the position of the register in the internal buffer array.
        let word_position = index / Self::NUMBER_OF_REGISTERS_IN_WORD;

        // Calculate the position of the register within the 32-bit word containing it.
        let register_position_in_u32 = index % Self::NUMBER_OF_REGISTERS_IN_WORD;

        // Extract the current value of the register at `index`.
        let register_value: u32 = (self.get_words()[word_position]
            >> (register_position_in_u32 * BITS))
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
    fn may_contain_all(&self, rhs: &Self) -> bool {
        for (left_word, right_word) in self
            .get_words()
            .iter_elements()
            .copied()
            .zip(rhs.get_words().iter_elements().copied())
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
    fn get_hash_and_index<T: Hash>(&self, value: &T) -> (u64, usize) {
        let mut hasher = SipHasher13::new();
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
    fn insert<T: Hash>(&mut self, rhs: T);
}