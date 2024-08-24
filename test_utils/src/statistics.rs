//! This module contains functions for calculating statistics.
use core::iter::Sum;
use core::ops::Div;
use stattest::test::StatisticalTest;
use stattest::test::WilcoxonWTest;
use colored::*;

pub fn standard_deviation(values: &[f64], mean: f64) -> f64 {
    // The values are always less than `u32::MAX`, so we can safely convert them.
    let number_of_values = f64::from(u32::try_from(values.len()).unwrap());
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / number_of_values;
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
pub fn mean_and_std(values: &[f64]) -> (f64, f64) {
    let mean = mean(values.iter().copied());
    let std = standard_deviation(values, mean);
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
    pub p_value: TestResult,
}

impl<'a> BenchmarkResults<'a> {

    /// Given the name of the feature provided, determines whether
    /// it should be to maximise (accuracy) or minimise (error, time, memory).
    fn estimate_feature_target(&self) -> bool {
        let maximising_needles = ["accuracy"];
        let minimising_needles = ["error", "time", "memory"];

        let feature = self.feature.to_lowercase();

        if maximising_needles.iter().any(|needle| feature.contains(needle)) {
            return true;
        }

        if minimising_needles.iter().any(|needle| feature.contains(needle)) {
            return false;
        }

        unimplemented!("Feature target not found for '{}'", self.feature);
    }

    pub fn new(
        feature: &'a str,
        new_stats: Stats,
        old_stats: Stats,
        p_value: TestResult,
    ) -> Self {
        Self {
            feature,
            new_stats,
            old_stats,
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
                "New Mean: {:.4} ± {:.4}",
                self.new_stats.mean, self.new_stats.std
            )
            .blue()
        );

        // Print old stats
        println!(
            "{}",
            format!(
                "Old Mean: {:.4} ± {:.4}",
                self.old_stats.mean, self.old_stats.std
            )
            .yellow()
        );

        // Calculate the improvement percentage
        let improvement_percentage = if self.estimate_feature_target() {
            ((self.new_stats.mean - self.old_stats.mean) / self.old_stats.mean) * 100.0
        } else {
            ((self.old_stats.mean - self.new_stats.mean) / self.old_stats.mean) * 100.0
        };

        // Print the p-value and significance
        match self.p_value {
            TestResult::Significant(p_value) => {
                println!(
                    "{}: {} ({:.4})",
                    "Statistical Significance".green(),
                    "YES",
                    p_value
                );
            },
            TestResult::NotSignificant(p_value) => {
                println!(
                    "{}: {} ({:.4})",
                    "Statistical Significance".red(),
                    "NO",
                    p_value
                );
            },
            TestResult::Unknown => {
                println!(
                    "{}: {}",
                    "Statistical Significance".red(),
                    "UNKNOWN"
                );
            }
        }

        // Print the improvement percentage
        if improvement_percentage.is_nan() {
            println!(
                "{}: {}",
                "Relative Improvement".cyan(),
                "N/A"
            );
        } else {
            println!(
                "{}: {:.2}%",
                "Relative Improvement".cyan(),
                improvement_percentage
            );
        }

        // Add spacing between results for readability
        println!();
    }
}

pub fn compare_features<'a>(
    new_results: &'a [f64],
    old_results: &'a [f64],
    feature: &'a str,
) -> BenchmarkResults<'a> {
    let (new_mean, new_std) = mean_and_std(new_results);
    let (old_mean, old_std) = mean_and_std(old_results);

    let test = WilcoxonWTest::voracious_paired(new_results, old_results);

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
        p_value: if let Ok(test) = test {
            if test.p_value() < 0.05 {
                TestResult::Significant(test.p_value())
            } else {
                TestResult::NotSignificant(test.p_value())
            }
        } else {
            if new_mean == old_mean {
                TestResult::NotSignificant(1.0)
            } else {
                TestResult::Unknown
            }
        },
    }
}
