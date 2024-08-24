//! Script to log and compare the progress of two variants of HLL.

use hyperloglog_rs::prelude::*;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use test_utils::prelude::{compare_features, read_csv, write_csv};

const ITERATIONS: usize = 1_0000;
const MINIMUM_CARDINALITY_FOR_SAMPLING: u64 = 0;
const MAX: u64 = 20_000;
const MEASUREMENT_STEP: u64 = 1;

type HLL = Hybrid<
    PlusPlus<
        Precision16,
        Bits6,
        <Precision16 as ArrayRegister<Bits6>>::Packed,
        twox_hash::XxHash64,
    >,
>;

#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone)]
struct Report {
    relative_error: f64,
}

/// Main function to compare the progress of two variants of HLL.
fn main() {
    let random_state = 7_536_558_723_694_876_u64;

    let progress_bar = ProgressBar::new(ITERATIONS as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let size = MAX / MEASUREMENT_STEP - MINIMUM_CARDINALITY_FOR_SAMPLING / MEASUREMENT_STEP;

    let mut total_reports = (0..ITERATIONS)
        .into_par_iter()
        .progress_with(progress_bar)
        .map(|i| {
            // In order to execute the test faster, we approximate the cardinality
            // of a set to this simple counter, as we assume that the iterator does
            // not yield duplicates that often, at least not in a small range.
            let mut counter = 0;
            let mut measurement_step = 0;
            let thread_random_state = splitmix64(random_state.wrapping_mul(i as u64 + 1));
            let mut hll = HLL::default();
            let mut reports: Vec<Report> = Vec::with_capacity((size) as usize);

            for value in iter_random_values::<u64>(MAX, None, Some(thread_random_state)) {
                counter += 1;
                hll.insert(&value);

                if counter <= MINIMUM_CARDINALITY_FOR_SAMPLING {
                    continue;
                }

                if measurement_step == MEASUREMENT_STEP {
                    let cardinality = hll.estimate_cardinality();
                    let relative_error = (counter as f64 - cardinality).abs() / counter as f64;
                    reports.push(Report {
                        relative_error,
                    });
                    measurement_step = 0;
                }

                measurement_step += 1;
            }

            if measurement_step != 0 {
                let cardinality = hll.estimate_cardinality();
                let relative_error = (counter as f64 - cardinality).abs() / counter as f64;
                reports.push(Report {
                    relative_error,
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

    assert_eq!(total_reports.len(), size as usize);

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
            write_csv(total_reports.into_iter(), "latest_report.csv");
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
