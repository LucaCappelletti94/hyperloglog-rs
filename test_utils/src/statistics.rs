//! This module contains functions for calculating statistics.
use colored::*;
use core::iter::Sum;
use core::ops::Div;
use stattest::test::StatisticalTest;
use stattest::test::WilcoxonWTest;

pub fn standard_deviation(values: &[f64], mean: f64, occurrences: &[usize]) -> f64 {
    // The values are always less than `u32::MAX`, so we can safely convert them.
    let number_of_values = occurrences.iter().sum::<usize>() as f64;
    let variance = values.iter().zip(occurrences.iter().copied()).map(|(v, o)| (v - mean).powi(2)*o as f64).sum::<f64>() / number_of_values;
    variance.sqrt()
}

pub fn mean<T, I>(values: I) -> T
where
    I: ExactSizeIterator<Item = T>,
    T: Sum + Div<T, Output = T> + TryFrom<u32>,
    <T as std::convert::TryFrom<u32>>::Error: std::fmt::Debug,
{
    // The values are always less than `u32::MAX`, so we can safely convert them.
    let number_of_values = T::try_from(u32::try_from(values.len()).unwrap()).unwrap();
    values.sum::<T>() / number_of_values
}

/// Returns a tuple with the mean and standard deviation of the values.
pub fn mean_and_std(values: &[f64], occurrences: &[usize]) -> (f64, f64) {
    let total_occurrences = occurrences.iter().sum::<usize>();
    let mean = values.iter().zip(occurrences.iter()).map(|(v, o)| v * *o as f64).sum::<f64>() / total_occurrences as f64;
    let std = standard_deviation(values, mean, occurrences);
    (mean, std)
}

pub enum TestResult {
    Significant(f64),
    NotSignificant(f64),
    Unknown,
}

pub struct Stats {
    pub mean: f64,
    pub std: f64,
}

pub struct BenchmarkResults<'a> {
    pub feature: &'a str,
    pub new_stats: Stats,
    pub old_stats: Stats,
    pub new_model: &'a str,
    pub old_model: &'a str,
    pub p_value: TestResult,
}

impl<'a> BenchmarkResults<'a> {
    /// Given the name of the feature provided, determines whether
    /// it should be to maximise (accuracy) or minimise (error, time, memory).
    fn estimate_feature_target(&self) -> bool {
        let maximising_needles = ["accuracy"];
        let minimising_needles = ["error", "time", "memory"];

        let feature = self.feature.to_lowercase();

        if maximising_needles
            .iter()
            .any(|needle| feature.contains(needle))
        {
            return true;
        }

        if minimising_needles
            .iter()
            .any(|needle| feature.contains(needle))
        {
            return false;
        }

        unimplemented!("Feature target not found for '{}'", self.feature);
    }

    pub fn new(feature: &'a str, new_stats: Stats, old_stats: Stats, new_model: &'a str, old_model: &'a str, p_value: TestResult) -> Self {
        Self {
            feature,
            new_stats,
            old_stats,
            new_model,
            old_model,
            p_value,
        }
    }

    /// Prints out the benchmark results.
    ///
    /// Specifically, it prints out the feature name, the new and old means,
    /// their standard deviations, and depending on the p-value, whether the
    /// difference is significant or not. Furthermore, depending on whether
    /// the feature is to be maximised or minimised, shows the relative improvement
    /// or degradation, using appropriate colors for readability.
    pub fn print(&self) {
        // Bold headers
        println!("{}", "Benchmark Results".bold().underline());

        // Print the feature name
        println!("{}", format!("Feature: {}", self.feature).bold());

        // Print new stats
        println!(
            "{}",
            format!(
                "{} Mean: {:.4} ± {:.4}",
                self.new_model,
                self.new_stats.mean, self.new_stats.std
            )
            .blue()
        );

        // Print old stats
        println!(
            "{}",
            format!(
                "{} Mean: {:.4} ± {:.4}",
                self.old_model,
                self.old_stats.mean, self.old_stats.std
            )
            .yellow()
        );

        // Calculate the improvement percentage
        let improvement_percentage =
            ((self.new_stats.mean - self.old_stats.mean).abs() / self.old_stats.mean) * 100.0;

        // Print the p-value and significance
        match self.p_value {
            TestResult::Significant(p_value) => {
                println!(
                    "{}: YES ({:+e})",
                    "Statistical Significance".green(),
                    p_value
                );
            }
            TestResult::NotSignificant(p_value) => {
                println!("{}: NO ({:+e})", "Statistical Significance".red(), p_value);
            }
            TestResult::Unknown => {
                println!("{}: UNKNOWN", "Statistical Significance".red());
            }
        }

        // Print the improvement percentage
        match (
            self.new_stats.mean > self.old_stats.mean,
            self.estimate_feature_target(),
        ) {
            (true, true) => {
                println!("{}: {:.2}%", "Improved by".green(), improvement_percentage);
            }
            (true, false) => {
                println!("{}: {:.2}%", "Degraded by".red(), improvement_percentage);
            }
            (false, true) => {
                println!("{}: {:.2}%", "Degraded by".red(), improvement_percentage);
            }
            (false, false) => {
                println!("{}: {:.2}%", "Improved by".green(), improvement_percentage);
            }
        }

        // Add spacing between results for readability
        println!();
    }
}

pub fn compare_features<'a>(
    new_results: &'a [f64],
    old_results: &'a [f64],
    occurrences: &'a [usize],
    feature: &'a str,
    new_model: &'a str,
    old_model: &'a str,
) -> BenchmarkResults<'a> {
    let (new_mean, new_std) = mean_and_std(new_results, occurrences);
    let (old_mean, old_std) = mean_and_std(old_results, occurrences);

    let test = WilcoxonWTest::weighted_paired(new_results, old_results, occurrences);

    BenchmarkResults {
        feature,
        new_stats: Stats {
            mean: new_mean,
            std: new_std,
        },
        old_stats: Stats {
            mean: old_mean,
            std: old_std,
        },
        new_model,
        old_model,
        p_value: if let Ok(test) = test {
            if test.p_value() < 0.05 {
                TestResult::Significant(test.p_value())
            } else {
                TestResult::NotSignificant(test.p_value())
            }
        } else if new_mean == old_mean {
            TestResult::NotSignificant(1.0)
        } else {
            TestResult::Unknown
        },
    }
}
