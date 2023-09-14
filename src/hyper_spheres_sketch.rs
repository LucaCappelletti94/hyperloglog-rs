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

pub trait SetLike<I> {
    /// Returns the estimated intersection cardinality between two sets.
    fn intersection_size(&self, other: &Self) -> I;

    /// Returns the estimated difference cardinality between two sets.
    fn difference_size(&self, other: &Self) -> I;
}

impl<F: Primitive<f32>, PRECISION: Precision + WordType<BITS>, const BITS: usize> SetLike<F>
    for HyperLogLog<PRECISION, BITS>
{
    fn intersection_size(&self, other: &Self) -> F {
        self.estimate_intersection_cardinality(other)
    }

    fn difference_size(&self, other: &Self) -> F {
        self.estimate_difference_cardinality(other)
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
            + std::ops::Add<Output = I>
            + std::ops::Sub<Output = I>
            + Sum
            + AddAssign
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
        let mut overlap_cardinality_matrix = [[I::default(); R]; L];
        let mut left_difference_cardinality_vector = [I::default(); L];
        let mut right_difference_cardinality_vector = [I::default(); R];

        // Populate the overlap cardinality matrix.
        for i in 0..L {
            for j in 0..R {
                overlap_cardinality_matrix[i][j] = (left[i].intersection_size(&right[j])
                    - (0..(i + 1).min(L))
                        .flat_map(|sub_i| {
                            (0..(j + 1).min(R))
                                .map(move |sub_j| overlap_cardinality_matrix[sub_i][sub_j])
                        })
                        .sum::<I>())
                .get_max(I::default());
            }
        }

        // Populate the difference cardinality vectors.
        let mut last_difference: I = I::default();

        // Populate the left difference cardinality vector.
        for i in 0..L {
            let this_difference = left[i].difference_size(&right[R - 1]);
            left_difference_cardinality_vector[i] = this_difference - last_difference;
            last_difference = this_difference;
        }

        last_difference = I::default();

        // Populate the right difference cardinality vector.
        for j in 0..R {
            let this_difference = right[j].difference_size(&left[L - 1]);
            right_difference_cardinality_vector[j] = this_difference - last_difference;
            last_difference = this_difference;
        }

        (
            overlap_cardinality_matrix,
            left_difference_cardinality_vector,
            right_difference_cardinality_vector,
        )
    }
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize> HyperSpheresSketch
    for HyperLogLog<PRECISION, BITS>
{
}
