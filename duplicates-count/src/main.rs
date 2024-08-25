//! Program to measure the number of subsequent identical hashes that may appear in the Listhash variant of HyperLogLog
//! after a downgrading event occurs, i.e. when the hashes are reduced to a smaller number of bits.
#![deny(unsafe_code)]
#![deny(unused_macro_rules)]
#![deny(missing_docs)]

use ahash::AHasher;
use hyperloglog_rs::composite_hash::{current::CurrentHash, switch::SwitchHash, CompositeHash};
use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use test_utils::prelude::{append_csv, read_csv, write_csv};
use twox_hash::XxHash64;
use wyhash::WyHash;

#[derive(Debug, Serialize, Deserialize)]
/// Report of the gap between subsequent hashes in the Listhash variant of HyperLogLog.
struct DuplicatesReport {
    /// The precision exponent of the HyperLogLog, determining
    /// the number of registers (2^precision).
    precision: u8,
    /// The number of bits used for the registers in the HyperLogLog.
    bit_size: u8,
    /// The hash size BEFORE the downgrading event.
    starting_hash_size: u8,
    /// The hash size AFTER the downgrading event.
    downgraded_hash_size: u8,
    /// The hasher used in the HyperLogLog.
    hasher: String,
    /// The composite hash approach used to encode index and registers
    /// in the HyperLogLog.
    composite_hash: String,
    /// The average number of duplicates observed.
    average_number_of_duplicates: f64,
    /// The absolute number of duplicates observed.
    absolute_average_number_of_duplicates: f64,
    /// The total number of hashes.
    total_number_of_hashes: u64,
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
fn count_duplicates<
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
    if let Ok(reports) = read_csv::<DuplicatesReport>("duplicates.csv") {
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

    let duplicates_report: HashMap<(u8, u8), (f64, u64, u64)> = ParallelIterator::reduce(
        (0..iterations)
            .into_par_iter()
            .progress_with(progress_bar)
            .map(|i| {
                let random_state = splitmix64(random_state.wrapping_mul(i + 1));
                let mut hll = hll.clone();
                let mut duplicates_report: HashMap<(u8, u8), (f64, u64, u64)> = HashMap::new();

                for value in iter_random_values::<u64>(1_000_000, None, Some(random_state)) {
                    let starting_hash_size: Option<u8> = hll.will_downgrade_upon_new_insert().then(|| {
                        hll.hash_bytes() * 8
                    }); 
                    hll.insert(&value);

                    if let Some(starting_hash_size) = starting_hash_size {
                        let downgraded_hash_size = hll.hash_bytes() * 8;
                        let mut last_hash: Option<u64> = None;
                        let mut duplicates = 0;
                        let mut total_number_of_hashes = 0;
                        for hash in hll.hashes().unwrap() {
                            duplicates += u64::from(last_hash == Some(hash));
                            last_hash = Some(hash);
                            total_number_of_hashes += 1;
                        }

                        let rate = duplicates as f64 / total_number_of_hashes as f64;

                        duplicates_report.insert(
                            (starting_hash_size, downgraded_hash_size),
                            (rate, duplicates, total_number_of_hashes),
                        );
                    }
                    if hll.will_dehybridize_upon_new_insert() {
                        break;
                    }
                }

                duplicates_report
            }),
        || HashMap::new(),
        |mut acc, report| {
            for ((starting_hash_size, downgraded_hash_size), (rate, count, total_hashes)) in report {
                let entry = acc.entry((starting_hash_size, downgraded_hash_size)).or_insert((0.0, 0, total_hashes));
                entry.0 += rate;
                entry.1 += count;
            }
            acc
        },
    );

    let path = "duplicates.csv";

    append_csv(
        duplicates_report.into_iter().map(|((starting_hash_size, downgraded_hash_size), (rate_sum, count_sum, total_hashes))| {
            let duplicates_rate = rate_sum / iterations as f64;
            let duplicates_count = count_sum as f64 / iterations as f64;

            DuplicatesReport {
                precision: P::EXPONENT,
                bit_size: B::NUMBER_OF_BITS,
                starting_hash_size,
                downgraded_hash_size,
                hasher: hash_name::<H>().to_string(),
                composite_hash: composite_hash_name::<CH>().to_string(),
                average_number_of_duplicates: duplicates_rate,
                absolute_average_number_of_duplicates: duplicates_count,
                total_number_of_hashes: total_hashes,
            }
        }),
        path,
    );
}

/// Proceral macro to generate the count_duplicates function for the provided precision,
/// bit size and hasher types.
macro_rules! generate_count_duplicates {
    ($multiprogress:ident, $precision:ty, $bit_size:ty, $($hasher:ty),*) => {
        let progress_bar = $multiprogress.add(ProgressBar::new(6 as u64));

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );

        progress_bar.tick();

        $(
            count_duplicates::<$precision, $bit_size, $hasher, CurrentHash<$precision, $bit_size>>($multiprogress);
            progress_bar.inc(1);
            count_duplicates::<$precision, $bit_size, $hasher, SwitchHash<$precision, $bit_size>>($multiprogress);
            progress_bar.inc(1);
        )*

        progress_bar.finish_and_clear();
    };
}

/// Procedural macro to generate the count_duplicates function for the provided precision,
/// and bit sizes.
macro_rules! generate_count_duplicates_for_precision {
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
            generate_count_duplicates!($multiprogress, $precision, $bit_size, XxHash64, WyHash, AHasher);
            progress_bar.inc(1);
        )*

        progress_bar.finish_and_clear();
    };
}

/// Procedural macro to generate the count_duplicates function for the provided precisions.
macro_rules! generate_count_duplicates_for_precisions {
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
            generate_count_duplicates_for_precision!($multiprogress, $precision, Bits4, Bits5, Bits6);
            progress_bar.inc(1);
        )*

        progress_bar.finish_and_clear();
    };
}

fn main() {
    let multiprogress = &MultiProgress::new();
    generate_count_duplicates_for_precisions!(
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
    let mut reports = read_csv::<DuplicatesReport>("duplicates.csv").unwrap();

    reports.sort_by(|a, b| {
        a.average_number_of_duplicates
            .partial_cmp(&b.average_number_of_duplicates)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.precision.cmp(&b.precision))
            .then_with(|| a.bit_size.cmp(&b.bit_size))
    });

    write_csv(reports.iter(), "duplicates.csv");
}
