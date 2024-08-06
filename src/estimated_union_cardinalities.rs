use crate::utils::*;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A struct for more readable code.
pub struct EstimatedUnionCardinalities<F> {
    /// The estimated cardinality of the left set.
    left_cardinality: F,
    /// The estimated cardinality of the right set.
    right_cardinality: F,
    /// The estimated cardinality of the union of the two sets.
    union_cardinality: F,
}

impl<F: Number> From<(F, F, F)> for EstimatedUnionCardinalities<F> {
    fn from(value: (F, F, F)) -> Self {
        debug_assert!(value.0 >= F::ZERO);
        debug_assert!(value.1 >= F::ZERO);
        debug_assert!(value.2 >= F::ZERO);
        debug_assert!(
            value.0 <= value.2,
            concat!(
                "The estimated cardinality of the left set should be less than ",
                "or equal to the estimated cardinality of the union of the two sets. ",
                "Received left: {}, right: {}, union: {}."
            ),
            value.0,
            value.1,
            value.2
        );
        debug_assert!(
            value.1 <= value.2,
            concat!(
                "The estimated cardinality of the right set should be less than ",
                "or equal to the estimated cardinality of the union of the two sets. ",
                "Received left: {}, right: {}, union: {}."
            ),
            value.0,
            value.1,
            value.2
        );
        debug_assert!(
            value.0 + value.1 >= value.2,
            concat!(
                "The sum of the estimated cardinalities of the two sets ",
                "should be greater than or equal to the estimated cardinality ",
                "of the union of the two sets. Received left: {}, right: {}, union: {}."
            ),
            value.0,
            value.1,
            value.2
        );
        Self {
            left_cardinality: value.0,
            right_cardinality: value.1,
            union_cardinality: value.2,
        }
    }
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

        Self::from((left_cardinality, right_cardinality, union_cardinality))
    }

    #[inline(always)]
    /// Returns the estimated cardinality of the left set.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::EstimatedUnionCardinalities;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let left_cardinality = estimated_union_cardinalities.get_left_cardinality();
    ///
    /// assert_eq!(left_cardinality, 2.0);
    /// ```
    pub fn get_left_cardinality(&self) -> F {
        self.left_cardinality
    }

    #[inline(always)]
    /// Returns the estimated cardinality of the right set.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::EstimatedUnionCardinalities;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let right_cardinality = estimated_union_cardinalities.get_right_cardinality();
    ///
    /// assert_eq!(right_cardinality, 3.0);
    /// ```
    pub fn get_right_cardinality(&self) -> F {
        self.right_cardinality
    }

    #[inline(always)]
    /// Returns the estimated cardinality of the union of the two sets.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::EstimatedUnionCardinalities;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let union_cardinality = estimated_union_cardinalities.get_union_cardinality();
    ///
    /// assert_eq!(union_cardinality, 4.0);
    /// ```
    pub fn get_union_cardinality(&self) -> F {
        self.union_cardinality
    }

    #[inline(always)]
    /// Returns the estimated cardinality of the intersection of the two sets.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::EstimatedUnionCardinalities;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let intersection_cardinality = estimated_union_cardinalities.get_intersection_cardinality();
    ///
    /// assert_eq!(intersection_cardinality, 1.0);
    /// ```
    pub fn get_intersection_cardinality(&self) -> F {
        self.left_cardinality + self.right_cardinality - self.union_cardinality
    }

    #[inline(always)]
    /// Returns the estimated cardinality of the left set minus the right set.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::EstimatedUnionCardinalities;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let left_minus_right_cardinality =
    ///     estimated_union_cardinalities.get_left_difference_cardinality();
    ///
    /// assert_eq!(left_minus_right_cardinality, 1.0);
    /// ```
    pub fn get_left_difference_cardinality(&self) -> F {
        self.union_cardinality - self.right_cardinality
    }

    #[inline(always)]
    /// Returns the estimated cardinality of the right set minus the left set.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::EstimatedUnionCardinalities;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let right_minus_left_cardinality =
    ///     estimated_union_cardinalities.get_right_difference_cardinality();
    ///
    /// assert_eq!(right_minus_left_cardinality, 2.0);
    /// ```
    pub fn get_right_difference_cardinality(&self) -> F {
        self.union_cardinality - self.left_cardinality
    }

    #[inline(always)]
    /// Returns the estimated cardinality of the symmetric difference of the two sets.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::EstimatedUnionCardinalities;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let symmetric_difference_cardinality =
    ///     estimated_union_cardinalities.get_symmetric_difference_cardinality();
    ///
    /// assert_eq!(symmetric_difference_cardinality, 3.0);
    /// ```
    pub fn get_symmetric_difference_cardinality(&self) -> F {
        self.union_cardinality + self.union_cardinality
            - self.left_cardinality
            - self.right_cardinality
    }

    #[inline(always)]
    /// Returns the estimated Jaccard index of the two sets.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::EstimatedUnionCardinalities;
    ///
    /// let estimated_union_cardinalities = EstimatedUnionCardinalities::from((2.0, 3.0, 4.0));
    ///
    /// let jaccard_index = estimated_union_cardinalities.get_jaccard_index();
    ///
    /// assert_eq!(
    ///     jaccard_index,
    ///     1.0 / 4.0,
    ///     "Example 1: Expected 1.0 / 4.0, got {}",
    ///     jaccard_index
    /// );
    /// ```
    pub fn get_jaccard_index(&self) -> F {
        let jaccard_index = self.get_intersection_cardinality() / self.union_cardinality;
        // Numerical (in)stability correction.
        if jaccard_index > F::ONE {
            F::ONE
        } else {
            jaccard_index
        }
    }
}
