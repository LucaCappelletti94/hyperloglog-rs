//!
mod float_bits_ops;
use crate::{
    composite_hash::{CompositeHash, GapHash},
    prelude::{Bits, HasherType, HyperLogLog, Precision, Registers},
    utils::{correct_union_estimate, FloatOps},
};

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperLogLog<P, B, R, H> {
    #[inline]
    /// Returns the union estimation from a decreasingly sorted iterator and a counter.
    ///
    /// # Implementative details
    /// The provided iterator is expected to be sorted in ascending order,
    /// in such a way that hash values that point to the same index are contiguos,
    /// and ordered by value of the register as well.
    pub(crate) fn union_estimation_from_sorted_iterator_and_counter(
        &self,
        other: &Self,
        left_cardinality: f64,
        right_cardinality: f64,
    ) -> f64 {
        let mut harmonic_sum = self.harmonic_sum;
        // We set the previous index to the NUMBER OF REGISTERS, which is a value higher
        // than the maximal possible index, so that the first value is always considered
        // as a new value.
        let mut previous_index = usize::MAX;

        for (left_register_value, index) in GapHash::<P, B>::decoded(
            self.registers.as_ref(),
            self.get_number_of_hashes(),
            self.get_hash_bits(),
            self.get_writer_tell(),
        ) {
            debug_assert!(
            index <= previous_index || previous_index == usize::MAX,
            "The index must be smaller than or equal to the previous index, but got {index} and {previous_index}",
        );

            // If the index is the same as the previous index, we skip the value
            // as the register value is necessarily less or equal to the previous one.
            if index == previous_index {
                continue;
            }

            // We update the previous index.
            previous_index = index;
            // Otherwise, we update the number of zeros and the harmonic sum.
            let right_register_value = other.registers.get_register(index);

            if left_register_value <= right_register_value {
                continue;
            }

            // If the right register value is a zero, we are surely now removing
            // it because the left register value cannot be a zero.
            harmonic_sum += f64::integer_exp2_minus(left_register_value)
                - f64::integer_exp2_minus(right_register_value);
        }

        correct_union_estimate(
            left_cardinality,
            right_cardinality,
            P::ALPHA * f64::integer_exp2(P::EXPONENT + P::EXPONENT) / harmonic_sum,
        )
    }
}
