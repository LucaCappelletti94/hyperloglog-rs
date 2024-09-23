//! Script to log and compare the progress of two variants of HLL.

use hyperloglog_rs::prelude::*;
use hypertwobits::h2b;
use mem_dbg::MemSize;
use serde::{Deserialize, Serialize};
use statistical_comparisons::prelude::{
    AlecHLL, CloudFlareHLL, HyperTwoBits, RustHLL, SimpleHLL, SourMash, TabacHLL, TabacHLLPlusPlus,
};
use test_utils::prelude::{cardinality_samples, compare_features, ExtendedCardinalitySample, Set};
use wyhash::WyHash;

const ITERATIONS: u64 = 10 * 64;

#[derive(Serialize, Deserialize)]
struct Measurement {
    measures: Vec<ExtendedCardinalitySample>,
    precision: Option<u8>,
    bits: Option<u8>,
    model_name: String
}

fn measure<S: Set + Default + MemSize>(reference: Option<&Measurement>) -> Measurement {
    let maximum_cardinality = 1 << 25;
    let model_name = S::default().model_name();
    let path = format!("{}.json", model_name);

    let stored_reports: Measurement = if let Some(stored_reports) =
        std::fs::File::open(path.as_str())
            .ok()
            .and_then(|file| serde_json::from_reader(&file).ok())
    {
        stored_reports
    } else {
        let total_reports = cardinality_samples::<S>(ITERATIONS, maximum_cardinality);

        let measurement = Measurement {
            measures: total_reports,
            precision: S::default().precision(),
            bits: S::default().bits(),
            model_name: model_name.clone(),
        };

        serde_json::to_writer(&std::fs::File::create(path.as_str()).unwrap(), &measurement)
            .unwrap();

        measurement
    };

    if let Some(reference) = reference {
        let old_errors: Vec<f64> = reference
            .measures
            .iter()
            .map(|report| report.absolute_relative_error_mean())
            .collect();
        let new_errors: Vec<f64> = stored_reports
            .measures
            .iter()
            .map(|report| report.absolute_relative_error_mean())
            .collect();

        debug_assert!(
            stored_reports.measures.iter().zip(reference.measures.iter()).all(|(new, old)| {
                new.count() == old.count()
            }),
            "The occurrences in both reports should be the same, or the buckets have changed!"
        );

        let occurrences: Vec<usize> = stored_reports
            .measures
            .iter()
            .map(|report| report.count())
            .collect();

        let error_benchmark = compare_features(
            new_errors.as_slice(),
            old_errors.as_slice(),
            occurrences.as_slice(),
            "Relative Error",
            model_name.as_ref(),
            reference.model_name.as_ref(),
        );

        error_benchmark.print();
    }

    stored_reports
}

/// Main function to compare the progress of two variants of HLL.
fn main() {
    let reference = measure::<
        HyperLogLog<Precision9, Bits6, <Precision9 as PackedRegister<Bits6>>::Vec, WyHash>,
    >(None);
    measure::<CloudFlareHLL<{ Precision9::EXPONENT as usize }, 6, WyHash>>(Some(&reference));
    measure::<SimpleHLL<WyHash, { Precision9::EXPONENT as usize }>>(Some(&reference));
    measure::<TabacHLLPlusPlus<Precision9, WyHash>>(Some(&reference));
    measure::<TabacHLL<Precision9, WyHash>>(Some(&reference));
    measure::<AlecHLL<Precision9>>(Some(&reference));
    measure::<RustHLL<Precision9>>(Some(&reference));
    measure::<SourMash<Precision9>>(Some(&reference));
    measure::<HyperTwoBits<h2b::M2048, WyHash>>(Some(&reference));
}
