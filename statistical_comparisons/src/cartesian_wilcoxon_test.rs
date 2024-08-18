//! Submodule loading all of the reports for a given task, one by one to avoid an excessive memory peak,
//! and then running the statistical tests.
use core::{f64, fmt};
use std::fmt::Display;
use std::iter::Sum;
use std::ops::Div;

use crate::csv_utils::{read_csv, write_csv};
use crate::estimation_tests::ErrorReport;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use stattest::test::StatisticalTest;
use stattest::test::WilcoxonWTest;

#[derive(Debug, PartialEq, Copy, serde::Deserialize, Clone, serde::Serialize)]
enum Outcome {
    /// When the first approach is better than the second.
    First,
    /// When the second approach is better than the first.
    Second,
    /// When the approaches are statistically equivalent.
    Tied,
    /// When the test failed.
    Failed,
}

impl Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::First => write!(f, "First"),
            Outcome::Second => write!(f, "Second"),
            Outcome::Tied => write!(f, "Tied"),
            Outcome::Failed => write!(f, "Failed"),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct WilcoxonTestResult {
    first_approach_name: String,
    second_approach_name: String,
    p_value: f64,
    outcome: Outcome,
    first_memsize: usize,
    first_mean: f64,
    first_std: f64,
    second_memsize: usize,
    second_mean: f64,
    second_std: f64,
}

#[derive(Debug)]
struct ReportInformations {
    absolute_errors: Vec<f64>,
    mean_memory_requirements: usize,
    mean_absolute_error: f64,
    std_absolute_error: f64,
    name: String,
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
            .template("Loading reports: [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
            .unwrap()
            .progress_chars("##-"),
    );

    // First, we load all of the reports in parallel.
    let reports: Vec<ReportInformations> = report_paths
        .par_iter()
        .progress_with(progress_bar)
        .map(|path| {
            let approach_name = path.file_stem().unwrap().to_str().unwrap();
            let path: &str = path.to_str().unwrap();
            let report: Vec<ErrorReport> = read_csv(path).unwrap();
            let absolute_errors = report.iter().map(ErrorReport::error).collect::<Vec<f64>>();
            let absolute_errors_mean = mean(absolute_errors.iter().copied());
            let mean_memory = mean(report.iter().map(ErrorReport::memory_requirement));
            let std_error = standard_deviation(&absolute_errors, absolute_errors_mean);

            ReportInformations {
                absolute_errors,
                mean_memory_requirements: mean_memory,
                mean_absolute_error: absolute_errors_mean,
                std_absolute_error: std_error,
                name: approach_name.to_string(),
            }
        })
        .collect::<Vec<ReportInformations>>();

    let progress_bar = indicatif::ProgressBar::new(reports.len() as u64);

    progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template(
                "Running Wilcoxon tests: [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}",
            )
            .unwrap()
            .progress_chars("##-"),
    );

    let results = reports
        .par_iter()
        .enumerate()
        .progress_with(progress_bar)
        .flat_map(|(i, report)| {
            reports.par_iter().skip(1 + i).map(|inner_report| {
                let w_test =
                    WilcoxonWTest::paired(&report.absolute_errors, &inner_report.absolute_errors);

                let outcome = if let Ok(w_test) = w_test {
                    if w_test.p_value() < 0.05 {
                        if report.mean_absolute_error < inner_report.mean_absolute_error {
                            Outcome::First
                        } else {
                            Outcome::Second
                        }
                    } else {
                        Outcome::Tied
                    }
                } else {
                    Outcome::Failed
                };

                WilcoxonTestResult {
                    first_approach_name: report.name.clone(),
                    second_approach_name: inner_report.name.clone(),
                    p_value: w_test.map_or(f64::NAN, |w| w.p_value()),
                    outcome,
                    first_memsize: report.mean_memory_requirements,
                    first_mean: report.mean_absolute_error,
                    first_std: report.std_absolute_error,
                    second_memsize: inner_report.mean_memory_requirements,
                    second_mean: inner_report.mean_absolute_error,
                    second_std: inner_report.std_absolute_error,
                }
            })
        })
        .collect::<Vec<WilcoxonTestResult>>();

    write_csv(
        results.into_iter(),
        &format!("reports/{feature_name}_wilcoxon.csv"),
    );
}
