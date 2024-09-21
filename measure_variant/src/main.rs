//! Script to log and compare the progress of two variants of HLL.

use hyperloglog_rs::prelude::*;
use mem_dbg::MemSize;
use statistical_comparisons::prelude::{CloudFlareHLL, SimpleHLL, TabacHLLPlusPlus};
use test_utils::prelude::{
    cardinality_samples, compare_features, read_csv, write_csv, ExtendedCardinalitySample, Set,
};
use wyhash::WyHash;

const ITERATIONS: u64 = 1_000 * 64;

fn measure<S: Set + Default + MemSize>(
    reference: Option<&[ExtendedCardinalitySample]>,
) -> Vec<ExtendedCardinalitySample> {
    let maximum_cardinality = 1 << 16;
    let model_name = S::default().model_name();
    let path = format!("{}.csv.gz", model_name);

    let stored_reports = if let Ok(stored_reports) = read_csv(path.as_str()) {
        stored_reports
    } else {
        let total_reports = cardinality_samples::<S>(ITERATIONS, maximum_cardinality);
        write_csv(total_reports.iter(), &path);
        total_reports
    };

    if let Some(reference) = reference {
        let old_errors: Vec<f64> = reference
            .iter()
            .map(|report| report.absolute_relative_error_mean())
            .collect();
        let new_errors: Vec<f64> = stored_reports
            .iter()
            .map(|report| report.absolute_relative_error_mean())
            .collect();
        let error_benchmark = compare_features(
            new_errors.as_slice(),
            old_errors.as_slice(),
            "Relative Error",
        );

        error_benchmark.print();
    }

    stored_reports
}

/// Main function to compare the progress of two variants of HLL.
fn main() {
    let reference = measure::<
        HyperLogLog<Precision4, Bits6, <Precision4 as PackedRegister<Bits6>>::Array, WyHash>,
    >(None);
    measure::<CloudFlareHLL<4, 6, WyHash>>(Some(&reference));
    measure::<SimpleHLL<WyHash, 4>>(Some(&reference));
    measure::<TabacHLLPlusPlus<Precision4, WyHash>>(Some(&reference));
}
