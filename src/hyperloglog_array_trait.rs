use core::hash::Hash;

use crate::prelude::*;
use crate::utils::FloatNumber;

pub trait HyperLogLogArrayTrait<P: Precision, B: Bits, H: HyperLogLogTrait<P, B, Hasher>, Hasher: core::hash::Hasher + Default, const N: usize>:
    Default + PartialEq + Eq + Clone
{
    /// Returns the estimated overlap and difference cardinality matrices and vectors with the provided HyperLogLogArray.
    ///
    /// # Arguments
    /// * `other`: The HyperLogLogArray to estimate the overlap and difference cardinality matrices and vectors with.
    ///
    /// # Returns
    /// The estimated overlap and difference cardinality matrices and vectors with the provided HyperLogLogArray.
    fn overlap_and_differences_cardinality_matrices<F: FloatNumber>(
        &self,
        other: &Self,
    ) -> ([[F; N]; N], [F; N], [F; N])
    where
        H: SetLike<F>,
        P: PrecisionConstants<F>;

    /// Returns the estimated normalized overlap and difference cardinality matrices and vectors with the provided HyperLogLogArray.
    ///
    /// # Arguments
    /// * `other`: The HyperLogLogArray to estimate the normalized overlap and difference cardinality matrices and vectors with.
    ///
    /// # Returns
    /// The estimated normalized overlap and difference cardinality matrices and vectors with the provided HyperLogLogArray.
    fn normalized_overlap_and_differences_cardinality_matrices<F: FloatNumber>(
        &self,
        other: &Self,
    ) -> ([[F; N]; N], [F; N], [F; N])
    where
        H: SetLike<F>,
        P: PrecisionConstants<F>;

    /// Inserts the provided value in the i-th HLL counter in the array.
    fn insert<T: Hash>(&mut self, i: usize, value: T);
}
