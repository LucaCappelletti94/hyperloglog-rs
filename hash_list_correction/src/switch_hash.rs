//! Error correction for switch hash.

use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::quote;
use syn::File;

use crate::utils::{hash_correction, CorrectionPerformance, HashCorrection};
use hyperloglog_rs::composite_hash::switch::SwitchHash;
use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use indicatif::{ProgressBar, ProgressStyle};
use test_utils::prelude::write_csv;

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
            let report = hash_correction::<SwitchHash<$precision, $bit_size>>($multiprogress);
            $reports.push(report);
            progress_bar.inc(1);
        )*

        progress_bar.finish_and_clear();
    };
}

/// Procedural macro to generate the switch_hash_correction function for the provided precisions.
macro_rules! generate_switch_hash_correction_for_precisions {
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
            generate_switch_hash_correction_for_precision!($reports, $multiprogress, $precision, Bits4, Bits5, Bits6);
            progress_bar.inc(1);
        )*

        progress_bar.finish_and_clear();
    };
}

pub fn compute_switch_hash_correction() {
    let mut reports: Vec<(HashCorrection, CorrectionPerformance)> = Vec::new();
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
                    let errors = correction.relative_errors.clone();
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

    let output = quote! {
        //! Correction coefficients for the switch hash birthday paradox.

        /// The cardinalities for the switch hash birthday paradox.
        pub(super) const SWITCH_HASH_BIRTHDAY_PARADOX_CARDINALITIES: [[&[u32]; 3]; 15] = [
            #(#cardinalities),*
        ];

        /// The relative errors for the switch hash birthday paradox.
        pub(super) const SWITCH_HASH_BIRTHDAY_PARADOX_ERRORS: [[&[f64]; 3]; 15] = [
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
