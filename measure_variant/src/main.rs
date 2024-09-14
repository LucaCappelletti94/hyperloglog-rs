//! Script to log and compare the progress of two variants of HLL.

use hyperloglog_rs::prelude::*;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use mem_dbg::MemSize;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use statistical_comparisons::prelude::*;
use std::collections::HashSet;
use test_utils::prelude::{compare_features, write_csv};
use twox_hash::XxHash64;

type HLL1 = HyperLogLog<
    Precision14,
    Bits6,
    <Precision14 as ArrayRegister<Bits6>>::Packed,
    twox_hash::XxHash64,
>;

const ITERATIONS: usize = 1024;

#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone)]
struct Report {
    relative_error: f64,
    estimated_cardinality: u64,
    cardinality: u64,
    memory_requirements: usize,
}

fn measure<S: Set + Default + MemSize>(reference: Option<&[Report]>) -> Vec<Report> {
    let random_state = 7_536_558_723_694_876_u64;

    let max_hashes = (1 << 18) * 12;

    let model_name = S::default().model_name();

    let progress_bar = ProgressBar::new(ITERATIONS as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(&format!("Measuring {model_name} [{{elapsed_precise}}] {{bar:40.cyan/blue}} {{pos:>7}}/{{len:7}} {{msg}}"))
            .unwrap()
            .progress_chars("##-"),
    );

    let mut total_reports = (0..ITERATIONS)
        .into_par_iter()
        .progress_with(progress_bar)
        .map(|i| {
            let thread_random_state = splitmix64(random_state.wrapping_mul(i as u64 + 1));
            let mut hll = S::default();
            let mut reports: Vec<Report> = Vec::with_capacity(max_hashes);
            let mut hashset: HashSet<u64> = HashSet::default();

            for value in iter_random_values::<u64>(max_hashes as u64, None, Some(thread_random_state)) {
                hll.insert_element(value);

                if hashset.insert(value) && hashset.len() > 128 {
                    let cardinality = hll.cardinality();
                    let exact_cardinality = hashset.len() as u64;
                    let relative_error =
                        (exact_cardinality as f64 - cardinality).abs() / exact_cardinality as f64;
                    reports.push(Report {
                        relative_error,
                        estimated_cardinality: cardinality as u64,
                        cardinality: exact_cardinality,
                        memory_requirements: hll.mem_size(mem_dbg::SizeFlags::default() | mem_dbg::SizeFlags::FOLLOW_REFS),
                    });
                }
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
                    acc_report.memory_requirements += report.memory_requirements;
                }
                acc
            },
        );

    total_reports.iter_mut().for_each(|report| {
        report.relative_error /= ITERATIONS as f64;
        report.memory_requirements /= ITERATIONS;
    });

    if let Some(reference) = reference {
        assert_eq!(reference.len(), total_reports.len());
    }

    // If the reference report has a length that is different from the total reports,
    // we assume that there has been a change in the code and we need to update the
    // reference report.
    write_csv(
        total_reports.iter(),
        format!("{}.csv.gz", S::default().model_name()).as_str(),
    );

    if let Some(reference) = reference {
        let old_errors = reference
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
    }

    total_reports
}

/// Main function to compare the progress of two variants of HLL.
fn main() {
    let reference = measure::<HLL1>(None);
    measure::<CloudFlareHLL<14, 6, XxHash64>>(Some(&reference));
    measure::<TabacHLLPlusPlus<Precision14, XxHash64>>(Some(&reference));
}
