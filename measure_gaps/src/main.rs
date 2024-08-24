//! Program to measure the gap between subsequent hashes in the Listhash variant of HyperLogLog,
//! for all 4 to 18 precisions a 4, 5, 6 bit sizes.

use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use test_utils::prelude::write_csv;
use twox_hash::XxHash64;
use wyhash::WyHash;
use ahash::AHasher;

#[derive(Serialize, Deserialize)]
struct GapReport {
    /// The largest hash we employ is a u32, therefore the largest
    /// possible gap is 2^32 - 1.
    gap: u32,
    /// The number of times we observed this gap.
    count: u64,
}



fn measure_gaps<P: Precision, B: Bits, H: HasherType>(multiprogress: &MultiProgress)
where
    P: ArrayRegister<B>,
{
    let iterations = 100_000;
    let hll = Hybrid::<PlusPlus<P, B, <P as ArrayRegister<B>>::Packed, H>>::default();

    let progress_bar = multiprogress.add(ProgressBar::new(iterations as u64));
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let random_state = 6539823745562884_u64;

    let gaps: HashMap<u8, HashMap<u32, u64>> = (0..iterations)
        .into_par_iter()
        .progress_with(progress_bar)
        .map(|i| {
            let random_state = splitmix64(random_state.wrapping_mul(i + 1));
            let mut hll = hll.clone();
            let mut gap_report: HashMap<u8, HashMap<u32, u64>> = HashMap::new();

            for value in iter_random_values::<u64>(1_000_000, None, Some(random_state)) {
                hll.insert(&value);

                if hll.will_downgrade_upon_new_insert() {
                    // We measure the hash at this point, which are sorted in
                    // descending order.
                    let hash_size = hll.hash_bytes();
                    let mut last_hash: Option<u64> = None;
                    for hash in hll.hashes().unwrap() {
                        if let Some(last_hash) = last_hash {
                            assert!(last_hash >= hash);
                            let gap = u32::try_from(last_hash - hash).unwrap();
                            let gap_report =
                                gap_report.entry(hash_size).or_insert_with(HashMap::new);
                            let gap_report = gap_report.entry(gap).or_insert(0);
                            *gap_report += 1;
                        }
                        last_hash = Some(hash);
                    }
                }
                if hll.will_dehybridize_upon_new_insert() {
                    // We measure the hash at this point.
                    let hash_size = hll.hash_bytes();
                    let mut last_hash: Option<u64> = None;
                    for hash in hll.hashes().unwrap() {
                        if let Some(last_hash) = last_hash {
                            assert!(last_hash >= hash);
                            let gap = u32::try_from(last_hash - hash).unwrap();
                            let gap_report =
                                gap_report.entry(hash_size).or_insert_with(HashMap::new);
                            let gap_report = gap_report.entry(gap).or_insert(0);
                            *gap_report += 1;
                        }
                        last_hash = Some(hash);
                    }
                    break;
                }
            }

            gap_report
        })
        .reduce(
            || HashMap::new(),
            |mut acc, report| {
                for (hash_size, gap_report) in report {
                    let hash_size_report = acc.entry(hash_size).or_insert_with(HashMap::new);
                    for (gap, count) in gap_report {
                        let gap_report = hash_size_report.entry(gap).or_insert(0);
                        *gap_report += count;
                    }
                }
                acc
            },
        );

    for (hash_size, gap_report) in gaps {
        let mut gap_reports = gap_report
            .into_iter()
            .map(|(gap, count)| GapReport { gap, count })
            .collect::<Vec<_>>();
        gap_reports.sort_by_key(|report| report.gap);
        write_csv(
            gap_reports.iter(),
            &format!(
                "reports/gap_report_precision_{}_bits_{}_hash_{}_{}.csv.gz",
                P::EXPONENT,
                B::NUMBER_OF_BITS,
                hash_size,
                core::any::type_name::<H>()
                    .split("::")
                    .last()
                    .unwrap()
                    .to_lowercase()
            ),
        );
    }
}

/// Proceral macro to generate the measure_gaps function for the provided precision,
/// bit size and hasher types.
macro_rules! generate_measure_gaps {
    ($multiprogress:ident, $precision:ty, $bit_size:ty, $($hasher:ty),*) => {
        let progress_bar = $multiprogress.add(ProgressBar::new(3 as u64));

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );

        progress_bar.tick();

        $(
            measure_gaps::<$precision, $bit_size, $hasher>($multiprogress);
            progress_bar.inc(1);
        )*

        progress_bar.finish();
    };
}

/// Procedural macro to generate the measure_gaps function for the provided precision,
/// and bit sizes.
macro_rules! generate_measure_gaps_for_precision {
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
            generate_measure_gaps!($multiprogress, $precision, $bit_size, WyHash, AHasher);
            progress_bar.inc(1);
        )*

        progress_bar.finish();
    };
}


/// Procedural macro to generate the measure_gaps function for the provided precisions.
macro_rules! generate_measure_gaps_for_precisions {
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
            generate_measure_gaps_for_precision!($multiprogress, $precision, Bits4, Bits5, Bits6);
            progress_bar.inc(1);
        )*

        progress_bar.finish();
    };
}


fn main() {
    let multiprogress = &MultiProgress::new();
    generate_measure_gaps_for_precisions!(
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
}
