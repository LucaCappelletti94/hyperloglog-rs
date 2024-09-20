//! Script to log and compare the progress of two variants of HLL.

use std::{ops::Add, u64};

use hyperloglog_rs::prelude::*;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use mem_dbg::MemSize;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use statistical_comparisons::prelude::*;
use test_utils::prelude::{
    cardinality_estimate_to_index, compare_features, read_csv, write_csv, ThreadUnsafeCell
};
use wyhash::WyHash;

// type HLL1 = HyperLogLog<
//     Precision4,
//     Bits6,
//     <Precision4 as PackedRegister<Bits6>>::Array,
//     twox_hash::XxHash64,
// >;

#[derive(Clone, MemSize)]
struct Uncorrected<P: Precision + PackedRegister<B>, B: Bits> {
    hll: HyperLogLog<P, B, <P as PackedRegister<B>>::Array, WyHash>,
}

impl<P: Precision + PackedRegister<B>, B: Bits> Default for Uncorrected<P, B> {
    fn default() -> Self {
        Self {
            hll: HyperLogLog::default(),
        }
    }
}

impl<P: Precision + PackedRegister<B>, B: Bits> Set for Uncorrected<P, B> {
    #[inline]
    fn cardinality(&self) -> f64 {
        self.hll.uncorrected_estimate_cardinality()
    }

    #[inline]
    fn insert_element(&mut self, value: u64) {
        self.hll.insert(&value);
    }

    #[inline]
    fn model_name(&self) -> String {
        format!("Uncorrected<P{}, B{}>", P::EXPONENT, B::NUMBER_OF_BITS)
    }

    #[inline]
    fn union(&self, _other: &Self) -> f64 {
        unimplemented!()
    }
}

const ITERATIONS: usize = 2 * 64;

#[derive(Default, Copy, Clone, Serialize, Deserialize)]
struct Sample {
    exact_cardinality_sum: f64,
    count: u64,
    cardinality_estimate_sum: f64,
    relative_error_sum: f64,
}

impl Sample {
    fn update(&mut self, exact_cardinality: f64, cardinality_estimate: f64) {
        self.relative_error_sum +=
            (cardinality_estimate - exact_cardinality).abs() / (exact_cardinality).max(1.0);
        self.exact_cardinality_sum += exact_cardinality;
        self.cardinality_estimate_sum += cardinality_estimate;
        self.count += 1;
    }

    fn into_stored_report(self) -> StoredReport {
        assert_ne!(self.count, 0, "There was no sample to compute the mean.");
        StoredReport {
            exact_cardinality_mean: self.exact_cardinality_sum / self.count as f64,
            estimated_cardinality_mean: self.cardinality_estimate_sum / self.count as f64,
            relative_error_mean: self.relative_error_sum / self.count as f64,
        }
    }
}

impl Add for Sample {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            exact_cardinality_sum: self.exact_cardinality_sum + other.exact_cardinality_sum,
            count: self.count + other.count,
            relative_error_sum: self.relative_error_sum + other.relative_error_sum,
            cardinality_estimate_sum: self.cardinality_estimate_sum
                + other.cardinality_estimate_sum,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct StoredReport {
    exact_cardinality_mean: f64,
    estimated_cardinality_mean: f64,
    relative_error_mean: f64,
}

fn measure<S: Set + Default + MemSize>(reference: Option<&[StoredReport]>) -> Vec<StoredReport> {
    let random_state = 7_536_558_723_694_876_u64;

    let max_hashes = 50_000_000_000;
    let step_size = u64::MAX / (max_hashes + 1);
    let capacity_to_allocate = cardinality_estimate_to_index(max_hashes) as usize + 1;

    let model_name = S::default().model_name();
    let path = format!("{}.csv.gz", model_name);

    let stored_reports = if let Ok(stored_reports) = read_csv(path.as_str()) {
        stored_reports
    } else {
        let progress_bar = ProgressBar::new(ITERATIONS as u64);
        progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(&format!("Measuring {model_name} [{{elapsed_precise}} {{eta_precise}}] {{bar:40.cyan/blue}} {{pos:>7}}/{{len:7}} {{msg}}"))
            .unwrap()
            .progress_chars("##-"),
        );

        let number_of_threads = rayon::current_num_threads();

        let reports: Vec<ThreadUnsafeCell<Vec<Sample>>> = (0..number_of_threads)
            .map(|_| ThreadUnsafeCell::new(vec![Sample::default(); capacity_to_allocate]))
            .collect();

        (0..ITERATIONS)
            .into_par_iter()
            .progress_with(progress_bar)
            .for_each(|i| {
                let bias = splitmix64(splitmix64(
                    random_state.wrapping_mul(splitmix64(i as u64 + 1)),
                ));
                let mut hll = S::default();
                let reports = reports[rayon::current_thread_index().unwrap()].get_mut();
                let mut starting_value = bias;

                for exact_cardinality in 0..=max_hashes {
                    let cardinality_estimate = hll.cardinality();

                    let index: usize = cardinality_estimate_to_index(exact_cardinality);
                    reports[index].update(exact_cardinality as f64, cardinality_estimate);
                    hll.insert_element(starting_value);

                    starting_value = starting_value.wrapping_add(step_size);
                }
            });

        // We flatten the reports from all threads into a single vector
        let total_reports: Vec<Sample> = reports
            .into_iter()
            .map(|report| report.into_inner())
            .reduce(|mut left: Vec<Sample>, right: Vec<Sample>| {
                left.iter_mut()
                    .zip(right)
                    .for_each(|(left, right)| *left = *left + right);
                left
            })
            .unwrap();

        // We convert the samples into stored reports
        let total_reports: Vec<StoredReport> = total_reports
            .into_iter()
            .map(|sample| sample.into_stored_report())
            .collect();

        // If the reference report has a length that is different from the total reports,
        // we assume that there has been a change in the code and we need to update the
        // reference report.
        write_csv(total_reports.iter(), &path);
        total_reports
    };

    if let Some(reference) = reference {
        let old_errors: Vec<f64> = reference
            .iter()
            .map(|report| report.relative_error_mean)
            .collect();
        let new_errors: Vec<f64> = stored_reports
            .iter()
            .map(|report| report.relative_error_mean)
            .collect();
        let error_benchmark = compare_features(
            new_errors.as_slice(),
            old_errors.as_slice(),
            "Relative Error",
        );

        error_benchmark.print();
    }

    stored_reports
}

/// Main function to compare the progress of two variants of HLL.
fn main() {
    let _reference = measure::<Uncorrected<Precision4, Bits5>>(None);
    // measure::<CloudFlareHLL<4, 6, XxHash64>>(Some(&reference));
    // measure::<TabacHLLPlusPlus<Precision4, XxHash64>>(Some(&reference));
}
