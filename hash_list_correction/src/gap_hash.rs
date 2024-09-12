//! Error correction for gap hash.

use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{File, Ident};

use crate::utils::{hash_correction, CorrectionPerformance, HashCorrection};
use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use indicatif::{ProgressBar, ProgressStyle};
use test_utils::prelude::write_csv;

/// Procedural macro to generate the gap_hash_correction function for the provided precision,
/// and bit sizes.
macro_rules! generate_gap_hash_correction_for_precision {
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
            let report = hash_correction::<$precision, $bit_size>($multiprogress);
            $reports.push(report);
            progress_bar.inc(1);
        )*

        progress_bar.finish_and_clear();
    };
}

/// Procedural macro to generate the gap_hash_correction function for the provided precisions.
macro_rules! generate_gap_hash_correction_for_precisions {
    ($reports:ident, $multiprogress:ident, $($precision:ty),*) => {
        let progress_bar = $multiprogress.add(ProgressBar::new(15));

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("Precisions: [{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );

        progress_bar.tick();

        $(
            generate_gap_hash_correction_for_precision!($reports, $multiprogress, $precision, Bits4, Bits5, Bits6);
            progress_bar.inc(1);
        )*

        progress_bar.finish_and_clear();
    };
}

pub fn compute_gap_hash_correction() {
    let mut reports: Vec<(HashCorrection, CorrectionPerformance)> = Vec::new();
    let multiprogress = &MultiProgress::new();
    generate_gap_hash_correction_for_precisions!(
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

    let path = "gap_hash_correction.csv";

    write_csv(reports.iter().map(|(_, c)| c), path);

    let maximal_precision = reports.iter().map(|(c, _)| c.precision).max().unwrap();

    let cardinalities = (4..=maximal_precision)
        .map(|exponent| {
            let bytes = (4..=6)
                .map(|bit_size| {
                    let (correction, _) = reports
                        .iter()
                        .find(|(correction, _)| {
                            correction.precision == exponent && correction.bits == bit_size
                        })
                        .unwrap();
                    let cardinalities = correction.cardinalities.clone();
                    quote! {
                        &[#(#cardinalities),*]
                    }
                })
                .collect::<Vec<TokenStream>>();
            quote! {
                [#(#bytes),*]
            }
        })
        .collect::<Vec<TokenStream>>();

    let errors = (4..=maximal_precision)
        .map(|exponent| {
            let bytes = (4..=6)
                .map(|bit_size| {
                    let (correction, _) = reports
                        .iter()
                        .find(|(correction, _)| {
                            correction.precision == exponent && correction.bits == bit_size
                        })
                        .unwrap();
                    let errors = correction.relative_errors.clone().into_iter().map(|error| {
                        (error*100.0).round() / 100.0
                    });
                    quote! {
                        &[#(#errors),*]
                    }
                })
                .collect::<Vec<TokenStream>>();
            quote! {
                [#(#bytes),*]
            }
        })
        .collect::<Vec<TokenStream>>();

    let paradox_cardinalities: Ident = Ident::new("GAP_HASH_BIRTHDAY_PARADOX_CARDINALITIES", proc_macro2::Span::call_site());

    let paradox_errors: Ident = Ident::new("GAP_HASH_BIRTHDAY_PARADOX_ERRORS", proc_macro2::Span::call_site());

    let output = quote! {
        //! Correction coefficients for the gap hash birthday paradox.

        #[expect(
            clippy::unreadable_literal,
            reason = "The values are used as a lookup table for the cardinalities."
        )]
        /// The cardinalities for the gap hash birthday paradox.
        pub(super) const #paradox_cardinalities: [[&[u32]; 3]; 15] = [
            #(#cardinalities),*
        ];

        #[expect(
            clippy::unreadable_literal,
            reason = "The values are used as a lookup table for the errors."
        )]
        /// The relative errors for the gap hash birthday paradox.
        pub(super) const #paradox_errors: [[&[f64]; 3]; 15] = [
            #(#errors),*
        ];
    };

    // We write out the output token stream to '../src/composite_hash/gap_birthday_paradox.rs'
    let output_path = "../src/composite_hash/gap_birthday_paradox.rs";

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
