//! CardinalitySampleBuilder is a helper struct for building samples for testing.
use std::ops::{Add, AddAssign};

use serde::{Deserialize, Serialize};

use crate::prelude::Point;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CardinalitySampleBuilder {
    count: u64,
    exact_cardinality_sum: f64,
    estimated_cardinality_sum: f64,
    absolute_relative_error_sum: f64,
    relative_error_sum: f64,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct CardinalitySample {
    pub exact_cardinality_mean: f64,
    pub estimated_cardinality_mean: f64,
    pub absolute_relative_error_mean: f64,
    pub relative_error_mean: f64,
}

impl From<CardinalitySample> for Point {
    #[inline]
    fn from(sample: CardinalitySample) -> Self {
        Point {
            x: sample.exact_cardinality_mean,
            y: sample.estimated_cardinality_mean,
        }
    }
}

impl CardinalitySampleBuilder {
    #[inline]
    pub fn update(&mut self, exact_cardinality: u64, estimated_cardinality: f64) {
        self.count += 1;
        self.exact_cardinality_sum += exact_cardinality as f64;
        self.estimated_cardinality_sum += estimated_cardinality;
        self.absolute_relative_error_sum +=
            (exact_cardinality as f64 - estimated_cardinality).abs() / exact_cardinality.max(1) as f64;
        self.relative_error_sum +=
            (exact_cardinality as f64 - estimated_cardinality) / exact_cardinality.max(1) as f64;
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

impl Add for CardinalitySampleBuilder {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            count: self.count + other.count,
            exact_cardinality_sum: self.exact_cardinality_sum + other.exact_cardinality_sum,
            estimated_cardinality_sum: self.estimated_cardinality_sum + other.estimated_cardinality_sum,
            absolute_relative_error_sum: self.absolute_relative_error_sum + other.absolute_relative_error_sum,
            relative_error_sum: self.relative_error_sum + other.relative_error_sum,
        }
    }
}

impl AddAssign for CardinalitySampleBuilder {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl From<CardinalitySampleBuilder> for CardinalitySample {
    #[inline]
    fn from(builder: CardinalitySampleBuilder) -> Self {
        assert_ne!(builder.count, 0);
        CardinalitySample {
            exact_cardinality_mean: builder.exact_cardinality_sum / builder.count as f64,
            estimated_cardinality_mean: builder.estimated_cardinality_sum / builder.count as f64,
            absolute_relative_error_mean: builder.absolute_relative_error_sum / builder.count as f64,
            relative_error_mean: builder.relative_error_sum / builder.count as f64,
        }
    }
}