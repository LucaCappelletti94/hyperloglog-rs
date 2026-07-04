//! Methods relative to the sorted hash list.
mod float_bits_ops;
use crate::{
    composite_hash::GapHash,
    prelude::{Bits, HasherType, HyperLogLog, Precision, Registers},
    utils::{correct_union_estimate, FloatOps},
};
use sketching_core::sparse_value_list::SparseValueCodec;

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> HyperLogLog<P, B, R, H, C>
where
    C: SparseValueCodec,
{
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
        // `self` is the sorted hash list (whose `harmonic_sum` field is a repurposed metadata word,
        // not a real sum) and `other` is the HyperLogLog register counter. Start from the registers'
        // actual harmonic sum and zero-register count, then raise the sum (and drop a zero) wherever a
        // hash in `self` decodes to a higher register rank than `other` holds at that index (the union
        // register is the element-wise maximum). This reconstructs the union's harmonic sum and zero
        // count exactly as if both operands were registers.
        let mut harmonic_sum = other.dense_harmonic_sum();
        let mut union_zeros = other
            .number_of_zero_registers()
            .expect("`other` is a HyperLogLog register counter");
        // We set the previous index to the NUMBER OF REGISTERS, which is a value higher
        // than the maximal possible index, so that the first value is always considered
        // as a new value.
        let mut previous_index = usize::MAX;

        for (left_register_value, index) in GapHash::<P, B>::decoded(
            self.registers.as_ref(),
            self.get_number_of_hashes().unwrap(),
            self.get_hash_bits().unwrap(),
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
            if right_register_value == 0 {
                union_zeros -= 1;
            }
            harmonic_sum += f64::integer_exp2_minus(left_register_value)
                - f64::integer_exp2_minus(right_register_value);
        }

        // Apply the same linear-counting/bias correction the both-registers union path applies,
        // rather than the badly-biased raw register estimate (which at low union load overshoots so
        // far it is clamped to `left + right`, silently discarding the overlap).
        correct_union_estimate(
            left_cardinality,
            right_cardinality,
            Self::corrected_register_cardinality(harmonic_sum, union_zeros),
        )
    }
}
