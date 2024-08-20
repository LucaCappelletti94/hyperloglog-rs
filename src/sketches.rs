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
//! While in the [`HyperLogLog`] case we provide the approximated version of this algorithm,
//! sometimes it is necessary, such as in test cases, to have the exact version of the
//! algorithm. The approximated version is faster and uses less memory, but it is not,
//! of course, guaranteed to be exact.
use crate::prelude::{Estimator, FloatOps, Number, Zero};

/// Trait for sketching algorithms that provide the overlap and differences cardinality matrices.
pub trait HyperSpheresSketch<N: Number>: Estimator<N> {
    #[inline]
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
    /// we expect to compute the exclusively overlap matrix $A_{ij}$ and the exclusively differences vectors $`B_i`$.    
    ///
    /// ![Illustration of overlaps](https://github.com/LucaCappelletti94/hyperloglog-rs/blob/main/triple_overlap.png?raw=true)
    ///
    /// Very similarly, for the case of vectors of two elements:
    ///
    /// ![Illustration of overlaps](https://github.com/LucaCappelletti94/hyperloglog-rs/blob/main/tuple_overlap.png?raw=true)
    fn overlap_and_differences_cardinality_matrices<const L: usize, const R: usize>(
        lefts: &[Self; L],
        rights: &[Self; R],
    ) -> ([[N; R]; L], [N; L], [N; R]) {
        // Initialize overlap and differences cardinality matrices/vectors.
        let mut last_row = [N::ZERO; R];
        let mut differential_overlap_cardinality_matrix = [[N::ZERO; R]; L];
        let mut left_difference_cardinality_vector = [N::ZERO; L];
        let mut right_cardinalities = [N::ZERO; R];

        rights
            .iter()
            .zip(right_cardinalities.iter_mut())
            .for_each(|(right, right_cardinality)| {
                *right_cardinality = right.estimate_cardinality();
            });

        let mut right_difference_cardinality_vector = [N::ZERO; R];
        let mut euc: EstimatedUnionCardinalities<N> = EstimatedUnionCardinalities {
            left: N::ZERO,
            right: N::ZERO,
            union: N::ZERO,
        };
        let mut last_left_difference = N::ZERO;

        // Populate the overlap cardinality matrix.
        for (i, left) in lefts.iter().enumerate() {
            let mut last_right_difference = N::ZERO;
            let left_cardinality = left.estimate_cardinality();
            let mut comulative_row = N::ZERO;
            for (j, (right, right_cardinality)) in
                rights.iter().zip(right_cardinalities).enumerate()
            {
                let union_cardinality = left.estimate_union_cardinality(right);
                euc = EstimatedUnionCardinalities {
                    left: left_cardinality,
                    right: right_cardinality,
                    union: union_cardinality,
                };
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
                debug_assert!(comulative_row >= N::ZERO, "Expected comulative_row to be larger than zero, but it is not. Got: comulative_row: {comulative_row:?}, delta: {delta:?}");

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
pub trait NormalizedHyperSpheresSketch: HyperSpheresSketch<f64> {
    #[inline]
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
        lefts: &[Self; L],
        rights: &[Self; R],
    ) -> ([[f64; R]; L], [f64; L], [f64; R]) {
        // Initialize overlap and differences cardinality matrices/vectors.
        let mut last_row = [f64::ZERO; R];
        let mut differential_overlap_cardinality_matrix = [[f64::ZERO; R]; L];
        let mut left_difference_cardinality_vector = [f64::ZERO; L];
        let mut right_cardinalities = [f64::ZERO; R];

        rights
            .iter()
            .zip(right_cardinalities.iter_mut())
            .for_each(|(right, right_cardinality)| {
                *right_cardinality = right.estimate_cardinality();
            });

        // We run a debug assert where we check that each right cardinality is
        // larger than the previous one.
        debug_assert!(
            right_cardinalities
                .iter()
                .zip(right_cardinalities.iter().skip(1))
                .all(|(left, right)| left <= right),
            "The right cardinalities should be sorted in ascending order."
        );

        let mut right_difference_cardinality_vector = [f64::ZERO; R];
        let mut euc: EstimatedUnionCardinalities<f64> = EstimatedUnionCardinalities {
            left: f64::ZERO,
            right: f64::ZERO,
            union: f64::ZERO,
        };
        let mut last_left_difference = f64::ZERO;
        let mut last_inner_left_differences = [f64::ZERO; R];
        let mut last_left_cardinality = f64::ZERO;

        // Populate the overlap cardinality matrix.
        for (i, left) in lefts.iter().enumerate() {
            let mut last_right_difference = f64::ZERO;
            let left_cardinality = left.estimate_cardinality();
            let mut comulative_row = f64::ZERO;
            let mut last_right_cardinality = f64::ZERO;
            for (j, (right, (right_cardinality, last_inner_left_difference))) in rights
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
                euc = EstimatedUnionCardinalities {
                    left: left_cardinality,
                    right: right_cardinality,
                    union: union_cardinality,
                };
                let delta = last_row[j] + comulative_row;
                let differential_intersection = euc
                    .get_intersection_cardinality()
                    .saturating_zero_sub(delta);

                debug_assert!(
                    differential_intersection >= f64::ZERO,
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
impl<M> NormalizedHyperSpheresSketch for M where M: Estimator<f64> {}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A struct for more readable code.
struct EstimatedUnionCardinalities<F> {
    /// The estimated cardinality of the left set.
    left: F,
    /// The estimated cardinality of the right set.
    right: F,
    /// The estimated cardinality of the union of the two sets.
    union: F,
}

impl<F: Number> EstimatedUnionCardinalities<F> {
    /// Returns the estimated cardinality of the intersection of the two sets.
    fn get_intersection_cardinality(&self) -> F {
        let intersection = self.left + self.right - self.union;
        debug_assert!(
            intersection >= F::ZERO,
            "Expected intersection to be larger than zero, but it is not. Got: intersection: {intersection:?}."
        );
        intersection
    }

    /// Returns the estimated cardinality of the left set minus the right set.
    fn get_left_difference_cardinality(&self) -> F {
        self.union - self.right
    }

    /// Returns the estimated cardinality of the right set minus the left set.
    fn get_right_difference_cardinality(&self) -> F {
        self.union - self.left
    }
}
