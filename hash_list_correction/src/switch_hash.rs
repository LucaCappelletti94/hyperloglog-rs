//! Error correction for switch hash.

use hyperloglog_rs::composite_hash::CompositeHash;
use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{File, Ident};

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

fn float_formatter<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{value:.7}"))
}

fn small_float_formatter<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{value:.2}"))
}

#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone)]
struct SwitchHashCorrection {
    precision: u8,
    bits: u8,
    #[serde(serialize_with = "float_formatter")]
    maximal_mean_relative_error: f64,
    #[serde(serialize_with = "small_float_formatter")]
    peak_estimated_cardinality: f64,
    #[serde(serialize_with = "small_float_formatter")]
    bias: f64,
    #[serde(serialize_with = "small_float_formatter")]
    error_reduction: f64,
}

impl SwitchHashCorrection {
    /// Implements the BirthDayParadoxCorrection trait for the SwitchHash struct at
    /// the associated precision and bit size.
    fn to_birthday_correction(&self) -> TokenStream {
        let precision = self.precision;
        let bits = self.bits;
        let bias = self.bias;
        let peak_estimated_cardinality = self.peak_estimated_cardinality;
        let maximal_mean_relative_error = self.maximal_mean_relative_error;
        let ident_precision: Ident = Ident::new(
            &format!("Precision{}", precision),
            proc_macro2::Span::call_site(),
        );
        let ident_bits: Ident =
            Ident::new(&format!("Bits{}", bits), proc_macro2::Span::call_site());
        let precision_flag = format!("precision_{}", precision);
        let precision_flag = quote! {
            #[cfg(feature = #precision_flag)]
        };

        quote! {
            #precision_flag
            impl crate::composite_hash::BirthDayParadoxCorrection for crate::composite_hash::SwitchHash<crate::precisions::#ident_precision, crate::bits::#ident_bits> {
                const MINIMAL_CARDINALITY_FOR_CORRECTION: f64 = #bias;
                const MAXIMAL_CARDINALITY_FOR_CORRECTION: f64 = #peak_estimated_cardinality;
                const MAXIMAL_ERROR_RATE: f64 = #maximal_mean_relative_error;
            }
        }
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
) -> SwitchHashCorrection
where
    P: ArrayRegister<B>,
    SwitchHash<P, B>: CompositeHash<Precision = P, Bits = B>,
{
    // We read the switch_hash_correction.csv file to check if the correction has already been
    // computed. If so, we return the correction.
    if let Some(correction) = read_csv::<SwitchHashCorrection>("switch_hash_correction.csv")
        .ok()
        .and_then(|corrections: Vec<SwitchHashCorrection>| {
            corrections.into_iter().find(|correction| {
                correction.precision == P::EXPONENT && correction.bits == B::NUMBER_OF_BITS
            })
        })
    {
        return correction;
    }

    let iterations: u32 = if P::EXPONENT < 9 {
        10_000_000
    } else if P::EXPONENT < 16 {
        1_000_000
    } else {
        50_000
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

                    for value in iter_random_values::<u64>(1_000_000, None, Some(random_state)) {
                        hash_set.insert(value);
                        hll.insert(&value);
                        if !hll.is_hash_list() {
                            break;
                        }
                        unique_hash_bits.insert(hll.hash_bits());
                        let cardinality =
                            f64::from(hll.duplicates().unwrap() + hll.number_of_hashes().unwrap());
                        let exact_cardinality = f64::from(u32::try_from(hash_set.len()).unwrap());
                        let error = (exact_cardinality - cardinality).abs() / exact_cardinality;
                        report.push(CardinalityError { cardinality, error });
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

    // We expect that there should be at most two steps.
    assert!(number_of_unique_hash_bits <= 2);
    assert!(number_of_unique_hash_bits > 0);

    // We expect at least one value in the report.
    assert!(!total_report.is_empty());

    // We normalize the results by the number of iterations.
    total_report.par_iter_mut().for_each(|report| {
        report.cardinality /= f64::from(iterations);
        report.error /= f64::from(iterations);
    });

    // We sort the results by the estimated cardinality, which most likely will be the
    // already sorted but it is not guaranteed.
    total_report.sort_by(|a, b| a.cardinality.partial_cmp(&b.cardinality).unwrap());

    // We find the maximal mean relative error and its corresponding estimated cardinality.
    let peak_error: CardinalityError = ParallelIterator::reduce(
        total_report.par_iter().copied(),
        || CardinalityError {
            cardinality: 0.0,
            error: 0.0,
        },
        |mut acc, report| {
            if report.error > acc.error {
                acc = report;
            }
            acc
        },
    );

    // We check that neither the maximal mean relative error nor the peak estimated cardinality
    // are zero.
    assert!(
        peak_error.error > 0.0,
        "The maximal error at precision {} and bits {} seems to be zero.",
        P::EXPONENT,
        B::NUMBER_OF_BITS
    );
    assert!(peak_error.cardinality > 0.0);

    // We compute the total error
    let total_error = total_report.iter().map(|report| report.error).sum::<f64>();

    // We iterate on the reports to find the point with minimal relative error.
    let (total_error_reduced, bias): (f64, f64) = (0..total_report.len())
        .into_par_iter()
        .map(|i| {
            let mut total_error_reduced = 0.0;
            let bias = total_report[i].cardinality;
            if bias >= peak_error.cardinality {
                return (total_error, bias);
            }
            for (j, this_report) in total_report.iter().enumerate() {
                if j < i {
                    total_error_reduced += this_report.error;
                } else {
                    let predicted_error = peak_error.error * (this_report.cardinality - bias)
                        / (peak_error.cardinality - bias);
                    assert!(predicted_error >= 0.0);
                    total_error_reduced += (this_report.error - predicted_error).abs();
                }
            }

            (total_error_reduced, bias)
        })
        .min_by(|(error_a, _), (error_b, _)| error_a.partial_cmp(error_b).unwrap())
        .unwrap();

    // We compute the error reduction
    let error_reduction = total_error / total_error_reduced;

    SwitchHashCorrection {
        precision: P::EXPONENT,
        bits: B::NUMBER_OF_BITS,
        maximal_mean_relative_error: peak_error.error,
        peak_estimated_cardinality: peak_error.cardinality,
        bias,
        error_reduction,
    }
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
    let mut reports: Vec<_> = Vec::new();
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

    write_csv(reports.iter(), "switch_hash_correction.csv");

    let valid_impls: Vec<TokenStream> = reports
        .iter()
        .map(|report| report.to_birthday_correction())
        .collect();

    let output = quote! {
        #(#valid_impls)*
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
