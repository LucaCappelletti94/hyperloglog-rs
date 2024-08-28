//! Program to measure the gap between subsequent hashes in the Listhash variant of HyperLogLog,
//! for all 4 to 18 precisions a 4, 5, 6 bit sizes.
#![deny(unsafe_code)]
#![deny(unused_macro_rules)]
#![deny(missing_docs)]
extern crate prettyplease;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{File, Ident};

use dsi_bitstream::prelude::{Code, CodesStats};
use hyperloglog_rs::composite_hash::{current::CurrentHash, switch::SwitchHash, CompositeHash};
use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::Serialize;
use serde::{Deserialize, Serializer};
use std::collections::HashMap;
use test_utils::prelude::{append_csv, read_csv, write_csv};
use twox_hash::XxHash64;

fn float_formatter<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{value:.4}"))
}

type CS = CodesStats<50, 50, 50, 50, 50>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
/// Report of the gap between subsequent hashes in the Listhash variant of HyperLogLog.
struct GapReport {
    /// The precision exponent of the HyperLogLog, determining
    /// the number of registers (2^precision).
    precision: u8,
    /// The number of bits used for the registers in the HyperLogLog.
    bit_size: u8,
    /// The number of bits used for the hash in the hash list variant
    /// of the HyperLogLog.
    hash_size: u8,
    /// The hasher used in the HyperLogLog.
    hasher: String,
    /// The composite hash approach used to encode index and registers
    /// in the HyperLogLog.
    composite_hash: String,
    /// The optimal code identified to encode this particular parametrization
    /// of HashList HyperLogLog
    code: String,
    /// The space usage of the optimal code.
    space_usage: u64,
    /// The uncompressed space usage if no code was used.
    uncompressed_space_usage: u64,
    /// The rate of the optimal code.
    #[serde(serialize_with = "float_formatter")]
    rate: f64,
    /// Mean encoded gap size in bits.
    #[serde(serialize_with = "float_formatter")]
    mean_gap_size: f64,
    /// The number of hashes that can fit without the optimal code.
    number_of_hashes: u64,
    /// The number of hashes that can fit with the optimal code.
    number_of_hashes_with_code: u64,
    /// Number of extra hashes that can fit with the optimal code and not
    /// without it.
    extra_hashes: u64,
}

impl GapReport {
    fn as_prefix_free_code_impl(&self) -> TokenStream {
        let precision = Ident::new(
            &format!("Precision{}", self.precision),
            proc_macro2::Span::call_site(),
        );
        let bits = Ident::new(
            &format!("Bits{}", self.bit_size),
            proc_macro2::Span::call_site(),
        );
        let code = Ident::new(
            &self.code.split("(").next().unwrap(),
            proc_macro2::Span::call_site(),
        );
        let code = if self.code.contains("(") {
            let number = self
                .code
                .split("(")
                .last()
                .unwrap()
                .split(")")
                .next()
                .unwrap();
            let number_usize = number.parse::<usize>().unwrap();

            quote! { #code<#number_usize> }
        } else {
            quote! { #code }
        };

        let hash_size = self.hash_size;
        let composite_hash = Ident::new(&self.composite_hash, proc_macro2::Span::call_site());

        let precision_flag = format!("precision_{}", self.precision);

        quote! {
            #[cfg(feature = #precision_flag)]
            impl super::PrefixFreeCode<#hash_size> for crate::composite_hash::#composite_hash<crate::precisions::#precision, crate::bits::#bits> {
                type Code = super::prefix_free_codes::#code;
            }
        }
    }

    fn as_test_only_prefix_free_code_impl(
        precision: u8,
        bit_size: u8,
        hash_size: u8,
        composite_hash: &str,
    ) -> TokenStream {
        let bits = Ident::new(&format!("Bits{}", bit_size), proc_macro2::Span::call_site());

        let hash_size = hash_size;
        let composite_hash = Ident::new(&composite_hash, proc_macro2::Span::call_site());

        let precision_flag = format!("precision_{}", precision);
        let precision = Ident::new(
            &format!("Precision{}", precision),
            proc_macro2::Span::call_site(),
        );

        quote! {
            #[cfg(feature = #precision_flag)]
            #[cfg(test)]
            impl super::PrefixFreeCode<#hash_size> for crate::composite_hash::#composite_hash<crate::precisions::#precision, crate::bits::#bits> {
                type Code = super::prefix_free_codes::NoPrefixCode<#hash_size>;
            }
        }
    }
}

/// Normalized the name of a hasher type.
fn hash_name<H>() -> &'static str {
    core::any::type_name::<H>().split("::").last().unwrap()
}

/// Normalized the name of a composite hash type.
fn composite_hash_name<CH>() -> &'static str {
    core::any::type_name::<CH>()
        .split("<")
        .next()
        .unwrap()
        .split("::")
        .last()
        .unwrap()
}

/// Measures the gap between subsequent hashes in the Listhash variant of HyperLogLog.
fn optimal_gap_codes<
    P: Precision,
    B: Bits,
    H: HasherType,
    CH: Clone + CompositeHash<Precision = P, Bits = B>,
>(
    multiprogress: &MultiProgress,
) where
    P: ArrayRegister<B>,
{
    // We check that this particular combination was not already measured.
    if let Ok(reports) = read_csv::<GapReport>("optimal-gap-codes.csv") {
        if reports.iter().any(|report| {
            report.precision == P::EXPONENT
                && report.bit_size == B::NUMBER_OF_BITS
                && report.hasher == hash_name::<H>()
                && report.composite_hash == composite_hash_name::<CH>()
        }) {
            return;
        }
    }

    let iterations = 20_000;
    let hll = Hybrid::<PlusPlus<P, B, <P as ArrayRegister<B>>::Packed, H>, CH>::default();

    let progress_bar = multiprogress.add(ProgressBar::new(iterations as u64));
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let random_state = 6539823745562884_u64;

    let gaps: HashMap<u8, (CS, u64)> = ParallelIterator::reduce(
        (0..iterations)
            .into_par_iter()
            .progress_with(progress_bar)
            .map(|i| {
                let random_state = splitmix64(random_state.wrapping_mul(i + 1));
                let mut hll = hll.clone();
                let mut gap_report: HashMap<u8, (CS, u64)> = HashMap::new();

                for value in iter_random_values::<u64>(1_000_000, None, Some(random_state)) {
                    hll.insert(&value);

                    if hll.will_downgrade_upon_new_insert() {
                        // We measure the hash at this point, which are sorted in
                        // descending order.
                        let hash_size = hll.hash_bytes() * 8;
                        let mut last_hash: Option<u64> = None;
                        for hash in hll.hashes().unwrap() {
                            let entry = gap_report
                                .entry(hash_size)
                                .or_insert_with(|| (CS::default(), 0));
                            entry.1 += 1;
                            if last_hash.is_none() {
                                last_hash = Some(hash);
                                continue;
                            }
                            let gap = last_hash.map(|last_hash| last_hash - hash - 1).unwrap();
                            entry.0.update(gap);
                            last_hash = Some(hash);
                        }
                    }
                    if hll.will_dehybridize_upon_new_insert() {
                        // We measure the hash at this point.
                        let hash_size = hll.hash_bytes() * 8;
                        let mut last_hash: Option<u64> = None;
                        for hash in hll.hashes().unwrap() {
                            let entry = gap_report
                                .entry(hash_size)
                                .or_insert_with(|| (CS::default(), 0));
                            entry.1 += 1;
                            if last_hash.is_none() {
                                last_hash = Some(hash);
                                continue;
                            }
                            let gap = last_hash.map(|last_hash| last_hash - hash - 1).unwrap();
                            entry.0.update(gap);
                            last_hash = Some(hash);
                        }
                        break;
                    }
                }

                gap_report
            }),
        || HashMap::new(),
        |mut acc, report| {
            for (hash_size, (gap_report, total)) in report {
                let hash_size_report = acc.entry(hash_size).or_insert_with(|| (CS::default(), 0));
                hash_size_report.0 += gap_report;
                hash_size_report.1 += total;
            }
            acc
        },
    );

    let path = "optimal-gap-codes.csv";

    append_csv(
        gaps.iter().map(|(hash_size, (gap_report, total))| {
            let (code, mut space_usage): (Code, u64) = gap_report.best_code();

            space_usage += u64::from(*hash_size) * iterations as u64;
            let uncompressed_space_usage = u64::from(*hash_size) * *total as u64;
            let rate = space_usage as f64 / uncompressed_space_usage as f64;
            let mean_gap_size = space_usage as f64 / *total as f64;
            let number_of_hashes = *total / iterations;
            let number_of_hashes_with_code =
                (uncompressed_space_usage as f64 / mean_gap_size / iterations as f64) as u64;
            let extra_hashes = number_of_hashes_with_code.saturating_sub(number_of_hashes);

            GapReport {
                precision: P::EXPONENT,
                bit_size: B::NUMBER_OF_BITS,
                hash_size: *hash_size,
                hasher: hash_name::<H>().to_string(),
                composite_hash: composite_hash_name::<CH>().to_string(),
                code: code.to_string(),
                space_usage,
                uncompressed_space_usage,
                rate,
                mean_gap_size,
                number_of_hashes,
                number_of_hashes_with_code,
                extra_hashes,
            }
        }),
        path,
    );
}

/// Proceral macro to generate the optimal_gap_codes function for the provided precision,
/// bit size and hasher types.
macro_rules! generate_optimal_gap_codes {
    ($multiprogress:ident, $precision:ty, $bit_size:ty, $($hasher:ty),*) => {
        let progress_bar = $multiprogress.add(ProgressBar::new(2 as u64));

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );

        progress_bar.tick();

        $(
            optimal_gap_codes::<$precision, $bit_size, $hasher, CurrentHash<$precision, $bit_size>>($multiprogress);
            progress_bar.inc(1);
            optimal_gap_codes::<$precision, $bit_size, $hasher, SwitchHash<$precision, $bit_size>>($multiprogress);
            progress_bar.inc(1);
        )*

        progress_bar.finish_and_clear();
    };
}

/// Procedural macro to generate the optimal_gap_codes function for the provided precision,
/// and bit sizes.
macro_rules! generate_optimal_gap_codes_for_precision {
    ($multiprogress:ident, $precision:ty, $($bit_size:ty),*) => {
        let progress_bar = $multiprogress.add(ProgressBar::new(3 as u64));

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );

        progress_bar.tick();

        $(
            generate_optimal_gap_codes!($multiprogress, $precision, $bit_size, XxHash64);
            progress_bar.inc(1);
        )*

        progress_bar.finish_and_clear();
    };
}

/// Procedural macro to generate the optimal_gap_codes function for the provided precisions.
macro_rules! generate_optimal_gap_codes_for_precisions {
    ($multiprogress:ident, $($precision:ty),*) => {
        let progress_bar = $multiprogress.add(ProgressBar::new(18-4));

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );

        progress_bar.tick();

        $(
            generate_optimal_gap_codes_for_precision!($multiprogress, $precision, Bits4, Bits5, Bits6);
            progress_bar.inc(1);
        )*

        progress_bar.finish_and_clear();
    };
}

fn main() {
    let multiprogress = &MultiProgress::new();
    generate_optimal_gap_codes_for_precisions!(
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

    // We reload the report one more time, sort it and re-write it.
    let mut reports = read_csv::<GapReport>("optimal-gap-codes.csv").unwrap();

    reports.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    write_csv(reports.iter(), "optimal-gap-codes.csv");

    // Next, we generate the implementation of the PrefixFreeCode trait for the optimal codes.
    // Of all reports, we keep only the first one we encounter for each combination of precision,
    // bit size, hash size and composite hash.
    let reports = reports
        .into_iter()
        .filter(|report| {
            // If the report shows that the optimal code achieves less than 1 extra hash, we do not
            // generate the implementation.
            report.extra_hashes > 0
        })
        .fold(HashMap::new(), |mut acc, report| {
            let key = (
                report.precision,
                report.bit_size,
                report.hash_size,
                report.composite_hash.clone(),
            );
            acc.entry(key).or_insert(report);
            acc
        });

    let valid_impls = reports
        .iter()
        .map(|(_, report)| report.as_prefix_free_code_impl());

    let test_impls = (4..=18)
        .flat_map(|precision| [4, 5, 6].map(|bits| (precision, bits)))
        .flat_map(|(precision, bits)| [8, 16, 24, 32].map(|hash_size| (precision, bits, hash_size)))
        .flat_map(|(precision, bits, hash_size)| {
            ["CurrentHash", "SwitchHash"]
                .map(move |composite_hash| (precision, bits, hash_size, composite_hash))
        })
        .filter(|(precision, bits, hash_size, composite_hash)| {
            !reports.contains_key(&(*precision, *bits, *hash_size, composite_hash.to_string()))
        })
        .map(|(precision, bits, hash_size, composite_hash)| {
            GapReport::as_test_only_prefix_free_code_impl(
                precision,
                bits,
                hash_size,
                composite_hash,
            )
        });

    let output = quote! {
        #(#valid_impls)*

        #(#test_impls)*
    };

    // We write out the output token stream to '../src/composite_hash/gaps/optimal_codes.rs'.
    let output_path = "../src/composite_hash/gaps/optimal_codes.rs";

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
