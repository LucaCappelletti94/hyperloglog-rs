//! Marker struct for the hybrid approach, that keeps the hash explicit up until they fit into the registers.

use crate::composite_hash::{GapHash, SaturationError};
use crate::correction_coefficients::{
    HASHLIST_CORRECTION_BIAS, HASHLIST_CORRECTION_CARDINALITIES, HYPERLOGLOG_CORRECTION_BIAS,
    HYPERLOGLOG_CORRECTION_CARDINALITIES, HYPERLOGLOG_CORRECTION_SLOPES,
};
use crate::prelude::*;
use core::f64;
use core::fmt::Debug;
use core::hash::Hash;
use core::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A struct representing the hybrid for approximate set cardinality estimation,
/// where the hash values are kept explicit up until they fit into the registers.
pub struct HyperLogLog<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    Hasher: HasherType = twox_hash::XxHash64,
> {
    /// The registers of the counter.
    pub(crate) registers: R,
    /// The harmonic sum of the registers, i.e. the sum of 2^(-register_value) for all registers.
    pub(crate) harmonic_sum: f64,
    /// Phantom data to ensure the type parameters are used.
    _phantom: PhantomData<(P, B, Hasher)>,
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> Default for HyperLogLog<P, B, R, H> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn correction_upper_bound<P: Precision>() -> f64 {
    f64::INFINITY
}

#[inline]
/// Returns the corrected estimate of the cardinality.
fn correct_cardinality<P: Precision, B: Bits>(
    raw_estimate: f64,
    cardinalities: &[[&[u32]; 3]; 6],
    biases: &[[&[f64]; 3]; 6],
) -> f64 {
    if raw_estimate >= correction_upper_bound::<P>() {
        return raw_estimate;
    }

    let cardinalities = &cardinalities[P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4];
    let biases = &biases[P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4];
    let estimate_u32 = raw_estimate as u32;

    if estimate_u32 <= cardinalities[0] {
        return raw_estimate + biases[0] * raw_estimate / f64::from(cardinalities[0]).max(1.0);
    }

    if estimate_u32 > cardinalities[cardinalities.len() - 1] {
        return raw_estimate
            + biases[cardinalities.len() - 1] * raw_estimate
                / f64::from(cardinalities[cardinalities.len() - 1]);
    }

    // We use a binary-search-based partition search to find the point where the raw estimate is
    // located in the cardinalities.

    let index = cardinalities.partition_point(|&x| x < estimate_u32);

    let lower_cardinality = cardinalities[index - 1];
    let upper_cardinality = cardinalities[index];

    let lower_bias = biases[index - 1];
    let upper_bias = biases[index];

    assert!(lower_cardinality < upper_cardinality);

    raw_estimate
        + (raw_estimate - f64::from(lower_cardinality))
            / f64::from(upper_cardinality - lower_cardinality)
            * (upper_bias - lower_bias)
        + lower_bias
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperLogLog<P, B, R, H> {
    #[inline]
    fn new() -> Self {
        let mut hll = Self {
            registers: R::default(),
            harmonic_sum: f64::NEG_INFINITY,
            _phantom: PhantomData,
        };

        hll.clear();

        hll
    }

    #[inline]
    /// Returns whether the counter is empty.
    pub fn is_empty(&self) -> bool {
        self.is_hash_list() && self.get_number_of_hashes() == 0
    }

    #[inline]
    /// Returns whether the counter is fully saturated.
    pub fn is_full(&self) -> bool {
        // The harmonic sum is defined as Sum(2^(-register_value)) for all registers.
        // When all registers are maximally filled, i.e. equal to the maximal multiplicity value,
        // the harmonic sum is equal to (2^(-max_multiplicity)) * number_of_registers.
        // Since number_of_registers is a power of 2, specifically 2^exponent, the harmonic sum
        // is equal to 2^(exponent - max_multiplicity).
        !self.is_hash_list()
            && self.harmonic_sum
                <= f64::integer_exp2_minus_signed(
                    (1_i16 << B::NUMBER_OF_BITS) - i16::from(P::EXPONENT) - 1,
                )
    }

    #[inline]
    /// Returns whether the provided element may be contained in the counter.
    pub fn may_contain<T: Hash>(&self, element: &T) -> bool {
        let (index, register, original_hash) = Self::index_and_register_and_hash(element);
        if self.is_hash_list() {
            GapHash::<P, B>::find(
                self.registers.as_ref(),
                self.get_number_of_hashes(),
                index,
                register,
                original_hash,
                self.get_hash_bits(),
                self.get_writer_tell(),
            )
        } else {
            self.registers.get_register(index) >= register
        }
    }

    #[inline]
    /// Returns whether the counter is in hybrid mode.
    pub fn is_hash_list(&self) -> bool {
        self.harmonic_sum.to_bits().leading_zeros() == 0
    }

    #[inline]
    fn clear(&mut self) {
        self.registers.clear_registers();
        self.harmonic_sum = f64::NEG_INFINITY;
        self.set_number_of_hashes(0);
        self.set_writer_tell(0);
        self.set_duplicates(0);
        self.set_hash_bits(GapHash::<P, B>::LARGEST_VIABLE_HASH_BITS);
        debug_assert!(self.is_hash_list());
    }

    #[inline]
    /// Inserts an element into the counter.
    pub fn insert<T: Hash>(&mut self, element: &T) -> bool {
        let (index, register, original_hash) = Self::index_and_register_and_hash(element);
        if self.is_hash_list() {
            let hash_bits = self.get_hash_bits();
            let number_of_hashes = self.get_number_of_hashes();
            let writer_tell = self.get_writer_tell();

            match GapHash::<P, B>::insert_sorted_desc(
                self.registers.as_mut(),
                number_of_hashes,
                writer_tell,
                index,
                register,
                original_hash,
                hash_bits,
            ) {
                Ok(Some(insert_metadata)) => {
                    self.set_number_of_hashes(number_of_hashes + 1 - insert_metadata.duplicates);
                    self.set_writer_tell(insert_metadata.bit_index);
                    self.add_duplicates(insert_metadata.duplicates);
                    self.set_hash_bits(insert_metadata.hash_bits);
                    true
                }
                Ok(None) => false,
                Err(err) => match err {
                    SaturationError::ExtendableSaturation => {
                        self.registers.increase_capacity();
                        self.insert(element)
                    }
                    SaturationError::Saturation(bit_index) => {
                        self.set_writer_tell(bit_index);
                        debug_assert_eq!(bit_index, self.get_writer_tell());
                        self.convert_hashlist_to_hyperloglog();
                        debug_assert!(!self.is_hash_list());
                        self.insert(element)
                    }
                },
            }
        } else {
            self.insert_register_value_and_index(register, index)
        }
    }

    #[inline]
    /// Converts the Hybrid counter to a regular [`HyperLogLog`] counter.
    fn convert_hashlist_to_hyperloglog(&mut self) {
        debug_assert!(self.is_hash_list());

        let current_cardinality = self.estimate_cardinality();
        let hash_bits = self.get_hash_bits();
        let mut new_registers = self.registers.clone();
        new_registers.clear_registers();
        let registers = core::mem::replace(&mut self.registers, new_registers);
        let number_of_hashes = self.get_number_of_hashes();
        let writer_tell = self.get_writer_tell();
        self.harmonic_sum = f64::integer_exp2(P::EXPONENT);

        debug_assert!(
            P::EXPONENT ==  4 && B::NUMBER_OF_BITS == 4 || GapHash::<P, B>::is_prefix_free_encoded(number_of_hashes, hash_bits, writer_tell),
            "Downgrading hash list to HyperLogLog counter with non-prefix-free encoding. Number of hashes: {number_of_hashes}, hash bits: {hash_bits}, writer tell: {writer_tell}, precision: {}, bits: {}.",
            P::EXPONENT,
            B::NUMBER_OF_BITS
        );

        let mut last_index = usize::MAX;
        GapHash::<P, B>::decoded(registers.as_ref(), number_of_hashes, hash_bits, writer_tell)
            .for_each(|(new_register_value, index)| {
                if last_index == index {
                    return;
                }
                last_index = index;
                self.insert_register_value_and_index(new_register_value, index);
            });

        // We set the harmonic sum precisely to the current cardinality, so that it is primed
        // to be more precise than what would otherwise be obtained by using directly the registers
        // without this additional information.
        let expected_harmonic_sum =
            P::ALPHA * f64::integer_exp2(2 * P::EXPONENT) / current_cardinality;
        // While the expected harmonic sum is greater than the current harmonic sum, we can adjust
        // it by inserting fake elements.
        while expected_harmonic_sum < self.harmonic_sum {
            // To do so in the most granular manner, we search for the smallest register value and
            // increase it by one.
            let mut smallest_register_value = u8::MAX;
            let mut smallest_index = usize::MAX;
            self.registers
                .iter_registers()
                .enumerate()
                .for_each(|(index, register_value)| {
                    if register_value < smallest_register_value {
                        smallest_register_value = register_value;
                        smallest_index = index;
                    }
                });
            // We increase the smallest register value by one.
            self.insert_register_value_and_index(smallest_register_value + 1, smallest_index);
        }

        debug_assert!(self.harmonic_sum.is_finite());
    }

    #[inline]
    /// Splits a hash into a register value and an index.
    fn insert_register_value_and_index(&mut self, new_register_value: u8, index: usize) -> bool {
        // Count leading zeros.
        debug_assert!(
            new_register_value <= u8::try_from(B::MASK).unwrap(),
            "Register value is too large: {new_register_value} > {}",
            B::MASK
        );
        debug_assert!(
            new_register_value > 0,
            "Register value is zero, which is not allowed."
        );

        let (old_register_value, larger_register_value) =
            self.registers.set_greater(index, new_register_value);

        self.harmonic_sum += f64::integer_exp2_minus(larger_register_value)
            - f64::integer_exp2_minus(old_register_value);

        debug_assert!(self.harmonic_sum.is_finite());
        debug_assert!(self.harmonic_sum > 0.0);

        old_register_value < new_register_value
    }

    #[inline]
    /// Returns the uncorrected estimate of the cardinality.
    pub fn uncorrected_estimate_cardinality(&self) -> f64 {
        if self.is_hash_list() {
            f64::from(self.get_number_of_hashes() + self.get_duplicates())
        } else {
            P::ALPHA * f64::integer_exp2(P::EXPONENT + P::EXPONENT) / self.harmonic_sum
        }
    }

    #[inline]
    /// Returns the corrected estimate of the cardinality.
    pub fn estimate_cardinality(&self) -> f64 {
        if self.is_hash_list() {
            correct_cardinality::<P, B>(
                f64::from(self.get_number_of_hashes() + self.get_duplicates()),
                &HASHLIST_CORRECTION_CARDINALITIES,
                &HASHLIST_CORRECTION_BIAS,
            )
        } else {
            let raw_estimate =
                P::ALPHA * f64::integer_exp2(P::EXPONENT + P::EXPONENT) / self.harmonic_sum;

            if raw_estimate >= correction_upper_bound::<P>() {
                return raw_estimate;
            }

            let cardinalities = HYPERLOGLOG_CORRECTION_CARDINALITIES[P::EXPONENT as usize - 4]
                [B::NUMBER_OF_BITS as usize - 4];
            let biases = HYPERLOGLOG_CORRECTION_BIAS[P::EXPONENT as usize - 4]
                [B::NUMBER_OF_BITS as usize - 4];
            let last_cardinality = f64::from(cardinalities[cardinalities.len() - 1]);
            let last_bias = biases[biases.len() - 1];
            if raw_estimate >= last_cardinality {
                let angular_coefficient = HYPERLOGLOG_CORRECTION_SLOPES[P::EXPONENT as usize - 4]
                    [B::NUMBER_OF_BITS as usize - 4];

                let delta = raw_estimate - last_cardinality;
                return last_cardinality
                    + last_bias
                    + (delta + last_bias * delta / last_cardinality) * angular_coefficient;
            }

            correct_cardinality::<P, B>(
                raw_estimate,
                &HYPERLOGLOG_CORRECTION_CARDINALITIES,
                &HYPERLOGLOG_CORRECTION_BIAS,
            )
        }
    }

    #[inline]
    /// Returns whether the provided [`HyperLogLog`] counter may be fully contained in the current [`HyperLogLog`] counter.
    ///
    /// # Arguments
    /// * `rhs` - The [`HyperLogLog`] counter to check.
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
    /// let mut hll1: PlusPlus<Precision8, Bits6, <Precision8 as PackedRegister<Bits6>>::Array> =
    ///     Default::default();
    /// let mut hll2: PlusPlus<Precision8, Bits6, <Precision8 as PackedRegister<Bits6>>::Array> =
    ///     Default::default();
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
        self.registers
            .iter_registers_zipped(&rhs.registers)
            .all(|[left_register, right_register]| left_register >= right_register)
    }

    #[inline]
    /// Hashes the element and returns the register value and the index of the register.
    pub fn index_and_register_and_hash<T: Hash>(element: &T) -> (usize, u8, u64) {
        let mut hasher = H::default();
        element.hash(&mut hasher);
        let hash = hasher.finish();

        let index: usize = usize::try_from(hash & ((1 << P::EXPONENT) - 1)).unwrap();

        debug_assert!(
            index < 1 << P::EXPONENT,
            "The index {index} must be less than the number of registers {}.",
            1 << P::EXPONENT
        );

        // And we censor we just used for the index.
        let mut censored_hash: u64 = hash | 1 << P::EXPONENT;

        // We need to add ones to the hash to make sure that the
        // the number of zeros we obtain afterwards is never higher
        // than the maximal value that may be represented in a register
        // with BITS bits.
        if <B as VariableWord>::NUMBER_OF_BITS < 6_u8 {
            censored_hash |= 1_u64 << (64_u64 - <B as VariableWord>::MASK);
        }

        let register_value = u8::try_from(censored_hash.leading_zeros() + 1).unwrap();

        debug_assert!(
            register_value <= u8::try_from(<B as VariableWord>::MASK).unwrap(),
            "The register value {} must be less than or equal to the maximum register value {}.",
            register_value,
            (1 << <B as VariableWord>::NUMBER_OF_BITS) - 1
        );

        debug_assert!(
            register_value > 0,
            "The register value must be greater than zero."
        );

        (index, register_value, hash)
    }

    #[inline]
    /// Returns an estimate of the intersection cardinality between two counters.
    pub fn estimate_intersection_cardinality(&self, other: &Self) -> f64 {
        let self_cardinality = self.estimate_cardinality();
        let other_cardinality = other.estimate_cardinality();
        let union_cardinality = self.estimate_union_cardinality_with_cardinalities(
            other,
            self_cardinality,
            other_cardinality,
        );

        // We apply correction to the union cardinality to get the intersection cardinality.
        if self_cardinality + other_cardinality < union_cardinality {
            0.0
        } else {
            self_cardinality + other_cardinality - union_cardinality
        }
    }

    #[inline]
    /// Returns an estimate of the Jaccard index between two counters.
    pub fn estimate_jaccard_index(&self, other: &Self) -> f64 {
        let self_cardinality = self.estimate_cardinality();
        let other_cardinality = other.estimate_cardinality();
        let union_cardinality = self.estimate_union_cardinality_with_cardinalities(
            other,
            self_cardinality,
            other_cardinality,
        );

        // We apply correction to the union cardinality to get the intersection cardinality.
        if self_cardinality + other_cardinality < union_cardinality || union_cardinality.is_zero() {
            0.0
        } else {
            (self_cardinality + other_cardinality - union_cardinality) / union_cardinality
        }
    }

    #[inline]
    /// Returns an estimate of the cardinality of the current counter minus the cardinality of the other counter.
    pub fn estimate_difference_cardinality(&self, other: &Self) -> f64 {
        let union_cardinality = self.estimate_union_cardinality(other);
        let other_cardinality = other.estimate_cardinality();
        if union_cardinality < other_cardinality {
            0.0
        } else {
            union_cardinality - other_cardinality
        }
    }

    #[inline]
    /// Returns the estimate of the cardinality of the union of two [`HyperLogLog`] counters.
    pub fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        self.estimate_union_cardinality_with_cardinalities(
            other,
            self.estimate_cardinality(),
            other.estimate_cardinality(),
        )
    }

    #[inline]
    fn estimate_union_cardinality_with_cardinalities(
        &self,
        other: &Self,
        self_cardinality: f64,
        other_cardinality: f64,
    ) -> f64 {
        match (self.is_hash_list(), other.is_hash_list()) {
            (true, true) => {
                let left_hash_bits = self.get_hash_bits();
                let right_hash_bits = other.get_hash_bits();
                assert!(left_hash_bits >= GapHash::<P, B>::SMALLEST_VIABLE_HASH_BITS);
                assert!(right_hash_bits >= GapHash::<P, B>::SMALLEST_VIABLE_HASH_BITS);

                let left_shift = if left_hash_bits <= right_hash_bits {
                    0
                } else {
                    left_hash_bits - right_hash_bits
                };
                let right_shift = if right_hash_bits <= left_hash_bits {
                    0
                } else {
                    right_hash_bits - left_hash_bits
                };

                let left_hashes = self.registers.as_ref();
                let right_hashes = other.registers.as_ref();
                let left_bit_index = self.get_writer_tell();
                let right_bit_index = other.get_writer_tell();

                let intersection_cardinality = f64::from(intersection_from_sorted_iterators(
                    GapHash::<P, B>::downgraded(
                        left_hashes,
                        self.get_number_of_hashes(),
                        left_hash_bits,
                        left_bit_index,
                        left_shift,
                    ),
                    GapHash::<P, B>::downgraded(
                        right_hashes,
                        other.get_number_of_hashes(),
                        right_hash_bits,
                        right_bit_index,
                        right_shift,
                    ),
                ));

                let union_cardinality =
                    self_cardinality + other_cardinality - intersection_cardinality;

                correct_union_estimate(self_cardinality, other_cardinality, union_cardinality)
            }
            (true, false) => {
                let hash_bits = self.get_hash_bits();
                assert!(hash_bits >= GapHash::<P, B>::SMALLEST_VIABLE_HASH_BITS);

                self.union_estimation_from_sorted_iterator_and_counter(
                    other,
                    self_cardinality,
                    other_cardinality,
                )
            }
            (false, true) => other.estimate_union_cardinality_with_cardinalities(
                self,
                self_cardinality,
                other_cardinality,
            ),
            (false, false) => {
                let union_estimate = correct_cardinality::<P, B>(
                    P::ALPHA * f64::integer_exp2(P::EXPONENT + P::EXPONENT)
                        / self.registers.get_union_harmonic_sum(&other.registers),
                    &HYPERLOGLOG_CORRECTION_CARDINALITIES,
                    &HYPERLOGLOG_CORRECTION_BIAS,
                );
                correct_union_estimate(self_cardinality, other_cardinality, union_estimate)
            }
        }
    }
}

#[cfg(test)]
mod test_hybrid_propertis {
    use super::*;
    use hyperloglog_derive::test_estimator;
    use twox_hash::XxHash;

    #[test_estimator]
    fn test_plusplus_properties<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>() {
        let mut hybrid: HyperLogLog<P, B, R, H> = Default::default();
        assert!(hybrid.is_hash_list());
        assert!(hybrid.is_empty());
        assert!(!hybrid.is_full());
        assert_eq!(hybrid.get_number_of_hashes(), 0);
        let mut normalized_error = 0.0;
        let mut non_normalized_error = 0.0;
        let mut random_state = 34567897654354_u64;
        let mut iterations = 0;

        while hybrid.is_hash_list() {
            iterations += 1;
            // To make the test a bit fairer using more random elements
            // than a numerical sequence.
            random_state = splitmix64(splitmix64(random_state));
            hybrid.insert(&random_state);
            assert!(
                !hybrid.insert(&random_state),
                "The Hybrid counter should NOT already contain the element {random_state}. Hash size: {}. Iteration n. {iterations}. Hash list status: {}",
                hybrid.get_hash_bits(),
                hybrid.is_hash_list()
            );
            assert!(
                hybrid.may_contain(&random_state),
                "The Hybrid counter must contain the element {random_state}. Iteration n. {iterations}.",
            );

            let estimated_cardinality = hybrid.estimate_cardinality();

            let error = iterations as f64 - estimated_cardinality;
            non_normalized_error += error;
            normalized_error += error / iterations as f64;
        }

        normalized_error /= iterations as f64;
        non_normalized_error /= iterations as f64;

        assert!(
            normalized_error <= P::error_rate() / 13.0,
            "The normalized error rate ({normalized_error}, {non_normalized_error}) must be less than or equal to the error rate ({}).",
            P::error_rate()
        );

        assert!(!hybrid.is_hash_list());
    }
}
