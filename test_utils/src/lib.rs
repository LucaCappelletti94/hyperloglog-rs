mod csv;
mod statistics;

pub mod prelude {
    pub use crate::csv::{read_csv, write_csv, append_csv};
    pub use crate::statistics::{
        compare_features, mean, mean_and_std, standard_deviation, BenchmarkResults, Stats,
    };
}
