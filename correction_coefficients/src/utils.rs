use hyperloglog_rs::prelude::*;
use hyperloglog_rs::sigma_tau::ertl_cardinality_from_moments;
use indicatif::MultiProgress;
use serde::{Deserialize, Serialize};
use test_utils::prelude::{
    force_dense_cardinality_samples, uncorrected_cardinality_samples_by_model, CardinalitySample,
    CardinalitySamplesByModel,
};

/// The per-cell linear-counting crossover: at or below this cardinality a force-dense counter
/// should report linear counting instead of Ertl's tau/sigma estimate, because linear counting is
/// the more accurate estimator in the low-load regime.
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterCorrection {
    pub precision: u8,
    pub bits: u8,
    /// The cardinality at or below which linear counting beats Ertl's tau/sigma register estimate
    /// (the largest sampled cardinality where linear-counting error is no worse than sigma/tau's).
    pub linear_count_threshold: u32,
}

/// Number of registers `m = 2^P` as an `f64`.
fn registers<P: Precision>() -> f64 {
    (1u64 << P::EXPONENT) as f64
}

/// Reconstructs the sigma/tau moments (`H`, `zeros`) from a mean-of-sample record and evaluates
/// [`ertl_cardinality_from_moments`] with saturation treated as zero. The reconstruction is
/// approximate (the sample stores means, not per-trial moments) but is only used to find the
/// crossover cardinality, which is a coarse decision insensitive to that approximation.
fn sigma_tau_estimate<P: Precision, B: Bits>(sample: &CardinalitySample) -> f64 {
    let m = registers::<P>();
    let harmonic_sum = P::ALPHA * m * m / sample.estimated_cardinality_mean.max(1.0);
    // `linear_counting_estimate = m * ln(m / zeros)`  =>  `zeros = m * exp(-linear_counting / m)`.
    let zeros = m * FloatOps::exp(-sample.linear_counting_estimate_mean / m);
    ertl_cardinality_from_moments::<P, B>(harmonic_sum, zeros, 0.0)
}

/// Returns the linear-counting threshold for the given force-dense hyperloglog samples: the largest
/// exact cardinality at which the measured linear-counting estimate is at least as accurate as the
/// sigma/tau register estimate. Returns 0 when linear counting never wins.
fn linear_count_threshold<P: Precision, B: Bits>(hyperloglog: &[CardinalitySample]) -> u32 {
    let mut samples: Vec<&CardinalitySample> = hyperloglog.iter().collect();
    samples.sort_by(|a, b| {
        a.exact_cardinality_mean
            .partial_cmp(&b.exact_cardinality_mean)
            .unwrap()
    });

    let mut threshold = 0u32;
    for sample in samples {
        let exact = sample.exact_cardinality_mean.max(1.0);
        let sigma_tau = sigma_tau_estimate::<P, B>(sample);
        let sigma_tau_error = (exact - sigma_tau).abs() / exact;
        let linear_counting_error = (exact - sample.linear_counting_estimate_mean).abs() / exact;
        if linear_counting_error <= sigma_tau_error {
            threshold = sample.exact_cardinality_mean.round() as u32;
        }
    }

    threshold
}

/// Locates the linear-counting crossover for the given precision and register width. Runs a Monte
/// Carlo sweep (or reuses the cached `<P>_<B>.report.json` output) to draw naturally-grown counter
/// samples, then a cheap force-dense sweep in the linear-counting region to pin down the
/// crossover, then emits the `RegisterCorrection` as `<P>_<B>.correction.json` for downstream
/// tooling.
pub fn register_correction<P: Precision, B: Bits>(
    multiprogress: &MultiProgress,
) -> RegisterCorrection
where
    P: PackedRegister<B>,
{
    let output_path = format!("{}_{}.report.json", P::EXPONENT, B::NUMBER_OF_BITS);

    let _report = if let Some(report) = std::fs::File::open(output_path.clone())
        .ok()
        .and_then(|file| serde_json::from_reader::<_, CardinalitySamplesByModel>(file).ok())
    {
        report
    } else {
        let iterations: u64 = 12_800_000 * 64 / (1 << (P::EXPONENT as u64 - 4));
        // The threshold only applies up to 7.5 * 2^P, so sampling to 8 * 2^P is sufficient. The
        // report is cached to disk so a repeated run reuses it without redoing Monte Carlo.
        let maximum_cardinality = 8 * (1 << P::EXPONENT);
        let cardinality_sample_by_model: CardinalitySamplesByModel =
            uncorrected_cardinality_samples_by_model::<P, B>(
                iterations,
                maximum_cardinality,
                multiprogress,
            );

        serde_json::to_writer(
            std::fs::File::create(output_path.clone()).unwrap(),
            &cardinality_sample_by_model,
        )
        .unwrap();

        cardinality_sample_by_model
    };

    // Force-dense sweep in the linear-counting region (capped at the sigma/tau upper bound), used
    // solely to pin down the crossover. Left independent of the natural-regime sampling above so
    // the heavy cached report is untouched.
    let linear_region_max_cardinality = (7.5 * (1u64 << P::EXPONENT) as f64) as u64;
    let force_dense_iterations: u64 = (12_800_000 * 64 / (1u64 << P::EXPONENT)).max(4096);
    let force_dense_samples = force_dense_cardinality_samples::<P, B>(
        force_dense_iterations,
        linear_region_max_cardinality,
        multiprogress,
    );

    let correction = RegisterCorrection {
        precision: P::EXPONENT,
        bits: B::NUMBER_OF_BITS,
        linear_count_threshold: linear_count_threshold::<P, B>(&force_dense_samples),
    };

    // Persist the threshold as JSON with the same naming scheme as before.
    let json_output_path = format!("{}_{}.correction.json", P::EXPONENT, B::NUMBER_OF_BITS);
    serde_json::to_writer(
        std::fs::File::create(json_output_path).unwrap(),
        &correction,
    )
    .unwrap();

    correction
}
