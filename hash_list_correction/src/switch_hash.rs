//! Error correction for switch hash.

use hyperloglog_rs::composite_hash::CompositeHash;
use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::quote;
use syn::File;

use hyperloglog_rs::composite_hash::switch::SwitchHash;
use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use test_utils::prelude::{read_csv, write_csv};
use twox_hash::XxHash64;

const NUMBER_OF_DISCONTINUITIES: usize = 6;

fn small_float_formatter<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{value:.2}"))
}

#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone)]
struct CorrectionPerformance {
    precision: u8,
    bits: u8,
    #[serde(serialize_with = "small_float_formatter")]
    rate_of_improvement: f64,
}

#[derive(Debug, Default, Copy, Clone)]
struct SwitchHashCorrection {
    precision: u8,
    bits: u8,
    relative_errors: [f64; NUMBER_OF_DISCONTINUITIES],
    cardinalities: [u32; NUMBER_OF_DISCONTINUITIES],
}

impl SwitchHashCorrection {
    /// Returns the estimated cardinality for the provided cardinality.
    fn estimate_error(&self, cardinality_estimate: u32) -> f64 {
        if cardinality_estimate >= self.cardinalities[NUMBER_OF_DISCONTINUITIES - 1] {
            return self.relative_errors[NUMBER_OF_DISCONTINUITIES - 1]
                * f64::from(cardinality_estimate)
                / f64::from(self.cardinalities[NUMBER_OF_DISCONTINUITIES - 1]);
        }

        if cardinality_estimate <= self.cardinalities[0] {
            return self.relative_errors[0] * f64::from(cardinality_estimate)
                / f64::from(self.cardinalities[0]).max(1.0);
        }

        // Otherwise, we find the partition that contains the cardinality estimate.
        let partition = self
            .cardinalities
            .windows(2)
            .enumerate()
            .find(|(_, window)| {
                let (lower, upper) = (window[0], window[1]);
                cardinality_estimate >= lower && cardinality_estimate < upper
            })
            .unwrap()
            .0;

        let (lower, upper) = (
            self.cardinalities[partition],
            self.cardinalities[partition + 1],
        );
        let (lower_error, upper_error) = (
            self.relative_errors[partition],
            self.relative_errors[partition + 1],
        );

        let slope = (upper_error - lower_error) / f64::from(upper - lower);

        lower_error + slope * f64::from(cardinality_estimate - lower)
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone)]
struct CardinalityError {
    cardinality: f64,
    error: f64,
}

/// Measures the gap between subsequent hashes in the Listhash variant of HyperLogLog.
fn switch_hash_correction<P: Precision, B: Bits>(
    multiprogress: &MultiProgress,
) -> (SwitchHashCorrection, CorrectionPerformance)
where
    P: ArrayRegister<B>,
    SwitchHash<P, B>: CompositeHash<Precision = P, Bits = B>,
{
    let iterations: u32 = if P::EXPONENT < 9 {
        50_000_000
    } else if P::EXPONENT < 16 {
        10_000_000
    } else if P::EXPONENT == 16 {
        5_000_000
    } else if P::EXPONENT == 17 {
        1_000_000
    } else {
        100_000
    };

    let progress_bar = multiprogress.add(ProgressBar::new(iterations as u64));
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "Samples: [{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
    );

    let random_state = 6_539_823_745_562_884_u64;

    let output_path = format!(
        "switch_hash_correction_{}_{}.csv",
        P::EXPONENT,
        B::NUMBER_OF_BITS
    );

    let total_report = if let Ok(reports) = read_csv::<CardinalityError>(&output_path) {
        reports
    } else {
        let (number_of_unique_hash_bits, mut total_report): (usize, Vec<CardinalityError>) =
            ParallelIterator::reduce(
                (0..iterations)
                    .into_par_iter()
                    .progress_with(progress_bar)
                    .map(|i| {
                        let random_state = splitmix64(random_state.wrapping_mul(u64::from(i) + 1));
                        let mut hash_set = HashSet::with_capacity(100_000);
                        let mut hll = Hybrid::<
                            PlusPlus<P, B, <P as ArrayRegister<B>>::Packed, XxHash64>,
                            SwitchHash<P, B>,
                        >::default();
                        let mut report: Vec<CardinalityError> = Vec::with_capacity(100_000);
                        let mut unique_hash_bits: HashSet<u8> = HashSet::with_capacity(2);

                        for value in iter_random_values::<u64>(1_000_000, None, Some(random_state))
                        {
                            unique_hash_bits.insert(hll.hash_bits());
                            let cardinality = f64::from(
                                hll.duplicates().unwrap() + hll.number_of_hashes().unwrap(),
                            );
                            let exact_cardinality =
                                f64::from(u32::try_from(hash_set.len()).unwrap());
                            let error = (exact_cardinality - cardinality).abs()
                                / exact_cardinality.max(1.0);
                            report.push(CardinalityError { cardinality, error });

                            hash_set.insert(value);
                            hll.insert(&value);
                            if !hll.is_hash_list() {
                                break;
                            }
                        }

                        (unique_hash_bits.len(), report)
                    }),
                || (0, Vec::new()),
                |mut acc, (unique_hash_bits, report)| {
                    acc.0 = unique_hash_bits;
                    if acc.1.is_empty() {
                        acc.1 = report;
                    } else {
                        for (acc, report) in acc.1.iter_mut().zip(report.iter()) {
                            acc.cardinality += report.cardinality;
                            acc.error += report.error;
                        }
                    }
                    acc
                },
            );

        // We normalize the results by the number of iterations.
        total_report.par_iter_mut().for_each(|report| {
            report.cardinality /= f64::from(iterations);
            report.error /= f64::from(iterations);
        });

        // We sort the results by the estimated cardinality, which most likely will be the
        // already sorted but it is not guaranteed.
        total_report.sort_by(|a, b| a.cardinality.partial_cmp(&b.cardinality).unwrap());

        // We store the mined data-points to a CSV so to avoid recomputing them
        // in the future.
        write_csv(total_report.iter(), &output_path);

        // We expect that there should be at most two steps.
        assert!(number_of_unique_hash_bits <= 2);
        assert!(number_of_unique_hash_bits > 0);

        // We expect at least one value in the report.
        assert!(!total_report.is_empty());

        total_report
    };

    // We compute the total error
    let total_error = total_report.iter().map(|report| report.error).sum::<f64>();

    // We compute the angular coefficients.
    let mut angular_coefficients: Vec<(usize, f64)> = total_report
        .windows(2)
        .map(|window| {
            let (first, second) = (window[0], window[1]);
            (second.error - first.error) / (second.cardinality - first.cardinality)
        })
        .enumerate()
        .collect();

    assert_eq!(angular_coefficients.len(), total_report.len() - 1);

    // We remove the last value from the angular coefficients, as we will be adding it
    // by default to the top k values.
    angular_coefficients.pop();

    assert!(NUMBER_OF_DISCONTINUITIES > 1);
    assert!(NUMBER_OF_DISCONTINUITIES < total_report.len());
    let k = NUMBER_OF_DISCONTINUITIES - 1;

    // We find the top k values with the highest angular coefficients.
    angular_coefficients.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let top_k_angular_coefficients: Vec<(usize, f64)> = angular_coefficients
        .iter()
        .rev()
        .take(k)
        .map(|(index, value)| (*index, *value))
        .collect();

    // We retrieve the top k values from the total report.
    let mut top_k_reports: Vec<&CardinalityError> = top_k_angular_coefficients
        .iter()
        .map(|(index, _)| total_report.get(*index).unwrap())
        .collect();

    // We add the last values to the top k values.
    let last_report = total_report.last().unwrap();

    top_k_reports.push(last_report);

    // We sort the top k values by the estimated cardinality.
    top_k_reports.sort_by(|a, b| a.cardinality.partial_cmp(&b.cardinality).unwrap());

    // We create the correction.
    let correction = SwitchHashCorrection {
        precision: P::EXPONENT,
        bits: B::NUMBER_OF_BITS,
        relative_errors: top_k_reports
            .iter()
            .map(|report| report.error)
            .collect::<Vec<f64>>()
            .try_into()
            .unwrap(),
        cardinalities: top_k_reports
            .iter()
            .map(|report| report.cardinality.round() as u32)
            .collect::<Vec<u32>>()
            .try_into()
            .unwrap(),
    };

    // We estimate how well the correction performs.
    let estimated_error = total_report
        .iter()
        .map(|report| {
            (report.error - correction.estimate_error(report.cardinality.round() as u32)).abs()
        })
        .sum::<f64>();

    // Rate of improvement.
    let rate_of_improvement = total_error / estimated_error;

    let performance = CorrectionPerformance {
        precision: P::EXPONENT,
        bits: B::NUMBER_OF_BITS,
        rate_of_improvement,
    };

    (correction, performance)
}

/// Procedural macro to generate the switch_hash_correction function for the provided precision,
/// and bit sizes.
macro_rules! generate_switch_hash_correction_for_precision {
    ($reports:ident, $multiprogress:ident, $precision:ty, $($bit_size:ty),*) => {
        let progress_bar = $multiprogress.add(ProgressBar::new(3 as u64));

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("Bits: [{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );

        progress_bar.tick();

        $(
            let report = switch_hash_correction::<$precision, $bit_size>($multiprogress);
            $reports.push(report);
            progress_bar.inc(1);
        )*

        progress_bar.finish_and_clear();
    };
}

/// Procedural macro to generate the switch_hash_correction function for the provided precisions.
macro_rules! generate_switch_hash_correction_for_precisions {
    ($reports:ident, $multiprogress:ident, $($precision:ty),*) => {
        let progress_bar = $multiprogress.add(ProgressBar::new(18-4));

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("Precisions: [{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );

        progress_bar.tick();

        $(
            generate_switch_hash_correction_for_precision!($reports, $multiprogress, $precision, Bits4, Bits5, Bits6);
            progress_bar.inc(1);
        )*

        progress_bar.finish_and_clear();
    };
}

pub fn compute_switch_hash_correction() {
    let mut reports: Vec<(SwitchHashCorrection, CorrectionPerformance)> = Vec::new();
    let multiprogress = &MultiProgress::new();
    generate_switch_hash_correction_for_precisions!(
        reports,
        multiprogress,
        Precision4,
        Precision5,
        Precision6,
        Precision7,
        Precision8,
        Precision9,
        Precision10,
        Precision11,
        Precision12,
        Precision13,
        Precision14,
        Precision15,
        Precision16,
        Precision17,
        Precision18
    );
    multiprogress.clear().unwrap();

    write_csv(reports.iter().map(|(_, c)| c), "switch_hash_correction.csv");

    let cardinalities = (4..=18)
        .map(|exponent| {
            let bytes = (4..=6)
                .map(|bit_size| {
                    let (correction, _) = reports
                        .iter()
                        .find(|(correction, _)| {
                            correction.precision == exponent && correction.bits == bit_size
                        })
                        .unwrap();
                    let cardinalities = correction.cardinalities;
                    quote! {
                        [#(#cardinalities),*]
                    }
                })
                .collect::<Vec<TokenStream>>();
            quote! {
                [#(#bytes),*]
            }
        })
        .collect::<Vec<TokenStream>>();

    let errors = (4..=18)
        .map(|exponent| {
            let bytes = (4..=6)
                .map(|bit_size| {
                    let (correction, _) = reports
                        .iter()
                        .find(|(correction, _)| {
                            correction.precision == exponent && correction.bits == bit_size
                        })
                        .unwrap();
                    let errors = correction.relative_errors;
                    quote! {
                        [#(#errors),*]
                    }
                })
                .collect::<Vec<TokenStream>>();
            quote! {
                [#(#bytes),*]
            }
        })
        .collect::<Vec<TokenStream>>();

    let output = quote! {
        //! Correction coefficients for the switch hash birthday paradox.

        /// The cardinalities for the switch hash birthday paradox.
        pub(super) const SWITCH_HASH_BIRTHDAY_PARADOX_CARDINALITIES: [[[u32; 6]; 3]; 15] = [
            #(#cardinalities),*
        ];

        /// The relative errors for the switch hash birthday paradox.
        pub(super) const SWITCH_HASH_BIRTHDAY_PARADOX_ERRORS: [[[f64; 6]; 3]; 15] = [
            #(#errors),*
        ];
    };

    // We write out the output token stream to '../src/composite_hash/switch_birthday_paradox.rs'
    let output_path = "../src/composite_hash/switch_birthday_paradox.rs";

    // Convert the generated TokenStream to a string
    let code_string = output.to_string();

    // Parse the generated code string into a syn::Item
    let syntax_tree: File = syn::parse_str(&code_string).unwrap();

    // Use prettyplease to format the syntax tree
    let formatted_code = unparse(&syntax_tree);

    // Write the formatted code to the output file
    std::fs::write(output_path, formatted_code).unwrap();

    println!("Generated optimal codes in '{}'", output_path);
}
