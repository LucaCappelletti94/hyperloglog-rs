//! Program to measure the gap between subsequent hashes in the Listhash variant of HyperLogLog,
//! for all 4 to 18 precisions a 4, 5, 6 bit sizes.
#![deny(unsafe_code)]
#![deny(unused_macro_rules)]
#![deny(missing_docs)]
extern crate prettyplease;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use hyperloglog_rs::composite_hash::gaps::GapFragment;
use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::quote;
use syn::File;

use dsi_bitstream::prelude::*;
use hyperloglog_rs::composite_hash::GapHash;
use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::Serialize;
use serde::{Deserialize, Serializer};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::{u64, u8};
use test_utils::prelude::{append_csv, read_report, write_report};

fn float_formatter<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{value:.4}"))
}

#[derive(Debug, Copy, Clone, Default)]
/// Collector of statistics for the different prefix-free codes.
pub struct CodesStats {
    /// The total number of elements observed.
    pub total: usize,
    /// The quantity of space needed to store the geometric and
    /// uniform parts of the gap, for each possible rice code,
    /// where the I-th axis represents the I-th rice code for the uniform
    /// part of the gap and the J-th axis represents the J-th rice code
    /// for the geometric part of the gap.
    pub rice: [usize; 20],
}

impl CodesStats {
    #[inline]
    /// Inserts a gap into the stats.
    pub fn insert(&mut self, gap: GapFragment) {
        self.total += 1;

        for (i_log2_b, val) in self.rice.iter_mut().enumerate() {
            *val += len_rice(gap.uniform_delta as u64, i_log2_b)
                + usize::from(gap.geometric_minus_one)
                + 1;
        }
    }

    #[inline]
    /// Removes the contribution of a gap from the stats.
    pub fn remove(&mut self, gap: GapFragment) {
        // Register contribution to the total.
        self.total -= 1;

        for (i_log2_b, val) in self.rice.iter_mut().enumerate() {
            *val -= len_rice(gap.uniform_delta as u64, i_log2_b)
                + usize::from(gap.geometric_minus_one)
                + 1;
        }
    }

    /// Combines additively this stats with another one.
    pub fn add(&mut self, rhs: &Self) {
        self.total += rhs.total;
        for (a, b) in self.rice.iter_mut().zip(rhs.rice.iter()) {
            *a += *b;
        }
    }

    /// Return the best code for the stream and its space usage.
    pub fn best_code(&self) -> (u8, usize) {
        let mut best = usize::MAX;
        let mut best_uniform_code = u8::MAX;

        for i in 0..self.rice.len() {
            let space_usage = self.rice[i];
            if space_usage < best {
                best = space_usage;
                best_uniform_code = u8::try_from(i).unwrap();
            }
        }

        (best_uniform_code, best)
    }
}

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
    /// The rice coefficient of the optimal code, for the uniform part.
    uniform_rice_coefficient: u8,
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

type H<P, B> = HyperLogLog<P, B, <P as PackedRegister<B>>::Array, twox_hash::XxHash>;

/// Measures the gap between subsequent hashes in the Listhash variant of HyperLogLog.
fn optimal_gap_codes<P: Precision, B: Bits>(multiprogress: &MultiProgress)
where
    P: PackedRegister<B>,
{
    // We check that this particular combination was not already measured.
    if let Some(reports) = read_report::<GapReport>("optimal-gap-codes.csv") {
        if reports
            .iter()
            .any(|report| report.precision == P::EXPONENT && report.bit_size == B::NUMBER_OF_BITS)
        {
            return;
        }
    }

    let iterations = 8_000_000 / (1 << (P::EXPONENT - 4));

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

    let number_of_bits =
        ((1usize << P::EXPONENT) * usize::from(B::NUMBER_OF_BITS)).div_ceil(64) * 64;

    let gaps: HashMap<u8, CodesStats> = ParallelIterator::reduce(
        (0..iterations as u64)
            .into_par_iter()
            .progress_with(progress_bar)
            .map(|i| {
                let random_state =
                    splitmix64(splitmix64(random_state.wrapping_mul(splitmix64(i + 1))));
                let mut gap_report: HashMap<u8, CodesStats> = HashMap::new();
                let mut hash_bits = GapHash::<P, B>::LARGEST_VIABLE_HASH_BITS;
                let padded_rank_index_size = GapHash::<P, B>::rank_index_padded_size(hash_bits);
                let preliminary_saturation_threshold =
                    (number_of_bits - padded_rank_index_size as usize) / usize::from(hash_bits);

                let mut previous_hash: Option<u32>;
                let mut next_hash;

                // We create a vector to store the hashes.
                let mut reference_hashes: Vec<Reverse<u32>> =
                    Vec::with_capacity(number_of_bits / usize::from(hash_bits) * 2);

                for value in iter_random_values::<u64>(1_000_000, None, Some(random_state)) {
                    let (index, register, original_hash) =
                        H::<P, B>::index_and_register_and_hash(&value);
                    let encoded_hash =
                        GapHash::<P, B>::encode(index, register, original_hash, hash_bits);

                    // We find the sorted position of the hash and insert it if it is not already present.
                    if let Err(position) = reference_hashes.binary_search(&Reverse(encoded_hash)) {
                        previous_hash = (position > 0).then(|| reference_hashes[position - 1].0);
                        next_hash = reference_hashes.get(position).map(|h| h.0);
                        reference_hashes.insert(position, Reverse(encoded_hash));
                    } else {
                        continue;
                    }

                    // We skip forwatd until we reach the preliminary saturation, at which point the
                    // uniformity of the hash functions should be good enough.
                    if reference_hashes.len() < preliminary_saturation_threshold {
                        continue;
                    }

                    // If we have just reached the preliminary saturation, we populate the gap report.
                    if reference_hashes.len() == preliminary_saturation_threshold {
                        let mut stats = CodesStats::default();

                        for window in reference_hashes.windows(2) {
                            let gap = GapHash::<P, B>::into_gap_fragment(
                                window[0].0,
                                window[1].0,
                                hash_bits,
                            );

                            stats.insert(gap);
                        }

                        gap_report.insert(hash_bits, stats);

                        continue;
                    }

                    // We insert the new gap.
                    // First, we insert the gap from previous_hash to encoded_hash.
                    if let Some(previous_hash) = previous_hash {
                        gap_report.get_mut(&hash_bits).unwrap().insert(
                            GapHash::<P, B>::into_gap_fragment(
                                previous_hash,
                                encoded_hash,
                                hash_bits,
                            ),
                        );
                    }

                    // Then, we insert the gap from encoded_hash to next_hash.
                    if let Some(next_hash) = next_hash {
                        gap_report.get_mut(&hash_bits).unwrap().insert(
                            GapHash::<P, B>::into_gap_fragment(encoded_hash, next_hash, hash_bits),
                        );
                    }

                    // We remove the previous gap, if it exists.
                    if let (Some(previous_hash), Some(next_hash)) = (previous_hash, next_hash) {
                        gap_report.get_mut(&hash_bits).unwrap().remove(
                            GapHash::<P, B>::into_gap_fragment(previous_hash, next_hash, hash_bits),
                        );
                    }

                    // We check whether the code that is currently performing best can still
                    // fit within the available number of bits or we need to downgrade the hash.
                    let (_, space_usage) = gap_report.get(&hash_bits).unwrap().best_code();

                    if space_usage
                        > number_of_bits
                            - hash_bits as usize
                            - GapHash::<P, B>::rank_index_total_size(hash_bits) as usize
                    {
                        // If we are forced to downgrade the hash, we need to revert the last
                        // insertion into the gap report.

                        // We remove the previous gap, if it exists.
                        if let (Some(previous_hash), Some(next_hash)) = (previous_hash, next_hash) {
                            gap_report.get_mut(&hash_bits).unwrap().insert(
                                GapHash::<P, B>::into_gap_fragment(
                                    previous_hash,
                                    next_hash,
                                    hash_bits,
                                ),
                            );
                        }

                        if let Some(previous_hash) = previous_hash {
                            gap_report.get_mut(&hash_bits).unwrap().remove(
                                GapHash::<P, B>::into_gap_fragment(
                                    previous_hash,
                                    encoded_hash,
                                    hash_bits,
                                ),
                            );
                        }

                        if let Some(next_hash) = next_hash {
                            gap_report.get_mut(&hash_bits).unwrap().remove(
                                GapHash::<P, B>::into_gap_fragment(
                                    encoded_hash,
                                    next_hash,
                                    hash_bits,
                                ),
                            );
                        }

                        // If we are already at the smallest hash size, we break.
                        if hash_bits == P::EXPONENT + B::NUMBER_OF_BITS {
                            break;
                        }

                        // We downgrade the hash.
                        hash_bits -= 1;

                        // We downgrade all the hashes to the new hash size.
                        reference_hashes.iter_mut().for_each(|hash| {
                            hash.0 = GapHash::<P, B>::downgrade(hash.0, hash_bits + 1, 1);
                        });

                        // The hashes should remain sorted, if they are not there is a serious bug.
                        assert!(reference_hashes.windows(2).all(|w| w[0] <= w[1]));

                        // The downgrade procedure may introduce duplications, we remove them.
                        reference_hashes.dedup();

                        let mut stats = CodesStats::default();

                        for window in reference_hashes.windows(2) {
                            let gap = GapHash::<P, B>::into_gap_fragment(
                                window[0].0,
                                window[1].0,
                                hash_bits,
                            );

                            stats.insert(gap);
                        }

                        gap_report.insert(hash_bits, stats);
                    }
                }

                gap_report
            }),
        || HashMap::new(),
        |mut acc, report| {
            for (hash_size, gap_report) in report {
                let hash_size_report = acc
                    .entry(hash_size)
                    .or_insert_with(|| CodesStats::default());
                hash_size_report.add(&gap_report);
            }
            acc
        },
    );

    // We collect the gap reports and write them to a CSV file.
    let mut gaps = gaps.into_iter().collect::<Vec<_>>();

    // We sort the gap reports by hash size.
    gaps.sort_by_key(|(hash_size, _)| *hash_size);

    // We determine the largest hash size that can be used.
    let max_hash_size = gaps.iter().map(|(hash_size, _)| *hash_size).max().unwrap();

    let path = "optimal-gap-codes.csv";

    append_csv(
        gaps.iter().map(|(hash_size, gap_report)| {
            let (uniform_rice_coefficient, space_usage): (u8, usize) = gap_report.best_code();

            let byte_padded_hash_size: u8 = max_hash_size.div_ceil(8) * 8;

            // We always represent the first hash as-is, not as an encoded gap. This first hash
            // is not counted as part of the mean compressed size.
            let mean_compressed_size = space_usage as f64 / gap_report.total as f64;
            let number_of_hashes = number_of_bits as u64 / u64::from(byte_padded_hash_size);
            let number_of_hashes_with_code = 1
                + ((number_of_bits as f64
                    - f64::from(*hash_size)
                    - GapHash::<P, B>::rank_index_total_size(*hash_size) as f64)
                    / mean_compressed_size as f64) as u64;
            let rate = number_of_hashes as f64 / number_of_hashes_with_code as f64;
            let extra_hashes = (number_of_hashes_with_code as u64).saturating_sub(number_of_hashes);

            GapReport {
                precision: P::EXPONENT,
                bit_size: B::NUMBER_OF_BITS,
                hash_size: *hash_size,
                uniform_rice_coefficient,
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
            optimal_gap_codes::<$precision, $bit_size>($multiprogress);
            progress_bar.inc(1);
        )*

        progress_bar.finish_and_clear();
    };
}

/// Procedural macro to generate the optimal_gap_codes function for the provided precisions.
macro_rules! generate_optimal_gap_codes_for_precisions {
    ($multiprogress:ident, $($precision:ty),*) => {
        let progress_bar = $multiprogress.add(ProgressBar::new(15));

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("Precisions: [{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
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
    let mut reports = read_report::<GapReport>("optimal-gap-codes.csv").unwrap();

    reports.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    write_report(reports.iter(), "optimal-gap-codes.csv");

    let rice_coefficients: Vec<TokenStream> = (4..=18)
        .map(|exponent| {
            let bytes = (4..=6)
                .map(|byte| {
                    let rice_coefficients = reports
                        .iter()
                        .filter(|report| {
                            report.precision == exponent
                                && report.bit_size == byte
                                && report.extra_hashes > 2
                                && report.rate < 0.8
                        })
                        .map(|report| {
                            let hash_size = report.hash_size;
                            let uniform_rice_coefficient = report.uniform_rice_coefficient;
                            quote! {
                                (#hash_size, #uniform_rice_coefficient)
                            }
                        });
                    quote! {
                        &[#(#rice_coefficients),*]
                    }
                })
                .collect::<Vec<TokenStream>>();
            quote! {
                [
                    #(#bytes),*
                ]
            }
        })
        .collect();

    let output = quote! {
        //! Optimal codes for the gap between subsequent hashes in the `ListHash` variant of [`HyperLogLog`].

        /// The optimal Rice code coefficients for the different precisions and bit sizes, when using hash-packing.
        pub(super) const OPTIMAL_RICE_COEFFICIENTS: [[&[(u8, u8)]; 3]; 15] = [
            #(#rice_coefficients),*
        ];
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
