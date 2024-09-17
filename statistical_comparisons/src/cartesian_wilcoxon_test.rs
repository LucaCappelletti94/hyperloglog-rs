//! Submodule loading all of the reports for a given task, one by one to avoid an excessive memory peak,
//! and then running the statistical tests.
use crate::estimation_tests::ErrorReport;
use core::f64;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use serde::Serializer;
use stattest::test::StatisticalTest;
use stattest::test::WilcoxonWTest;
use std::iter::Sum;
use std::ops::Div;
use test_utils::prelude::{read_csv, write_csv};

fn float_formatter<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{value:.4}"))
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct WilcoxonTestResult {
    first_approach_name: String,
    second_approach_name: String,
    #[serde(serialize_with = "float_formatter")]
    p_value: f64,
}

#[derive(Debug)]
struct ErrorReports {
    absolute_errors: Vec<f64>,
    mean_memory_requirements: usize,
    mean_time_requirements: u128,
    mean_absolute_error: f64,
    std_absolute_error: f64,
    name: String,
}

impl ErrorReports {
    fn from_path(path: &std::path::Path, normalized: bool) -> Self {
        let path: &str = path.to_str().unwrap();
        let reports: Vec<ErrorReport> = match read_csv(path) {
            Ok(reports) => reports,
            Err(err) => {
                eprintln!("Error reading the CSV file: {err}");

                // We delete the file and exit with an error code.
                std::fs::remove_file(path).unwrap();

                std::process::exit(1);
            }
        };
        let approach_name = path
            .split('/')
            .last()
            .unwrap()
            .split('.')
            .next()
            .unwrap()
            .to_owned();

        let absolute_errors = if normalized {
            reports
                .iter()
                .map(|report| {
                    report.error().max(f64::from(f32::EPSILON))
                        * f64::from(u32::try_from(report.memory_requirement()).unwrap())
                        * (report.time_requirement() as f64)
                })
                .collect::<Vec<f64>>()
        } else {
            reports.iter().map(ErrorReport::error).collect::<Vec<f64>>()
        };

        let mean_memory_requirements = mean(reports.iter().map(ErrorReport::memory_requirement));
        let mean_time_requirements = mean(reports.iter().map(ErrorReport::time_requirement));
        let mean_absolute_error = mean(absolute_errors.iter().copied());
        let std_absolute_error = standard_deviation(&absolute_errors, mean_absolute_error);

        ErrorReports {
            absolute_errors,
            mean_memory_requirements,
            mean_time_requirements,
            mean_absolute_error,
            std_absolute_error,
            name: approach_name,
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct ModelPerformance {
    name: String,
    mean_absolute_error: f64,
    std_absolute_error: f64,
    mean_time_requirements: u128,
    mean_memory_requirements: usize,
    number_of_wins: usize,
    number_of_losses: usize,
    number_of_draws: usize,
}

fn standard_deviation(values: &[f64], mean: f64) -> f64 {
    // The values are always less than `u32::MAX`, so we can safely convert them.
    let number_of_values = f64::from(u32::try_from(values.len()).unwrap());
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / number_of_values;
    variance.sqrt()
}

fn mean<T, I>(values: I) -> T
where
    I: ExactSizeIterator<Item = T>,
    T: Sum + Div<T, Output = T> + TryFrom<u32>,
    <T as std::convert::TryFrom<u32>>::Error: std::fmt::Debug,
{
    // The values are always less than `u32::MAX`, so we can safely convert them.
    let number_of_values = T::try_from(u32::try_from(values.len()).unwrap()).unwrap();
    values.sum::<T>() / number_of_values
}

/// Runs the Wilcoxon test and writes the results to a CSV file, alongside the win-loss-draw statistics.
fn wilcoxon_test(mut reports: Vec<ErrorReports>, feature_name: &str) {
    // We sort the reports by approach name, so that later when we update the
    // number of wins, losses and draws, we can find the correct approach quickly
    // by using a binary search.
    reports.par_sort_unstable_by(|a, b| a.name.cmp(&b.name));

    let progress_bar = indicatif::ProgressBar::new(reports.len() as u64);

    progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("Preparing tasks: [{elapsed_precise} | {eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
            .unwrap()
            .progress_chars("##-"),
    );

    // Next, we prepare the tasks to run by pre-rendering the test tuples to evaluate.
    let tasks: Vec<(&ErrorReports, &ErrorReports)> = reports
        .par_iter()
        .enumerate()
        .progress_with(progress_bar)
        .flat_map(|(i, report)| {
            reports[1 + i..]
                .par_iter()
                .map(move |inner_report| (report, inner_report))
        })
        .collect();

    let progress_bar = indicatif::ProgressBar::new(tasks.len() as u64);

    progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template(
                "Running Wilcoxon tests: [{elapsed_precise} | {eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}",
            )
            .unwrap()
            .progress_chars("##-"),
    );

    let results = tasks
        .into_par_iter()
        .progress_with(progress_bar)
        .map(|(left, right)| {
            let w_test =
                WilcoxonWTest::voracious_paired(&left.absolute_errors, &right.absolute_errors);

            WilcoxonTestResult {
                first_approach_name: left.name.clone(),
                second_approach_name: right.name.clone(),
                p_value: w_test.map_or(f64::NAN, |w| w.p_value()),
            }
        })
        .collect::<Vec<WilcoxonTestResult>>();

    write_csv(
        results.iter(),
        &format!("reports/{feature_name}_wilcoxon.csv.gz"),
    );

    // We create a set of reports of model performance with the same order as the reports.
    let mut models_performance = reports
        .into_iter()
        .map(|report| ModelPerformance {
            name: report.name,
            mean_absolute_error: report.mean_absolute_error,
            std_absolute_error: report.std_absolute_error,
            mean_memory_requirements: report.mean_memory_requirements,
            mean_time_requirements: report.mean_time_requirements,
            number_of_wins: 0,
            number_of_losses: 0,
            number_of_draws: 0,
        })
        .collect::<Vec<ModelPerformance>>();

    // We now compute the number of wins, losses and draws for each approach.
    for result in results {
        let left = models_performance
            .binary_search_by(|r| r.name.cmp(&result.first_approach_name))
            .unwrap();
        let right = models_performance
            .binary_search_by(|r| r.name.cmp(&result.second_approach_name))
            .unwrap();

        if result.p_value < 0.01 {
            let left_mean = models_performance[left].mean_absolute_error;
            let right_mean = models_performance[right].mean_absolute_error;

            if left_mean < right_mean {
                models_performance[left].number_of_wins += 1;
                models_performance[right].number_of_losses += 1;
            } else {
                models_performance[left].number_of_losses += 1;
                models_performance[right].number_of_wins += 1;
            }
        } else {
            models_performance[left].number_of_draws += 1;
            models_performance[right].number_of_draws += 1;
        }
    }

    // We sort the model performance by the number of wins, and then by the number of losses.
    models_performance.sort_unstable_by(|a, b| {
        a.number_of_wins
            .cmp(&b.number_of_wins)
            .then_with(|| a.number_of_losses.cmp(&b.number_of_losses))
            .reverse()
    });

    // We write the model performance to a CSV file.
    write_csv(
        models_performance.iter(),
        &format!("reports/{feature_name}_performance.csv"),
    );
}

/// Runs a cartesian (`NxN`) Paired Signed-rank Wilcoxon test on the reports of a given feature.
///
/// Since these tests are by nature simmetrical, we only run the tests for the upper triangular
/// matrix of the comparisons.
///
/// # Arguments
/// * `feature_name` - The name of the feature to compare.
///
/// # Panics
/// * If the reports are not found.
pub fn cartesian_wilcoxon_test(feature_name: &str) {
    // We load the list of files in the 'reports/{feature_name}' directory,
    // which will be all in the form 'reports/{feature_name}/{approach_name}.csv'.

    let report_paths = std::fs::read_dir(format!("reports/{feature_name}"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            // We check that the document is a file and the file name ends with '.csv.gz'.
            path.is_file() && path.to_str().unwrap().ends_with(".csv.gz")
        })
        .collect::<Vec<_>>();

    let progress_bar = indicatif::ProgressBar::new(report_paths.len() as u64);

    progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("Loading reports: [{elapsed_precise} | {eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
            .unwrap()
            .progress_chars("##-"),
    );

    // First, we load all of the reports in parallel.
    let reports: Vec<ErrorReports> = report_paths
        .par_iter()
        .progress_with(progress_bar)
        .map(|path| ErrorReports::from_path(path, false))
        .collect::<Vec<ErrorReports>>();

    wilcoxon_test(reports, feature_name);

    let progress_bar = indicatif::ProgressBar::new(report_paths.len() as u64);

    progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("Loading memory normalized reports: [{elapsed_precise} | {eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
            .unwrap()
            .progress_chars("##-"),
    );

    // First, we load all of the reports in parallel.
    let normalized_reports: Vec<ErrorReports> = report_paths
        .par_iter()
        .progress_with(progress_bar)
        .map(|path| ErrorReports::from_path(path, true))
        .collect::<Vec<ErrorReports>>();

    wilcoxon_test(normalized_reports, &format!("{feature_name}_normalized"));
}
