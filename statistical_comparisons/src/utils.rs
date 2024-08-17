include!("../../benches/utils.rs");
use std::collections::HashSet;
use std::fmt::Display;
use std::hash::RandomState;

use hyperloglog_rs::prelude::*;
use indicatif::ProgressIterator;
use mem_dbg::{MemSize, SizeFlags};
use stattest::test::StatisticalTest;
use stattest::test::WilcoxonWTest;

pub(crate) enum SetLikeObjects<const EXPONENT: usize, P: Precision>
where
    P: ArrayRegister<Bits8> + ArrayRegister<Bits6>,
{
    HashSet(HashSet<u64>),
    TabacHyperLogLogPlus(TabacHLLPlusPlus<P>),
    TabacHyperLogLogPF(TabacHLL<P>),
    SAHyperLogLog(SAHyperLogLog<u64>),
    RustHyperLogLog(RustHyperLogLog),
    CardinalityEstimator(CardinalityEstimator<u64, wyhash::WyHash, EXPONENT, 6>),
    HLL6Xxhasher(
        PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>,
    ),
    HLL6WyHash(PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, wyhash::WyHash>),
    HLL8Xxhasher(
        PlusPlus<P, Bits8, <P as ArrayRegister<Bits8>>::ArrayRegister, twox_hash::XxHash64>,
    ),
    HLL8WyHash(PlusPlus<P, Bits8, <P as ArrayRegister<Bits8>>::ArrayRegister, wyhash::WyHash>),
    Beta6Xxhasher(
        LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>,
    ),
    Beta6WyHash(LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, wyhash::WyHash>),
    Beta8Xxhasher(
        LogLogBeta<P, Bits8, <P as ArrayRegister<Bits8>>::ArrayRegister, twox_hash::XxHash64>,
    ),
    Beta8WyHash(LogLogBeta<P, Bits8, <P as ArrayRegister<Bits8>>::ArrayRegister, wyhash::WyHash>),
    #[cfg(feature = "mle")]
    MLEPPWyHash(
        MLE<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, wyhash::WyHash>, 2>,
    ),
    #[cfg(feature = "mle")]
    MLEPPXxhasher(
        MLE<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>, 2>,
    ),
    #[cfg(feature = "mle")]
    MLEBetaWyHash(
        MLE<LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, wyhash::WyHash>, 2>,
    ),
    #[cfg(feature = "mle")]
    MLEBetaXxhasher(
        MLE<
            LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>,
            2,
        >,
    ),
    #[cfg(feature = "mle")]
    HybridMLEPPWyHash(
        Hybrid<
            MLE<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, wyhash::WyHash>, 2>,
        >,
    ),
    #[cfg(feature = "mle")]
    HybridMLEPPXxhasher(
        Hybrid<
            MLE<
                PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>,
                2,
            >,
        >,
    ),
    #[cfg(feature = "mle")]
    HybridMLEBetaWyHash(
        Hybrid<
            MLE<
                LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, wyhash::WyHash>,
                2,
            >,
        >,
    ),
    #[cfg(feature = "mle")]
    HybridMLEBetaXxhasher(
        Hybrid<
            MLE<
                LogLogBeta<
                    P,
                    Bits6,
                    <P as ArrayRegister<Bits6>>::ArrayRegister,
                    twox_hash::XxHash64,
                >,
                2,
            >,
        >,
    ),
    HybridPPWyHash(
        Hybrid<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, wyhash::WyHash>>,
    ),
    HybridPPXxhasher(
        Hybrid<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>>,
    ),
    HybridBetaWyHash(
        Hybrid<LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, wyhash::WyHash>>,
    ),
    HybridBetaXxhasher(
        Hybrid<
            LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>,
        >,
    ),
}

impl<const EXPONENT: usize, P: Precision> Named for SetLikeObjects<EXPONENT, P> {
    fn name(&self) -> String {
        
    }
}

impl<const EXPONENT: usize, P: Precision> SetLikeObjects<EXPONENT, P>
where
    P: ArrayRegister<Bits8> + ArrayRegister<Bits6>,
    P: MemSize,
    <P as ArrayRegister<Bits8>>::ArrayRegister: MemSize + Words<Word = u64>,
    <P as ArrayRegister<Bits6>>::ArrayRegister: MemSize + Words<Word = u64>,
{
    pub(crate) fn all_cardinalities() -> Vec<Self> {
        vec![
            SetLikeObjects::HashSet(HashSet::create(P::EXPONENT)),
            SetLikeObjects::TabacHyperLogLogPlus(TabacHyperLogLogPlus::create(P::EXPONENT)),
            SetLikeObjects::TabacHyperLogLogPF(TabacHyperLogLogPF::create(P::EXPONENT)),
            SetLikeObjects::SAHyperLogLog(SAHyperLogLog::create(P::EXPONENT)),
            SetLikeObjects::RustHyperLogLog(<RustHyperLogLog as TestSetLike<u64>>::create(
                P::EXPONENT,
            )),
            SetLikeObjects::CardinalityEstimator(CardinalityEstimator::create(P::EXPONENT)),
            SetLikeObjects::HLL6Xxhasher(Default::default()),
            SetLikeObjects::HLL6WyHash(Default::default()),
            SetLikeObjects::HLL8Xxhasher(Default::default()),
            SetLikeObjects::HLL8WyHash(Default::default()),
            SetLikeObjects::Beta6Xxhasher(Default::default()),
            SetLikeObjects::Beta6WyHash(Default::default()),
            SetLikeObjects::Beta8Xxhasher(Default::default()),
            SetLikeObjects::Beta8WyHash(Default::default()),
            SetLikeObjects::HybridPPWyHash(Default::default()),
            SetLikeObjects::HybridPPXxhasher(Default::default()),
            SetLikeObjects::HybridBetaWyHash(Default::default()),
            SetLikeObjects::HybridBetaXxhasher(Default::default()),
        ]
    }

    pub(crate) fn all_union() -> Vec<Self> {
        vec![
            SetLikeObjects::HashSet(HashSet::create(P::EXPONENT)),
            SetLikeObjects::TabacHyperLogLogPlus(TabacHyperLogLogPlus::create(P::EXPONENT)),
            SetLikeObjects::TabacHyperLogLogPF(TabacHyperLogLogPF::create(P::EXPONENT)),
            SetLikeObjects::SAHyperLogLog(SAHyperLogLog::create(P::EXPONENT)),
            SetLikeObjects::RustHyperLogLog(<RustHyperLogLog as TestSetLike<u64>>::create(
                P::EXPONENT,
            )),
            SetLikeObjects::CardinalityEstimator(CardinalityEstimator::create(P::EXPONENT)),
            SetLikeObjects::HLL6Xxhasher(Default::default()),
            SetLikeObjects::HLL6WyHash(Default::default()),
            SetLikeObjects::HLL8Xxhasher(Default::default()),
            SetLikeObjects::HLL8WyHash(Default::default()),
            SetLikeObjects::Beta6Xxhasher(Default::default()),
            SetLikeObjects::Beta6WyHash(Default::default()),
            SetLikeObjects::Beta8Xxhasher(Default::default()),
            SetLikeObjects::Beta8WyHash(Default::default()),
            #[cfg(feature = "mle")]
            SetLikeObjects::MLEPPXxhasher(Default::default()),
            #[cfg(feature = "mle")]
            SetLikeObjects::MLEBetaXxhasher(Default::default()),
            #[cfg(feature = "mle")]
            SetLikeObjects::HybridMLEPPXxhasher(Default::default()),
            #[cfg(feature = "mle")]
            SetLikeObjects::HybridMLEBetaXxhasher(Default::default()),
            SetLikeObjects::HybridPPWyHash(Default::default()),
            SetLikeObjects::HybridPPXxhasher(Default::default()),
            SetLikeObjects::HybridBetaWyHash(Default::default()),
            SetLikeObjects::HybridBetaXxhasher(Default::default()),
        ]
    }
}

impl<const EXPONENT: usize, P: Precision> MemSize for SetLikeObjects<EXPONENT, P>
where
    P: ArrayRegister<Bits8> + ArrayRegister<Bits6>,
    P: MemSize,
    <P as ArrayRegister<Bits8>>::ArrayRegister: MemSize,
    <P as ArrayRegister<Bits6>>::ArrayRegister: MemSize,
    P::NumberOfZeros: MemSize,
{
    fn mem_size(&self, flags: SizeFlags) -> usize {
        match self {
            SetLikeObjects::HashSet(set) => set.mem_size(flags),
            SetLikeObjects::TabacHyperLogLogPlus(set) => set.mem_size(flags),
            SetLikeObjects::TabacHyperLogLogPF(set) => set.mem_size(flags),
            SetLikeObjects::SAHyperLogLog(set) => set.mem_size(flags),
            SetLikeObjects::RustHyperLogLog(set) => set.mem_size(flags),
            SetLikeObjects::CardinalityEstimator(set) => set.mem_size(flags),
            SetLikeObjects::HLL6Xxhasher(set) => set.mem_size(flags),
            SetLikeObjects::HLL6WyHash(set) => set.mem_size(flags),
            SetLikeObjects::HLL8Xxhasher(set) => set.mem_size(flags),
            SetLikeObjects::HLL8WyHash(set) => set.mem_size(flags),
            SetLikeObjects::Beta6Xxhasher(set) => set.mem_size(flags),
            SetLikeObjects::Beta6WyHash(set) => set.mem_size(flags),
            SetLikeObjects::Beta8Xxhasher(set) => set.mem_size(flags),
            SetLikeObjects::Beta8WyHash(set) => set.mem_size(flags),
            #[cfg(feature = "mle")]
            SetLikeObjects::MLEPPWyHash(set) => set.mem_size(flags),
            #[cfg(feature = "mle")]
            SetLikeObjects::MLEPPXxhasher(set) => set.mem_size(flags),
            #[cfg(feature = "mle")]
            SetLikeObjects::MLEBetaWyHash(set) => set.mem_size(flags),
            #[cfg(feature = "mle")]
            SetLikeObjects::MLEBetaXxhasher(set) => set.mem_size(flags),
            #[cfg(feature = "mle")]
            SetLikeObjects::HybridMLEPPWyHash(set) => set.mem_size(flags),
            #[cfg(feature = "mle")]
            SetLikeObjects::HybridMLEPPXxhasher(set) => set.mem_size(flags),
            #[cfg(feature = "mle")]
            SetLikeObjects::HybridMLEBetaWyHash(set) => set.mem_size(flags),
            #[cfg(feature = "mle")]
            SetLikeObjects::HybridMLEBetaXxhasher(set) => set.mem_size(flags),
            SetLikeObjects::HybridPPWyHash(set) => set.mem_size(flags),
            SetLikeObjects::HybridPPXxhasher(set) => set.mem_size(flags),
            SetLikeObjects::HybridBetaWyHash(set) => set.mem_size(flags),
            SetLikeObjects::HybridBetaXxhasher(set) => set.mem_size(flags),
        }
    }
}

impl<const EXPONENT: usize, P: Precision> TestSetLike<u64> for SetLikeObjects<EXPONENT, P>
where
    P: ArrayRegister<Bits8> + ArrayRegister<Bits6>,
    <P as ArrayRegister<Bits8>>::ArrayRegister: Words<Word = u64> + MemSize,
    <P as ArrayRegister<Bits6>>::ArrayRegister: Words<Word = u64> + MemSize,
    Self: MemSize,
{
    fn create(_precision: usize) -> Self {
        unimplemented!()
    }

    fn insert(&mut self, value: &u64) {
        match self {
            SetLikeObjects::HashSet(set) => {
                <HashSet<u64> as TestSetLike<u64>>::insert(set, value);
            }
            SetLikeObjects::TabacHyperLogLogPlus(set) => {
                <TabacHyperLogLogPlus<u64, RandomState> as TestSetLike<u64>>::insert(set, value);
            }
            SetLikeObjects::TabacHyperLogLogPF(set) => {
                <TabacHyperLogLogPF<u64, RandomState> as TestSetLike<u64>>::insert(set, value);
            }
            SetLikeObjects::SAHyperLogLog(set) => {
                set.insert(value);
            }
            SetLikeObjects::RustHyperLogLog(set) => {
                set.insert(&value);
            }
            SetLikeObjects::CardinalityEstimator(set) => {
                set.insert(&value);
            }
            SetLikeObjects::HLL6Xxhasher(set) => {
                set.insert(value);
            }
            SetLikeObjects::HLL6WyHash(set) => {
                set.insert(value);
            }
            SetLikeObjects::HLL8Xxhasher(set) => {
                set.insert(value);
            }
            SetLikeObjects::HLL8WyHash(set) => {
                set.insert(value);
            }
            SetLikeObjects::Beta6Xxhasher(set) => {
                set.insert(value);
            }
            SetLikeObjects::Beta6WyHash(set) => {
                set.insert(value);
            }
            SetLikeObjects::Beta8Xxhasher(set) => {
                set.insert(value);
            }
            SetLikeObjects::Beta8WyHash(set) => {
                set.insert(value);
            }
            #[cfg(feature = "mle")]
            SetLikeObjects::MLEPPWyHash(set) => {
                set.insert(value);
            }
            #[cfg(feature = "mle")]
            SetLikeObjects::MLEPPXxhasher(set) => {
                set.insert(value);
            }
            #[cfg(feature = "mle")]
            SetLikeObjects::MLEBetaWyHash(set) => {
                set.insert(value);
            }
            #[cfg(feature = "mle")]
            SetLikeObjects::MLEBetaXxhasher(set) => {
                set.insert(value);
            }
            #[cfg(feature = "mle")]
            SetLikeObjects::HybridMLEPPWyHash(set) => {
                set.insert(value);
            }
            #[cfg(feature = "mle")]
            SetLikeObjects::HybridMLEPPXxhasher(set) => {
                set.insert(value);
            }
            #[cfg(feature = "mle")]
            SetLikeObjects::HybridMLEBetaWyHash(set) => {
                set.insert(value);
            }
            #[cfg(feature = "mle")]
            SetLikeObjects::HybridMLEBetaXxhasher(set) => {
                set.insert(value);
            }
            SetLikeObjects::HybridPPWyHash(set) => {
                set.insert(value);
            }
            SetLikeObjects::HybridPPXxhasher(set) => {
                set.insert(value);
            }
            SetLikeObjects::HybridBetaWyHash(set) => {
                set.insert(value);
            }
            SetLikeObjects::HybridBetaXxhasher(set) => {
                set.insert(value);
            }
        }
    }
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

/// Transposes a provided vector of vectors.
pub fn transpose<T: Copy + Default>(vec: &Vec<Vec<T>>) -> Vec<Vec<T>> {
    let mut transposed_vector = vec![vec![T::default(); vec.len()]; vec[0].len()];

    let progress_bar = indicatif::ProgressBar::new(vec.len() as u64);
    progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("Transposing: [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
            .unwrap()
            .progress_chars("##-"),
    );

    for i in (0..vec.len()).progress_with(progress_bar) {
        for j in 0..vec[i].len() {
            transposed_vector[j][i] = vec[i][j];
        }
    }

    transposed_vector
}

fn write_csv<P: Precision, T: Display + Default + Copy>(
    transposed_data: &Vec<Vec<T>>,
    feature_name: &str,
    data_name: &str,
    approach_names: &[&str],
) {
    let mut writer = csv::Writer::from_path(&format!(
        "./statistical_tests_reports/{feature_name}_{data_name}_{exponent}.csv",
        feature_name = feature_name,
        data_name = data_name,
        exponent = P::EXPONENT
    ))
    .unwrap();

    writer.write_record(approach_names.iter().copied()).unwrap();

    let progress_bar = indicatif::ProgressBar::new(transposed_data.len() as u64);

    progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("Writing CSV: [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
            .unwrap()
            .progress_chars("##-"),
    );

    for row in transposed_data.iter().progress_with(progress_bar) {
        assert_eq!(row.len(), approach_names.len());
        let row: Vec<String> = row.iter().map(|v| v.to_string()).collect();
        writer.write_record(row).unwrap();
    }

    writer.flush().unwrap();
}

pub fn statistical_report<P: Precision>(
    approach_names: &[&str],
    features: Vec<f64>,
    transposed_absolute_errors: Vec<Vec<f64>>,
    transposed_memory_requirements: Vec<Vec<usize>>,
    feature_name: &str,
) {
    assert_eq!(transposed_absolute_errors[0].len(), approach_names.len());
    assert_eq!(
        transposed_memory_requirements[0].len(),
        approach_names.len()
    );

    // We write out to three CSVs the features, absolute errors and memory requirements
    // with as header the approach names.
    let transposed_features = transpose(&vec![features]);
    write_csv::<P, f64>(&transposed_features, feature_name, "features", &["HashSet"]);
    write_csv::<P, f64>(
        &transposed_absolute_errors,
        feature_name,
        "absolute_errors",
        approach_names,
    );
    // write_csv::<P, usize>(&memory_requirements, feature_name, "memory_requirements", approach_names);

    let absolute_errors = transpose(&transposed_absolute_errors);
    let memory_requirements = transpose(&transposed_memory_requirements);

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
        "./statistical_tests_reports/{feature_name}_{}.csv",
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

    let progress_bar = indicatif::ProgressBar::new(approach_names.len() as u64);

    progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("Running tests: [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
            .unwrap()
            .progress_chars("##-"),
    );

    for (
        i,
        ((first_approach_name, first_memsize), (first_absolute_errors, (first_mean, first_std))),
    ) in approach_names
        .iter()
        .zip(memory_requirements.iter())
        .zip(absolute_errors.iter().zip(means.iter().zip(stds.iter())))
        .enumerate()
        .progress_with(progress_bar)
    {
        for (
            j,
            (
                (second_approach_name, second_memsize),
                (second_absolute_errors, (second_mean, second_std)),
            ),
        ) in approach_names
            .iter()
            .zip(memory_requirements.iter())
            .zip(absolute_errors.iter().zip(means.iter().zip(stds.iter())))
            .enumerate()
        {
            if i >= j {
                continue;
            }

            let w_test = WilcoxonWTest::paired(first_absolute_errors, second_absolute_errors);

            writer
                .write_record(&[
                    feature_name,
                    first_approach_name,
                    second_approach_name,
                    w_test
                        .as_ref()
                        .map(|w_test| format!("{:.5}", w_test.p_value()))
                        .unwrap_or("Unknown".to_owned())
                        .as_str(),
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
        }
    }
    // We close the CSV document.
    writer.flush().unwrap();
}
