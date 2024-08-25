//! Program to measure the gap between subsequent hashes in the Listhash variant of HyperLogLog,
//! for all 4 to 18 precisions a 4, 5, 6 bit sizes.
#![deny(unsafe_code)]
#![deny(unused_macro_rules)]
#![deny(missing_docs)]

use dsi_bitstream::prelude::{Code, CodesStats};
use hyperloglog_rs::composite_hash::{switch::SwitchHash, CompositeHash};
use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use test_utils::prelude::{append_csv, read_csv, write_csv};
use twox_hash::XxHash64;

type CS = CodesStats<50, 50, 50, 50, 50>;

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
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
    rate: f64,
    /// Mean encoded gap size in bits.
    mean_gap_size: f64,
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

    let iterations = 50_000;
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
                            if let Some(last_hash) = last_hash {
                                assert!(last_hash >= hash);
                                let entry = gap_report
                                    .entry(hash_size)
                                    .or_insert_with(|| (CS::default(), 0));
                                entry.0.update(last_hash - hash);
                                entry.1 += 1;
                            }
                            last_hash = Some(hash);
                        }
                    }
                    if hll.will_dehybridize_upon_new_insert() {
                        // We measure the hash at this point.
                        let hash_size = hll.hash_bytes() * 8;
                        let mut last_hash: Option<u64> = None;
                        for hash in hll.hashes().unwrap() {
                            if let Some(last_hash) = last_hash {
                                assert!(last_hash >= hash);
                                let entry = gap_report
                                    .entry(hash_size)
                                    .or_insert_with(|| (CS::default(), 0));
                                entry.0.update(last_hash - hash);
                                entry.1 += 1;
                            }
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
                let hash_size_report = acc
                    .entry(hash_size)
                    .or_insert_with(|| (CS::default(), 0));
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

            let uncompressed_space_usage = u64::from(*hash_size) * (total + 1);
            let rate = space_usage as f64 / uncompressed_space_usage as f64;
            let mean_gap_size = space_usage as f64 / *total as f64;

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
                mean_gap_size
            }
        }),
        path,
    );
}

/// Proceral macro to generate the optimal_gap_codes function for the provided precision,
/// bit size and hasher types.
macro_rules! generate_optimal_gap_codes {
    ($multiprogress:ident, $precision:ty, $bit_size:ty, $($hasher:ty),*) => {
        // let progress_bar = $multiprogress.add(ProgressBar::new(1 as u64));

        // progress_bar.set_style(
        //     ProgressStyle::default_bar()
        //         .template("[{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        //         .unwrap()
        //         .progress_chars("##-"),
        // );

        // progress_bar.tick();

        $(
            // optimal_gap_codes::<$precision, $bit_size, $hasher, CurrentHash<$precision, $bit_size>>($multiprogress);
            // progress_bar.inc(1);
            optimal_gap_codes::<$precision, $bit_size, $hasher, SwitchHash<$precision, $bit_size>>($multiprogress);
            // progress_bar.inc(1);
        )*

        // progress_bar.finish_and_clear();
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
}
