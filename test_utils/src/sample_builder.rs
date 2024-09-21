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

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ExtendedCardinalitySampleBuilder {
    cardinality_sample_builder: CardinalitySampleBuilder,
    memory_requirements_sum: usize,
    time_requirements_sum: u128,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct ExtendedCardinalitySample {
    pub cardinality_sample: CardinalitySample,
    pub memory_requirements_mean: f64,
    pub time_requirements_mean: f64,
}

impl ExtendedCardinalitySample {
    #[inline]
    pub fn absolute_relative_error_mean(&self) -> f64 {
        self.cardinality_sample.absolute_relative_error_mean
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct CardinalitySample {
    pub exact_cardinality_mean: f64,
    pub estimated_cardinality_mean: f64,
    pub absolute_relative_error_mean: f64,
    pub relative_error_mean: f64,
}

impl CardinalitySample {
    #[inline]
    pub fn subtraction(&self) -> f64 {
        self.exact_cardinality_mean - self.estimated_cardinality_mean
    }
}

impl Point for CardinalitySample {
    #[inline]
    fn x(&self) -> f64 {
        self.estimated_cardinality_mean
    }

    #[inline]
    fn y(&self) -> f64 {
        // (self.exact_cardinality_mean - self.estimated_cardinality_mean).abs() / self.exact_cardinality_mean.max(1.0)
        self.subtraction()
    }
}

impl ExtendedCardinalitySampleBuilder {
    #[inline]
    pub fn update(
        &mut self,
        exact_cardinality: u64,
        estimated_cardinality: f64,
        memory_requirements: usize,
        time_requirements: u128,
    ) {
        self.cardinality_sample_builder
            .update(exact_cardinality, estimated_cardinality);
        self.memory_requirements_sum += memory_requirements;
        self.time_requirements_sum += time_requirements;
    }
}

impl CardinalitySampleBuilder {
    #[inline]
    pub fn update(&mut self, exact_cardinality: u64, estimated_cardinality: f64) {
        self.count += 1;
        self.exact_cardinality_sum += exact_cardinality as f64;
        self.estimated_cardinality_sum += estimated_cardinality;
        self.absolute_relative_error_sum += (exact_cardinality as f64 - estimated_cardinality)
            .abs()
            / exact_cardinality.max(1) as f64;
        self.relative_error_sum +=
            (exact_cardinality as f64 - estimated_cardinality) / exact_cardinality.max(1) as f64;
    }

    #[inline]
    pub fn count(&self) -> u64 {
        self.count
    }
}

impl Add for CardinalitySampleBuilder {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            count: self.count + other.count,
            exact_cardinality_sum: self.exact_cardinality_sum + other.exact_cardinality_sum,
            estimated_cardinality_sum: self.estimated_cardinality_sum
                + other.estimated_cardinality_sum,
            absolute_relative_error_sum: self.absolute_relative_error_sum
                + other.absolute_relative_error_sum,
            relative_error_sum: self.relative_error_sum + other.relative_error_sum,
        }
    }
}

impl Add for ExtendedCardinalitySampleBuilder {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            cardinality_sample_builder: self.cardinality_sample_builder
                + other.cardinality_sample_builder,
            memory_requirements_sum: self.memory_requirements_sum + other.memory_requirements_sum,
            time_requirements_sum: self.time_requirements_sum + other.time_requirements_sum,
        }
    }
}

impl AddAssign for CardinalitySampleBuilder {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl AddAssign for ExtendedCardinalitySampleBuilder {
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
            absolute_relative_error_mean: builder.absolute_relative_error_sum
                / builder.count as f64,
            relative_error_mean: builder.relative_error_sum / builder.count as f64,
        }
    }
}

impl From<ExtendedCardinalitySampleBuilder> for ExtendedCardinalitySample {
    #[inline]
    fn from(builder: ExtendedCardinalitySampleBuilder) -> Self {
        assert_ne!(builder.cardinality_sample_builder.count, 0);
        ExtendedCardinalitySample {
            cardinality_sample: builder.cardinality_sample_builder.into(),
            memory_requirements_mean: builder.memory_requirements_sum as f64
                / builder.cardinality_sample_builder.count as f64,
            time_requirements_mean: (builder.time_requirements_sum
                / builder.cardinality_sample_builder.count as u128)
                as f64,
        }
    }
}
