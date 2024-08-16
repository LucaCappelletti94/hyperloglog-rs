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
use crate::utils::*;
use crate::estimated_union_cardinalities::EstimatedUnionCardinalities;

/// Trait for sketching algorithms that provide the overlap and differences cardinality matrices.
pub trait HyperSpheresSketch<N: Number>: Estimator<N> {
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
    ) -> ([[N; R]; L], [N; L], [N; R]) {
        // Initialize overlap and differences cardinality matrices/vectors.
        let mut last_row = [N::default(); R];
        let mut differential_overlap_cardinality_matrix = [[N::default(); R]; L];
        let mut left_difference_cardinality_vector = [N::default(); L];
        let mut right_cardinalities = [N::default(); R];

        right
            .iter()
            .zip(right_cardinalities.iter_mut())
            .for_each(|(right, right_cardinality)| {
                *right_cardinality = right.estimate_cardinality();
            });

        let mut right_difference_cardinality_vector = [N::default(); R];
        let mut euc: EstimatedUnionCardinalities<N> = EstimatedUnionCardinalities::default();
        let mut last_left_difference = N::default();

        // Populate the overlap cardinality matrix.
        for (i, left) in left.iter().enumerate() {
            let mut last_right_difference = N::default();
            let left_cardinality = left.estimate_cardinality();
            let mut comulative_row = N::default();
            for (j, (right, right_cardinality)) in right.iter().zip(right_cardinalities).enumerate()
            {
                let union_cardinality = left.estimate_union_cardinality(right);
                euc = EstimatedUnionCardinalities::with_correction(
                    left_cardinality,
                    right_cardinality,
                    union_cardinality,
                );
                let delta = last_row[j] + comulative_row;
                differential_overlap_cardinality_matrix[i][j] = euc
                    .get_intersection_cardinality()
                    .saturating_zero_sub(delta);
                last_row[j] = if euc.get_intersection_cardinality() > delta {
                    euc.get_intersection_cardinality()
                } else {
                    delta
                };
                comulative_row += differential_overlap_cardinality_matrix[i][j];

                // We always set the value of the right difference so that the
                // last time we write this will necessarily be with the last
                // and largest left set.
                right_difference_cardinality_vector[j] = euc
                    .get_right_difference_cardinality()
                    .saturating_zero_sub(last_right_difference);

                last_right_difference = euc.get_right_difference_cardinality();
            }
            left_difference_cardinality_vector[i] = euc
                .get_left_difference_cardinality()
                .saturating_zero_sub(last_left_difference);
            last_left_difference = euc.get_left_difference_cardinality();
        }

        (
            differential_overlap_cardinality_matrix,
            left_difference_cardinality_vector,
            right_difference_cardinality_vector,
        )
    }
}

/// Trait for sketching algorithms that provide the normalized overlap and differences cardinality matrices.
pub trait NormalizedHyperSpheresSketch<F: Float>: HyperSpheresSketch<F> {
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
    ) -> ([[F; R]; L], [F; L], [F; R]) {
        // Initialize overlap and differences cardinality matrices/vectors.
        let mut last_row = [F::default(); R];
        let mut differential_overlap_cardinality_matrix = [[F::default(); R]; L];
        let mut left_difference_cardinality_vector = [F::default(); L];
        let mut right_cardinalities = [F::default(); R];

        right
            .iter()
            .zip(right_cardinalities.iter_mut())
            .for_each(|(right, right_cardinality)| {
                *right_cardinality = right.estimate_cardinality();
            });

        // We run a debug assert where we check that each right cardinality is
        // larger than the previous one.
        debug_assert!(right_cardinalities
            .iter()
            .zip(right_cardinalities.iter().skip(1))
            .all(|(left, right)| left <= right));

        let mut right_difference_cardinality_vector = [F::default(); R];
        let mut euc: EstimatedUnionCardinalities<F> = EstimatedUnionCardinalities::default();
        let mut last_left_difference: F = F::default();
        let mut last_inner_left_differences = [F::default(); R];
        let mut last_left_cardinality: F = F::default();

        // Populate the overlap cardinality matrix.
        for (i, left) in left.iter().enumerate() {
            let mut last_right_difference: F = F::default();
            let left_cardinality = left.estimate_cardinality();
            let mut comulative_row = F::default();
            let mut last_right_cardinality = F::default();
            for (j, (right, (right_cardinality, last_inner_left_difference))) in right
                .iter()
                .zip(
                    right_cardinalities
                        .iter()
                        .copied()
                        .zip(last_inner_left_differences.iter_mut()),
                )
                .enumerate()
            {
                let union_cardinality = left.estimate_union_cardinality(right);
                euc = EstimatedUnionCardinalities::with_correction(
                    left_cardinality,
                    right_cardinality,
                    union_cardinality,
                );
                let delta = last_row[j] + comulative_row;
                let differential_intersection = euc
                    .get_intersection_cardinality()
                    .saturating_zero_sub(delta);

                debug_assert!(
                    differential_intersection >= F::default(),
                    concat!(
                        "Expected differential_intersection to be larger than zero, but it is not. ",
                        "Got: differential_intersection: {:?}, delta: {:?}",
                    ),
                    differential_intersection,
                    delta,
                );

                let maximal_differential_intersection_cardinality =
                    (euc.get_left_difference_cardinality() + right_cardinality)
                        .saturating_zero_sub(*last_inner_left_difference + last_right_cardinality);
                *last_inner_left_difference = euc.get_left_difference_cardinality();

                differential_overlap_cardinality_matrix[i][j] = differential_intersection
                    .saturating_one_div(maximal_differential_intersection_cardinality);
                last_row[j] = if euc.get_intersection_cardinality() > delta {
                    euc.get_intersection_cardinality()
                } else {
                    delta
                };
                comulative_row += differential_intersection;

                // We always set the value of the right difference so that the
                // last time we write this will necessarily be with the last
                // and largest left set.

                let differential_right_difference = euc
                    .get_right_difference_cardinality()
                    .saturating_zero_sub(last_right_difference);

                right_difference_cardinality_vector[j] = differential_right_difference
                    .saturating_one_div(
                        right_cardinality.saturating_zero_sub(last_right_cardinality),
                    );
                last_right_difference = euc.get_right_difference_cardinality();
                last_right_cardinality = right_cardinality;
            }
            left_difference_cardinality_vector[i] = euc
                .get_left_difference_cardinality()
                .saturating_zero_sub(last_left_difference)
                .saturating_one_div(left_cardinality.saturating_zero_sub(last_left_cardinality));
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

impl<N: Number, M> HyperSpheresSketch<N> for M where M: Estimator<N> {}
impl<F: Float, M> NormalizedHyperSpheresSketch<F> for M where M: Estimator<F> {}
