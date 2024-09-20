use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use rayon::prelude::*;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::u32;
use test_utils::prelude::{
    cardinality_samples_by_model, rdp, CardinalitySample, CardinalitySamplesByModel, Point,
};

fn small_float_formatter<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{value:.4}"))
}

#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone)]
pub struct CorrectionPerformance {
    pub precision: u8,
    pub bits: u8,
    #[serde(serialize_with = "small_float_formatter")]
    pub rate_of_improvement: f64,
    #[serde(serialize_with = "small_float_formatter")]
    pub uncorrected_error: f64,
    #[serde(serialize_with = "small_float_formatter")]
    pub corrected_error: f64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HashCorrection {
    pub precision: u8,
    pub bits: u8,
    pub hash_list_bias: Vec<f64>,
    pub hyperloglog_relative_bias: Vec<f64>,
    pub hash_list_cardinalities: Vec<u32>,
    pub hyperloglog_cardinalities: Vec<u32>,
}

impl HashCorrection {
    /// Returns the estimated cardinality for the provided cardinality.
    fn adjust_cardinality(
        cardinality_estimate: u32,
        cardinalities: &[u32],
        relative_errors: &[f64],
    ) -> f64 {
        if cardinality_estimate >= *cardinalities.last().unwrap() {
            return f64::from(cardinality_estimate)
                + f64::from(*relative_errors.last().unwrap()) * f64::from(cardinality_estimate)
                    / f64::from(*cardinalities.last().unwrap()).max(1.0);
        }

        if cardinality_estimate <= cardinalities[0] {
            return f64::from(cardinality_estimate)
                + f64::from(relative_errors[0]) * f64::from(cardinality_estimate)
                    / f64::from(cardinalities[0]).max(1.0);
        }

        // Otherwise, we find the partition that contains the cardinality estimate.
        let partition = cardinalities
            .windows(2)
            .position(|window| {
                let (lower, upper) = (window[0], window[1]);
                cardinality_estimate >= lower && cardinality_estimate < upper
            })
            .unwrap();

        let (lower, upper) = (cardinalities[partition], cardinalities[partition + 1]);
        let (lower_error, upper_error) =
            (relative_errors[partition], relative_errors[partition + 1]);

        let slope = f64::from(cardinality_estimate - lower) / f64::from(upper - lower)
            * f64::from(upper_error - lower_error);

        f64::from(cardinality_estimate) + f64::from(lower_error) + slope
    }

    /// Returns the estimated cardinality for the provided cardinality.
    fn adjust_hash_list_cardinality(&self, cardinality_estimate: u32) -> f64 {
        Self::adjust_cardinality(
            cardinality_estimate,
            &self.hash_list_cardinalities,
            &self.hash_list_bias,
        )
    }

    /// Returns the estimated cardinality for the provided cardinality.
    fn adjust_hyperloglog_cardinality(&self, cardinality_estimate: u32) -> f64 {
        Self::adjust_cardinality(
            cardinality_estimate,
            &self.hyperloglog_cardinalities,
            &self.hyperloglog_relative_bias,
        )
    }
}

fn correction<P: Precision>(report: &[CardinalitySample]) -> (Vec<u32>, Vec<f64>) {
    // We split the data into k partitions, and we identify the largest discontinuity
    // in each partition.
    let top_k_reports: Vec<Point> = rdp(report, 2.0);

    let number_of_reports = report.len();

    // We create the correction.
    let errors = top_k_reports
        .par_iter()
        .map(|report| report.x() - report.y())
        .collect::<Vec<f64>>();

    let cardinalities = top_k_reports
        .par_iter()
        .map(|report| report.y().round() as u32)
        .collect::<Vec<u32>>();

    // We remove in both the cardinalities and errors the entries that have the
    // same cardinality. In such cases, we average the errors.
    let mut filtered_errors = Vec::with_capacity(errors.len());
    let mut filtered_cardinalities: Vec<u32> = Vec::with_capacity(cardinalities.len());

    let mut previous_cardinality = cardinalities[0];
    let mut previous_error = errors[0];
    let mut count = 1.0;

    for (cardinality, error) in cardinalities.iter().zip(errors.iter()).skip(1) {
        if *cardinality <= previous_cardinality + number_of_reports as u32 / 42 {
            count += 1.0;
            previous_error += *error;
        } else {
            filtered_cardinalities.push(previous_cardinality);
            filtered_errors.push(previous_error / count);

            previous_cardinality = *cardinality;
            previous_error = *error;
            count = 1.0;
        }
    }

    filtered_cardinalities.push(previous_cardinality);
    filtered_errors.push(previous_error / count);

    assert!(filtered_cardinalities.len() == filtered_errors.len());
    assert!(filtered_cardinalities.len() <= cardinalities.len());

    (filtered_cardinalities, filtered_errors)
}

/// Returns the start and end CardinalitySample representing the largest stretch of near-zero relative error.
fn find_largest_stretch_of_near_zero_relative_error(report: &[CardinalitySample]) -> (usize, usize) {
    let mut start = 0;
    let mut end = 0;
    let mut current_start = 0;
    let mut current_end = 0;
    let mut current_relative_error = 0.0;

    for (index, sample) in report.iter().enumerate() {
        if sample.relative_error_mean < 0.01 {
            if current_relative_error < 0.01 {
                current_end = index;
            } else {
                current_start = index;
                current_end = index;
                current_relative_error = sample.relative_error_mean;
            }
        } else {
            if current_end - current_start > end - start {
                start = current_start;
                end = current_end;
            }
            current_relative_error = 0.0;
        }
    }

    (start, end)
}


#[allow(unsafe_code)]
/// Measures the gap between subsequent hashes in the Listhash variant of HyperLogLog.
pub fn hash_correction<P: Precision, B: Bits>(
    _multiprogress: &MultiProgress,
    only_hash_list: bool,
) -> (HashCorrection, CorrectionPerformance)
where
    P: PackedRegister<B>,
{
    let iterations: u64 = 10 * 64;
    let maximum_cardinality = 1 << 33;

    let output_path = format!("{}_{}.report.json", P::EXPONENT, B::NUMBER_OF_BITS);

    let report = if let Some(report) = std::fs::File::open(output_path.clone())
        .ok()
        .and_then(|file| serde_json::from_reader(file).ok())
    {
        report
    } else {
        let cardinality_sample_by_model: CardinalitySamplesByModel =
            cardinality_samples_by_model::<P, B, wyhash::WyHash>(iterations, maximum_cardinality);

        // We store the reports to a JSON file.

        serde_json::to_writer(
            std::fs::File::create(output_path.clone()).unwrap(),
            &cardinality_sample_by_model,
        )
        .unwrap();

        cardinality_sample_by_model
    };

    let (hash_list_cardinalities, hash_list_bias) = correction::<P>(&report.hash_list);
    let (hyperloglog_cardinalities, hyperloglog_relative_bias) = if only_hash_list {
        (vec![], vec![])
    } else {
        correction::<P>(&report.hyperloglog)
    };

    // We create the correction.
    let correction = HashCorrection {
        precision: P::EXPONENT,
        bits: B::NUMBER_OF_BITS,
        hash_list_bias,
        hyperloglog_relative_bias,
        hash_list_cardinalities,
        hyperloglog_cardinalities,
    };

    // We dump the HasHCorrection as a JSON file with the same path
    // as the output path but with a .json extension.
    let json_output_path = format!("{}_{}.correction.json", P::EXPONENT, B::NUMBER_OF_BITS);

    serde_json::to_writer(
        std::fs::File::create(json_output_path.clone()).unwrap(),
        &correction,
    )
    .unwrap();

    let uncorrected_error = report
        .hash_list
        .iter()
        .chain(report.hyperloglog.iter())
        .map(|report| report.absolute_relative_error_mean)
        .sum::<f64>()
        / (report.hash_list.len() + report.hyperloglog.len()) as f64;

    let corrected_error = (report
        .hash_list
        .iter()
        .map(|report| {
            (report.exact_cardinality_mean
                - correction
                    .adjust_hash_list_cardinality(report.estimated_cardinality_mean.round() as u32))
            .abs()
                / report.exact_cardinality_mean.max(1.0)
        })
        .sum::<f64>()
        / report.hash_list.len() as f64
        + report
            .hyperloglog
            .iter()
            .map(|report| {
                (report.exact_cardinality_mean
                    - correction.adjust_hyperloglog_cardinality(
                        report.estimated_cardinality_mean.round() as u32,
                    ))
                .abs()
                    / report.exact_cardinality_mean.max(1.0)
            })
            .sum::<f64>()
            / report.hyperloglog.len() as f64)
        / 2.0;

    // Rate of improvement.
    let rate_of_improvement = uncorrected_error / corrected_error;

    let performance = CorrectionPerformance {
        precision: P::EXPONENT,
        bits: B::NUMBER_OF_BITS,
        rate_of_improvement,
        uncorrected_error,
        corrected_error,
    };

    (correction, performance)
}
