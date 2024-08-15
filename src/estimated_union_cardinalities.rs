use crate::utils::*;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A struct for more readable code.
pub(crate) struct EstimatedUnionCardinalities<F> {
    /// The estimated cardinality of the left set.
    left_cardinality: F,
    /// The estimated cardinality of the right set.
    right_cardinality: F,
    /// The estimated cardinality of the union of the two sets.
    union_cardinality: F,
}

impl<F: Number> EstimatedUnionCardinalities<F> {
    #[inline(always)]
    /// Returns a new instance of `EstimatedUnionCardinalities` with applied corrections.
    pub(crate) fn with_correction(
        left_cardinality: F,
        right_cardinality: F,
        mut union_cardinality: F,
    ) -> Self {
        if union_cardinality > left_cardinality + right_cardinality {
            union_cardinality = left_cardinality + right_cardinality;
        }

        // The following can happen when the HLL registers start to be saturated.
        if union_cardinality < left_cardinality || union_cardinality < right_cardinality {
            union_cardinality = if left_cardinality < right_cardinality {
                right_cardinality
            } else {
                left_cardinality
            };
        }

        debug_assert!(left_cardinality >= F::ZERO);
        debug_assert!(right_cardinality >= F::ZERO);
        debug_assert!(union_cardinality >= F::ZERO);
        debug_assert!(
            left_cardinality <= union_cardinality,
            concat!(
                "The estimated cardinality of the left set should be less than ",
                "or equal to the estimated cardinality of the union of the two sets. ",
                "Received left: {}, right: {}, union: {}."
            ),
            left_cardinality,
            right_cardinality,
            union_cardinality
        );
        debug_assert!(
            right_cardinality <= union_cardinality,
            concat!(
                "The estimated cardinality of the right set should be less than ",
                "or equal to the estimated cardinality of the union of the two sets. ",
                "Received left: {}, right: {}, union: {}."
            ),
            left_cardinality,
            right_cardinality,
            union_cardinality
        );
        debug_assert!(
            left_cardinality + right_cardinality >= union_cardinality,
            concat!(
                "The sum of the estimated cardinalities of the two sets ",
                "should be greater than or equal to the estimated cardinality ",
                "of the union of the two sets. Received left: {}, right: {}, union: {}."
            ),
            left_cardinality,
            right_cardinality,
            union_cardinality
        );
        Self {
            left_cardinality,
            right_cardinality,
            union_cardinality,
        }
    }

    #[inline(always)]
    /// Returns the estimated cardinality of the intersection of the two sets.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let intersection_cardinality = estimated_union_cardinalities.get_intersection_cardinality();
    ///
    /// assert_eq!(intersection_cardinality, 1.0);
    /// ```
    pub(crate) fn get_intersection_cardinality(&self) -> F {
        self.left_cardinality + self.right_cardinality - self.union_cardinality
    }

    #[inline(always)]
    /// Returns the estimated cardinality of the left set minus the right set.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let left_minus_right_cardinality =
    ///     estimated_union_cardinalities.get_left_difference_cardinality();
    ///
    /// assert_eq!(left_minus_right_cardinality, 1.0);
    /// ```
    pub(crate) fn get_left_difference_cardinality(&self) -> F {
        self.union_cardinality - self.right_cardinality
    }

    #[inline(always)]
    /// Returns the estimated cardinality of the right set minus the left set.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let right_minus_left_cardinality =
    ///     estimated_union_cardinalities.get_right_difference_cardinality();
    ///
    /// assert_eq!(right_minus_left_cardinality, 2.0);
    /// ```
    pub(crate) fn get_right_difference_cardinality(&self) -> F {
        self.union_cardinality - self.left_cardinality
    }
}
