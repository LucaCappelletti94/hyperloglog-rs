//! Generator for the dense register-regime bias-correction coefficients and the linear-counting
//! thresholds written to `../src/correction_coefficients.rs`.
#![deny(unsafe_code)]
#![deny(unused_macro_rules)]
#![deny(missing_docs)]
extern crate prettyplease;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

mod polyfit;
mod utils;

use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::quote;
use syn::File;

use crate::utils::{register_correction, CorrectionPerformance, RegisterCorrection};
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
            let report = register_correction::<$precision, $bit_size>($multiprogress);
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
    let mut reports: Vec<(RegisterCorrection, CorrectionPerformance)> = Vec::new();
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
        Precision15,
        Precision16,
        Precision17,
        Precision18
    );
    multiprogress.clear().unwrap();

    let path = "correction.csv";

    write_report(reports.iter().map(|(_, c)| c), path);

    let maximal_precision = reports.iter().map(|(c, _)| c.precision).max().unwrap();

    // Emit one nested array per statistic, indexed `[precision - 4][bits - 4]`. The bias correction is
    // now a fitted degree-8 polynomial per cell: nine coefficients in the domain-mapped load `u` plus
    // the load domain `[t_lo, t_hi]`, evaluated by `PolyCorrection::corrected` in the library.
    let round = |value: f64, places: i32| {
        let scale = 10f64.powi(places);
        (value * scale).round() / scale
    };

    let mut hyperloglog_coeffs: Vec<TokenStream> = Vec::new();
    let mut hyperloglog_domains: Vec<TokenStream> = Vec::new();
    let mut linear_count_thresholds: Vec<TokenStream> = Vec::new();

    let coeff_tokens = |poly: &polyfit::PolyFit| {
        let cs = poly.coeffs.iter().map(|c| round(*c, 9));
        quote! { [#(#cs),*] }
    };
    let domain_tokens = |poly: &polyfit::PolyFit| {
        let lo = round(poly.t_lo, 9);
        let hi = round(poly.t_hi, 9);
        quote! { [#lo, #hi] }
    };

    (4..=maximal_precision).for_each(|exponent| {
        let mut this_hyperloglog_coeffs: Vec<TokenStream> = Vec::new();
        let mut this_hyperloglog_domains: Vec<TokenStream> = Vec::new();
        let mut this_linear_count_thresholds: Vec<TokenStream> = Vec::new();

        (4..=6).for_each(|bit_size| {
            let (correction, _) = reports
                .iter()
                .find(|(correction, _)| {
                    correction.precision == exponent && correction.bits == bit_size
                })
                .unwrap();
            let sub_linear_count_threshold = correction.linear_count_threshold;
            this_linear_count_thresholds.push(quote! { #sub_linear_count_threshold });
            this_hyperloglog_coeffs.push(coeff_tokens(&correction.hyperloglog));
            this_hyperloglog_domains.push(domain_tokens(&correction.hyperloglog));
        });

        hyperloglog_coeffs.push(quote! { [#(#this_hyperloglog_coeffs),*] });
        hyperloglog_domains.push(quote! { [#(#this_hyperloglog_domains),*] });
        linear_count_thresholds.push(quote! { [#(#this_linear_count_thresholds),*] });
    });

    let number_of_precisions = hyperloglog_coeffs.len();
    let ncoeff = polyfit::NCOEFF;

    let output = quote! {
        //! Register-regime correction coefficients (generated by the `correction_coefficients` crate).
        //!
        //! The hash-list regime no longer needs a table: it is estimated analytically by the
        //! width-aware occupancy inverse in `src/hyperloglog.rs`
        //! (`HyperLogLog::hash_list_cardinality`). Only the dense register regime keeps a fitted
        //! bias correction here.
        //!
        //! Each cell holds a fitted degree-8 polynomial bias correction: nine coefficients in the
        //! domain-mapped load `u in [-1, 1]` (low order first) plus the load domain `[t_lo, t_hi]`
        //! (with `t = raw / 2^P`). Evaluated by `correct_cardinality` in `src/hyperloglog.rs`.
        //! Indexed `[precision - 4][bits - 4]`.

        /// Register-regime polynomial coefficients.
        pub(super) const HYPERLOGLOG_CORRECTION_COEFFS: [[[f64; #ncoeff]; 3]; #number_of_precisions] = [
            #(#hyperloglog_coeffs),*
        ];

        /// Register-regime polynomial load domains `[t_lo, t_hi]`.
        pub(super) const HYPERLOGLOG_CORRECTION_DOMAIN: [[[f64; 2]; 3]; #number_of_precisions] = [
            #(#hyperloglog_domains),*
        ];

        /// The cardinality at or below which a force-dense HyperLogLog counter should report linear
        /// counting instead of the bias-corrected raw estimate, indexed `[precision - 4][bits - 4]`.
        pub(super) const HYPERLOGLOG_LINEAR_COUNT_THRESHOLD: [[u32; 3]; #number_of_precisions] = [
            #(#linear_count_thresholds),*
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
