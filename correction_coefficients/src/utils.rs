use crate::polyfit::{fit, FitSample, PolyFit};
use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::u32;
use test_utils::prelude::{
    force_dense_cardinality_samples, uncorrected_cardinality_samples_by_model, CardinalitySample,
    CardinalitySamplesByModel,
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
    pub rate_of_hyperloglog_improvement: f64,
    #[serde(serialize_with = "small_float_formatter")]
    pub uncorrected_hyperloglog_error: f64,
    #[serde(serialize_with = "small_float_formatter")]
    pub corrected_hyperloglog_error: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterCorrection {
    pub precision: u8,
    pub bits: u8,
    /// Fitted polynomial correction for the register regime. The sorted hash list regime no longer
    /// needs a fitted table: it is estimated analytically by the width-aware occupancy inverse in the
    /// library (`HyperLogLog::hash_list_cardinality`).
    pub hyperloglog: PolyFit,
    /// The cardinality at or below which linear counting beats the bias-corrected raw register
    /// estimate (the largest sampled cardinality where linear-counting error is no worse than the
    /// corrected-raw error). Below this, a force-dense counter should report linear counting.
    pub linear_count_threshold: u32,
}

/// The number of registers `m = 2^P` as an `f64`, the normalizer for the polynomial's load variable.
fn registers<P: Precision>() -> f64 {
    (1u64 << P::EXPONENT) as f64
}

impl RegisterCorrection {
    /// Corrected cardinality in the register regime.
    fn adjust_hyperloglog_cardinality<P: Precision, B: Bits>(&self, raw: f64) -> f64 {
        self.hyperloglog.corrected(raw, registers::<P>())
    }
}

/// Builds the fit observations from per-bucket samples: the raw estimate, the exact cardinality, and
/// the bucket count (the fit weight). Empty and zero-cardinality buckets are dropped.
fn fit_samples(report: &[CardinalitySample]) -> Vec<FitSample> {
    report
        .iter()
        .filter(|s| s.count > 0 && s.exact_cardinality_mean > 0.0)
        .map(|s| FitSample {
            raw: s.estimated_cardinality_mean,
            exact: s.exact_cardinality_mean,
            count: s.count as f64,
        })
        .collect()
}

/// Returns the linear-counting threshold for the given force-dense hyperloglog samples: the largest
/// exact cardinality at which the measured linear-counting estimate is at least as accurate as the
/// bias-corrected raw register estimate. Returns 0 when linear counting never wins.
fn linear_count_threshold<P: Precision, B: Bits>(
    hyperloglog: &[CardinalitySample],
    correction: &RegisterCorrection,
) -> u32 {
    let mut samples: Vec<&CardinalitySample> = hyperloglog.iter().collect();
    samples.sort_by(|a, b| {
        a.exact_cardinality_mean
            .partial_cmp(&b.exact_cardinality_mean)
            .unwrap()
    });

    let mut threshold = 0u32;
    for sample in samples {
        let exact = sample.exact_cardinality_mean.max(1.0);
        let corrected_raw =
            correction.adjust_hyperloglog_cardinality::<P, B>(sample.estimated_cardinality_mean);
        let corrected_raw_error = (exact - corrected_raw).abs() / exact;
        let linear_counting_error = (exact - sample.linear_counting_estimate_mean).abs() / exact;
        if linear_counting_error <= corrected_raw_error {
            threshold = sample.exact_cardinality_mean.round() as u32;
        }
    }

    threshold
}

#[allow(unsafe_code)]
/// Fits the dense register-regime bias-correction polynomial and locates the linear-counting
/// threshold for the given precision and register width, returning the correction and its measured
/// performance.
pub fn register_correction<P: Precision, B: Bits>(
    multiprogress: &MultiProgress,
) -> (RegisterCorrection, CorrectionPerformance)
where
    P: PackedRegister<B>,
{
    let output_path = format!("{}_{}.report.json", P::EXPONENT, B::NUMBER_OF_BITS);

    let report = if let Some(report) = std::fs::File::open(output_path.clone())
        .ok()
        .and_then(|file| serde_json::from_reader(file).ok())
    {
        report
    } else {
        let iterations: u64 = 12_800_000 * 64 / (1 << (P::EXPONENT as u64 - 4));
        // A degree-8 polynomial fit needs far fewer samples than the old dense RDP table, and the
        // register correction only applies up to 7.5 * 2^P (above it the raw estimate is used
        // uncorrected). Sampling to 8 * 2^P covers both arms (the hash-list capacity sits well below
        // it) at a fraction of the old 200 * 2^P cost, which keeps a from-scratch regeneration fast.
        let maximum_cardinality = 8 * (1 << P::EXPONENT);
        let cardinality_sample_by_model: CardinalitySamplesByModel =
            uncorrected_cardinality_samples_by_model::<P, B>(
                iterations,
                maximum_cardinality,
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

    let m = registers::<P>();

    // The register correction only applies up to the 7.5 * 2^P bound (above it the raw estimate is
    // used uncorrected), so restrict the fit to that domain.
    let filtered_hyperloglog: Vec<CardinalitySample> = report
        .hyperloglog
        .iter()
        .filter(|report| report.exact_cardinality_mean < 7.5 * m)
        .cloned()
        .collect();
    let hyperloglog = fit(&fit_samples(&filtered_hyperloglog), m);

    // We create the correction.
    let mut correction = RegisterCorrection {
        precision: P::EXPONENT,
        bits: B::NUMBER_OF_BITS,
        hyperloglog,
        linear_count_threshold: 0,
    };

    // Locate the linear-counting threshold: the largest cardinality where the linear-counting
    // estimate is at least as accurate as the bias-corrected raw register estimate. Below this load a
    // force-dense counter is more accurate using linear counting; above it the bias correction wins.
    // This uses a separate, cheap force-dense sweep (capped at the bias-correction upper bound, the
    // linear-counting region) so the heavy, cached natural-regime sampling above is left untouched.
    let linear_region_max_cardinality = (7.5 * (1u64 << P::EXPONENT) as f64) as u64;
    let force_dense_iterations: u64 = (12_800_000 * 64 / (1u64 << P::EXPONENT)).max(4096);
    let force_dense_samples = force_dense_cardinality_samples::<P, B>(
        force_dense_iterations,
        linear_region_max_cardinality,
        multiprogress,
    );
    correction.linear_count_threshold =
        linear_count_threshold::<P, B>(&force_dense_samples, &correction);

    // We dump the RegisterCorrection as a JSON file with the same path
    // as the output path but with a .json extension.
    let json_output_path = format!("{}_{}.correction.json", P::EXPONENT, B::NUMBER_OF_BITS);

    serde_json::to_writer(
        std::fs::File::create(json_output_path.clone()).unwrap(),
        &correction,
    )
    .unwrap();

    let uncorrected_hyperloglog_error = report
        .hyperloglog
        .iter()
        .map(|report| report.absolute_relative_error_mean)
        .sum::<f64>()
        / report.hyperloglog.len() as f64;

    let corrected_hyperloglog_error = report
        .hyperloglog
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
    let rate_of_hyperloglog_improvement =
        uncorrected_hyperloglog_error / corrected_hyperloglog_error;

    let performance = CorrectionPerformance {
        precision: P::EXPONENT,
        bits: B::NUMBER_OF_BITS,
        rate_of_hyperloglog_improvement,
        uncorrected_hyperloglog_error,
        corrected_hyperloglog_error,
    };

    (correction, performance)
}
