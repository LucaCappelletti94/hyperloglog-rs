//! This program evaluates the collision rate of the composite hash for different precisions and bits.
use hyperloglog_rs::prelude::*;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::Hash;
use std::hash::Hasher;
use twox_hash::XxHash64;

#[derive(Serialize, Deserialize)]
/// Struct to store the collision report.
struct CollisionReport {
    /// Average normalized number of elements that are missing in the set of composite hashes.
    mean_collision_rate: f64,
    /// Average number of elements that are missing in the set of composite hashes.
    mean_number_of_collisions: f64,
    /// The number of elements expected to be handled correctly by this particular precision and bits.
    number_of_elements: u64,
    /// The number of bits in the composite hash.
    number_of_bits: u8,
    /// The precision exponent.
    exponent: u8,
    /// The number of bits in the composite hash.
    number_of_bits_composite_hash: u8,
}

/// Function to evaluate the collision rate of the composite hash.
fn collision_rate<H: CompositeHash, P: Precision, B: Bits>() -> Option<CollisionReport>
where
    P: ArrayRegister<B>,
{
    if P::EXPONENT + B::NUMBER_OF_BITS > H::NUMBER_OF_BITS {
        return None;
    }

    let random_state = 768768565446575749_u64;

    // We consider the case with packed registers, which is the most conservative
    // since there are no extra bits that we can use for the composite hash in the hybrid
    // case.
    let number_of_elements: u64 =
        (1_u64 << P::EXPONENT) * B::NUMBER_OF_BITS_U64 / H::NUMBER_OF_BITS_U64;

    let number_of_iterations = 50_000;

    let (total_collision_rate, total_number_of_collisions) = (0..number_of_iterations)
        .into_par_iter()
        .map(|i| {
            let mut hashset = HashSet::new();
            let mut hashset_composite = HashSet::new();
            let mut random_state = splitmix64(random_state.wrapping_mul(i + 1));
            while hashset.len() < number_of_elements as usize {
                random_state = splitmix64(random_state);
                let value = random_state;

                let mut hasher = XxHash64::default();
                value.hash(&mut hasher);
                let hash = hasher.finish();

                // We insert the value in the hashset.
                hashset.insert(value);

                // We extract from
                let (register, index) =
                    PlusPlus::<P, B, <P as ArrayRegister<B>>::Packed, XxHash64>::split_hash(hash);

                let encoded = H::encode::<P, B>(register, index, hash);
                let (decoded_register, decoded_index) = H::decode::<P, B>(encoded);

                assert_eq!(register, decoded_register);
                assert_eq!(index, decoded_index);

                // We insert the composite hash in the hashset.
                hashset_composite.insert(encoded);
            }

            let exact_cardinality = hashset.len();
            debug_assert_eq!(exact_cardinality, number_of_elements as usize);
            let composite_cardinality = hashset_composite.len();
            let delta = exact_cardinality - composite_cardinality;
            let collision_rate = delta as f64 / (exact_cardinality as f64);

            (collision_rate, delta)
        })
        .reduce(|| (0.0, 0), |(a, b), (c, d)| (a + c, b + d));

    let mean_collision_rate = total_collision_rate / number_of_iterations as f64;
    let mean_number_of_collisions = total_number_of_collisions as f64 / number_of_iterations as f64;

    Some(CollisionReport {
        mean_collision_rate,
        mean_number_of_collisions,
        number_of_elements,
        number_of_bits: B::NUMBER_OF_BITS,
        exponent: P::EXPONENT,
        number_of_bits_composite_hash: H::NUMBER_OF_BITS,
    })
}

/// Macro to generate test_composite_hash runs for a given precision and bits.
macro_rules! test_composite_hash {
    ($progress_bar:ident, $reports:ident, $hash:ty, $exponent:expr, $($bits:ty),*) => {
        $(
            paste::paste! {
                if let Some(report) = collision_rate::<$hash, [<Precision $exponent>], $bits>() {
                    $reports.push(report);
                    $progress_bar.inc(1);
                }
            }
        )*
    };
}

/// Macro to generate test_composite_hash runs for a given precision.
macro_rules! test_composite_hash_precisions {
    ($progress_bar:ident, $reports:ident, $hash:ty, $($precision:expr),*) => {
        $(
            test_composite_hash!($progress_bar, $reports, $hash, $precision, Bits3, Bits4, Bits5, Bits6);
        )*
    };
}

/// Macro to generate test_collision_rate runs for a hash.
macro_rules! test_collision_rate {
    ($progress_bar:ident, $reports:ident, $( $hash:ty ),*) => {
        $(
            test_composite_hash_precisions!($progress_bar, $reports, $hash, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);
        )*
    };
}

fn main() {
    let progress_bar = indicatif::ProgressBar::new(157);

    progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("Computing hash collision rates: [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let mut reports: Vec<CollisionReport> = vec![];
    test_collision_rate!(progress_bar, reports, u8, u16, u32, u64);

    // We write the reports to a CSV using csv and serde.

    let file = std::fs::File::create("collision_rates.csv").unwrap();
    let mut writer = csv::Writer::from_writer(file);

    for record in reports {
        writer.serialize(record).unwrap();
    }

    writer.flush().unwrap();
}
