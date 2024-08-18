//! This module contains the code to generate the reports for the cardinality and union tests.
use std::collections::HashSet;

use crate::estimation_tests::{cardinality_test, union_test};
use crate::csv_utils::{write_csv, read_csv};
use crate::{
    estimation_tests::{ErrorReport, PerformanceReport},
    traits::TransparentMemSize,
};
use indicatif::ProgressIterator;
use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use strum::IntoEnumIterator;

fn prepare_reports<S, T1, T2>(test_for_hashset: T1, test: T2, test_name: &str)
where
    T1: Fn(&HashSet<u64>, Option<&MultiProgress>) -> Vec<PerformanceReport>,
    T2: Fn(&S, Option<&MultiProgress>) -> Vec<PerformanceReport>,
    S: Named
        + Clone
        + ExtendableApproximatedSet<u64>
        + Estimator<f64>
        + TransparentMemSize
        + IntoEnumIterator,
{
    // We create a directory called "reports" if it does not exist.
    let _ = std::fs::create_dir("reports");
    // Next, we create a directory called "cardinality" if it does not exist
    // inside the "reports" directory.
    let _ = std::fs::create_dir(format!("reports/{test_name}"));

    // We start by computing the exact cardinality of the set.
    let exact_estimator = HashSet::<u64>::new();
    // We determine the path where we will store the report.
    let path = format!("reports/{test_name}-{}.csv.gz", exact_estimator.name());
    // If the path does not already exist, we create it.
    let correct_report: Vec<PerformanceReport> = if std::path::Path::new(&path).exists() {
        read_csv(&path).unwrap()
    } else {
        // We log the progress of the computation.
        log::info!("Computing the exact {} of the set.", test_name);

        let correct_report = test_for_hashset(&exact_estimator, None);
        // And we store it into the "reports/cardinality/{estimator_name}.csv" file.
        write_csv(correct_report.iter().copied(), &path);
        correct_report
    };

    let entries = S::iter();

    // We filter the entries, so to be able to provide a loading bar that
    // only includes entry we actually process.
    let entries = entries.filter(|entry| {
        let path = format!("reports/{test_name}/{}.csv", entry.name());
        !std::path::Path::new(&path).exists()
    }).filter(|entry|{
        // If the test name is "cardinality", we exclude all models whose
        // name contains the word "MLE", as we solely have the joint union
        // estimation to evaluate from the MLE models at this time.
        !(test_name == "cardinality" && entry.name().contains("MLE"))
    });

    // We clone the iterator and compute the actual number of entries.
    let number_of_entries = entries.clone().count();

    // We prepare a multi-progress bar to display the progress of the computation,
    // and we insert the progress bars for these entries. We then pass this progress
    // bar to the test function.

    let multi_progress = MultiProgress::new();

    let entries_progress_bar = multi_progress.add(indicatif::ProgressBar::new(number_of_entries as u64));
    entries_progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template(&format!(
                "{test_name}: [{{elapsed_precise}}] {{bar:40.cyan/blue}} {{pos:>7}}/{{len:7}}"
            )).unwrap()
            .progress_chars("##-"),
    );

    for enum_entry in entries.progress_with(entries_progress_bar) {
        // We do the same for each estimator.
        let path = format!("reports/{test_name}/{}.csv.gz", enum_entry.name());
        let report = test(&enum_entry, Some(&multi_progress));
        write_csv(
            report
                .into_iter()
                .zip(correct_report.iter().copied())
                .map(|(a, b)| ErrorReport::from_performance_reports(a, b)),
            &path,
        );
    }

    multi_progress.clear().unwrap();
}

/// Trait to prepare the reports for the cardinality and union tests.
pub trait SetTester:
    Named
    + Clone
    + ExtendableApproximatedSet<u64>
    + Estimator<f64>
    + TransparentMemSize
    + IntoEnumIterator
{
    /// Prepare the reports for the cardinality test.
    fn prepare_cardinality_reports() {
        prepare_reports::<Self, _, _>(cardinality_test, cardinality_test, "cardinality");
    }

    /// Prepare the reports for the union test.
    fn prepare_union_reports() {
        prepare_reports::<Self, _, _>(union_test, union_test, "union");
    }
}

impl<
        S: Named
            + Clone
            + ExtendableApproximatedSet<u64>
            + Estimator<f64>
            + TransparentMemSize
            + IntoEnumIterator,
    > SetTester for S
{
}
