use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::iter::Sum;
use std::u32;
use test_utils::prelude::{rdp, read_csv, write_csv, Point};
use twox_hash::XxHash64;

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
    pub hashlist_smallest_maximal_cardinality: u32,
    pub hashlist_mean_maximal_cardinality: u32,
    pub hashlist_largest_maximal_cardinality: u32,
    pub hashlist_relative_errors: Vec<f64>,
    pub hyperloglog_relative_errors: Vec<f64>,
    pub hashlist_cardinalities: Vec<u32>,
    pub hyperloglog_cardinalities: Vec<u32>,
    pub hyperloglog_slope: f64,
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
    fn adjust_hashlist_cardinality(&self, cardinality_estimate: u32) -> f64 {
        Self::adjust_cardinality(
            cardinality_estimate,
            &self.hashlist_cardinalities,
            &self.hashlist_relative_errors,
        )
    }

    /// Returns the estimated cardinality for the provided cardinality.
    fn adjust_hyperloglog_cardinality(&self, cardinality_estimate: u32) -> f64 {
        if cardinality_estimate >= *self.hyperloglog_cardinalities.last().unwrap() {
            return f64::from(cardinality_estimate)
                + self.hyperloglog_slope
                    * f64::from(
                        cardinality_estimate - *self.hyperloglog_cardinalities.last().unwrap(),
                    );
        }

        Self::adjust_cardinality(
            cardinality_estimate,
            &self.hyperloglog_cardinalities,
            &self.hyperloglog_relative_errors,
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone)]
struct CardinalityError {
    exact_cardinality: f64,
    cardinality: f64,
    times_we_saw_this_cardinality: u32,
}

impl CardinalityError {
    fn new(exact_cardinality: f64, cardinality: f64) -> Self {
        Self {
            exact_cardinality,
            cardinality,
            times_we_saw_this_cardinality: 1,
        }
    }
}

impl Sum for CardinalityError {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut sum = Self::default();
        for item in iter {
            sum.exact_cardinality += item.exact_cardinality;
            sum.cardinality += item.cardinality;
        }
        sum
    }
}

impl From<CardinalityError> for Point {
    fn from(error: CardinalityError) -> Self {
        Self::from((
            f64::from(error.cardinality),
            f64::from(error.exact_cardinality),
        ))
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone)]
pub struct Metadata {
    hashlist_minimum_cardinality: u32,
    hashlist_mean_cardinality: u32,
    hashlist_maximal_cardinality: u32,
}

impl Metadata {
    pub fn new(cardinality: u32) -> Self {
        Self {
            hashlist_maximal_cardinality: cardinality,
            hashlist_minimum_cardinality: if cardinality == 0 {
                u32::MAX
            } else {
                cardinality
            },
            hashlist_mean_cardinality: cardinality,
        }
    }
}

fn correction<P: Precision>(report: &[Point]) -> (Vec<u32>, Vec<f64>) {
    // We determine the maximal number of points to consider.
    let k = 9;

    // We split the data into k partitions, and we identify the largest discontinuity
    // in each partition.
    let top_k_reports: Vec<Point> = rdp(report, 0.25, k);

    // We create the correction.
    let errors = top_k_reports
        .par_iter()
        .map(|report| report.y() - report.x())
        .collect::<Vec<f64>>();

    let cardinalities = top_k_reports
        .par_iter()
        .map(|report| report.x().round() as u32)
        .collect::<Vec<u32>>();

    // We check that the cardinalities are sorted.
    debug_assert!(cardinalities.is_sorted());

    // We remove in both the cardinalities and errors the entries that have the
    // same cardinality. In such cases, we average the errors.
    let mut filtered_errors = Vec::with_capacity(errors.len());
    let mut filtered_cardinalities = Vec::with_capacity(cardinalities.len());

    let mut previous_cardinality = cardinalities[0];
    let mut previous_error = errors[0];
    let mut count = 1.0;

    for (cardinality, error) in cardinalities.iter().zip(errors.iter()).skip(1) {
        if *cardinality <= previous_cardinality + 1 {
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

    // We check that the filtered cardinalities are sorted.
    debug_assert!(filtered_cardinalities.is_sorted());

    (filtered_cardinalities, filtered_errors)
}

/// Returns the bias and angular coefficient that best fits the provided points.
fn least_squares_linear_regression(points: &[Point]) -> f64 {
    let n = points.len() as f64;

    let sum_x = points.iter().map(|point| point.x()).sum::<f64>();
    let sum_y = points.iter().map(|point| point.y()).sum::<f64>();

    let sum_y_squared = points
        .iter()
        .map(|point| point.y() * point.y())
        .sum::<f64>();
    let sum_xy = points
        .iter()
        .map(|point| point.x() * point.y())
        .sum::<f64>();

    (n * sum_xy - sum_x * sum_y) / (n * sum_y_squared - sum_y * sum_y)
}

/// Merges the sorted two arrays of CardinalityError into a single array.
fn merge_sorted_arrays(
    left: &[CardinalityError],
    right: &[CardinalityError],
) -> Vec<CardinalityError> {
    let mut merged: Vec<CardinalityError> = Vec::with_capacity(left.len() + right.len());

    let mut left_iter = left.iter();
    let mut right_iter = right.iter();

    let mut maybe_left = left_iter.next();
    let mut maybe_right = right_iter.next();

    while let (Some(left), Some(right)) = (maybe_left, maybe_right) {
        let left_cardinality = left.cardinality;
        let right_cardinality = right.cardinality;

        if (left_cardinality - right_cardinality).abs() < 0.4 {
            merged.push(CardinalityError {
                exact_cardinality: left.exact_cardinality + right.exact_cardinality,
                cardinality: left_cardinality,
                times_we_saw_this_cardinality: left.times_we_saw_this_cardinality
                    + right.times_we_saw_this_cardinality,
            });
            maybe_left = left_iter.next();
            maybe_right = right_iter.next();
            continue;
        }

        if left_cardinality < right_cardinality {
            merged.push(*left);
            maybe_left = left_iter.next();
        } else if left_cardinality > right_cardinality {
            merged.push(*right);
            maybe_right = right_iter.next();
        }
    }

    merged.extend(maybe_left);
    merged.extend(maybe_right);

    merged.extend(left_iter);
    merged.extend(right_iter);

    // Due to the merger of cardinalities, we may introduce small errors in the
    // ordering, sor we need to sort it once more.
    merged.sort_by(|a, b| a.cardinality.partial_cmp(&b.cardinality).unwrap());

    merged
}

/// Measures the gap between subsequent hashes in the Listhash variant of HyperLogLog.
pub fn hash_correction<P: Precision, B: Bits>(
    multiprogress: &MultiProgress,
) -> (HashCorrection, CorrectionPerformance)
where
    P: ArrayRegister<B>,
{
    let iterations: u32 = 8_000_000 / (1 << (P::EXPONENT - 4));

    let progress_bar = multiprogress.add(ProgressBar::new(iterations as u64));
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "Samples: [{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
    );

    let maximal_cardinality = 30 * (1 << P::EXPONENT);

    let random_state = 6_539_823_745_562_884_u64;

    let hashlist_output_path = format!("{}_{}.hashlist.csv.gz", P::EXPONENT, B::NUMBER_OF_BITS);
    let hyperloglog_output_path =
        format!("{}_{}.hyperloglog.csv.gz", P::EXPONENT, B::NUMBER_OF_BITS);

    let (metadata, hashlist_report, hyperloglog_report) = if let Ok(hashlist_report) =
        read_csv::<Point>(&hashlist_output_path)
    {
        // We also load the metadata.
        let metadata_output_path = hashlist_output_path.replace(".csv.gz", ".metadata.json");

        let metadata: Metadata =
            serde_json::from_reader(std::fs::File::open(metadata_output_path).unwrap()).unwrap();

        let hyperloglog_report = read_csv::<Point>(&hyperloglog_output_path).unwrap();

        (metadata, hashlist_report, hyperloglog_report)
    } else {
        let (mut metadata, hashlist_report, hyperloglog_report): (
            Metadata,
            Vec<CardinalityError>,
            Vec<CardinalityError>,
        ) = (0..iterations)
            .into_par_iter()
            .progress_with(progress_bar)
            .map(|i| {
                let random_state = splitmix64(random_state.wrapping_mul(u64::from(i) + 1));
                let mut hash_set = HashSet::with_capacity(maximal_cardinality);
                let mut hll =
                    HyperLogLog::<P, B, <P as ArrayRegister<B>>::Packed, XxHash64>::default();
                let mut hashlist_report: Vec<CardinalityError> =
                    Vec::with_capacity(maximal_cardinality);
                let mut hyperloglog_report: Vec<CardinalityError> =
                    Vec::with_capacity(maximal_cardinality);
                let mut hashlist_maximal_cardinality = 0;

                for value in
                    iter_random_values::<u64>(maximal_cardinality as u64, None, Some(random_state))
                {
                    let cardinality = hll.uncorrected_estimate_cardinality();
                    let exact_cardinality = f64::from(u32::try_from(hash_set.len()).unwrap());

                    if hll.is_hash_list() {
                        if let Some(report) = hashlist_report.last_mut() {
                            if (report.cardinality - cardinality).abs() < 0.4 {
                                report.exact_cardinality += exact_cardinality;
                                report.times_we_saw_this_cardinality += 1;
                            } else {
                                hashlist_report
                                    .push(CardinalityError::new(exact_cardinality, cardinality));
                            }
                        } else {
                            hashlist_report
                                .push(CardinalityError::new(exact_cardinality, cardinality));
                        }
                    } else {
                        if let Some(report) = hyperloglog_report.last_mut() {
                            if (report.cardinality - cardinality).abs() < 0.4 {
                                report.exact_cardinality += exact_cardinality;
                                report.times_we_saw_this_cardinality += 1;
                            } else {
                                hyperloglog_report
                                    .push(CardinalityError::new(exact_cardinality, cardinality));
                            }
                        } else {
                            hyperloglog_report
                                .push(CardinalityError::new(exact_cardinality, cardinality));
                        }
                    }

                    if hll.is_hash_list() {
                        hashlist_maximal_cardinality = exact_cardinality.round() as u32;
                    }

                    hash_set.insert(value);
                    hll.insert(&value);
                }

                debug_assert!(hashlist_report.is_sorted_by(|a, b| a
                    .cardinality
                    .partial_cmp(&b.cardinality)
                    .unwrap()
                    == std::cmp::Ordering::Less));

                let metadata = Metadata::new(hashlist_maximal_cardinality);

                (metadata, hashlist_report, hyperloglog_report)
            })
            .reduce(
                || (Metadata::new(0), Vec::new(), Vec::new()),
                |(acc_metadata, acc_hashlist, acc_hyperloglog),
                 (metadata, hashlist_report, hyperloglog_report)| {
                    (
                        Metadata {
                            hashlist_minimum_cardinality: metadata
                                .hashlist_minimum_cardinality
                                .min(acc_metadata.hashlist_minimum_cardinality),
                            hashlist_mean_cardinality: metadata
                                .hashlist_mean_cardinality
                                .saturating_add(acc_metadata.hashlist_mean_cardinality),
                            hashlist_maximal_cardinality: metadata
                                .hashlist_maximal_cardinality
                                .max(acc_metadata.hashlist_maximal_cardinality),
                        },
                        merge_sorted_arrays(&acc_hashlist, &hashlist_report),
                        merge_sorted_arrays(&acc_hyperloglog, &hyperloglog_report),
                    )
                },
            );

        metadata.hashlist_mean_cardinality /= iterations;

        debug_assert!(hashlist_report.is_sorted_by(|a, b| a
            .cardinality
            .partial_cmp(&b.cardinality)
            .unwrap()
            == std::cmp::Ordering::Less));
        debug_assert!(hyperloglog_report.is_sorted_by(|a, b| a
            .cardinality
            .partial_cmp(&b.cardinality)
            .unwrap()
            == std::cmp::Ordering::Less));

        // We denoise the hashlist_report and hyperloglog_report by averaging the
        // errors for the same cardinality and by removing entries that are rare.
        let minimum_occurrance = iterations / 15;

        let averaged_hashlist_report: Vec<Point> = hashlist_report
            .into_iter()
            .filter(|report| report.times_we_saw_this_cardinality >= minimum_occurrance)
            .map(|report| {
                Point::from(CardinalityError {
                    exact_cardinality: report.exact_cardinality
                        / report.times_we_saw_this_cardinality as f64,
                    cardinality: report.cardinality,
                    times_we_saw_this_cardinality: 1,
                })
            })
            .collect();

        let averaged_hyperloglog_report: Vec<Point> = hyperloglog_report
            .into_iter()
            .filter(|report| report.times_we_saw_this_cardinality >= minimum_occurrance)
            .map(|report| {
                Point::from(CardinalityError {
                    exact_cardinality: report.exact_cardinality
                        / report.times_we_saw_this_cardinality as f64,
                    cardinality: report.cardinality,
                    times_we_saw_this_cardinality: 1,
                })
            })
            .collect();

        // We filter out values with exact cardinality higher than 5 * 2^P::EXPONENT,
        // since we only want to target the range [0, 5 * 2^P::EXPONENT] and the latter
        // half is subject to more noise.
        let averaged_hashlist_report: Vec<Point> = averaged_hashlist_report
            .into_iter()
            .filter(|report| report.x() <= 10.0 * f64::from(1 << P::EXPONENT))
            .collect();

        let averaged_hyperloglog_report: Vec<Point> = averaged_hyperloglog_report
            .into_iter()
            .filter(|report| report.x() <= 10.0 * f64::from(1 << P::EXPONENT))
            .collect();

        // We store the mined data-points to a CSV so to avoid recomputing them
        // in the future.
        write_csv(averaged_hashlist_report.iter(), &hashlist_output_path);
        write_csv(averaged_hyperloglog_report.iter(), &hyperloglog_output_path);

        // We store the maximal cardinality of the hashlist as JSON metadata.
        let metadata_hashlist_output_path =
            hashlist_output_path.replace(".csv.gz", ".metadata.json");

        serde_json::to_writer(
            std::fs::File::create(metadata_hashlist_output_path.clone()).unwrap(),
            &metadata,
        )
        .unwrap();

        (
            metadata,
            averaged_hashlist_report,
            averaged_hyperloglog_report,
        )
    };

    let (hashlist_cardinalities, hashlist_relative_errors) = correction::<P>(&hashlist_report);
    let (hyperloglog_cardinalities, hyperloglog_relative_errors) =
        correction::<P>(&hyperloglog_report);

    let hyperloglog_slope =
        least_squares_linear_regression(&hyperloglog_report[hyperloglog_report.len() / 2..]);

    // We create the correction.
    let correction = HashCorrection {
        precision: P::EXPONENT,
        bits: B::NUMBER_OF_BITS,
        hashlist_largest_maximal_cardinality: metadata.hashlist_maximal_cardinality,
        hashlist_mean_maximal_cardinality: metadata.hashlist_mean_cardinality,
        hashlist_smallest_maximal_cardinality: metadata.hashlist_minimum_cardinality,
        hashlist_relative_errors,
        hyperloglog_relative_errors,
        hashlist_cardinalities,
        hyperloglog_cardinalities,
        hyperloglog_slope,
    };

    // We dump the HasHCorrection as a JSON file with the same path
    // as the output path but with a .json extension.
    let json_output_path = format!("{}_{}.hashlist.json", P::EXPONENT, B::NUMBER_OF_BITS);

    serde_json::to_writer(
        std::fs::File::create(json_output_path.clone()).unwrap(),
        &correction,
    )
    .unwrap();

    let uncorrected_error = hashlist_report
        .iter()
        .chain(hyperloglog_report.iter())
        .map(|report| ((report.y()) - report.x()).abs() / report.y().max(1.0))
        .sum::<f64>()
        / (hashlist_report.len() + hyperloglog_report.len()) as f64;

    let corrected_error = (hashlist_report
        .iter()
        .map(|report| {
            ((report.y()) - correction.adjust_hashlist_cardinality(report.x().round() as u32)).abs()
                / (report.y().abs()).max(1.0)
        })
        .sum::<f64>()
        / hashlist_report.len() as f64
        + hyperloglog_report
            .iter()
            .map(|report| {
                ((report.y())
                    - correction.adjust_hyperloglog_cardinality(report.x().round() as u32))
                .abs()
                    / (report.y()).max(1.0)
            })
            .sum::<f64>()
            / hyperloglog_report.len() as f64)
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
