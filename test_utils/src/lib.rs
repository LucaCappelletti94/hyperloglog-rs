mod csv;
mod statistics;
mod readable_number;
mod ramer_douglas_peucker;

pub mod prelude {
    pub use crate::csv::{append_csv, read_csv, write_csv};
    pub use crate::statistics::{
        compare_features, mean, mean_and_std, standard_deviation, BenchmarkResults, Stats,
    };
    pub use crate::ramer_douglas_peucker::{rdp, Point};
    pub use crate::readable_number::ReadableNumber;
}
