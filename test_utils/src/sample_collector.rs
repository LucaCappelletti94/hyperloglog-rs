//! Cardinality sample collector.
use crate::cardinality_to_index::cardinality_estimate_to_index;
use crate::parallel::ThreadUnsafeCell;
use crate::sample_builder::{
    CardinalitySample, CardinalitySampleBuilder, ExtendedCardinalitySample,
    ExtendedCardinalitySampleBuilder,
};
use crate::set::{Set, Uncorrected};
use hyperloglog_rs::prelude::*;
use indicatif::{MultiProgress, ParallelProgressIterator, ProgressBar, ProgressStyle};
use mem_dbg::{MemSize, SizeFlags};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

const RANDOM_STATE: u64 = 5_435_765_765_854_357_668u64;

#[inline]
pub fn cardinality_samples<S: MemSize + Set + Default>(
    iterations: u64,
    maximum_cardinality: u64,
) -> Vec<ExtendedCardinalitySample> {
    let progress_bar = ProgressBar::new(iterations);
    let model_name = S::default().model_name();
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(&format!("Measuring {model_name} [{{elapsed_precise}} {{eta_precise}}] {{bar:40.cyan/blue}} {{pos:>7}}/{{len:7}} {{msg}}"))
            .unwrap()
            .progress_chars("##-"),
        );

    let step_size = u64::MAX / (maximum_cardinality + 1);
    let capacity_to_allocate = cardinality_estimate_to_index(maximum_cardinality) as usize + 1;

    let number_of_threads = rayon::current_num_threads();

    let reports: Vec<ThreadUnsafeCell<Vec<ExtendedCardinalitySampleBuilder>>> = (0
        ..number_of_threads)
        .map(|_| {
            ThreadUnsafeCell::new(vec![
                ExtendedCardinalitySampleBuilder::default();
                capacity_to_allocate
            ])
        })
        .collect();

    (0..iterations)
        .into_par_iter()
        .progress_with(progress_bar)
        .for_each(|i| {
            let bias = splitmix64(splitmix64(
                RANDOM_STATE.wrapping_mul(splitmix64(i as u64 + 1)),
            ));
            let mut hll = S::default();
            let reports = reports[rayon::current_thread_index().unwrap()].get_mut();

            // We iterate over the reports and increase the measurements.
            reports.iter_mut().for_each(|report| report.increase_measuremenet_count());

            let mut starting_value = bias;

            for exact_cardinality in 0..=maximum_cardinality {
                let start = std::time::Instant::now();
                let cardinality_estimate = hll.cardinality();
                hll.insert_element(starting_value);
                let end = std::time::Instant::now();

                // We insert the same value several times so that models that
                // employ obvious counters can be detected.
                hll.insert_element(starting_value);
                hll.insert_element(starting_value);
                hll.insert_element(starting_value);
                hll.insert_element(starting_value);
                hll.insert_element(starting_value);


                let index: usize = cardinality_estimate_to_index(exact_cardinality);
                reports[index].update(
                    exact_cardinality,
                    cardinality_estimate,
                    hll.mem_size(SizeFlags::default() | SizeFlags::FOLLOW_REFS),
                    (end - start).as_nanos(),
                );

                starting_value = starting_value.wrapping_add(step_size);
            }
        });

    // We flatten the reports from all threads into a single vector
    let total_reports: Vec<ExtendedCardinalitySampleBuilder> = reports
        .into_iter()
        .map(|report| report.into_inner())
        .reduce(
            |mut left: Vec<ExtendedCardinalitySampleBuilder>,
             right: Vec<ExtendedCardinalitySampleBuilder>| {
                left.iter_mut()
                    .zip(right)
                    .for_each(|(left, right)| *left += right);
                left
            },
        )
        .unwrap();

    total_reports.into_iter().map(Into::into).collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardinalitySamplesByModel {
    pub mean_hash_list_saturation: Option<f64>,
    pub mean_hyperloglog_saturation: Option<f64>,
    pub hyperloglog: Vec<CardinalitySample>,
    pub hash_list: Vec<CardinalitySample>,
}

struct ReportsBuilder {
    reports: Vec<ThreadUnsafeCell<Vec<CardinalitySampleBuilder>>>,
}

impl ReportsBuilder {
    #[inline]
    fn new(capacity_to_allocate: usize) -> Self {
        let number_of_threads = rayon::current_num_threads();
        let reports: Vec<ThreadUnsafeCell<Vec<CardinalitySampleBuilder>>> = (0..number_of_threads)
            .map(|_| {
                ThreadUnsafeCell::new(vec![
                    CardinalitySampleBuilder::default();
                    capacity_to_allocate
                ])
            })
            .collect();

        Self { reports }
    }

    #[inline]
    fn get_mut(&self) -> &mut Vec<CardinalitySampleBuilder> {
        self.reports[rayon::current_thread_index().unwrap()].get_mut()
    }

    #[inline]
    fn flatten(self, iterations: u64) -> Vec<CardinalitySample> {
        let mut reports = self
            .reports
            .into_iter()
            .map(|report| report.into_inner())
            .reduce(
                |mut left: Vec<CardinalitySampleBuilder>, right: Vec<CardinalitySampleBuilder>| {
                    left.iter_mut()
                        .zip(right)
                        .for_each(|(left, right)| *left += right);
                    left
                },
            )
            .unwrap()
            .into_iter()
            .filter_map(|report| {
                if report.count() < iterations / 20 {
                    None
                } else {
                    Some(report)
                }
            })
            .map(Into::into)
            .collect::<Vec<CardinalitySample>>();

        reports.sort_by(|a, b| {
            a.exact_cardinality_mean
                .partial_cmp(&b.exact_cardinality_mean)
                .unwrap()
        });

        reports
    }
}

struct ScalarCounter {
    counters: Vec<ThreadUnsafeCell<(u64, f64)>>,
}

impl ScalarCounter {
    #[inline]
    fn new() -> Self {
        let number_of_threads = rayon::current_num_threads();
        let counters: Vec<ThreadUnsafeCell<(u64, f64)>> = (0..number_of_threads)
            .map(|_| ThreadUnsafeCell::new((0, 0.0)))
            .collect();

        Self { counters }
    }

    #[inline]
    fn into_mean(self) -> Option<f64> {
        let (count, sum) = self
            .counters
            .into_iter()
            .map(|counter| counter.into_inner())
            .reduce(|(count, sum), (other_count, other_sum)| (count + other_count, sum + other_sum))
            .unwrap();

        if count == 0 {
            None
        } else {
            Some(sum / count as f64)
        }
    }

    #[inline]
    fn update(&self, value: f64) {
        let entry = self.counters[rayon::current_thread_index().unwrap()].get_mut();
        entry.0 += 1;
        entry.1 += value;
    }
}

#[inline]
pub fn uncorrected_cardinality_samples_by_model<P: Precision + PackedRegister<B>, B: Bits>(
    iterations: u64,
    maximum_cardinality: u64,
    multiprogress: &MultiProgress,
) -> CardinalitySamplesByModel {
    let progress_bar = multiprogress.add(ProgressBar::new(iterations));
    let model_not_imprinted = Uncorrected::<P, B>::default();

    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(&format!("Sampling [{{elapsed_precise}} {{eta_precise}}] {{bar:40.cyan/blue}} {{pos:>7}}/{{len:7}} {{msg}}"))
            .unwrap()
            .progress_chars("##-"),
        );

    let step_size = u64::MAX / (2 * maximum_cardinality + 1);
    let capacity_to_allocate = cardinality_estimate_to_index(maximum_cardinality) as usize + 1;

    let hyperloglog_reports = ReportsBuilder::new(capacity_to_allocate);
    let hash_list_reports = ReportsBuilder::new(capacity_to_allocate);

    // We create two arrays to keep track of the sums of saturation cardinalities
    // and their counts. We will use these to compute the mean saturation.
    let hash_list_saturations = ScalarCounter::new();
    let hyperloglog_saturations = ScalarCounter::new();

    (0..iterations)
        .into_par_iter()
        .progress_with(progress_bar)
        .for_each(|i| {
            let bias = splitmix64(splitmix64(
                RANDOM_STATE.wrapping_mul(splitmix64(i as u64 + 1)),
            ));
            let mut model_not_imprinted = model_not_imprinted.clone();

            let hash_list_reports = hash_list_reports.get_mut();
            let hyperloglog_reports = hyperloglog_reports.get_mut();

            // We iterate over the reports and increase the measurements.
            hash_list_reports.iter_mut().for_each(|report| report.increase_measuremenet_count());
            hyperloglog_reports.iter_mut().for_each(|report| report.increase_measuremenet_count());

            let mut starting_value = bias;

            for exact_cardinality in 0..=maximum_cardinality {
                let cardinality_estimate_not_imprinted = model_not_imprinted.cardinality();

                let index: usize = cardinality_estimate_to_index(exact_cardinality);

                if model_not_imprinted.is_hash_list() {
                    hash_list_reports[index].update(
                        exact_cardinality,
                        cardinality_estimate_not_imprinted,
                    );
                } else {
                    hyperloglog_reports[index].update(
                        exact_cardinality,
                        cardinality_estimate_not_imprinted,
                    );
                }

                let was_hash_list = model_not_imprinted.is_hash_list();
                let was_not_full_not_imprinted = !model_not_imprinted.is_full();
                model_not_imprinted.insert_element(starting_value);

                if was_hash_list != model_not_imprinted.is_hash_list() {
                    hash_list_saturations.update(exact_cardinality as f64);
                }

                if was_not_full_not_imprinted && model_not_imprinted.is_full() {
                    hyperloglog_saturations.update(exact_cardinality as f64);
                }

                starting_value = starting_value.wrapping_add(step_size);
            }
        });

    // We flatten the reports from all threads into a single vector
    let total_hyperloglog_reports = hyperloglog_reports.flatten(iterations);

    let total_hash_list_reports = hash_list_reports.flatten(iterations);

    // We compute the mean saturation for both models
    let mean_hash_list_saturation = hash_list_saturations.into_mean();

    let mean_hyperloglog_saturation = hyperloglog_saturations.into_mean();

    CardinalitySamplesByModel {
        mean_hash_list_saturation,
        mean_hyperloglog_saturation,
        hyperloglog: total_hyperloglog_reports,
        hash_list: total_hash_list_reports,
    }
}
