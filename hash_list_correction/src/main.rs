//! Rust script to identify the optimal correction
#![deny(unsafe_code)]
#![deny(unused_macro_rules)]
#![deny(missing_docs)]
extern crate prettyplease;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

mod utils;

use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::quote;
use syn::File;

use crate::utils::{hash_correction, CorrectionPerformance, HashCorrection};
use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use indicatif::{ProgressBar, ProgressStyle};
use test_utils::prelude::write_report;

/// Procedural macro to generate the correction function for the provided precision,
/// and bit sizes.
macro_rules! generate_gap_for_precision {
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

/// Procedural macro to generate the correction function for the provided precisions.
macro_rules! generate_gap_for_precisions {
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
            generate_gap_for_precision!($reports, $multiprogress, $precision, Bits4, Bits5, Bits6);
            progress_bar.inc(1);
        )*

        progress_bar.finish_and_clear();
    };
}

fn correction() {
    let mut reports: Vec<(HashCorrection, CorrectionPerformance)> = Vec::new();
    let multiprogress = &MultiProgress::new();
    generate_gap_for_precisions!(
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
        Precision15
        // Precision16,
        // Precision17,
        // Precision18
    );
    multiprogress.clear().unwrap();

    let path = "correction.csv";

    write_report(reports.iter().map(|(_, c)| c), path);

    let maximal_precision = reports.iter().map(|(c, _)| c.precision).max().unwrap();

    let mut hash_list_cardinalities: Vec<TokenStream> = Vec::new();
    let mut hash_list_errors: Vec<TokenStream> = Vec::new();
    let mut hyperloglog_cardinalities: Vec<TokenStream> = Vec::new();
    let mut hyperloglog_errors: Vec<TokenStream> = Vec::new();

    (4..=maximal_precision).for_each(|exponent| {
        let mut this_hash_list_cardinalities: Vec<TokenStream> = Vec::new();
        let mut this_hash_list_errors: Vec<TokenStream> = Vec::new();
        let mut this_hyperloglog_cardinalities: Vec<TokenStream> = Vec::new();
        let mut this_hyperloglog_errors: Vec<TokenStream> = Vec::new();

        (4..=6).for_each(|bit_size| {
            let (correction, _) = reports
                .iter()
                .find(|(correction, _)| {
                    correction.precision == exponent && correction.bits == bit_size
                })
                .unwrap();
            let sub_hash_list_cardinalities = correction.hash_list_cardinalities.clone();
            this_hash_list_cardinalities.push(quote! {
                &[#(#sub_hash_list_cardinalities),*]
            });
            let sub_hash_list_errors = correction
                .hash_list_bias
                .clone()
                .into_iter()
                .map(|error| (error * 100.0).round() / 100.0);
            this_hash_list_errors.push(quote! {
                &[#(#sub_hash_list_errors),*]
            });
            let sub_hyperloglog_cardinalities = correction.hyperloglog_cardinalities.clone();
            this_hyperloglog_cardinalities.push(quote! {
                &[#(#sub_hyperloglog_cardinalities),*]
            });
            let sub_hyperloglog_errors = correction
                .hyperloglog_relative_bias
                .clone()
                .into_iter()
                .map(|error| (error * 100.0).round() / 100.0);
            this_hyperloglog_errors.push(quote! {
                &[#(#sub_hyperloglog_errors),*]
            });
        });

        hash_list_cardinalities.push(quote! {
            [#(#this_hash_list_cardinalities),*]
        });
        hash_list_errors.push(quote! {
            [#(#this_hash_list_errors),*]
        });
        hyperloglog_cardinalities.push(quote! {
            [#(#this_hyperloglog_cardinalities),*]
        });
        hyperloglog_errors.push(quote! {
            [#(#this_hyperloglog_errors),*]
        });
    });

    let number_of_precisions = hash_list_cardinalities.len();

    let output = quote! {
        //! Correction coefficients.

        /// The hash_list-correction cardinalities for the gap hash birthday paradox.
        pub(super) const HASHLIST_CORRECTION_CARDINALITIES: [[&[u32]; 3]; #number_of_precisions] = [
            #(#hash_list_cardinalities),*
        ];

        /// The hash_list-correction errors for the gap hash birthday paradox.
        pub(super) const HASHLIST_CORRECTION_BIAS: [[&[f64]; 3]; #number_of_precisions] = [
            #(#hash_list_errors),*
        ];

        /// The hyperloglog-correction cardinalities for the gap hash birthday paradox.
        pub(super) const HYPERLOGLOG_CORRECTION_CARDINALITIES: [[&[u32]; 3]; #number_of_precisions] = [
            #(#hyperloglog_cardinalities),*
        ];

        /// The hyperloglog-correction errors for the gap hash birthday paradox.
        pub(super) const HYPERLOGLOG_CORRECTION_BIAS: [[&[f64]; 3]; #number_of_precisions] = [
            #(#hyperloglog_errors),*
        ];
    };

    // We write out the output token stream to '../src/composite_hash/gap_birthday_paradox.rs'
    let output_path = "../src/correction_coefficients.rs";

    // Convert the generated TokenStream to a string
    let code_string = output.to_string();

    // Parse the generated code string into a syn::Item
    let syntax_tree: File = syn::parse_str(&code_string).unwrap();

    // Use prettyplease to format the syntax tree
    let formatted_code = unparse(&syntax_tree);

    // Write the formatted code to the output file
    std::fs::write(output_path, formatted_code).unwrap();

    println!("Generated correction coefficients in '{}'", output_path);
}

fn main() {
    correction();
}
