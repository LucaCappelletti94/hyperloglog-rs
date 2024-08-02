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
use crate::prelude::*;
use core::fmt::Debug;
use core::ops::AddAssign;

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

impl<F: Primitive<f32>, P: Precision + WordType<BITS>, const BITS: usize> SetLike<F>
    for HyperLogLog<P, BITS>
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

        // union_estimate = union_estimate.get_min(self_cardinality + other_cardinality);

        EstimatedUnionCardinalities::from((self_cardinality, other_cardinality, union_estimate))
    }

    fn get_cardinality(&self) -> F {
        F::reverse(self.estimate_cardinality())
    }
}

pub trait HyperSpheresSketch<I>: Sized + SetLike<I>
where
    I: Copy
        + Default
        + core::ops::Add<Output = I>
        + core::ops::Sub<Output = I>
        + core::ops::Div<Output = I>
        + core::ops::Mul<Output = I>
        + AddAssign
        + Debug
        + One
        + MaxMin,
{
    #[inline(always)]
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
    fn overlap_and_differences_cardinality_matrices<const L: usize, const R: usize>(
        left: &[Self; L],
        right: &[Self; R],
    ) -> ([[I; R]; L], [I; L], [I; R]) {
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
                right_difference_cardinality_vector[j] = (euc.get_right_difference_cardinality()
                    - last_right_difference)
                    .get_max(I::default());

                last_right_difference = euc.get_right_difference_cardinality();
            }
            left_difference_cardinality_vector[i] = (euc.get_left_difference_cardinality()
                - last_left_difference)
                .get_max(I::default());
            last_left_difference = euc.get_left_difference_cardinality();
        }

        (
            differential_overlap_cardinality_matrix,
            left_difference_cardinality_vector,
            right_difference_cardinality_vector,
        )
    }

    #[cfg(feature = "std")]
    #[inline(always)]
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
    fn overlap_and_differences_cardinality_matrices_vec(
        left: &[Self],
        right: &[Self],
    ) -> (Vec<Vec<I>>, Vec<I>, Vec<I>) {
        // Initialize overlap and differences cardinality matrices/vectors.
        let mut last_row = vec![I::default(); right.len()];
        let mut differential_overlap_cardinality_matrix =
            vec![vec![I::default(); right.len()]; left.len()];
        let mut left_difference_cardinality_vector = vec![I::default(); left.len()];
        let mut right_cardinalities = vec![I::default(); right.len()];

        right.iter().zip(right_cardinalities.iter_mut()).for_each(
            |(right_hll, right_cardinality)| {
                *right_cardinality = right_hll.get_cardinality();
            },
        );

        let mut right_difference_cardinality_vector = vec![I::default(); right.len()];
        let mut euc: EstimatedUnionCardinalities<I> = EstimatedUnionCardinalities::default();
        let mut last_left_difference: I = I::default();

        // Populate the overlap cardinality matrix.
        for (i, left_hll) in left.iter().enumerate() {
            let mut last_right_difference: I = I::default();
            let left_cardinality = left_hll.get_cardinality();
            let mut comulative_row = I::default();
            for (j, (right_hll, right_cardinality)) in right
                .iter()
                .zip(right_cardinalities.iter().copied())
                .enumerate()
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
                right_difference_cardinality_vector[j] = (euc.get_right_difference_cardinality()
                    - last_right_difference)
                    .get_max(I::default());

                last_right_difference = euc.get_right_difference_cardinality();
            }
            left_difference_cardinality_vector[i] = (euc.get_left_difference_cardinality()
                - last_left_difference)
                .get_max(I::default());
            last_left_difference = euc.get_left_difference_cardinality();
        }

        (
            differential_overlap_cardinality_matrix,
            left_difference_cardinality_vector,
            right_difference_cardinality_vector,
        )
    }

    #[inline(always)]
    /// Returns the normalized overlap and differences cardinality matrices of two lists of sets.
    ///
    /// # Arguments
    /// * `left` - The first list of sets.
    /// * `right` - The second list of sets.
    ///
    /// # Returns
    /// * `overlap_cardinality_matrix` - Matrix of normalized estimated overlapping cardinalities between the elements of the left and right arrays.
    /// * `left_difference_cardinality_vector` - Vector of normalized estimated difference cardinalities between the elements of the left array and the last element of the right array.
    /// * `right_difference_cardinality_vector` - Vector of normalized estimated difference cardinalities between the elements of the right array and the last element of the left array.
    fn normalized_overlap_and_differences_cardinality_matrices<const L: usize, const R: usize>(
        left: &[Self; L],
        right: &[Self; R],
    ) -> ([[I; R]; L], [I; L], [I; R]) {
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

        // We run a debug assert where we check that each right cardinality is
        // larger than the previous one.
        debug_assert!(right_cardinalities
            .iter()
            .zip(right_cardinalities.iter().skip(1))
            .all(|(left, right)| left <= right));

        let mut right_difference_cardinality_vector = [I::default(); R];
        let mut euc: EstimatedUnionCardinalities<I> = EstimatedUnionCardinalities::default();
        let mut last_left_difference: I = I::default();
        let mut last_inner_left_differences = [I::default(); R];
        let mut last_left_cardinality: I = I::default();

        // Populate the overlap cardinality matrix.
        for (i, left_hll) in left.iter().enumerate() {
            let mut last_right_difference: I = I::default();
            let left_cardinality = left_hll.get_cardinality();
            let mut comulative_row = I::default();
            let mut last_right_cardinality = I::default();
            for (j, (right_hll, (right_cardinality, last_inner_left_difference))) in right
                .iter()
                .zip(
                    right_cardinalities
                        .iter()
                        .copied()
                        .zip(last_inner_left_differences.iter_mut()),
                )
                .enumerate()
            {
                euc = left_hll.get_estimated_union_cardinality(
                    left_cardinality,
                    right_hll,
                    right_cardinality,
                );
                let delta = last_row[j] + comulative_row;
                let differential_intersection =
                    (euc.get_intersection_cardinality() - delta).get_max(I::default());

                debug_assert!(
                    differential_intersection >= I::default(),
                    concat!(
                        "Expected differential_intersection to be larger than zero, but it is not. ",
                        "Got: differential_intersection: {:?}, delta: {:?}",
                    ),
                    differential_intersection,
                    delta,
                );

                let maximal_differential_intersection_cardinality =
                    (euc.get_left_difference_cardinality() - *last_inner_left_difference
                        + right_cardinality
                        - last_right_cardinality)
                        .get_max(I::non_zero_positive_min_value());
                *last_inner_left_difference = euc.get_left_difference_cardinality();

                differential_overlap_cardinality_matrix[i][j] = (differential_intersection
                    / maximal_differential_intersection_cardinality)
                    .get_min(I::ONE);
                last_row[j] = euc.get_intersection_cardinality().get_max(delta);
                comulative_row += differential_intersection;

                // We always set the value of the right difference so that the
                // last time we write this will necessarily be with the last
                // and largest left set.

                let differential_right_difference = (euc.get_right_difference_cardinality()
                    - last_right_difference)
                    .get_max(I::default());
                let maximal_differential_right_difference = (right_cardinality
                    - last_right_cardinality)
                    .get_max(I::non_zero_positive_min_value());

                right_difference_cardinality_vector[j] = (differential_right_difference
                    / maximal_differential_right_difference)
                    .get_min(I::ONE);
                last_right_difference = euc.get_right_difference_cardinality();
                last_right_cardinality = right_cardinality;
            }
            left_difference_cardinality_vector[i] = ((euc.get_left_difference_cardinality()
                - last_left_difference)
                .get_max(I::default())
                / (left_cardinality - last_left_cardinality)
                    .get_max(I::non_zero_positive_min_value()))
            .get_min(I::ONE);
            last_left_cardinality = left_cardinality;
            last_left_difference = euc.get_left_difference_cardinality();
        }

        (
            differential_overlap_cardinality_matrix,
            left_difference_cardinality_vector,
            right_difference_cardinality_vector,
        )
    }

    #[cfg(feature = "std")]
    #[inline(always)]
    /// Returns the normalized overlap and differences cardinality matrices of two vectors of sets.
    ///
    /// # Arguments
    /// * `left` - The first list of sets.
    /// * `right` - The second list of sets.
    ///
    /// # Returns
    /// * `overlap_cardinality_matrix` - Matrix of normalized estimated overlapping cardinalities between the elements of the left and right arrays.
    /// * `left_difference_cardinality_vector` - Vector of normalized estimated difference cardinalities between the elements of the left array and the last element of the right array.
    /// * `right_difference_cardinality_vector` - Vector of normalized estimated difference cardinalities between the elements of the right array and the last element of the left array.
    fn normalized_overlap_and_differences_cardinality_matrices_vec(
        left: &[Self],
        right: &[Self],
    ) -> (Vec<Vec<I>>, Vec<I>, Vec<I>) {
        // Initialize overlap and differences cardinality matrices/vectors.
        let mut last_row = vec![I::default(); right.len()];
        let mut differential_overlap_cardinality_matrix =
            vec![vec![I::default(); right.len()]; left.len()];
        let mut left_difference_cardinality_vector = vec![I::default(); left.len()];
        let mut right_cardinalities = vec![I::default(); right.len()];

        right.iter().zip(right_cardinalities.iter_mut()).for_each(
            |(right_hll, right_cardinality)| {
                *right_cardinality = right_hll.get_cardinality();
            },
        );

        // We run a debug assert where we check that each right cardinality is
        // larger than the previous one.
        debug_assert!(right_cardinalities
            .iter()
            .zip(right_cardinalities.iter().skip(1))
            .all(|(left, right)| left <= right));

        let mut right_difference_cardinality_vector = vec![I::default(); right.len()];
        let mut euc: EstimatedUnionCardinalities<I> = EstimatedUnionCardinalities::default();
        let mut last_left_difference: I = I::default();
        let mut last_left_cardinality: I = I::default();
        let mut last_inner_left_differences = vec![I::default(); right.len()];

        // Populate the overlap cardinality matrix.
        for (i, left_hll) in left.iter().enumerate() {
            let mut last_right_difference: I = I::default();
            let left_cardinality = left_hll.get_cardinality();
            let mut comulative_row = I::default();
            let mut last_right_cardinality = I::default();
            for (j, (right_hll, (right_cardinality, last_inner_left_difference))) in right
                .iter()
                .zip(
                    right_cardinalities
                        .iter()
                        .copied()
                        .zip(last_inner_left_differences.iter_mut()),
                )
                .enumerate()
            {
                euc = left_hll.get_estimated_union_cardinality(
                    left_cardinality,
                    right_hll,
                    right_cardinality,
                );
                let delta = last_row[j] + comulative_row;
                let differential_intersection =
                    (euc.get_intersection_cardinality() - delta).get_max(I::default());

                debug_assert!(
                    differential_intersection >= I::default(),
                    concat!(
                        "Expected differential_intersection to be larger than zero, but it is not. ",
                        "Got: differential_intersection: {:?}, delta: {:?}",
                    ),
                    differential_intersection,
                    delta,
                );

                let maximal_differential_intersection_cardinality =
                    (euc.get_left_difference_cardinality() - *last_inner_left_difference
                        + right_cardinality
                        - last_right_cardinality)
                        .get_max(I::non_zero_positive_min_value());
                *last_inner_left_difference = euc.get_left_difference_cardinality();

                differential_overlap_cardinality_matrix[i][j] = (differential_intersection
                    / maximal_differential_intersection_cardinality)
                    .get_min(I::ONE);
                last_row[j] = euc.get_intersection_cardinality().get_max(delta);
                comulative_row += differential_intersection;

                // We always set the value of the right difference so that the
                // last time we write this will necessarily be with the last
                // and largest left set.

                let differential_right_difference = (euc.get_right_difference_cardinality()
                    - last_right_difference)
                    .get_max(I::default());
                let maximal_differential_right_difference = (right_cardinality
                    - last_right_cardinality)
                    .get_max(I::non_zero_positive_min_value());

                right_difference_cardinality_vector[j] = (differential_right_difference
                    / maximal_differential_right_difference)
                    .get_min(I::ONE);
                last_right_difference = euc.get_right_difference_cardinality();
                last_right_cardinality = right_cardinality;
            }
            left_difference_cardinality_vector[i] = ((euc.get_left_difference_cardinality()
                - last_left_difference)
                .get_max(I::default())
                / (left_cardinality - last_left_cardinality)
                    .get_max(I::non_zero_positive_min_value()))
            .get_min(I::ONE);
            last_left_cardinality = left_cardinality;
            last_left_difference = euc.get_left_difference_cardinality();
        }

        (
            differential_overlap_cardinality_matrix,
            left_difference_cardinality_vector,
            right_difference_cardinality_vector,
        )
    }
}

impl<P: Precision + WordType<BITS>, const BITS: usize, I: Primitive<f32>> HyperSpheresSketch<I>
    for HyperLogLog<P, BITS>
{
}
