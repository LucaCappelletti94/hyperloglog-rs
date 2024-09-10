use hyperloglog_rs::composite_hash::CompositeHash;

use crate::ramer_douglas_peucker::{rdp, Point};
use hyperloglog_rs::prelude::*;
use hyperloglog_rs::utils::VariableWord;
use indicatif::MultiProgress;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use test_utils::prelude::{read_csv, write_csv};
use twox_hash::XxHash64;

fn small_float_formatter<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{value:.2}"))
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
    pub relative_errors: Vec<f64>,
    pub cardinalities: Vec<u32>,
}

impl HashCorrection {
    /// Returns the estimated cardinality for the provided cardinality.
    fn estimate_cardinality(&self, cardinality_estimate: u32) -> f64 {
        if cardinality_estimate >= *self.cardinalities.last().unwrap() {
            return f64::from(cardinality_estimate)
                + self.relative_errors.last().unwrap() * f64::from(cardinality_estimate)
                    / f64::from(*self.cardinalities.last().unwrap());
        }

        if cardinality_estimate <= self.cardinalities[0] {
            return f64::from(cardinality_estimate)
                + self.relative_errors[0] * f64::from(cardinality_estimate)
                    / f64::from(self.cardinalities[0]).max(1.0);
        }

        // Otherwise, we find the partition that contains the cardinality estimate.
        let partition = self
            .cardinalities
            .windows(2)
            .position(|window| {
                let (lower, upper) = (window[0], window[1]);
                cardinality_estimate >= lower && cardinality_estimate < upper
            })
            .unwrap();

        let (lower, upper) = (
            self.cardinalities[partition],
            self.cardinalities[partition + 1],
        );
        let (lower_error, upper_error) = (
            self.relative_errors[partition],
            self.relative_errors[partition + 1],
        );

        let slope = f64::from(cardinality_estimate - lower) / f64::from(upper - lower).max(1.0)
            * (upper_error - lower_error);

        f64::from(cardinality_estimate) + lower_error + slope
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone)]
struct CardinalityError {
    exact_cardinality: f64,
    cardinality: f64,
}

impl From<CardinalityError> for Point {
    fn from(error: CardinalityError) -> Self {
        Self::from((error.cardinality, error.exact_cardinality))
    }
}

/// Measures the gap between subsequent hashes in the Listhash variant of HyperLogLog.
pub fn hash_correction<CH>(multiprogress: &MultiProgress) -> (HashCorrection, CorrectionPerformance)
where
    CH::Precision: ArrayRegister<CH::Bits>,
    CH: CompositeHash,
{
    let iterations: u32 = if CH::Precision::EXPONENT < 9 {
        4_000_000
    } else if CH::Precision::EXPONENT < 10 {
        1_000_000
    } else {
        10_000 / u32::from(CH::Precision::EXPONENT)
    };

    let progress_bar = multiprogress.add(ProgressBar::new(iterations as u64));
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "Samples: [{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
    );

    let random_state = 6_539_823_745_562_884_u64;

    let output_path = format!(
        "{}.csv",
        core::any::type_name::<CH>()
            .to_lowercase()
            .replace("::", "_")
            .replace("hyperloglog_rs_", "")
            .replace("composite_hash_", "")
            .replace("switch_", "")
            .replace("gaps_", "")
            .replace("precisions_precision", "")
            .replace("bits_bits", "")
    );

    let total_report = if let Ok(reports) = read_csv::<CardinalityError>(&output_path) {
        reports
    } else {
        let total_report: HashMap<u32, (u32, CardinalityError)> = ParallelIterator::reduce(
            ParallelIterator::fold(
                (0..iterations)
                    .into_par_iter()
                    .progress_with(progress_bar)
                    .map(|i| {
                        let random_state = splitmix64(random_state.wrapping_mul(u64::from(i) + 1));
                        let mut hash_set = HashSet::with_capacity(100_000);
                        let mut hll = Hybrid::<
                            PlusPlus<
                                CH::Precision,
                                CH::Bits,
                                <CH::Precision as ArrayRegister<CH::Bits>>::Packed,
                                XxHash64,
                            >,
                            CH,
                        >::default();
                        let mut report: Vec<CardinalityError> = Vec::with_capacity(100_000);

                        for value in iter_random_values::<u64>(1_000_000, None, Some(random_state))
                        {
                            let cardinality = f64::from(
                                hll.duplicates().unwrap() + hll.number_of_hashes().unwrap(),
                            );
                            let exact_cardinality =
                                f64::from(u32::try_from(hash_set.len()).unwrap());
                            report.push(CardinalityError {
                                exact_cardinality,
                                cardinality,
                            });

                            hash_set.insert(value);
                            hll.insert(&value);
                            if !hll.is_hash_list() {
                                break;
                            }
                        }

                        report
                    }),
                || HashMap::new(),
                |mut acc, report| {
                    for r in report {
                        acc.entry((r.cardinality * 10.0).round() as u32)
                            .and_modify(|(count, reports): &mut (u32, CardinalityError)| {
                                reports.exact_cardinality += r.exact_cardinality;
                                reports.cardinality += r.cardinality;
                                *count += 1;
                            })
                            .or_insert((1, r));
                    }
                    acc
                },
            ),
            || HashMap::new(),
            |mut acc, report| {
                for (key, (count, report)) in report {
                    acc.entry(key)
                        .and_modify(|(acc_count, acc_report)| {
                            acc_report.cardinality += report.cardinality;
                            acc_report.exact_cardinality += report.exact_cardinality;
                            *acc_count += count;
                        })
                        .or_insert((count, report));
                }
                acc
            },
        );

        // We convert the hashmap to a vector.
        let mut total_report: Vec<_> = total_report
            .into_iter()
            .filter_map(|(_, (count, report))| {
                let mut report = report;
                report.cardinality /= f64::from(count);
                report.exact_cardinality /= f64::from(count);
                Some(report)
            })
            .collect();

        // We store the mined data-points to a CSV so to avoid recomputing them
        // in the future.
        write_csv(total_report.iter(), &output_path);

        // We sort the results by the estimated cardinality, which most likely will be the
        // already sorted but it is not guaranteed.
        total_report.sort_by(|a, b| a.cardinality.partial_cmp(&b.cardinality).unwrap());

        // We expect at least one value in the report.
        assert!(!total_report.is_empty());

        total_report
    };

    // We convert the total_report to a list of points.
    let points: Vec<Point> = total_report.iter().copied().map(|report| report.into()).collect();

    // We determine the maximal number of points to consider.
    let k = usize::from(CH::Precision::EXPONENT) * 4;

    // We split the data into k partitions, and we identify the largest discontinuity
    // in each partition.
    let top_k_reports: Vec<Point> = rdp(points.as_slice(), 0.0000001, k);

    // We create the correction.
    let correction = HashCorrection {
        precision: CH::Precision::EXPONENT,
        bits: CH::Bits::NUMBER_OF_BITS,
        relative_errors: top_k_reports
            .iter()
            .map(|report| report.y() - report.x())
            .collect::<Vec<f64>>(),
        cardinalities: top_k_reports
            .iter()
            .map(|report| report.x().round() as u32)
            .collect::<Vec<u32>>(),
    };

    // We dump the HasHCorrection as a JSON file with the same path
    // as the output path but with a .json extension.
    let json_output_path = output_path.replace(".csv", ".json");

    serde_json::to_writer(
        std::fs::File::create(json_output_path.clone()).unwrap(),
        &correction,
    )
    .unwrap();

    let uncorrected_error = total_report
        .iter()
        .map(|report| {
            (f64::from(report.exact_cardinality) - report.cardinality).abs()
                / f64::from(report.exact_cardinality).max(1.0)
        })
        .sum::<f64>()
        / total_report.len() as f64;

    let corrected_error = total_report
        .iter()
        .map(|report| {
            (f64::from(report.exact_cardinality)
                - correction.estimate_cardinality(report.cardinality.round() as u32))
            .abs()
                / f64::from(report.exact_cardinality).max(1.0)
        })
        .sum::<f64>()
        / total_report.len() as f64;

    // Rate of improvement.
    let rate_of_improvement = uncorrected_error / corrected_error;

    let performance = CorrectionPerformance {
        precision: CH::Precision::EXPONENT,
        bits: CH::Bits::NUMBER_OF_BITS,
        rate_of_improvement,
        uncorrected_error,
        corrected_error,
    };

    (correction, performance)
}
