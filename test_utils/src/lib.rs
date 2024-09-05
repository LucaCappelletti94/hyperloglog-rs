mod csv;
mod statistics;
mod readable_number;

pub mod prelude {
    pub use crate::csv::{append_csv, read_csv, write_csv};
    pub use crate::statistics::{
        compare_features, mean, mean_and_std, standard_deviation, BenchmarkResults, Stats,
    };
    pub use crate::readable_number::ReadableNumber;
}
