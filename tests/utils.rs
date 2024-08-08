use std::hash::{BuildHasher, Hash};

use hyperloglog_rs::prelude::*;
use hyperloglogplus::HyperLogLog as TabacHyperLogLog;
use hyperloglogplus::HyperLogLogPF as TabacHyperLogLogPF;
use hyperloglogplus::HyperLogLogPlus as TabacHyperLogLogPlus;
use cardinality_estimator::CardinalityEstimator;
use rust_hyperloglog::HyperLogLog as RustHyperLogLog;
use mem_dbg::MemSize;
use stattest::test::WilcoxonWTest;
use stattest::test::StatisticalTest;
use streaming_algorithms::HyperLogLog as SAHyperLogLog;

pub trait MutableSet<I: Hash>: MemSize {
    fn create(precision: usize) -> Self;
    fn insert(&mut self, x: I);
}

impl MutableSet<usize> for std::collections::HashSet<usize> {
    fn create(_precision: usize) -> Self {
        Self::new()
    }
    fn insert(&mut self, x: usize) {
        self.insert(x);
    }
}

impl<I: Hash> MutableSet<I> for RustHyperLogLog {
    fn create(precision: usize) -> Self {
        let exponent = (precision as f64) / 2.0;
        let error_rate = 1.04 / 2f64.powf(exponent);
        RustHyperLogLog::new(error_rate)
    }
    fn insert(&mut self, x: I) {
        self.insert(&x);
    }
}

impl<I: Hash, const P: usize, H: core::hash::Hasher + Default, const W: usize> MutableSet<I> for CardinalityEstimator<I, H, P, W> {
    fn create(precision: usize) -> Self {
        assert!(precision == P);
        Self::new()
    }
    fn insert(&mut self, x: I) {
        self.insert(&x);
    }
}

impl<
        I: Hash,
        Hasher: Default + core::hash::Hasher,
        P: MemSize + Precision + PrecisionConstants<f64>,
        B: MemSize + Bits,
        R: MemSize + Registers<P, B>,
    > MutableSet<I> for HyperLogLog<P, B, R, Hasher>
where
    Self: HyperLogLogTrait<P, B, Hasher>,
    <P as Precision>::NumberOfZeros: MemSize,
{
    fn create(precision: usize) -> Self {
        assert!(precision == P::EXPONENT);
        Self::default()
    }
    fn insert(&mut self, x: I) {
        <Self as HyperLogLogTrait<P, B, Hasher>>::insert(self, x);
    }
}

impl<
        const ERROR: i32,
        I: Hash,
        P: MemSize + Precision + PrecisionConstants<f64>,
        B: MemSize + Bits,
        R: MemSize + Registers<P, B>,
        Hasher: Default + core::hash::Hasher,
    > MutableSet<I> for MLE<HyperLogLog<P, B, R, Hasher>, ERROR>
where
    Self: HyperLogLogTrait<P, B, Hasher>,
    <P as Precision>::NumberOfZeros: MemSize,
    HyperLogLog<P, B, R, Hasher>: MemSize,
{
    fn create(precision: usize) -> Self {
        assert!(precision == P::EXPONENT);
        Self::default()
    }
    fn insert(&mut self, x: I) {
        <Self as HyperLogLogTrait<P, B, Hasher>>::insert(self, x);
    }
}

impl<I: Hash + Eq + PartialEq, B: MemSize + BuildHasher + Default> MutableSet<I>
    for TabacHyperLogLogPF<I, B>
where
    Self: Clone,
{
    fn create(precision: usize) -> Self {
        TabacHyperLogLogPF::new(precision as u8, B::default()).unwrap()
    }
    fn insert(&mut self, x: I) {
        TabacHyperLogLog::insert(self, &x);
    }
}

impl<I: Hash + Eq + PartialEq, B: MemSize + BuildHasher + Default> MutableSet<I>
    for TabacHyperLogLogPlus<I, B>
where
    Self: Clone,
{
    fn create(precision: usize) -> Self {
        TabacHyperLogLogPlus::new(precision as u8, B::default()).unwrap()
    }

    fn insert(&mut self, x: I) {
        TabacHyperLogLog::insert(self, &x);
    }
}

impl<I: Hash + Eq + PartialEq> MutableSet<I> for SAHyperLogLog<I> {
    fn create(precision: usize) -> Self {
        let exponent = (precision as f64) / 2.0;
        let error_rate = 1.04 / 2f64.powf(exponent);
        SAHyperLogLog::new(error_rate)
    }

    fn insert(&mut self, x: I) {
        self.push(&x);
    }
}

pub fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

pub fn xorshift64(mut x: u64) -> u64 {
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

fn standard_deviation(values: &[f64], mean: f64) -> f64 {
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn mean_usize(values: &[usize]) -> f64 {
    values.iter().sum::<usize>() as f64 / values.len() as f64
}

pub fn iter_random_vector(mut random_state: u64) -> impl Iterator<Item = usize> {
    let maximal_size = 1_000_000;
    let maximal_value = 1_000_000;
    random_state = splitmix64(random_state);
    let maximal_size = 1 + xorshift64(random_state) as usize % maximal_size;
    random_state = splitmix64(random_state);
    let maximal_value = 1 + xorshift64(random_state) as usize % maximal_value;
    (0..maximal_size).map(move |_| {
        random_state = splitmix64(random_state);
        random_state = xorshift64(random_state);
        random_state as usize % maximal_value
    })
}

pub fn populate<P: Precision, S: MutableSet<usize>>(random_state: u64) -> (S, usize) {
    let mut set_like: S = S::create(P::EXPONENT);
    for l in iter_random_vector(random_state) {
        set_like.insert(l.clone());
    }

    let size = set_like.mem_size(mem_dbg::SizeFlags::default());
    (set_like, size)
}

pub fn statistical_report<const N: usize, P: Precision>(
    approach_names: &[&str; N],
    absolute_errors: &[&[f64]; N],
    memory_requirements: &[&[usize]; N],
    feature_name: &str,
) {
    // We compute the actual means
    let means: Vec<f64> = absolute_errors.iter().map(|errors| mean(errors)).collect();

    // And the standard deviations
    let stds: Vec<f64> = absolute_errors
        .iter()
        .zip(means.iter())
        .map(|(errors, mean)| standard_deviation(errors, *mean))
        .collect();

    // We open a CSV document where to store the results of the test.
    let mut writer = csv::Writer::from_path(&format!(
        "tests/statistical_tests_reports/{feature_name}_{}.csv",
        P::EXPONENT
    ))
    .unwrap();

    // We write the header of the CSV document.
    writer
        .write_record(&[
            "feature",
            "first_approach",
            "second_approach",
            "p-value",
            "winner",
            "first_memsize",
            "first_mean",
            "first_std",
            "second_memsize",
            "second_mean",
            "second_std",
            "precision",
            "theoretical_error",
        ])
        .unwrap();

    for (
        i,
        ((first_approach_name, first_memsize), (first_absolute_errors, (first_mean, first_std))),
    ) in approach_names
        .iter()
        .zip(memory_requirements.iter())
        .zip(absolute_errors.iter().zip(means.iter().zip(stds.iter())))
        .enumerate()
    {
        for (
            (second_approach_name, second_memsize),
            (second_absolute_errors, (second_mean, second_std)),
        ) in approach_names[i + 1..]
            .iter()
            .zip(memory_requirements[i + 1..].iter())
            .zip(
                absolute_errors[i + 1..]
                    .iter()
                    .zip(means[i + 1..].iter().zip(stds[i + 1..].iter())),
            )
        {
            let w_test =
                WilcoxonWTest::paired(first_absolute_errors, second_absolute_errors);

            writer
                .write_record(&[
                    feature_name,
                    first_approach_name,
                    second_approach_name,
                    w_test.as_ref().map(|w_test| format!("{:.5}", w_test.p_value())).unwrap_or("Unknown".to_owned()).as_str(),
                    if let Ok(w_test) = w_test.as_ref() {
                        if w_test.p_value() < 0.05 {
                            if first_mean < second_mean {
                                "First"
                            } else {
                                "Second"
                            }
                        } else {
                            "None"
                        }
                    } else {
                        "Unknown"
                    },
                    format!("{}", mean_usize(first_memsize)).as_str(),
                    format!("{}", first_mean).as_str(),
                    format!("{}", first_std).as_str(),
                    format!("{}", mean_usize(second_memsize)).as_str(),
                    format!("{}", second_mean).as_str(),
                    format!("{}", second_std).as_str(),
                    format!("{}", P::EXPONENT).as_str(),
                    format!("{}", P::error_rate()).as_str(),
                ])
                .unwrap();

            // We check with a Wilcoxon signed-rank test if the difference between the
            // two approaches is significant. If it is, we compare the mean absolute error
            // of the two approaches. If the mean absolute error of the competitor library
            // is lower, we fail the test.
            // if w_test.p_value() < 0.05 {
            //     assert!(
            //         their_mean >= ours_hll6_mean,
            //         "MAE ({:.4}±{:.4}) of {} > MAE ({:.4}±{:.4}) of {} approach (p-value {:.4}), expected at most {:.4}.",
            //         ours_hll6_mean,
            //         ours_hll6_std,
            //         "HLL<bits=6>",
            //         their_mean,
            //         their_std,
            //         library_name,
            //         w_test.p_value(),
            //         P::error_rate()
            //     );
            // }
        }
    }
    // We close the CSV document.
    writer.flush().unwrap();
}
