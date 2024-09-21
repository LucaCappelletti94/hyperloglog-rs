//! Cardinality sample collector.
use crate::cardinality_to_index::cardinality_estimate_to_index;
use crate::parallel::ThreadUnsafeCell;
use crate::sample_builder::{
    CardinalitySample, CardinalitySampleBuilder, ExtendedCardinalitySample,
    ExtendedCardinalitySampleBuilder,
};
use crate::set::{
    Set, UncorrectedFullyImprinted, UncorrectedImprintedDown, UncorrectedImprintedUp,
    UncorrectedNotImprinted,
};
use hyperloglog_rs::prelude::*;
use indicatif::{MultiProgress, ParallelProgressIterator, ProgressBar, ProgressStyle};
use mem_dbg::{MemSize, SizeFlags};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

const RANDOM_STATE: u64 = 5_435_765_765_854_357_668u64;

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
            let mut starting_value = bias;

            for exact_cardinality in 0..=maximum_cardinality {
                let start = std::time::Instant::now();
                let cardinality_estimate = hll.cardinality();
                hll.insert_element(starting_value);
                let end = std::time::Instant::now();
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
            |mut left: Vec<ExtendedCardinalitySampleBuilder>, right: Vec<ExtendedCardinalitySampleBuilder>| {
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
    pub hyperloglog_fully_imprinted: Vec<CardinalitySample>,
    pub hyperloglog_imprinted_up: Vec<CardinalitySample>,
    pub hyperloglog_imprinted_down: Vec<CardinalitySample>,
    pub hash_list: Vec<CardinalitySample>,
}

pub fn uncorrected_cardinality_samples_by_model<P: Precision + PackedRegister<B>, B: Bits>(
    iterations: u64,
    maximum_cardinality: u64,
    only_hash_list: bool,
    multiprogress: &MultiProgress,
) -> CardinalitySamplesByModel {
    let progress_bar = multiprogress.add(ProgressBar::new(iterations));
    let model_not_imprinted = UncorrectedNotImprinted::<P, B>::default();
    let model_fully_imprinted = UncorrectedFullyImprinted::<P, B>::default();
    let model_imprinted_up = UncorrectedImprintedUp::<P, B>::default();
    let model_imprinted_down = UncorrectedImprintedDown::<P, B>::default();
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(&format!("Sampling [{{elapsed_precise}} {{eta_precise}}] {{bar:40.cyan/blue}} {{pos:>7}}/{{len:7}} {{msg}}"))
            .unwrap()
            .progress_chars("##-"),
        );

    let step_size = u64::MAX / (2 * maximum_cardinality + 1);
    let capacity_to_allocate = cardinality_estimate_to_index(maximum_cardinality) as usize + 1;

    let number_of_threads = rayon::current_num_threads();

    let hyperloglog_reports: Vec<ThreadUnsafeCell<Vec<CardinalitySampleBuilder>>> = (0
        ..number_of_threads)
        .map(|_| {
            ThreadUnsafeCell::new(vec![
                CardinalitySampleBuilder::default();
                capacity_to_allocate
            ])
        })
        .collect();

    let hyperloglog_fully_imprinted_reports: Vec<ThreadUnsafeCell<Vec<CardinalitySampleBuilder>>> =
        (0..number_of_threads)
            .map(|_| {
                ThreadUnsafeCell::new(vec![
                    CardinalitySampleBuilder::default();
                    capacity_to_allocate
                ])
            })
            .collect();

    let hyperloglog_imprinted_up_reports: Vec<ThreadUnsafeCell<Vec<CardinalitySampleBuilder>>> = (0
        ..number_of_threads)
        .map(|_| {
            ThreadUnsafeCell::new(vec![
                CardinalitySampleBuilder::default();
                capacity_to_allocate
            ])
        })
        .collect();

    let hyperloglog_imprinted_down_reports: Vec<ThreadUnsafeCell<Vec<CardinalitySampleBuilder>>> =
        (0..number_of_threads)
            .map(|_| {
                ThreadUnsafeCell::new(vec![
                    CardinalitySampleBuilder::default();
                    capacity_to_allocate
                ])
            })
            .collect();

    let hash_list_reports: Vec<ThreadUnsafeCell<Vec<CardinalitySampleBuilder>>> = (0
        ..number_of_threads)
        .map(|_| {
            ThreadUnsafeCell::new(vec![
                CardinalitySampleBuilder::default();
                capacity_to_allocate
            ])
        })
        .collect();

    // We create two arrays to keep track of the sums of saturation cardinalities
    // and their counts. We will use these to compute the mean saturation.
    let hash_list_saturations: Vec<ThreadUnsafeCell<Option<(usize, f64)>>> = (0..number_of_threads)
        .map(|_| ThreadUnsafeCell::new(None))
        .collect();

    let hyperloglog_saturations: Vec<ThreadUnsafeCell<Option<(usize, f64)>>> = (0
        ..number_of_threads)
        .map(|_| ThreadUnsafeCell::new(None))
        .collect();

    (0..iterations)
        .into_par_iter()
        .progress_with(progress_bar)
        .for_each(|i| {
            let bias = splitmix64(splitmix64(
                RANDOM_STATE.wrapping_mul(splitmix64(i as u64 + 1)),
            ));
            let mut model_not_imprinted = model_not_imprinted.clone();
            let mut model_fully_imprinted = model_fully_imprinted.clone();
            let mut model_imprinted_up = model_imprinted_up.clone();
            let mut model_imprinted_down = model_imprinted_down.clone();

            let hash_list_reports =
                hash_list_reports[rayon::current_thread_index().unwrap()].get_mut();
            let hyperloglog_reports =
                hyperloglog_reports[rayon::current_thread_index().unwrap()].get_mut();
            let hyperloglog_fully_imprinted_reports = hyperloglog_fully_imprinted_reports
                [rayon::current_thread_index().unwrap()]
            .get_mut();
            let hyperloglog_imprinted_up_reports =
                hyperloglog_imprinted_up_reports[rayon::current_thread_index().unwrap()].get_mut();
            let hyperloglog_imprinted_down_reports = hyperloglog_imprinted_down_reports
                [rayon::current_thread_index().unwrap()]
            .get_mut();

            let mut starting_value = bias;

            for exact_cardinality in 0..=(2 * maximum_cardinality) {
                let cardinality_estimate_not_imprinted = model_not_imprinted.cardinality();
                let cardinality_estimate_fully_imprinted = model_fully_imprinted.cardinality();
                let cardinality_estimate_imprinted_up = model_imprinted_up.cardinality();
                let cardinality_estimate_imprinted_down = model_imprinted_down.cardinality();

                let index: usize = cardinality_estimate_to_index(exact_cardinality);

                if index >= capacity_to_allocate {
                    continue;
                }

                assert_eq!(
                    model_not_imprinted.is_hash_list(),
                    model_fully_imprinted.is_hash_list()
                );

                if model_not_imprinted.is_hash_list() {
                    hash_list_reports[index]
                        .update(exact_cardinality, cardinality_estimate_not_imprinted);
                    assert_eq!(
                        cardinality_estimate_not_imprinted,
                        cardinality_estimate_fully_imprinted
                    );
                } else {
                    if only_hash_list {
                        break;
                    }
                    hyperloglog_reports[index]
                        .update(exact_cardinality, cardinality_estimate_not_imprinted);
                    hyperloglog_fully_imprinted_reports[index]
                        .update(exact_cardinality, cardinality_estimate_fully_imprinted);
                    hyperloglog_imprinted_up_reports[index]
                        .update(exact_cardinality, cardinality_estimate_imprinted_up);
                    hyperloglog_imprinted_down_reports[index]
                        .update(exact_cardinality, cardinality_estimate_imprinted_down);
                }

                let was_hash_list = model_not_imprinted.is_hash_list();
                let was_not_full_not_imprinted = !model_not_imprinted.is_full();
                // let was_not_full_imprinted = !model_imprinted.is_full();
                model_not_imprinted.insert_element(starting_value);
                model_fully_imprinted.insert_element(starting_value);
                model_imprinted_up.insert_element(starting_value);
                model_imprinted_down.insert_element(starting_value);

                if was_hash_list != model_not_imprinted.is_hash_list() {
                    let saturation_reports =
                        hash_list_saturations[rayon::current_thread_index().unwrap()].get_mut();
                    let (count, sum) = saturation_reports.get_or_insert((0, 0.0));
                    *count += 1;
                    *sum += exact_cardinality as f64;
                }

                if was_not_full_not_imprinted && model_not_imprinted.is_full() {
                    let saturation_reports =
                        hyperloglog_saturations[rayon::current_thread_index().unwrap()].get_mut();
                    let (count, sum) = saturation_reports.get_or_insert((0, 0.0));
                    *count += 1;
                    *sum += exact_cardinality as f64;
                }

                starting_value = starting_value.wrapping_add(step_size);
            }
        });

    // We flatten the reports from all threads into a single vector
    let total_hyperloglog_reports: Vec<CardinalitySampleBuilder> = hyperloglog_reports
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
        .unwrap();

    let total_hyperloglog_fully_imprinted_reports: Vec<CardinalitySampleBuilder> =
        hyperloglog_fully_imprinted_reports
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
            .unwrap();

    let total_hyperloglog_imprinted_up_reports: Vec<CardinalitySampleBuilder> =
        hyperloglog_imprinted_up_reports
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
            .unwrap();

    let total_hyperloglog_imprinted_down_reports: Vec<CardinalitySampleBuilder> =
        hyperloglog_imprinted_down_reports
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
            .unwrap();

    let total_hash_list_reports: Vec<CardinalitySampleBuilder> = hash_list_reports
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
        .unwrap();

    let mut total_hash_list_reports: Vec<CardinalitySample> = total_hash_list_reports
        .into_iter()
        .filter_map(|report| {
            if report.count() < iterations / 20 {
                None
            } else {
                Some(report)
            }
        })
        .map(Into::into)
        .collect();

    total_hash_list_reports.sort_by(|a, b| {
        a.estimated_cardinality_mean
            .partial_cmp(&b.estimated_cardinality_mean)
            .unwrap()
    });

    let mut total_hyperloglog_reports: Vec<CardinalitySample> = total_hyperloglog_reports
        .into_iter()
        .filter_map(|report| {
            if report.count() < iterations / 20 {
                None
            } else {
                Some(report)
            }
        })
        .map(Into::into)
        .collect();

    total_hyperloglog_reports.sort_by(|a, b| {
        a.estimated_cardinality_mean
            .partial_cmp(&b.estimated_cardinality_mean)
            .unwrap()
    });

    let mut total_hyperloglog_fully_imprinted_reports: Vec<CardinalitySample> =
        total_hyperloglog_fully_imprinted_reports
            .into_iter()
            .filter_map(|report| {
                if report.count() < iterations / 20 {
                    None
                } else {
                    Some(report)
                }
            })
            .map(Into::into)
            .collect();

    total_hyperloglog_fully_imprinted_reports.sort_by(|a, b| {
        a.estimated_cardinality_mean
            .partial_cmp(&b.estimated_cardinality_mean)
            .unwrap()
    });

    let mut total_hyperloglog_imprinted_up_reports: Vec<CardinalitySample> =
        total_hyperloglog_imprinted_up_reports
            .into_iter()
            .filter_map(|report| {
                if report.count() < iterations / 20 {
                    None
                } else {
                    Some(report)
                }
            })
            .map(Into::into)
            .collect();

    total_hyperloglog_imprinted_up_reports.sort_by(|a, b| {
        a.estimated_cardinality_mean
            .partial_cmp(&b.estimated_cardinality_mean)
            .unwrap()
    });

    let mut total_hyperloglog_imprinted_down_reports: Vec<CardinalitySample> =
        total_hyperloglog_imprinted_down_reports
            .into_iter()
            .filter_map(|report| {
                if report.count() < iterations / 20 {
                    None
                } else {
                    Some(report)
                }
            })
            .map(Into::into)
            .collect();

    total_hyperloglog_imprinted_down_reports.sort_by(|a, b| {
        a.estimated_cardinality_mean
            .partial_cmp(&b.estimated_cardinality_mean)
            .unwrap()
    });

    // We compute the mean saturation for both models
    let mean_hash_list_saturation = hash_list_saturations
        .into_iter()
        .map(|saturation| saturation.into_inner())
        .filter_map(|saturation| saturation)
        .reduce(|(count, sum), (other_count, other_sum)| (count + other_count, sum + other_sum))
        .map(|(count, sum)| sum / count as f64);

    let mean_hyperloglog_saturation = hyperloglog_saturations
        .into_iter()
        .map(|saturation| saturation.into_inner())
        .filter_map(|saturation| saturation)
        .reduce(|(count, sum), (other_count, other_sum)| (count + other_count, sum + other_sum))
        .map(|(count, sum)| sum / count as f64);

    CardinalitySamplesByModel {
        mean_hash_list_saturation,
        mean_hyperloglog_saturation,
        hyperloglog_imprinted_down: total_hyperloglog_imprinted_down_reports,
        hyperloglog_imprinted_up: total_hyperloglog_imprinted_up_reports,
        hyperloglog_fully_imprinted: total_hyperloglog_fully_imprinted_reports,
        hyperloglog: total_hyperloglog_reports,
        hash_list: total_hash_list_reports,
    }
}
