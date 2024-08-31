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
use hyperloglog_rs::composite_hash::GapHash;
use hyperloglog_rs::composite_hash::{current::CurrentHash, switch::SwitchHash, CompositeHash};
use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::Serialize;
use serde::{Deserialize, Serializer};
use std::collections::HashMap;
use std::u64;
use test_utils::prelude::{append_csv, read_csv, write_csv};
use twox_hash::XxHash64;

fn float_formatter<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{value:.4}"))
}

type CS = CodesStats<32, 32, 32, 32, 32>;

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
    /// The composite hash approach used to encode index and registers
    /// in the HyperLogLog.
    composite_hash: String,
    /// The optimal code identified to encode this particular parametrization
    /// of HashList HyperLogLog
    code: String,
    /// The rate of the optimal code.
    #[serde(serialize_with = "float_formatter")]
    rate: f64,
    /// Mean encoded gap size in bits.
    #[serde(serialize_with = "float_formatter")]
    mean_compressed_size: f64,
    /// The number of hashes that can fit without the optimal code.
    number_of_hashes: u64,
    /// The number of hashes that can fit with the optimal code.
    number_of_hashes_with_code: u64,
    /// Number of extra hashes that can fit with the optimal code and not
    /// without it.
    extra_hashes: u64,
}

impl GapReport {
    fn code_token_stream(&self) -> TokenStream {
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

        quote! { super::prefix_free_codes::#code }
    }
}

fn as_prefix_free_code_impl(
    gap_report_u8: Option<GapReport>,
    gap_report_u16: Option<GapReport>,
    gap_report_u24: Option<GapReport>,
    gap_report_u32: Option<GapReport>,
) -> TokenStream {
    // We check that at least one of the gap reports is not None.
    if gap_report_u8.is_none()
        && gap_report_u16.is_none()
        && gap_report_u24.is_none()
        && gap_report_u32.is_none()
    {
        panic!("At least one gap report must be provided.");
    }

    // Gap report u8 must have hash size u8.
    if let Some(gap_report) = gap_report_u8.as_ref() {
        if gap_report.hash_size != 8 {
            panic!("Gap report u8 must have hash size 8.");
        }
    }
    // Gap report u16 must have hash size u16.
    if let Some(gap_report) = gap_report_u16.as_ref() {
        if gap_report.hash_size != 16 {
            panic!("Gap report u16 must have hash size 16.");
        }
    }
    // Gap report u24 must have hash size u24.
    if let Some(gap_report) = gap_report_u24.as_ref() {
        if gap_report.hash_size != 24 {
            panic!("Gap report u24 must have hash size 24.");
        }
    }
    // Gap report u32 must have hash size u32.
    if let Some(gap_report) = gap_report_u32.as_ref() {
        if gap_report.hash_size != 32 {
            panic!("Gap report u32 must have hash size 32.");
        }
    }

    // We get the first report that is not None.
    let gap_report: &GapReport = gap_report_u8
        .as_ref()
        .or(gap_report_u16.as_ref())
        .or(gap_report_u24.as_ref())
        .or(gap_report_u32.as_ref())
        .unwrap();

    // We check that all gap reports have the same precision and bit size.
    for maybe_gap_report in [
        &gap_report_u8,
        &gap_report_u16,
        &gap_report_u24,
        &gap_report_u32,
    ] {
        if let Some(maybe_gap_report) = maybe_gap_report {
            if gap_report.precision != maybe_gap_report.precision
                || gap_report.bit_size != maybe_gap_report.bit_size
                || gap_report.composite_hash != maybe_gap_report.composite_hash
            {
                panic!("All gap reports must have the same precision and bit size.");
            }
        }
    }

    let precision = Ident::new(
        &format!("Precision{}", gap_report.precision),
        proc_macro2::Span::call_site(),
    );
    let bits = Ident::new(
        &format!("Bits{}", gap_report.bit_size),
        proc_macro2::Span::call_site(),
    );

    let code_u8 = gap_report_u8
        .as_ref()
        .map(|gap_report| gap_report.code_token_stream())
        .unwrap_or_else(|| quote! { () });
    let code_u16 = gap_report_u16
        .as_ref()
        .map(|gap_report| gap_report.code_token_stream())
        .unwrap_or_else(|| quote! { () });
    let code_u24 = gap_report_u24
        .as_ref()
        .map(|gap_report| gap_report.code_token_stream())
        .unwrap_or_else(|| quote! { () });
    let code_u32 = gap_report_u32
        .as_ref()
        .map(|gap_report| gap_report.code_token_stream())
        .unwrap_or_else(|| quote! { () });

    let composite_hash = Ident::new(&gap_report.composite_hash, proc_macro2::Span::call_site());

    let precision_flag = format!("precision_{}", gap_report.precision);

    quote! {
        #[cfg(feature = #precision_flag)]
        impl super::PrefixFreeCode for crate::composite_hash::#composite_hash<crate::precisions::#precision, crate::bits::#bits> {
            type Code8 = #code_u8;
            type Code16 = #code_u16;
            type Code24 = #code_u24;
            type Code32 = #code_u32;
        }
    }
}

/// Normalized the name of a composite hash type.
fn composite_hash_name<CH>() -> &'static str {
    core::any::type_name::<CH>()
        .split("<")
        .nth(1)
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
                && report.composite_hash == composite_hash_name::<CH>()
        }) {
            return;
        }
    }

    let iterations = 100;
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
                let mut count: HashMap<u32, u32> = HashMap::new();
                let mut total_count = 0;
                let mut hash_bytes = hll.hash_bytes();
                let mut gap_report: HashMap<u8, (CS, u64)> = HashMap::new();

                for value in iter_random_values::<u64>(1_000_000, None, Some(random_state)) {
                    if !hll.insert(&value) {
                        continue;
                    }

                    if hash_bytes != hll.hash_bytes() || !hll.is_hash_list() {
                        let mut stats = CS::default();

                        for (gap, count) in count.iter() {
                            stats.update_many(u64::from(*gap), u64::from(*count));
                        }

                        gap_report.insert(hash_bytes * 8, (stats, total_count));
                        count.clear();

                        if !hll.is_hash_list() {
                            break;
                        }

                        hash_bytes = hll.hash_bytes();
                        total_count = 0;
                    }

                    let number_of_hashes = hll.hashes().unwrap().len();
                    let saturation_level = (1 << P::EXPONENT) * usize::from(B::NUMBER_OF_BITS)
                        / usize::from(hll.hash_bytes() * 8);

                    // If we have not yet reached saturation, we do not care about compressing
                    // these gaps as they will not be yet needed nor the gap themselves will be
                    // uniformely distributed.
                    if number_of_hashes < saturation_level {
                        continue;
                    }

                    // We measure the hash at this point, which are sorted in
                    // descending order.
                    let mut last_hash = u64::MAX;
                    for hash in hll.hashes().unwrap() {
                        total_count += 1;
                        if last_hash == u64::MAX {
                            last_hash = hash;
                            continue;
                        }
                        let gap = last_hash - hash - 1;
                        last_hash = hash;
                        *count.entry(gap as u32).or_insert(0) += 1;
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
            let (code, space_usage): (Code, u64) = gap_report.best_code();

            // We always represent the first hash as-is, not as an encoded gap.
            let mean_compressed_size =
                (f64::from(*hash_size) * iterations as f64 + space_usage as f64) / *total as f64;
            let number_of_hashes = (1_u64 << P::EXPONENT)
                * u64::try_from(B::NUMBER_OF_BITS_USIZE).unwrap()
                / u64::from(*hash_size);
            let rate = mean_compressed_size / f64::from(*hash_size);
            let number_of_hashes_with_code = ((1_u64 << P::EXPONENT)
                * u64::try_from(B::NUMBER_OF_BITS_USIZE).unwrap())
                / (mean_compressed_size as u64);
            let extra_hashes = (number_of_hashes_with_code as u64).saturating_sub(number_of_hashes);

            GapReport {
                precision: P::EXPONENT,
                bit_size: B::NUMBER_OF_BITS,
                hash_size: *hash_size,
                composite_hash: composite_hash_name::<CH>().to_string(),
                code: code.to_string(),
                rate,
                mean_compressed_size,
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
            optimal_gap_codes::<$precision, $bit_size, $hasher, GapHash<CurrentHash<$precision, $bit_size>>>($multiprogress);
            progress_bar.inc(1);
            optimal_gap_codes::<$precision, $bit_size, $hasher, GapHash<SwitchHash<$precision, $bit_size>>>($multiprogress);
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
        // Precision4,
        // Precision5,
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
            report.extra_hashes > 1
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

    let mut valid_impls = Vec::new();

    for precision in 4_u8..=18 {
        for bit_size in 4_u8..=6 {
            for composite_hash in ["CurrentHash", "SwitchHash"] {
                let gap_report_u8 =
                    reports.get(&(precision, bit_size, 8, composite_hash.to_string()));
                let gap_report_u16 =
                    reports.get(&(precision, bit_size, 16, composite_hash.to_string()));
                let gap_report_u24 =
                    reports.get(&(precision, bit_size, 24, composite_hash.to_string()));
                let gap_report_u32 =
                    reports.get(&(precision, bit_size, 32, composite_hash.to_string()));

                if gap_report_u8.is_none()
                    && gap_report_u16.is_none()
                    && gap_report_u24.is_none()
                    && gap_report_u32.is_none()
                {
                    continue;
                }

                valid_impls.push(as_prefix_free_code_impl(
                    gap_report_u8.cloned(),
                    gap_report_u16.cloned(),
                    gap_report_u24.cloned(),
                    gap_report_u32.cloned(),
                ));
            }
        }
    }

    let output = quote! {
        #(#valid_impls)*
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
