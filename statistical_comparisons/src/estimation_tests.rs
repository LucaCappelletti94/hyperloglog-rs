//! Comparative test making sure that the quality of the estimation
//! obtain by the hyperloglog-rs library is equal or better than the
//! competitor libraries.
//! Evaluation of set-like properties across different data structures.
use crate::traits::TransparentMemSize;
use hyperloglog_rs::prelude::*;
use indicatif::ParallelProgressIterator;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use rayon::prelude::*;

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct PerformanceReport {
    prediction: f64,
    memory_requirement: usize,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct ErrorReport {
    error: f64,
    memory_requirement: usize,
}

impl ErrorReport {
    pub fn error(&self) -> f64 {
        self.error
    }

    pub fn memory_requirement(&self) -> usize {
        self.memory_requirement
    }

    pub fn from_performance_reports(
        performance_report: PerformanceReport,
        real_value: PerformanceReport,
    ) -> Self {
        ErrorReport {
            error: (performance_report.prediction - real_value.prediction).abs()
                / real_value.prediction.max(1.0),
            memory_requirement: performance_report.memory_requirement,
        }
    }
}

pub(crate) fn cardinality_test<
    H: Estimator<f64> + Clone + TransparentMemSize + Named + ExtendableApproximatedSet<u64>,
>(
    estimator: &H,
) -> Vec<PerformanceReport> {
    let number_of_vectors = 2_000_u64;
    let minimum_sample_interval = 5_u64;
    let maximum_sample_interval = 10_000_u64;
    let random_state = splitmix64(9_516_748_163_234_878_233_u64);

    let estimator_name = estimator.name();

    let progress_bar = ProgressBar::new(number_of_vectors);
    progress_bar.set_style(
        ProgressStyle::default_bar()
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
            let mut random_state = splitmix64(splitmix64(random_state.wrapping_mul(thread_number + 1)));
            let mut performance_reports = Vec::new();
            let mut estimator = estimator.clone();

            let mut current_sample_rate = minimum_sample_interval;

            for (i, element) in iter_random_values(2_000_000, None, random_state).enumerate() {
                estimator.insert(&element);

                if u64::try_from(i).unwrap() % current_sample_rate == 0 {
                    if current_sample_rate < maximum_sample_interval {
                        random_state = splitmix64(random_state);
                        current_sample_rate += random_state % current_sample_rate;
                    }

                    performance_reports.push(PerformanceReport {
                        prediction: estimator.estimate_cardinality(),
                        memory_requirement: estimator.transparent_mem_size()
                    });
                }
            }

            performance_reports
        })
        .collect::<Vec<PerformanceReport>>()
}

pub(crate) fn union_test<
    H: Estimator<f64> + Clone + TransparentMemSize + Named + ExtendableApproximatedSet<u64>,
>(
    estimator: &H,
) -> Vec<PerformanceReport> {
    let number_of_vectors = 2_000_u64;
    let minimum_sample_interval = 5_u64;
    let maximum_sample_interval = 10_000_u64;
    let left_random_state = splitmix64(5_647_315_671_326_798_672_u64);
    let right_random_state = splitmix64(4_457_567_787_334_878_233_u64);

    let estimator_name = estimator.name();

    let progress_bar = ProgressBar::new(number_of_vectors);
    progress_bar.set_style(
        ProgressStyle::default_bar()
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
            let mut performance_reports = Vec::new();
            let mut left_estimator = estimator.clone();
            let mut right_estimator = estimator.clone();

            let mut left_random_state = splitmix64(splitmix64(
                left_random_state.wrapping_mul(thread_number + 1),
            ));
            let right_random_state = splitmix64(splitmix64(
                right_random_state.wrapping_mul(thread_number + 1),
            ));

            let mut left_iter = iter_random_values(2_000_000, Some(1_000_000), left_random_state);
            let mut right_iter = iter_random_values(2_000_000, Some(1_000_000), right_random_state);

            let mut current_sample_rate = minimum_sample_interval;

            let mut i = 0;

            loop {
                let mut new_object = false;
                if let Some(left) = left_iter.next() {
                    left_estimator.insert(&left);
                    new_object = true;
                }
                if let Some(right) = right_iter.next() {
                    right_estimator.insert(&right);
                    new_object = true;
                }
                if !new_object {
                    break;
                }

                if i % current_sample_rate == 0 {
                    if current_sample_rate < maximum_sample_interval {
                        left_random_state = splitmix64(left_random_state);
                        current_sample_rate += left_random_state % current_sample_rate;
                    }

                    let union_cardinality =
                        left_estimator.estimate_union_cardinality(&right_estimator);
                    let memory_requirements = (left_estimator.transparent_mem_size()
                        + right_estimator.transparent_mem_size())
                        / 2;

                    performance_reports.push(PerformanceReport {
                        prediction: union_cardinality,
                        memory_requirement: memory_requirements,
                    });
                }

                i += 1;
            }

            performance_reports
        })
        .collect::<Vec<PerformanceReport>>()
}
