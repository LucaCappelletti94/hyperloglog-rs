use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::u32;
use test_utils::prelude::{
    rdp, uncorrected_cardinality_samples_by_model, CardinalitySample, CardinalitySamplesByModel,
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
    pub rate_of_hash_list_improvement: f64,
    #[serde(serialize_with = "small_float_formatter")]
    pub uncorrected_hash_list_error: f64,
    #[serde(serialize_with = "small_float_formatter")]
    pub corrected_hash_list_error: f64,
    #[serde(serialize_with = "small_float_formatter")]
    pub rate_of_hyperloglog_improvement: f64,
    #[serde(serialize_with = "small_float_formatter")]
    pub uncorrected_hyperloglog_error: f64,
    #[serde(serialize_with = "small_float_formatter")]
    pub corrected_hyperloglog_error: f64,
}

#[derive(Debug, Serialize, Deserialize)]
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
    fn adjust_hash_list_cardinality<P: Precision, B: Bits>(
        &self,
        cardinality_estimate: f64,
    ) -> f64 {
        correct_cardinality::<P, B>(
            cardinality_estimate,
            &self.hash_list_cardinalities,
            &self.hash_list_bias,
        )
    }

    /// Returns the estimated cardinality for the provided cardinality.
    fn adjust_hyperloglog_cardinality<P: Precision, B: Bits>(
        &self,
        cardinality_estimate: f64,
    ) -> f64 {
        correct_cardinality::<P, B>(
            cardinality_estimate,
            &self.hyperloglog_cardinalities,
            &self.hyperloglog_relative_bias,
        )
    }
}

fn correction<P: Precision>(report: &[CardinalitySample], tolerance: f64) -> (Vec<u32>, Vec<f64>) {
    // We split the data into k partitions, and we identify the largest discontinuity
    // in each partition.
    let mut top_k_reports: Vec<CardinalitySample> = rdp(report, tolerance);

    // We sort the samples by estimated_cardinality_mean
    top_k_reports.sort_by(|a, b| {
        a.estimated_cardinality_mean
            .partial_cmp(&b.estimated_cardinality_mean)
            .unwrap()
    });

    // We remove in both the cardinalities and errors the entries that have the
    // same cardinality. In such cases, we average the errors.
    let mut biases: Vec<f64> = Vec::with_capacity(top_k_reports.len());
    let mut cardinalities: Vec<u32> = Vec::with_capacity(top_k_reports.len());

    let mut previous_cardinality = top_k_reports[0].estimated_cardinality_mean.round() as u32;
    let mut previous_error = top_k_reports[0].subtraction();
    let mut count = 1.0;

    for report in top_k_reports.into_iter().skip(1) {
        if report.estimated_cardinality_mean.round() as u32 == previous_cardinality {
            count += 1.0;
            previous_error += report.subtraction();
        } else {
            cardinalities.push(previous_cardinality);
            biases.push(previous_error / count);

            previous_cardinality = report.estimated_cardinality_mean.round() as u32;
            previous_error = report.subtraction();
            count = 1.0;
        }
    }

    cardinalities.push(previous_cardinality);
    biases.push(previous_error / count);

    assert!(cardinalities.len() == biases.len());
    assert!(cardinalities.len() <= cardinalities.len());

    (cardinalities, biases)
}

#[allow(unsafe_code)]
/// Measures the gap between subsequent hashes in the Listhash variant of HyperLogLog.
pub fn hash_correction<P: Precision, B: Bits>(
    multiprogress: &MultiProgress,
    only_hash_list: bool,
) -> (HashCorrection, CorrectionPerformance)
where
    P: PackedRegister<B>,
{
    let iterations: u64 = 10_000 * 64;
    let maximum_cardinality = 25 * (1 << P::EXPONENT);

    let output_path = format!("{}_{}.report.json", P::EXPONENT, B::NUMBER_OF_BITS);

    let report = if let Some(report) = std::fs::File::open(output_path.clone())
        .ok()
        .and_then(|file| serde_json::from_reader(file).ok())
    {
        report
    } else {
        let cardinality_sample_by_model: CardinalitySamplesByModel =
            uncorrected_cardinality_samples_by_model::<P, B>(
                iterations,
                maximum_cardinality,
                only_hash_list,
                multiprogress,
            );

        // We store the reports to a JSON file.

        serde_json::to_writer(
            std::fs::File::create(output_path.clone()).unwrap(),
            &cardinality_sample_by_model,
        )
        .unwrap();

        cardinality_sample_by_model
    };

    let (hash_list_cardinalities, hash_list_bias) = correction::<P>(&report.hash_list, 0.01);
    let (hyperloglog_cardinalities, hyperloglog_relative_bias) = if only_hash_list {
        (vec![], vec![])
    } else {
        correction::<P>(&report.hyperloglog_fully_imprinted, 0.15)
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

    let uncorrected_hash_list_error = report
        .hash_list
        .iter()
        .map(|report| report.absolute_relative_error_mean)
        .sum::<f64>()
        / report.hash_list.len() as f64;

    let uncorrected_hyperloglog_error = report
        .hyperloglog_fully_imprinted
        .iter()
        .map(|report| report.absolute_relative_error_mean)
        .sum::<f64>()
        / report.hyperloglog.len() as f64;

    let corrected_hash_list_error = report
        .hash_list
        .iter()
        .map(|report| {
            (report.exact_cardinality_mean
                - correction
                    .adjust_hash_list_cardinality::<P, B>(report.estimated_cardinality_mean))
            .abs()
                / report.exact_cardinality_mean.max(1.0)
        })
        .sum::<f64>()
        / report.hash_list.len() as f64;

    let corrected_hyperloglog_error = report
        .hyperloglog_fully_imprinted
        .iter()
        .map(|report| {
            (report.exact_cardinality_mean
                - correction
                    .adjust_hyperloglog_cardinality::<P, B>(report.estimated_cardinality_mean))
            .abs()
                / report.exact_cardinality_mean.max(1.0)
        })
        .sum::<f64>()
        / report.hyperloglog.len() as f64;

    // Rate of improvement.
    let rate_of_hash_list_improvemet = uncorrected_hash_list_error / corrected_hash_list_error;
    let rate_of_hyperloglog_improvemet =
        uncorrected_hyperloglog_error / corrected_hyperloglog_error;

    let performance = CorrectionPerformance {
        precision: P::EXPONENT,
        bits: B::NUMBER_OF_BITS,
        rate_of_hash_list_improvement: rate_of_hash_list_improvemet,
        uncorrected_hash_list_error,
        corrected_hash_list_error,
        rate_of_hyperloglog_improvement: rate_of_hyperloglog_improvemet,
        uncorrected_hyperloglog_error,
        corrected_hyperloglog_error,
    };

    (correction, performance)
}
