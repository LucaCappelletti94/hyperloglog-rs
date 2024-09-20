//! Cardinality sample collector.
use crate::cardinality_to_index::cardinality_estimate_to_index;
use crate::parallel::ThreadUnsafeCell;
use crate::sample_builder::{CardinalitySample, CardinalitySampleBuilder};
use crate::set::Set;
use serde::{Deserialize, Serialize};
use hyperloglog_rs::prelude::*;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;

const RANDOM_STATE: u64 = 5_435_765_765_854_357_668u64;

pub fn cardinality_samples<S: Set + Default>(
    iterations: u64,
    maximum_cardinality: u64,
) -> Vec<CardinalitySample> {
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

    let reports: Vec<ThreadUnsafeCell<Vec<CardinalitySampleBuilder>>> = (0..number_of_threads)
        .map(|_| {
            ThreadUnsafeCell::new(vec![
                CardinalitySampleBuilder::default();
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
                let cardinality_estimate = hll.cardinality();

                let index: usize = cardinality_estimate_to_index(exact_cardinality);
                reports[index].update(exact_cardinality, cardinality_estimate);
                hll.insert_element(starting_value);

                starting_value = starting_value.wrapping_add(step_size);
            }
        });

    // We flatten the reports from all threads into a single vector
    let total_reports: Vec<CardinalitySampleBuilder> = reports
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

    total_reports.into_iter().map(Into::into).collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardinalitySamplesByModel {
    pub mean_hash_list_saturation: Option<f64>,
    pub mean_hyperloglog_saturation: Option<f64>,
    pub hyperloglog: Vec<CardinalitySample>,
    pub hash_list: Vec<CardinalitySample>,
}

pub fn cardinality_samples_by_model<P: Precision + PackedRegister<B>, B: Bits, H: HasherType>(
    iterations: u64,
    maximum_cardinality: u64,
) -> CardinalitySamplesByModel {
    let progress_bar = ProgressBar::new(iterations);
    let model = HyperLogLog::<P, B, <P as PackedRegister<B>>::Array, H>::default();
    let model_name = model.model_name();
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(&format!("Measuring {model_name} [{{elapsed_precise}} {{eta_precise}}] {{bar:40.cyan/blue}} {{pos:>7}}/{{len:7}} {{msg}}"))
            .unwrap()
            .progress_chars("##-"),
        );

    let step_size = u64::MAX / (maximum_cardinality + 1);
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
            let mut hll = model.clone();
            let hash_list_reports =
                hash_list_reports[rayon::current_thread_index().unwrap()].get_mut();
            let hyperloglog_reports =
                hyperloglog_reports[rayon::current_thread_index().unwrap()].get_mut();
            let mut starting_value = bias;

            for exact_cardinality in 0..=maximum_cardinality {
                let cardinality_estimate = hll.cardinality();

                let index: usize = cardinality_estimate_to_index(exact_cardinality);
                if hll.is_hash_list() {
                    hash_list_reports[index].update(exact_cardinality, cardinality_estimate);
                } else {
                    hyperloglog_reports[index].update(exact_cardinality, cardinality_estimate);
                }

                let was_hash_list = hll.is_hash_list();
                let was_not_full = !hll.is_full();
                hll.insert_element(starting_value);

                if was_hash_list != hll.is_hash_list() {
                    let saturation_reports =
                        hash_list_saturations[rayon::current_thread_index().unwrap()].get_mut();
                    let (count, sum) = saturation_reports.get_or_insert((0, 0.0));
                    *count += 1;
                    *sum += exact_cardinality as f64;
                }

                if was_not_full && hll.is_full() {
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

    let mut total_hash_list_reports: Vec<CardinalitySampleBuilder> = hash_list_reports
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

    // We drop the hashlist values after they become all zeros as all iterations
    // at that point had switched to hyperloglog
    let mut last_hash_list_index = 0;
    for (index, report) in total_hash_list_reports.iter().enumerate() {
        if report.is_empty() {
            last_hash_list_index = index;
            break;
        }
    }
    total_hash_list_reports.truncate(last_hash_list_index);

    // Simmetrically, we drop the hyperloglog reports from when the model was using
    // the hashlist and therefore there are no hyperloglog reports
    let total_hyperloglog_reports: Vec<CardinalitySampleBuilder> = total_hyperloglog_reports
        .into_iter()
        .skip_while(|report| report.is_empty())
        .collect();

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
        hyperloglog: total_hyperloglog_reports
            .into_iter()
            .map(Into::into)
            .collect(),
        hash_list: total_hash_list_reports
            .into_iter()
            .map(Into::into)
            .collect(),
    }
}
