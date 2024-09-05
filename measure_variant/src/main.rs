//! Script to log and compare the progress of two variants of HLL.

use hyperloglog_rs::composite_hash::*;
use hyperloglog_rs::prelude::*;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use test_utils::prelude::{compare_features, read_csv, write_csv};

type HLL1 = Hybrid<
    PlusPlus<
        Precision17,
        Bits6,
        <Precision17 as ArrayRegister<Bits6>>::Packed,
        twox_hash::XxHash64,
    >,
    SwitchHash<Precision17, Bits6>,
>;

type HLL2 = Hybrid<
    PlusPlus<
        Precision17,
        Bits6,
        <Precision17 as ArrayRegister<Bits6>>::Packed,
        twox_hash::XxHash64,
    >,
    GapHash<Precision17, Bits6, false>,
>;

const ITERATIONS: usize = 64;
const MINIMUM_CARDINALITY_FOR_SAMPLING: u64 = 0;
const MEASUREMENT_STEP: u64 = 1;

#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone)]
struct Report {
    relative_error: f64,
    estimated_cardinality: u64,
    cardinality: u64,
}

/// Main function to compare the progress of two variants of HLL.
fn main() {
    let random_state = 7_536_558_723_694_876_u64;

    let max_hashes = (1 << 17) * 6 / 4;
    let max = max_hashes as u64;

    let progress_bar = ProgressBar::new(ITERATIONS as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let size = max / MEASUREMENT_STEP - MINIMUM_CARDINALITY_FOR_SAMPLING / MEASUREMENT_STEP;

    let mut total_reports = (0..ITERATIONS)
        .into_par_iter()
        .progress_with(progress_bar)
        .map(|i| {
            let mut measurement_step = 0;
            let thread_random_state = splitmix64(random_state.wrapping_mul(i as u64 + 1));
            let mut hll = HLL2::default();
            let mut reports: Vec<Report> = Vec::with_capacity((size) as usize);
            let mut hashset: HashSet<u64> = HashSet::default();

            for value in iter_random_values::<u64>(max, None, Some(thread_random_state)) {
                hll.insert(&value);

                if hashset.insert(value) {
                    let cardinality = hll.estimate_cardinality();
                    let exact_cardinality = hashset.len() as u64;
                    let relative_error =
                        (exact_cardinality as f64 - cardinality).abs() / exact_cardinality as f64;
                    reports.push(Report {
                        relative_error,
                        estimated_cardinality: cardinality as u64,
                        cardinality: exact_cardinality,
                    });
                    measurement_step = 0;
                }

                measurement_step += 1;
            }

            if measurement_step != 0 {
                let cardinality = hll.estimate_cardinality();
                let exact_cardinality = hashset.len() as u64;
                let relative_error =
                    (exact_cardinality as f64 - cardinality).abs() / exact_cardinality as f64;
                reports.push(Report {
                    relative_error,
                    estimated_cardinality: cardinality as u64,
                    cardinality: exact_cardinality,
                });
            }

            reports
        })
        .reduce(
            || Vec::new(),
            |mut acc, mut reports| {
                if acc.is_empty() {
                    acc = reports;
                    return acc;
                }

                for (acc_report, report) in acc.iter_mut().zip(reports.iter_mut()) {
                    acc_report.relative_error += report.relative_error;
                }
                acc
            },
        );

    // assert_eq!(total_reports.len(), size as usize);

    total_reports.iter_mut().for_each(|report| {
        report.relative_error /= ITERATIONS as f64;
    });

    // Now we check whether there is already a stored report.
    if let Ok(reference_reports) = read_csv::<Report>("reference_report.csv") {
        // If the reference report has a length that is different from the total reports,
        // we assume that there has been a change in the code and we need to update the
        // reference report.
        if reference_reports.len() != total_reports.len() {
            write_csv(total_reports.into_iter(), "reference_report.csv");
            return;
        } else {
            write_csv(total_reports.iter(), "latest_report.csv");
        }

        let old_errors = reference_reports
            .iter()
            .map(|report| report.relative_error)
            .collect::<Vec<_>>();
        let new_errors = total_reports
            .iter()
            .map(|report| report.relative_error)
            .collect::<Vec<_>>();
        let error_benchmark = compare_features(
            new_errors.as_slice(),
            old_errors.as_slice(),
            "Relative Error",
        );

        error_benchmark.print();
    } else {
        write_csv(total_reports.into_iter(), "reference_report.csv");
    }
}
