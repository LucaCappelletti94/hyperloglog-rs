//! Submodule loading all of the reports for a given task, one by one to avoid an excessive memory peak,
//! and then running the statistical tests.
use core::{f64, fmt};
use std::fmt::Display;

use crate::estimation_tests::{ErrorReport, Header};
use crate::reports_generator::read_report_from_csv;
use crate::utils::{write_csv, mean, mean_usize, standard_deviation};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use stattest::test::WilcoxonWTest;
use stattest::test::StatisticalTest;

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

impl Into<Vec<String>> for WilcoxonTestResult {
    fn into(self) -> Vec<String> {
        vec![
            self.first_approach_name,
            self.second_approach_name,
            format!("{:.5}", self.p_value),
            self.outcome.to_string(),
            format!("{}", self.first_memsize),
            format!("{:.6}", self.first_mean),
            format!("{:.6}", self.first_std),
            format!("{}", self.second_memsize),
            format!("{:.6}", self.second_mean),
            format!("{:.6}", self.second_std),
        ]
    }
}

impl Header for WilcoxonTestResult {
    fn header() -> &'static [&'static str] {
        &[
            "first_approach",
            "second_approach",
            "p-value",
            "winner",
            "first_memsize",
            "first_mean_error",
            "first_std_error",
            "second_memsize",
            "second_mean_error",
            "second_std_error",
        ]
    }
}

pub fn cartesian_wilcoxon_test(feature_name: &str) {
    // We load the list of files in the 'reports/{feature_name}' directory,
    // which will be all in the form 'reports/{feature_name}/{approach_name}.csv'.

    let reports = std::fs::read_dir(format!("reports/{feature_name}"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file() && path.extension().unwrap() == "csv")
        .collect::<Vec<_>>();

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
        .flat_map(|(i, path)| {
            let approach_name = path.file_stem().unwrap().to_str().unwrap();
            let path: &str = path.to_str().unwrap();
            let report: Vec<ErrorReport> = read_report_from_csv(path).unwrap();
            let absolute_errors = report.iter().map(|r| r.error()).collect::<Vec<_>>();
            let memory_requirements = report.iter().map(|r| r.memory_requirement()).collect::<Vec<_>>();
            let absolute_errors_mean = mean(&absolute_errors);
            let mean_memory = mean_usize(&memory_requirements);
            let std_error = standard_deviation(&absolute_errors, absolute_errors_mean);

            reports
                .iter()
                .skip(1 + i)
                .map(|inner_path| {
                    let inner_approach_name = inner_path.file_stem().unwrap().to_str().unwrap();
                    let inner_path: &str = inner_path.to_str().unwrap();
                    let inner_report: Vec<ErrorReport> = read_report_from_csv(inner_path).unwrap();
                    let inner_absolute_errors = inner_report.iter().map(|r| r.error()).collect::<Vec<_>>();
                    let inner_memory_requirements = inner_report.iter().map(|r| r.memory_requirement()).collect::<Vec<_>>();
                    let inner_absolute_errors_mean = mean(&inner_absolute_errors);
                    let inner_mean_memory = mean_usize(&inner_memory_requirements);
                    let inner_std_error = standard_deviation(&inner_absolute_errors, inner_absolute_errors_mean);

                    let w_test = WilcoxonWTest::paired(&absolute_errors, &inner_absolute_errors);

                    let outcome = if let Ok(w_test) = w_test {
                        if w_test.p_value() < 0.05 {
                            if mean_memory < inner_mean_memory {
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
                        first_approach_name: approach_name.to_string(),
                        second_approach_name: inner_approach_name.to_string(),
                        p_value: w_test.map_or(f64::NAN, |w_test| w_test.p_value()),
                        outcome,
                        first_memsize: memory_requirements.len(),
                        first_mean: absolute_errors_mean,
                        first_std: std_error,
                        second_memsize: inner_memory_requirements.len(),
                        second_mean: inner_absolute_errors_mean,
                        second_std: inner_std_error,
                    }
                })
                .collect::<Vec<WilcoxonTestResult>>()
        })
        .collect::<Vec<WilcoxonTestResult>>();

    write_csv(
        results.into_iter(),
        &format!("reports/{feature_name}_wilcoxon.csv"),
    );
}
