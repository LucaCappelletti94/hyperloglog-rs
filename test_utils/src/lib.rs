#![deny(unconditional_recursion)]
mod cardinality_to_index;
mod csv;
mod ramer_douglas_peucker;
mod readable_number;
mod statistics;
mod parallel;
mod sample_builder;
mod sample_collector;
mod set;

pub mod prelude {
    pub use crate::cardinality_to_index::{
        cardinality_estimate_to_index, index_to_cardinality_estimate,
    };
    pub use crate::csv::{append_csv, read_csv, write_csv};
    pub use crate::ramer_douglas_peucker::{rdp, Point};
    pub use crate::readable_number::ReadableNumber;
    pub use crate::statistics::{
        compare_features, mean, mean_and_std, standard_deviation, BenchmarkResults, Stats,
    };
    pub use crate::parallel::*;
    pub use crate::sample_builder::{CardinalitySampleBuilder, CardinalitySample};
    pub use crate::sample_collector::{cardinality_samples, cardinality_samples_by_model, CardinalitySamplesByModel};
    pub use crate::set::Set;
}
