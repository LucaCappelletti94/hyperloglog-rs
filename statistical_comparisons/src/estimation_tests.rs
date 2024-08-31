//! Comparative test making sure that the quality of the estimation
//! obtain by the hyperloglog-rs library is equal or better than the
//! competitor libraries.
//! Evaluation of set-like properties across different data structures.
use crate::traits::TransparentMemSize;
use hyperloglog_rs::prelude::*;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct PerformanceReport {
    prediction: f64,
    memory_requirement: usize,
    time_requirement: u128,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct ErrorReport {
    error: f64,
    memory_requirement: usize,
    time_requirement: u128,
}

impl ErrorReport {
    pub fn error(&self) -> f64 {
        self.error
    }

    pub fn memory_requirement(&self) -> usize {
        self.memory_requirement
    }

    pub fn time_requirement(&self) -> u128 {
        self.time_requirement
    }

    pub fn from_performance_reports(
        performance_report: PerformanceReport,
        real_value: PerformanceReport,
    ) -> Self {
        ErrorReport {
            error: (performance_report.prediction - real_value.prediction).abs()
                / real_value.prediction.max(1.0),
            memory_requirement: performance_report.memory_requirement,
            time_requirement: performance_report.time_requirement,
        }
    }
}

fn sample_interval_by_range(cardinality: u64) -> u64 {
    if cardinality < 1_00 {
        // 10
        10
    } else if cardinality < 1_000 {
        // 10
        100
    } else if cardinality < 10_000 {
        // 10
        1000
    } else if cardinality < 100_000 {
        // 10
        10_000
    } else if cardinality < 1_000_000 {
        // 10
        100_000
    } else if cardinality <= 10_000_000 {
        // 10
        1_000_000
    } else {
        unimplemented!()
    }
}

pub(crate) fn cardinality_test<
    H: Estimator<f64> + Clone + TransparentMemSize + Named + ExtendableApproximatedSet<u64>,
>(
    estimator: &H,
    multiprogress: &indicatif::MultiProgress,
) -> Vec<PerformanceReport> {
    let number_of_vectors = 1_000_u64;
    let number_of_elements = 10_000_000;
    let sequence_random_state = splitmix64(9_516_748_163_234_878_233_u64);
    let sample_index_random_state = splitmix64(2_348_782_399_516_748_163_u64);

    let estimator_name = estimator.name();
    let progress_bar = multiprogress.add(indicatif::ProgressBar::new(number_of_vectors));

    progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template(&format!(
                "{estimator_name}: [{{elapsed_precise}}] {{bar:40.cyan/blue}} {{pos:>7}}/{{len:7}}"
            ))
            .unwrap()
            .progress_chars("##-"),
    );

    (0..number_of_vectors)
        .into_par_iter()
        .progress_with(progress_bar)
        .flat_map(|thread_number| {
            let sequence_random_state = splitmix64(splitmix64(
                sequence_random_state.wrapping_mul(thread_number + 1),
            ));
            let mut sample_index_random_state = splitmix64(splitmix64(
                sample_index_random_state.wrapping_mul(thread_number + 1),
            ));
            let mut performance_reports = Vec::new();
            let mut estimator = estimator.clone();
            let mut next_sample_index = sample_interval_by_range(0);

            for (i, element) in
                iter_random_values::<u64>(number_of_elements, None, Some(sequence_random_state))
                    .enumerate()
            {
                estimator.insert(&element);

                if next_sample_index == i as u64 {
                    sample_index_random_state = splitmix64(sample_index_random_state);
                    next_sample_index +=
                        sample_index_random_state % sample_interval_by_range(i as u64);

                    let start = std::time::Instant::now();
                    let prediction = estimator.estimate_cardinality();
                    let time_requirement = start.elapsed().as_micros();

                    performance_reports.push(PerformanceReport {
                        prediction,
                        memory_requirement: estimator.transparent_mem_size(),
                        time_requirement,
                    });
                }
            }

            performance_reports
        })
        .collect::<Vec<PerformanceReport>>()
}

// pub(crate) fn union_test<
//     H: Estimator<f64> + Clone + TransparentMemSize + Named + ExtendableApproximatedSet<u64>,
// >(
//     estimator: &H,
//     multi_progress: Option<&indicatif::MultiProgress>,
// ) -> Vec<PerformanceReport> {
//     let number_of_vectors = 1_000_u64;
//     let minimum_sample_interval = 5_u64;
//     let maximum_sample_interval = 20_000_u64;
//     let left_random_state = splitmix64(5_647_315_671_326_798_672_u64);
//     let right_random_state = splitmix64(4_457_567_787_334_878_233_u64);

//     let estimator_name = estimator.name();

//     let progress_bar = ProgressBar::new(number_of_vectors);
//     progress_bar.set_style(
//         ProgressStyle::default_bar()
//             .template(&format!(
//                 "{estimator_name}: [{{elapsed_precise}}] {{bar:40.cyan/blue}} {{pos:>7}}/{{len:7}}"
//             ))
//             .unwrap()
//             .progress_chars("##-"),
//     );

//     (0..number_of_vectors)
//         .into_par_iter()
//         .progress_with(if let Some(multi_progress) = multi_progress {
//             multi_progress.add(progress_bar)
//         } else {
//             progress_bar
//         })
//         .flat_map(|thread_number| {
//             let mut performance_reports = Vec::new();
//             let mut left_estimator = estimator.clone();
//             let mut right_estimator = estimator.clone();

//             let mut left_random_state = splitmix64(splitmix64(
//                 left_random_state.wrapping_mul(thread_number + 1),
//             ));
//             let right_random_state = splitmix64(splitmix64(
//                 right_random_state.wrapping_mul(thread_number + 1),
//             ));

//             let mut left_iter =
//                 iter_random_values::<u64>(2_000_000, Some(1_000_000), Some(left_random_state));
//             let mut right_iter =
//                 iter_random_values::<u64>(2_000_000, Some(1_000_000), Some(right_random_state));

//             let mut current_sample_rate = minimum_sample_interval;

//             let mut i = 0;

//             loop {
//                 let mut new_object = false;
//                 if let Some(left) = left_iter.next() {
//                     left_estimator.insert(&left);
//                     new_object = true;
//                 }
//                 if let Some(right) = right_iter.next() {
//                     right_estimator.insert(&right);
//                     new_object = true;
//                 }
//                 if !new_object {
//                     break;
//                 }

//                 if i % current_sample_rate == 0 {
//                     if current_sample_rate < maximum_sample_interval {
//                         left_random_state = splitmix64(left_random_state);
//                         current_sample_rate += left_random_state % current_sample_rate;
//                     }

//                     todo!();
//                 }

//                 i += 1;
//             }

//             performance_reports
//         })
//         .collect::<Vec<PerformanceReport>>()
// }
