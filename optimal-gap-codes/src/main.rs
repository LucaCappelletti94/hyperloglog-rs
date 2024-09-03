//! Program to measure the gap between subsequent hashes in the Listhash variant of HyperLogLog,
//! for all 4 to 18 precisions a 4, 5, 6 bit sizes.
#![deny(unsafe_code)]
#![deny(unused_macro_rules)]
#![deny(missing_docs)]
extern crate prettyplease;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use core::ops::{Add, Div, Sub};
use hyperloglog_rs::composite_hash::gaps::{GapFragment, PrefixFreeCode};
use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{File, Ident};

use dsi_bitstream::prelude::*;
use hyperloglog_rs::composite_hash::{switch::SwitchHash, CompositeHash};
use hyperloglog_rs::composite_hash::{BirthDayParadoxCorrection, GapHash};
use hyperloglog_rs::prelude::*;
use indicatif::MultiProgress;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::Serialize;
use serde::{Deserialize, Serializer};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::u64;
use test_utils::prelude::{append_csv, read_csv, write_csv};

fn float_formatter<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{value:.4}"))
}

const MAX_CODE_COEFFICIENT: usize = 30;

#[derive(Debug, Copy, Clone)]
/// Collector of statistics for the different prefix-free codes.
pub struct CodesStats {
    /// The total number of elements observed.
    pub total: u64,
    /// The total space used to store the elements if
    /// they were stored using the unary code.
    pub unary: u64,
    /// The total space used to store the elements if
    /// they were stored using the gamma code.
    pub gamma: u64,
    /// The total space used to store the elements if
    /// they were stored using the delta code.
    pub delta: u64,
    /// The total space used to store the elements if
    /// they were stored using the omega code.
    pub omega: u64,
    /// The total space used to store the elements if
    /// they were stored using the zeta code.
    pub zeta: [u64; MAX_CODE_COEFFICIENT],
    /// The total space used to store the elements if
    /// they were stored using the Golomb code.
    pub golomb: [u64; MAX_CODE_COEFFICIENT],
    /// The total space used to store the elements if
    /// they were stored using the exponential Golomb code.
    pub exp_golomb: [u64; MAX_CODE_COEFFICIENT],
    /// The total space used to store the elements if
    /// they were stored using the Rice code.
    pub rice: [u64; MAX_CODE_COEFFICIENT],
    /// The total space used to store the elements if
    /// they were stored using the Pi code.
    pub pi: [u64; MAX_CODE_COEFFICIENT],
    /// The total space used to store the elements if
    /// they were stored using the Pi web code.
    pub pi_web: [u64; MAX_CODE_COEFFICIENT],
}

impl core::default::Default for CodesStats {
    fn default() -> Self {
        Self {
            total: 0,
            unary: 0,
            gamma: 0,
            delta: 0,
            omega: 0,
            zeta: [0; MAX_CODE_COEFFICIENT],
            golomb: [0; MAX_CODE_COEFFICIENT],
            exp_golomb: [0; MAX_CODE_COEFFICIENT],
            rice: [0; MAX_CODE_COEFFICIENT],
            pi: [0; MAX_CODE_COEFFICIENT],
            pi_web: [0; MAX_CODE_COEFFICIENT],
        }
    }
}

/// Returns the length of the gamma code for the provided integer.
fn ceil<I>(numerator: I, denominator: I) -> I
where
    I: One + Add<Output = I> + Sub<Output = I> + Div<Output = I> + From<u8> + Copy,
{
    (numerator + denominator - I::ONE) / denominator
}

impl CodesStats {
    #[inline]
    /// Inserts a gap into the stats.
    pub fn insert(&mut self, gap: GapFragment, vbyte: bool) {
        // Register contribution to the total.
        let unary_encoded_register = u64::from(gap.geometric) + 1;
        self.total += 1;

        let mut unary_delta = unary_encoded_register + (gap.uniform + 1);
        if vbyte {
            unary_delta = ceil(unary_delta, 8) * 8;
        }
        self.unary += unary_delta;
        let mut gamma_delta = unary_encoded_register + len_gamma(gap.uniform) as u64;
        if vbyte {
            gamma_delta = ceil(gamma_delta, 8) * 8;
        }
        self.gamma += gamma_delta;
        let mut delta_delta = unary_encoded_register + len_delta(gap.uniform) as u64;
        if vbyte {
            delta_delta = ceil(delta_delta, 8) * 8;
        }
        self.delta += delta_delta;
        let mut omega_delta = unary_encoded_register + len_omega(gap.uniform) as u64;
        if vbyte {
            omega_delta = ceil(omega_delta, 8) * 8;
        }
        self.omega += omega_delta;

        for (k, val) in self.zeta.iter_mut().enumerate() {
            let mut zeta_delta =
                unary_encoded_register + (len_zeta(gap.uniform, (k + 1) as _) as u64);
            if vbyte {
                zeta_delta = ceil(zeta_delta, 8) * 8;
            }
            *val += zeta_delta;
        }
        for (b, val) in self.golomb.iter_mut().enumerate() {
            let mut golomb_delta =
                unary_encoded_register + (len_golomb(gap.uniform, (b + 1) as _) as u64);
            if vbyte {
                golomb_delta = ceil(golomb_delta, 8) * 8;
            }
            *val += golomb_delta;
        }
        for (k, val) in self.exp_golomb.iter_mut().enumerate() {
            let mut exp_golomb_delta =
                unary_encoded_register + (len_exp_golomb(gap.uniform, k as _) as u64);
            if vbyte {
                exp_golomb_delta = ceil(exp_golomb_delta, 8) * 8;
            }
            *val += exp_golomb_delta;
        }
        for (log2_b, val) in self.rice.iter_mut().enumerate() {
            let mut rice_delta =
                unary_encoded_register + (len_rice(gap.uniform, log2_b as _) as u64);
            if vbyte {
                rice_delta = ceil(rice_delta, 8) * 8;
            }
            *val += rice_delta;
        }
        // +2 because π0 = gamma and π1 = zeta_2
        for (k, val) in self.pi.iter_mut().enumerate() {
            let mut pi_delta = unary_encoded_register + (len_pi(gap.uniform, (k + 2) as _) as u64);
            if vbyte {
                pi_delta = ceil(pi_delta, 8) * 8;
            }
            *val += pi_delta;
        }
        for (k, val) in self.pi_web.iter_mut().enumerate() {
            let mut pi_web_delta =
                unary_encoded_register + (len_pi_web(gap.uniform, k as _) as u64);
            if vbyte {
                pi_web_delta = ceil(pi_web_delta, 8) * 8;
            }
            *val += pi_web_delta;
        }
    }

    #[inline]
    /// Removes the contribution of a gap from the stats.
    pub fn remove(&mut self, gap: GapFragment, vbyte: bool) {
        // Register contribution to the total.
        let unary_encoded_register = u64::from(gap.geometric) + 1;
        self.total -= 1;

        let mut unary_delta = unary_encoded_register + (gap.uniform + 1);
        if vbyte {
            unary_delta = ceil(unary_delta, 8) * 8;
        }
        self.unary -= unary_delta;
        let mut gamma_delta = unary_encoded_register + len_gamma(gap.uniform) as u64;
        if vbyte {
            gamma_delta = ceil(gamma_delta, 8) * 8;
        }
        self.gamma -= gamma_delta;
        let mut delta_delta = unary_encoded_register + len_delta(gap.uniform) as u64;
        if vbyte {
            delta_delta = ceil(delta_delta, 8) * 8;
        }
        self.delta -= delta_delta;
        let mut omega_delta = unary_encoded_register + len_omega(gap.uniform) as u64;
        if vbyte {
            omega_delta = ceil(omega_delta, 8) * 8;
        }
        self.omega -= omega_delta;

        for (k, val) in self.zeta.iter_mut().enumerate() {
            let mut zeta_delta =
                unary_encoded_register + (len_zeta(gap.uniform, (k + 1) as _) as u64);
            if vbyte {
                zeta_delta = ceil(zeta_delta, 8) * 8;
            }
            *val -= zeta_delta;
        }
        for (b, val) in self.golomb.iter_mut().enumerate() {
            let mut golomb_delta =
                unary_encoded_register + (len_golomb(gap.uniform, (b + 1) as _) as u64);
            if vbyte {
                golomb_delta = ceil(golomb_delta, 8) * 8;
            }
            *val -= golomb_delta;
        }
        for (k, val) in self.exp_golomb.iter_mut().enumerate() {
            let mut exp_golomb_delta =
                unary_encoded_register + (len_exp_golomb(gap.uniform, k as _) as u64);
            if vbyte {
                exp_golomb_delta = ceil(exp_golomb_delta, 8) * 8;
            }
            *val -= exp_golomb_delta;
        }
        for (log2_b, val) in self.rice.iter_mut().enumerate() {
            let mut rice_delta =
                unary_encoded_register + (len_rice(gap.uniform, log2_b as _) as u64);
            if vbyte {
                rice_delta = ceil(rice_delta, 8) * 8;
            }
            *val -= rice_delta;
        }
        // +2 because π0 = gamma and π1 = zeta_2
        for (k, val) in self.pi.iter_mut().enumerate() {
            let mut pi_delta = unary_encoded_register + (len_pi(gap.uniform, (k + 2) as _) as u64);
            if vbyte {
                pi_delta = ceil(pi_delta, 8) * 8;
            }
            *val -= pi_delta;
        }
        for (k, val) in self.pi_web.iter_mut().enumerate() {
            let mut pi_web_delta =
                unary_encoded_register + (len_pi_web(gap.uniform, k as _) as u64);
            if vbyte {
                pi_web_delta = ceil(pi_web_delta, 8) * 8;
            }
            *val -= pi_web_delta;
        }
    }

    /// Combines additively this stats with another one.
    pub fn add(&mut self, rhs: &Self) {
        self.total += rhs.total;
        self.unary += rhs.unary;
        self.gamma += rhs.gamma;
        self.delta += rhs.delta;
        self.omega += rhs.omega;
        for (a, b) in self.zeta.iter_mut().zip(rhs.zeta.iter()) {
            *a += *b;
        }
        for (a, b) in self.golomb.iter_mut().zip(rhs.golomb.iter()) {
            *a += *b;
        }
        for (a, b) in self.exp_golomb.iter_mut().zip(rhs.exp_golomb.iter()) {
            *a += *b;
        }
        for (a, b) in self.rice.iter_mut().zip(rhs.rice.iter()) {
            *a += *b;
        }
        for (a, b) in self.pi.iter_mut().zip(rhs.pi.iter()) {
            *a += *b;
        }
        for (a, b) in self.pi_web.iter_mut().zip(rhs.pi_web.iter()) {
            *a += *b;
        }
    }

    /// Return the best code for the stream and its space usage.
    pub fn best_code(&self) -> (Code, u64) {
        let mut best = self.unary;
        let mut best_code = Code::Unary;

        macro_rules! check {
            ($code:expr, $len:expr) => {
                if $len < best {
                    best = $len;
                    best_code = $code;
                }
            };
        }

        for (log2_b, val) in self.rice.iter().enumerate() {
            check!(
                Code::Rice {
                    log2_b: log2_b as _
                },
                *val
            );
        }

        check!(Code::Gamma, self.gamma);
        check!(Code::Delta, self.delta);
        check!(Code::Omega, self.omega);

        for (b, val) in self.golomb.iter().enumerate() {
            check!(Code::Golomb { b: (b + 1) as _ }, *val);
        }

        for (k, val) in self.exp_golomb.iter().enumerate() {
            check!(Code::ExpGolomb { k: k as _ }, *val);
        }
        
        for (k, val) in self.zeta.iter().enumerate() {
            check!(Code::Zeta { k: (k + 1) as _ }, *val);
        }
        
        
        for (k, val) in self.pi.iter().enumerate() {
            check!(Code::Pi { k: (k + 2) as _ }, *val);
        }
        for (k, val) in self.pi_web.iter().enumerate() {
            check!(Code::PiWeb { k: k as _ }, *val);
        }

        (best_code, best)
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
    /// Whether the optimal code uses variable byte encoding.
    vbyte: bool,
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

// fn as_prefix_free_code_impl(
//     gap_report_u8: Option<GapReport>,
//     gap_report_u16: Option<GapReport>,
//     gap_report_u24: Option<GapReport>,
//     gap_report_u32: Option<GapReport>,
// ) -> TokenStream {
//     // We check that at least one of the gap reports is not None.
//     if gap_report_u8.is_none()
//         && gap_report_u16.is_none()
//         && gap_report_u24.is_none()
//         && gap_report_u32.is_none()
//     {
//         panic!("At least one gap report must be provided.");
//     }

//     // Gap report u8 must have hash size u8.
//     if let Some(gap_report) = gap_report_u8.as_ref() {
//         if gap_report.hash_size != 8 {
//             panic!("Gap report u8 must have hash size 8.");
//         }
//     }
//     // Gap report u16 must have hash size u16.
//     if let Some(gap_report) = gap_report_u16.as_ref() {
//         if gap_report.hash_size != 16 {
//             panic!("Gap report u16 must have hash size 16.");
//         }
//     }
//     // Gap report u24 must have hash size u24.
//     if let Some(gap_report) = gap_report_u24.as_ref() {
//         if gap_report.hash_size != 24 {
//             panic!("Gap report u24 must have hash size 24.");
//         }
//     }
//     // Gap report u32 must have hash size u32.
//     if let Some(gap_report) = gap_report_u32.as_ref() {
//         if gap_report.hash_size != 32 {
//             panic!("Gap report u32 must have hash size 32.");
//         }
//     }

//     // We get the first report that is not None.
//     let gap_report: &GapReport = gap_report_u8
//         .as_ref()
//         .or(gap_report_u16.as_ref())
//         .or(gap_report_u24.as_ref())
//         .or(gap_report_u32.as_ref())
//         .unwrap();

//     // We check that all gap reports have the same precision and bit size.
//     for maybe_gap_report in [
//         &gap_report_u8,
//         &gap_report_u16,
//         &gap_report_u24,
//         &gap_report_u32,
//     ] {
//         if let Some(maybe_gap_report) = maybe_gap_report {
//             if gap_report.precision != maybe_gap_report.precision
//                 || gap_report.bit_size != maybe_gap_report.bit_size
//                 || gap_report.vbyte != maybe_gap_report.vbyte
//             {
//                 panic!("All gap reports must have the same precision and bit size.");
//             }
//         }
//     }

//     let precision = Ident::new(
//         &format!("Precision{}", gap_report.precision),
//         proc_macro2::Span::call_site(),
//     );
//     let bits = Ident::new(
//         &format!("Bits{}", gap_report.bit_size),
//         proc_macro2::Span::call_site(),
//     );

//     let code_u8 = gap_report_u8
//         .as_ref()
//         .map(|gap_report| gap_report.code_token_stream())
//         .unwrap_or_else(|| quote! { () });
//     let code_u16 = gap_report_u16
//         .as_ref()
//         .map(|gap_report| gap_report.code_token_stream())
//         .unwrap_or_else(|| quote! { () });
//     let code_u24 = gap_report_u24
//         .as_ref()
//         .map(|gap_report| gap_report.code_token_stream())
//         .unwrap_or_else(|| quote! { () });
//     let code_u32 = gap_report_u32
//         .as_ref()
//         .map(|gap_report| gap_report.code_token_stream())
//         .unwrap_or_else(|| quote! { () });

//     let composite_hash = Ident::new(&gap_report.composite_hash, proc_macro2::Span::call_site());

//     let precision_flag = format!("precision_{}", gap_report.precision);

//     quote! {
//         #[cfg(feature = #precision_flag)]
//         impl super::PrefixFreeCode for crate::composite_hash::#composite_hash<crate::precisions::#precision, crate::bits::#bits> {
//             type Code8 = #code_u8;
//             type Code16 = #code_u16;
//             type Code24 = #code_u24;
//             type Code32 = #code_u32;
//         }
//     }
// }

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

type H<P, B> = PlusPlus<P, B, <P as ArrayRegister<B>>::Packed, twox_hash::XxHash>;

/// Measures the gap between subsequent hashes in the Listhash variant of HyperLogLog.
fn optimal_gap_codes<P: Precision, B: Bits>(multiprogress: &MultiProgress, vbyte: bool)
where
    P: ArrayRegister<B>,
    SwitchHash<P, B>: BirthDayParadoxCorrection,
{
    // We check that this particular combination was not already measured.
    if let Ok(reports) = read_csv::<GapReport>("optimal-gap-codes.csv") {
        if reports.iter().any(|report| {
            report.precision == P::EXPONENT
                && report.bit_size == B::NUMBER_OF_BITS
                && report.vbyte == vbyte
        }) {
            return;
        }
    }

    let iterations = 800_000 / (1 << (P::EXPONENT - 4));

    let progress_bar = multiprogress.add(ProgressBar::new(iterations as u64));
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("Samples: [{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let random_state = 6_539_823_745_562_884_u64;

    let gaps: HashMap<u8, CodesStats> = ParallelIterator::reduce(
        (0..iterations)
            .into_par_iter()
            .progress_with(progress_bar)
            .map(|i| {
                let random_state = splitmix64(random_state.wrapping_mul(i + 1));
                let mut gap_report: HashMap<u8, CodesStats> = HashMap::new();
                let mut hash_bits = SwitchHash::<P, B>::LARGEST_VIABLE_HASH_BITS;
                let number_of_bits = (1usize << P::EXPONENT) * usize::from(B::NUMBER_OF_BITS);
                let preliminary_saturation_threshold = number_of_bits / usize::from(hash_bits);
                let mut previous_hash: Option<u64>;
                let mut next_hash;

                // We create a vector to store the hashes.
                let mut reference_hashes: Vec<Reverse<u64>> =
                    Vec::with_capacity(number_of_bits / usize::from(hash_bits) * 2);

                for value in iter_random_values::<u64>(1_000_000, None, Some(random_state)) {
                    let (index, register, original_hash) =
                        H::<P, B>::index_and_register_and_hash(&value);
                    let encoded_hash =
                        SwitchHash::<P, B>::encode(index, register, original_hash, hash_bits);

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
                            let gap = GapHash::<SwitchHash<P, B>>::into_gap_fragment(
                                window[0].0,
                                window[1].0,
                                hash_bits,
                            );

                            stats.insert(gap, vbyte);
                        }

                        gap_report.insert(hash_bits, stats);

                        continue;
                    }

                    // We insert the new gap.
                    // First, we insert the gap from previous_hash to encoded_hash.
                    if let Some(previous_hash) = previous_hash {
                        gap_report.get_mut(&hash_bits).unwrap().insert(
                            GapHash::<SwitchHash<P, B>>::into_gap_fragment(
                                previous_hash,
                                encoded_hash,
                                hash_bits,
                            ),
                            vbyte,
                        );
                    }

                    // Then, we insert the gap from encoded_hash to next_hash.
                    if let Some(next_hash) = next_hash {
                        gap_report.get_mut(&hash_bits).unwrap().insert(
                            GapHash::<SwitchHash<P, B>>::into_gap_fragment(
                                encoded_hash,
                                next_hash,
                                hash_bits,
                            ),
                            vbyte,
                        );
                    }

                    // We remove the previous gap, if it exists.
                    if let (Some(previous_hash), Some(next_hash)) = (previous_hash, next_hash) {
                        gap_report.get_mut(&hash_bits).unwrap().remove(
                            GapHash::<SwitchHash<P, B>>::into_gap_fragment(
                                previous_hash,
                                next_hash,
                                hash_bits,
                            ),
                            vbyte,
                        );
                    }

                    // We check whether the code that is currently performing best can still
                    // fit within the available number of bits or we need to downgrade the hash.
                    let (_, space_usage) = gap_report.get(&hash_bits).unwrap().best_code();

                    if space_usage > number_of_bits as u64 {
                        // If we are forced to downgrade the hash, we need to revert the last
                        // insertion into the gap report.

                        // We remove the previous gap, if it exists.
                        if let (Some(previous_hash), Some(next_hash)) = (previous_hash, next_hash) {
                            gap_report.get_mut(&hash_bits).unwrap().insert(
                                GapHash::<SwitchHash<P, B>>::into_gap_fragment(
                                    previous_hash,
                                    next_hash,
                                    hash_bits,
                                ),
                                vbyte,
                            );
                        }

                        if let Some(previous_hash) = previous_hash {
                            gap_report.get_mut(&hash_bits).unwrap().remove(
                                GapHash::<SwitchHash<P, B>>::into_gap_fragment(
                                    previous_hash,
                                    encoded_hash,
                                    hash_bits,
                                ),
                                vbyte,
                            );
                        }

                        if let Some(next_hash) = next_hash {
                            gap_report.get_mut(&hash_bits).unwrap().remove(
                                GapHash::<SwitchHash<P, B>>::into_gap_fragment(
                                    encoded_hash,
                                    next_hash,
                                    hash_bits,
                                ),
                                vbyte,
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
                            hash.0 = SwitchHash::<P, B>::downgrade(hash.0, hash_bits + 1, 1);
                        });

                        // The hashes should remain sorted, if they are not there is a serious bug.
                        assert!(reference_hashes.windows(2).all(|w| w[0] <= w[1]));

                        // The downgrade procedure may introduce duplications, we remove them.
                        reference_hashes.dedup();

                        let mut stats = CodesStats::default();

                        for window in reference_hashes.windows(2) {
                            let gap = GapHash::<SwitchHash<P, B>>::into_gap_fragment(
                                window[0].0,
                                window[1].0,
                                hash_bits,
                            );

                            stats.insert(gap, vbyte);
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

    let path = "optimal-gap-codes.csv";

    append_csv(
        gaps.iter().map(|(hash_size, gap_report)| {
            let (code, space_usage): (Code, u64) = gap_report.best_code();

            let byte_padded_hash_size: u8 = ceil(*hash_size, 8) * 8;

            // We always represent the first hash as-is, not as an encoded gap.
            let mean_compressed_size = (f64::from(*hash_size) * iterations as f64
                + space_usage as f64)
                / gap_report.total as f64;
            let number_of_hashes = (1_u64 << P::EXPONENT) * u64::from(B::NUMBER_OF_BITS)
                / u64::from(byte_padded_hash_size);
            let rate = mean_compressed_size / f64::from(byte_padded_hash_size);
            let number_of_hashes_with_code = ((1_u64 << P::EXPONENT)
                * u64::from(B::NUMBER_OF_BITS))
                / (mean_compressed_size as u64);
            let extra_hashes = (number_of_hashes_with_code as u64).saturating_sub(number_of_hashes);

            GapReport {
                precision: P::EXPONENT,
                bit_size: B::NUMBER_OF_BITS,
                hash_size: *hash_size,
                vbyte,
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

/// Procedural macro to generate the optimal_gap_codes function for the provided precision,
/// and bit sizes.
macro_rules! generate_optimal_gap_codes_for_precision {
    ($multiprogress:ident, $precision:ty, $($bit_size:ty),*) => {
        let progress_bar = $multiprogress.add(ProgressBar::new(6 as u64));

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise} | {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );

        progress_bar.tick();

        $(
            optimal_gap_codes::<$precision, $bit_size>($multiprogress, true);
            progress_bar.inc(1);
            optimal_gap_codes::<$precision, $bit_size>($multiprogress, false);
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
    let mut reports = read_csv::<GapReport>("optimal-gap-codes.csv").unwrap();

    reports.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    write_csv(reports.iter(), "optimal-gap-codes.csv");

    // Next, we generate the implementation of the PrefixFreeCode trait for the optimal codes.
    // Of all reports, we keep only the first one we encounter for each combination of precision,
    // bit size, hash size and composite hash.
    // let reports = reports
    //     .into_iter()
    //     .filter(|report| {
    //         // If the report shows that the optimal code achieves less than 1 extra hash, we do not
    //         // generate the implementation.
    //         report.extra_hashes > 1
    //     })
    //     .fold(HashMap::new(), |mut acc, report| {
    //         let key = (
    //             report.precision,
    //             report.bit_size,
    //             report.hash_size,
    //             report.vbyte.clone(),
    //         );
    //         acc.entry(key).or_insert(report);
    //         acc
    //     });

    // let mut valid_impls = Vec::new();

    // for precision in 4_u8..=18 {
    //     for bit_size in 4_u8..=6 {
    //         for composite_hash in ["CurrentHash", "SwitchHash"] {
    //             let gap_report_u8 =
    //                 reports.get(&(precision, bit_size, 8, composite_hash.to_string()));
    //             let gap_report_u16 =
    //                 reports.get(&(precision, bit_size, 16, composite_hash.to_string()));
    //             let gap_report_u24 =
    //                 reports.get(&(precision, bit_size, 24, composite_hash.to_string()));
    //             let gap_report_u32 =
    //                 reports.get(&(precision, bit_size, 32, composite_hash.to_string()));

    //             if gap_report_u8.is_none()
    //                 && gap_report_u16.is_none()
    //                 && gap_report_u24.is_none()
    //                 && gap_report_u32.is_none()
    //             {
    //                 continue;
    //             }

    //             valid_impls.push(as_prefix_free_code_impl(
    //                 gap_report_u8.cloned(),
    //                 gap_report_u16.cloned(),
    //                 gap_report_u24.cloned(),
    //                 gap_report_u32.cloned(),
    //             ));
    //         }
    //     }
    // }

    // let output = quote! {
    //     #(#valid_impls)*
    // };

    // // We write out the output token stream to '../src/composite_hash/gaps/optimal_codes.rs'.
    // let output_path = "../src/composite_hash/gaps/optimal_codes.rs";

    // // Convert the generated TokenStream to a string
    // let code_string = output.to_string();

    // // Parse the generated code string into a syn::Item
    // let syntax_tree: File = syn::parse_str(&code_string).unwrap();

    // // Use prettyplease to format the syntax tree
    // let formatted_code = unparse(&syntax_tree);

    // // Write the formatted code to the output file
    // std::fs::write(output_path, formatted_code).unwrap();

    // println!("Generated optimal codes in '{}'", output_path);
}
