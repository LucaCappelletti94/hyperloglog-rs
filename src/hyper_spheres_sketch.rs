//! Exact sketching algorithms.
//!
//! This submodule contains the implementation of the exact sketching algorithms
//! as part of a trait.
//!
//! A sketch is a representation of the similarity between two list of sets.
//!
//! It is used in cases such as in graphs for representing the similarity between
//! two nodes, de facto providing features that characterize a candidate edge
//! between two nodes.
//!
//! While in the HyperLogLog case we provide the approximated version of this algorithm,
//! sometimes it is necessary, such as in test cases, to have the exact version of the
//! algorithm. The approximated version is faster and uses less memory, but it is not,
//! of course, guaranteed to be exact. Learn more about the approximated version in the
//! [HyperLogLog.estimated_overlap_and_differences_cardinality_matrices] method.
//!
use crate::prelude::*;
use core::iter::Sum;
use core::ops::AddAssign;
use core::ops::{DivAssign, MulAssign, SubAssign};

pub trait SetLike<I> {
    /// Returns the estimated intersection and left and right difference cardinality between two sets.
    fn get_estimated_union_cardinality(
        &self,
        self_cardinality: I,
        other: &Self,
        other_cardinality: I,
    ) -> EstimatedUnionCardinalities<I>;

    /// Returns the cardinality of the set.
    fn get_cardinality(&self) -> I;
}

impl<F: Primitive<f32>, PRECISION: Precision + WordType<BITS>, const BITS: usize> SetLike<F>
    for HyperLogLog<PRECISION, BITS>
{
    fn get_estimated_union_cardinality(
        &self,
        self_cardinality: F,
        other: &Self,
        other_cardinality: F,
    ) -> EstimatedUnionCardinalities<F> {
        let mut raw_union_estimate = 0.0;

        let mut union_zeros = 0;
        for (left_word, right_word) in self
            .get_words()
            .iter_elements()
            .copied()
            .zip(other.get_words().iter_elements().copied())
        {
            let mut union_partial: f32 = 0.0;
            for i in 0..Self::NUMBER_OF_REGISTERS_IN_WORD {
                let left_register = (left_word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                let right_register = (right_word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                let maximal_register = (left_register).max(right_register);
                union_partial += f32::from_le_bytes(((127 - maximal_register) << 23).to_le_bytes());
                union_zeros += (maximal_register == 0) as usize;
            }
            raw_union_estimate += union_partial;
        }

        union_zeros -= Self::get_number_of_padding_registers();

        // We need to subtract the padding registers from the raw estimates
        // as for each such register we are adding a one.
        raw_union_estimate -= Self::get_number_of_padding_registers() as f32;

        let union_estimate = F::reverse(Self::adjust_estimate_with_zeros(
            raw_union_estimate,
            union_zeros,
        ));

        EstimatedUnionCardinalities::from((self_cardinality, other_cardinality, union_estimate))
    }

    fn get_cardinality(&self) -> F {
        F::reverse(self.estimate_cardinality())
    }
}

pub trait HyperSpheresSketch: Sized {
    /// Returns the overlap and differences cardinality matrices of two lists of sets.
    ///
    /// # Arguments
    /// * `left` - The first list of sets.
    /// * `right` - The second list of sets.
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
    fn overlap_and_differences_cardinality_matrices<
        I: Copy
            + Default
            + Primitive<f32>
            + core::ops::Add<Output = I>
            + core::ops::Sub<Output = I>
            + core::ops::Div<Output = I>
            + core::ops::Mul<Output = I>
            + Sum
            + Send
            + Sync
            + AddAssign
            + SubAssign
            + MulAssign
            + DivAssign
            + MaxMin,
        const L: usize,
        const R: usize,
    >(
        left: &[Self; L],
        right: &[Self; R],
    ) -> ([[I; R]; L], [I; L], [I; R])
    where
        Self: SetLike<I>,
    {
        // Initialize overlap and differences cardinality matrices/vectors.
        let mut last_row = [I::default(); R];
        let mut differential_overlap_cardinality_matrix = [[I::default(); R]; L];
        let mut left_difference_cardinality_vector = [I::default(); L];
        let mut right_cardinalities = [I::default(); R];

        right.iter().zip(right_cardinalities.iter_mut()).for_each(
            |(right_hll, right_cardinality)| {
                *right_cardinality = right_hll.get_cardinality();
            },
        );

        let mut right_difference_cardinality_vector = [I::default(); R];
        let mut euc: EstimatedUnionCardinalities<I> = EstimatedUnionCardinalities::default();
        let mut last_left_difference: I = I::default();

        // Populate the overlap cardinality matrix.
        for (i, left_hll) in left.iter().enumerate() {
            let mut last_right_difference: I = I::default();
            let left_cardinality = left_hll.get_cardinality();
            let mut comulative_row = I::default();
            for (j, (right_hll, right_cardinality)) in
                right.iter().zip(right_cardinalities).enumerate()
            {
                euc = left_hll.get_estimated_union_cardinality(
                    left_cardinality,
                    right_hll,
                    right_cardinality,
                );
                let delta = last_row[j] + comulative_row;
                differential_overlap_cardinality_matrix[i][j] =
                    (euc.get_intersection_cardinality() - delta).get_max(I::default());
                last_row[j] = euc.get_intersection_cardinality().get_max(delta);
                comulative_row += differential_overlap_cardinality_matrix[i][j];

                // We always set the value of the right difference so that the
                // last time we write this will necessarily be with the last
                // and largest left set.
                let this_difference = euc.get_right_difference_cardinality();
                right_difference_cardinality_vector[j] =
                    (this_difference - last_right_difference).get_max(I::default());
                last_right_difference = this_difference;
            }
            let this_difference = euc.get_left_difference_cardinality();
            left_difference_cardinality_vector[i] =
                (this_difference - last_left_difference).get_max(I::default());
            last_left_difference = this_difference;
        }

        (
            differential_overlap_cardinality_matrix,
            left_difference_cardinality_vector,
            right_difference_cardinality_vector,
        )
    }

    /// Returns the overlap cardinality matrices and outer difference shells cardinality of two lists of sets.
    ///
    /// # Arguments
    /// * `left` - The first list of sets.
    /// * `right` - The second list of sets.
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
    fn overlap_matrix_and_outer_difference_shells_cardinality<
        I: Copy
            + Default
            + Primitive<f32>
            + core::ops::Add<Output = I>
            + core::ops::Sub<Output = I>
            + core::ops::Div<Output = I>
            + core::ops::Mul<Output = I>
            + Sum
            + Send
            + Sync
            + AddAssign
            + SubAssign
            + MulAssign
            + DivAssign
            + MaxMin,
        const L: usize,
        const R: usize,
    >(
        left: &[Self; L],
        right: &[Self; R],
    ) -> ([[I; R]; L], I, I)
    where
        Self: SetLike<I>,
    {
        // Initialize overlap and differences cardinality matrices/vectors.
        let mut last_row = [I::default(); R];
        let mut differential_overlap_cardinality_matrix = [[I::default(); R]; L];
        let mut left_difference_outer_shell = I::default();
        let mut right_difference_outer_shell = I::default();
        let mut right_cardinalities = [I::default(); R];

        right.iter().zip(right_cardinalities.iter_mut()).for_each(
            |(right_hll, right_cardinality)| {
                *right_cardinality = right_hll.get_cardinality();
            },
        );

        let mut euc: EstimatedUnionCardinalities<I> = EstimatedUnionCardinalities::default();
        let mut last_left_difference: I = I::default();

        // Populate the overlap cardinality matrix.
        for (i, left_hll) in left.iter().enumerate() {
            let mut last_right_difference: I = I::default();
            let left_cardinality = left_hll.get_cardinality();
            let mut comulative_row = I::default();
            for (j, (right_hll, right_cardinality)) in
                right.iter().zip(right_cardinalities).enumerate()
            {
                euc = left_hll.get_estimated_union_cardinality(
                    left_cardinality,
                    right_hll,
                    right_cardinality,
                );
                let delta = last_row[j] + comulative_row;
                differential_overlap_cardinality_matrix[i][j] =
                    (euc.get_intersection_cardinality() - delta).get_max(I::default());
                last_row[j] = euc.get_intersection_cardinality().get_max(delta);
                comulative_row += differential_overlap_cardinality_matrix[i][j];

                // We always set the value of the right difference so that the
                // last time we write this will necessarily be with the last
                // and largest left set.
                let this_difference = euc.get_right_difference_cardinality();
                right_difference_outer_shell =
                    (this_difference - last_right_difference).get_max(I::default());
                last_right_difference = this_difference;
            }
            let this_difference = euc.get_left_difference_cardinality();
            left_difference_outer_shell =
                (this_difference - last_left_difference).get_max(I::default());
            last_left_difference = this_difference;
        }

        (
            differential_overlap_cardinality_matrix,
            left_difference_outer_shell,
            right_difference_outer_shell,
        )
    }
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize> HyperSpheresSketch
    for HyperLogLog<PRECISION, BITS>
{
}
