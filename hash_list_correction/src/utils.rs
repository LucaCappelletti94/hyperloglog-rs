use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use rayon::prelude::*;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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

    pub fn merge(self, other: Self) -> Self {
        Self {
            hashlist_minimum_cardinality: self
                .hashlist_minimum_cardinality
                .min(other.hashlist_minimum_cardinality),
            hashlist_mean_cardinality: self
                .hashlist_mean_cardinality
                .saturating_add(other.hashlist_mean_cardinality),
            hashlist_maximal_cardinality: self
                .hashlist_maximal_cardinality
                .max(other.hashlist_maximal_cardinality),
        }
    }
}

fn correction<P: Precision>(report: &[Point]) -> (Vec<u32>, Vec<f64>) {
    // We determine the maximal number of points to consider.
    let k = 9;

    // We split the data into k partitions, and we identify the largest discontinuity
    // in each partition.
    let top_k_reports: Vec<Point> = rdp(report, 0.1, k);

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
    let mut filtered_cardinalities: Vec<u32> = Vec::with_capacity(cardinalities.len());

    let mut previous_cardinality = cardinalities[0];
    let mut previous_error = errors[0];
    let mut count = 1.0;
    let mut total_previous_cardinality: u64 = previous_cardinality as u64;

    for (cardinality, error) in cardinalities.iter().zip(errors.iter()).skip(1) {
        if *cardinality
            <= previous_cardinality
                + (P::EXPONENT as u32) * (P::EXPONENT as u32) * (P::EXPONENT as u32) / 9
        {
            count += 1.0;
            previous_error += *error;
            total_previous_cardinality += *cardinality as u64;
        } else {
            filtered_cardinalities.push((total_previous_cardinality / count as u64) as u32);
            filtered_errors.push(previous_error / count);

            previous_cardinality = *cardinality;
            previous_error = *error;
            count = 1.0;
        }
    }

    filtered_cardinalities.push((total_previous_cardinality / count as u64) as u32);
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

struct ThreadSafeUnsafeCell<T>(std::cell::UnsafeCell<T>);

#[allow(unsafe_code)]
unsafe impl<T> Sync for ThreadSafeUnsafeCell<T> {}
#[allow(unsafe_code)]
unsafe impl<T> Send for ThreadSafeUnsafeCell<T> {}

#[allow(unsafe_code)]
/// Measures the gap between subsequent hashes in the Listhash variant of HyperLogLog.
pub fn hash_correction<P: Precision, B: Bits>(
    multiprogress: &MultiProgress,
) -> (HashCorrection, CorrectionPerformance)
where
    P: PackedRegister<B>,
{
    let number_of_cpus = rayon::current_num_threads();

    let iterations = 50_000 * 64;

    let progress_bar = multiprogress.add(ProgressBar::new(iterations as u64));
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "Samples: [{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
    );

    let maximal_cardinality = 10 * (1 << P::EXPONENT);

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
        // We generate an hashset with 4 times the requested cardinality.
        let mut hashset = HashSet::with_capacity(maximal_cardinality * 4);
        let mut random_state = 6_539_823_745_562_884_u64;

        while hashset.len() < maximal_cardinality * 4 {
            random_state = splitmix64(splitmix64(random_state));
            let value = xorshift64(random_state);
            hashset.insert(value);
        }

        // We convert the hashset into a vector.
        let hashset: Vec<u64> = hashset.into_iter().collect();

        // We create a copy of this vector for each thread
        let hashsets: Vec<Vec<u64>> = (0..number_of_cpus)
            .map(|_| hashset.clone())
            .collect::<Vec<Vec<u64>>>();

        // We place these vectors into UnsafeCells, so that each thread can
        // access its own copy.
        let hashsets: Vec<ThreadSafeUnsafeCell<Vec<u64>>> = hashsets
            .into_iter()
            .map(|hashset| ThreadSafeUnsafeCell(std::cell::UnsafeCell::new(hashset)))
            .collect();

        let (mut metadata, hashlist_report, hyperloglog_report): (
            Metadata,
            Vec<(u32, u64)>,
            Vec<(u32, u64)>,
        ) = ParallelIterator::reduce(
            (0..iterations)
                .into_par_iter()
                .progress_with(progress_bar)
                .map(|_| {
                    // We access the hashset of the current thread.
                    let hashset =
                        unsafe { &mut *hashsets[rayon::current_thread_index().unwrap()].0.get() };

                    // We shuffle the hashset.
                    hashset.shuffle(&mut rand::thread_rng());

                    let mut hll =
                        HyperLogLog::<P, B, <P as PackedRegister<B>>::Array, XxHash64>::default();
                    let mut hashlist_maximal_estimated_cardinality = 0;
                    let mut hashlist_maximal_exact_cardinality = 0;

                    let mut hashlist_cardinality_estimates: Vec<(u32, u64)> =
                        vec![(0, 0); maximal_cardinality];
                    let mut hyperloglog_cardinality_estimates: Vec<(u32, u64)> =
                        vec![(0, 0); maximal_cardinality];

                    let mut hll_estimate = hll.uncorrected_estimate_cardinality().round() as usize;
                    hashlist_cardinality_estimates[hll_estimate] = (1, 0);

                    for (exact_cardinality, value) in hashset.iter().enumerate() {
                        hll.insert(value);
                        let raw_hll_estimate = hll.uncorrected_estimate_cardinality();
                        assert!(raw_hll_estimate.is_finite());
                        hll_estimate = raw_hll_estimate.round() as usize;

                        if hll_estimate >= maximal_cardinality {
                            break;
                        }

                        if hll.is_hash_list() {
                            hashlist_maximal_estimated_cardinality = hll_estimate as u32;
                            hashlist_maximal_exact_cardinality = exact_cardinality as u32;

                            hashlist_cardinality_estimates[hll_estimate] = (
                                hashlist_cardinality_estimates[hll_estimate].0 + 1,
                                hashlist_cardinality_estimates[hll_estimate].1
                                    + exact_cardinality as u64,
                            );
                        } else {
                            hyperloglog_cardinality_estimates[hll_estimate] = (
                                hyperloglog_cardinality_estimates[hll_estimate].0 + 1,
                                hyperloglog_cardinality_estimates[hll_estimate].1
                                    + exact_cardinality as u64,
                            );
                        }
                    }

                    let metadata = Metadata::new(hashlist_maximal_exact_cardinality);

                    // Since we need to return them, we need to convert them into vectors:
                    hashlist_cardinality_estimates
                        .truncate(hashlist_maximal_estimated_cardinality as usize);

                    (
                        metadata,
                        hashlist_cardinality_estimates,
                        hyperloglog_cardinality_estimates,
                    )
                }),
            || {
                (
                    Metadata::new(0),
                    Vec::new(),
                    vec![(0, 0); maximal_cardinality],
                )
            },
            |(mut acc_metadata, mut acc_hashlist, mut acc_hyperloglog),
             (metadata, hashlist_report, hyperloglog_report)| {
                acc_metadata = acc_metadata.merge(metadata);

                acc_hashlist = if acc_hashlist.is_empty() {
                    hashlist_report
                } else {
                    let (mut larger, smaller) = if hashlist_report.len() > acc_hashlist.len() {
                        (hashlist_report, acc_hashlist)
                    } else {
                        (acc_hashlist, hashlist_report)
                    };

                    larger.iter_mut().zip(smaller.iter()).for_each(|(l, s)| {
                        l.0 += s.0;
                        l.1 += s.1;
                    });

                    larger
                };

                acc_hyperloglog
                    .iter_mut()
                    .zip(hyperloglog_report.iter())
                    .for_each(|(l, s)| {
                        l.0 += s.0;
                        l.1 += s.1;
                    });

                (acc_metadata, acc_hashlist, acc_hyperloglog)
            },
        );

        metadata.hashlist_mean_cardinality /= iterations as u32;

        // Now we determine the medians.
        let hashlist_report: Vec<Point> = hashlist_report
            .into_par_iter()
            .enumerate()
            .filter(|(_, (count, _))| *count > iterations as u32 / 15)
            .map(|(cardinality, (count, estimates_sum))| {
                let exact_cardinality = estimates_sum as f64 / count as f64;
                Point::from((cardinality as f64, exact_cardinality))
            })
            .collect();

        // Analogously for the hyperloglog, with the caveat that some of the entries
        // may be empty of with a small number of entries. We filter out those that
        // have less than 'iterations / 10' entries.
        let hyperloglog_report: Vec<Point> = hyperloglog_report
            .into_par_iter()
            .enumerate()
            .filter(|(_, (count, _))| *count > iterations as u32 / 15)
            .map(|(cardinality, (count, estimates_sum))| {
                let exact_cardinality = estimates_sum as f64 / count as f64;
                Point::from((cardinality as f64, exact_cardinality))
            })
            .collect();

        // We store the mined data-points to a CSV so to avoid recomputing them
        // in the future.
        write_csv(hashlist_report.iter(), &hashlist_output_path);
        write_csv(hyperloglog_report.iter(), &hyperloglog_output_path);

        // We store the maximal cardinality of the hashlist as JSON metadata.
        let metadata_hashlist_output_path =
            hashlist_output_path.replace(".csv.gz", ".metadata.json");

        serde_json::to_writer(
            std::fs::File::create(metadata_hashlist_output_path.clone()).unwrap(),
            &metadata,
        )
        .unwrap();

        (metadata, hashlist_report, hyperloglog_report)
    };

    let (hashlist_cardinalities, hashlist_relative_errors) = correction::<P>(&hashlist_report);
    let (hyperloglog_cardinalities, hyperloglog_relative_errors) =
        correction::<P>(&hyperloglog_report);

    let hyperloglog_slope =
        least_squares_linear_regression(&hyperloglog_report[hyperloglog_report.len() * 3 / 4..]);

    assert!(hyperloglog_slope.is_finite());

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
