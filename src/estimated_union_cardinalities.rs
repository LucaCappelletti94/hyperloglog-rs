use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Default, Deserialize, Serialize)]
/// A struct for more readable code.
pub struct EstimatedUnionCardinalities {
    /// The estimated cardinality of the left set.
    left_cardinality: f32,
    /// The estimated cardinality of the right set.
    right_cardinality: f32,
    /// The estimated cardinality of the union of the two sets.
    union_cardinality: f32,
}

impl From<(f32, f32, f32)> for EstimatedUnionCardinalities {
    fn from(value: (f32, f32, f32)) -> Self {
        Self {
            left_cardinality: value.0,
            right_cardinality: value.1,
            union_cardinality: value.2,
        }
    }
}

impl EstimatedUnionCardinalities {
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
    ///
    /// ```
    pub fn get_left_cardinality(&self) -> f32 {
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
    ///
    /// ```
    pub fn get_right_cardinality(&self) -> f32 {
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
    ///
    /// ```
    pub fn get_union_cardinality(&self) -> f32 {
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
    ///
    /// ```
    pub fn get_intersection_cardinality(&self) -> f32 {
        (self.left_cardinality + self.right_cardinality - self.union_cardinality).max(0.0)
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
    /// let left_minus_right_cardinality = estimated_union_cardinalities.get_left_difference_cardinality();
    ///
    /// assert_eq!(left_minus_right_cardinality, 1.0);
    ///
    /// ```
    pub fn get_left_difference_cardinality(&self) -> f32 {
        (self.left_cardinality - self.get_intersection_cardinality()).max(0.0)
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
    /// let right_minus_left_cardinality = estimated_union_cardinalities.get_right_difference_cardinality();
    ///
    /// assert_eq!(right_minus_left_cardinality, 2.0);
    ///
    /// ```
    pub fn get_right_difference_cardinality(&self) -> f32 {
        (self.right_cardinality - self.get_intersection_cardinality()).max(0.0)
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
    /// let symmetric_difference_cardinality = estimated_union_cardinalities.get_symmetric_difference_cardinality();
    ///
    /// assert_eq!(symmetric_difference_cardinality, 3.0);
    ///
    /// ```
    pub fn get_symmetric_difference_cardinality(&self) -> f32 {
        (self.left_cardinality + self.right_cardinality - 2.0 * self.get_intersection_cardinality())
            .max(0.0)
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
    /// assert_eq!(jaccard_index, 1.0 / 4.0, "Example 1: Expected 1.0 / 4.0, got {}", jaccard_index);
    ///
    /// ```
    ///
    pub fn get_jaccard_index(&self) -> f32 {
        (self.get_intersection_cardinality() / (self.union_cardinality).max(f32::EPSILON))
            .max(0.0)
            .min(1.0)
    }
}