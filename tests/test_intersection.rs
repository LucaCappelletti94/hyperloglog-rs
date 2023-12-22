#![cfg(feature="std")]
/// Example file which writes a reference TSV with two random sets and their exact intersection similarity,
/// and the estimated intersection similarity using HyperLogLog. The file can be used to benchmark the
/// accuracy of the HyperLogLog algorithm against other implementations. Of course, we need to run this
/// for multiple precisions and number of bits, which we will log as different rows in the TSV.
///
/// The TSV will have the following columns:
///
/// - `precision`: The precision of the HyperLogLog algorithm.
/// - `bits`: The number of bits used by the HyperLogLog algorithm.
/// - `exact`: The exact intersection similarity between the two sets.
/// - `hll`: The estimated intersection similarity using HyperLogLog.
/// - `seed`: The seed used to generate the two sets.
/// - `set1`: The first set, with values separated by commas
/// - `set2`: The second set, with values separated by commas
///
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
// use indicatif::ParallelProgressIterator;
use indicatif::ProgressIterator;

use hyperloglog_rs::prelude::*;

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

fn xorshift(mut x: u64) -> u64 {
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

fn intersection(set1: &HashSet<u64>, set2: &HashSet<u64>) -> usize {
    set1.intersection(set2).count()
}

fn old_intersection_hll<PRECISION: Precision + WordType<BITS>, const BITS: usize>(
    set1: &HashSet<u64>,
    set2: &HashSet<u64>,
) -> f32 {
    let hll1: HyperLogLog<PRECISION, BITS> = set1.iter().collect();
    let hll2: HyperLogLog<PRECISION, BITS> = set2.iter().collect();

    hll1.estimate_intersection_cardinality(&hll2)
}

fn intersection_with_set_estimation<PRECISION: Precision + WordType<BITS>, const BITS: usize>(
    set1: &HashSet<u64>,
    set2: &HashSet<u64>,
) -> f32 {
    let hll1: HyperLogLog<PRECISION, BITS> = set1.iter().collect();
    let hll2: HyperLogLog<PRECISION, BITS> = set2.iter().collect();

    (hll1 & hll2).estimate_cardinality()
}

fn intersection_with_mle<
    PRECISION: Precision + WordType<BITS>,
    const ERROR: i32,
    const BITS: usize,
>(
    set1: &HashSet<u64>,
    set2: &HashSet<u64>,
) -> (f32, f32) {
    let start = std::time::Instant::now();
    let hll1: HyperLogLog<PRECISION, BITS> = set1.iter().collect();
    let hll2: HyperLogLog<PRECISION, BITS> = set2.iter().collect();
    let hll1_mle: MLE<ERROR, HyperLogLog<PRECISION, BITS>> = hll1.into();
    let hll2_mle: MLE<ERROR, HyperLogLog<PRECISION, BITS>> = hll2.into();

    let estimate = hll1_mle
        .estimate_intersection_cardinality(&hll2_mle);

    (estimate, start.elapsed().as_secs_f32())
}

fn write_line<PRECISION: Precision + WordType<BITS>, const BITS: usize>(
    set1: &HashSet<u64>,
    set2: &HashSet<u64>,
    exact_intersection: usize,
    file: &mut File,
) -> std::io::Result<()> {
    let old_hll_start = std::time::Instant::now();
    let old_hll = old_intersection_hll::<PRECISION, BITS>(&set1, &set2);
    let old_hll_time = old_hll_start.elapsed().as_secs_f32();
    let intersection_hll_start = std::time::Instant::now();
    let intersection_hll = intersection_with_set_estimation::<PRECISION, BITS>(&set1, &set2);
    let intersection_hll_time = intersection_hll_start.elapsed().as_secs_f32();
    let (mle_1, time_1) = intersection_with_mle::<PRECISION, 1, BITS>(&set1, &set2);
    let (mle_2, time_2) = intersection_with_mle::<PRECISION, 2, BITS>(&set1, &set2);
    let (mle_3, time_3) = intersection_with_mle::<PRECISION, 3, BITS>(&set1, &set2);
    let (mle_4, time_4) = intersection_with_mle::<PRECISION, 4, BITS>(&set1, &set2);

    let line = format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
        PRECISION::EXPONENT,
        BITS,
        exact_intersection,
        old_hll,
        intersection_hll,
        mle_1,
        mle_2,
        mle_3,
        mle_4,
        old_hll_time,
        intersection_hll_time,
        time_1,
        time_2,
        time_3,
        time_4,
    );

    file.write_all(line.as_bytes())
}

fn write_line_set<
    PRECISION: Precision + WordType<1> + WordType<2> + WordType<3> + WordType<4> + WordType<5> + WordType<6>,
>(
    set1: &HashSet<u64>,
    set2: &HashSet<u64>,
    exact_intersection: usize,
    file: &mut File,
) {
    // write_line::<PRECISION, 1>(&set1, &set2, exact_intersection, file)
    //     .unwrap();
    // write_line::<PRECISION, 2>(&set1, &set2, exact_intersection, file)
    //     .unwrap();
    // write_line::<PRECISION, 3>(&set1, &set2, exact_intersection, file)
    //     .unwrap();
    write_line::<PRECISION, 4>(&set1, &set2, exact_intersection, file)
        .unwrap();
    write_line::<PRECISION, 5>(&set1, &set2, exact_intersection, file)
        .unwrap();
    write_line::<PRECISION, 6>(&set1, &set2, exact_intersection, file)
        .unwrap();
}

fn write_line_set_for_hasher(
    set1: &HashSet<u64>,
    set2: &HashSet<u64>,
    exact_intersection: usize,
    file: &mut File,
) {
    write_line_set::<Precision4>(&set1, &set2, exact_intersection, file);
    write_line_set::<Precision5>(&set1, &set2, exact_intersection, file);
    write_line_set::<Precision6>(&set1, &set2, exact_intersection, file);
    write_line_set::<Precision7>(&set1, &set2, exact_intersection, file);
    write_line_set::<Precision8>(&set1, &set2, exact_intersection, file);
    write_line_set::<Precision9>(&set1, &set2, exact_intersection, file);
    write_line_set::<Precision10>(&set1, &set2, exact_intersection, file);
    write_line_set::<Precision11>(&set1, &set2, exact_intersection, file);
    write_line_set::<Precision12>(&set1, &set2, exact_intersection, file);
    write_line_set::<Precision13>(&set1, &set2, exact_intersection, file);
    write_line_set::<Precision14>(&set1, &set2, exact_intersection, file);
    write_line_set::<Precision15>(&set1, &set2, exact_intersection, file);
    write_line_set::<Precision16>(&set1, &set2, exact_intersection, file);
}

//#[test]
fn test_intersection_cardinality_perfs() {
    // since both the precision and the number of bits are compile time constants, we can
    // not iterate over the precision and bits, but we need to manually change them, making
    // the code a bit verbose:

    // precision 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
    // bits 4, 5, 6

    // For each precision and number of bits, we generate 1000 random sets and write them to the file.
    // We also write the exact intersection similarity and the estimated intersection similarity using HyperLogLog.

    let number_of_tests = 10_000;
    let loading_bar = indicatif::ProgressBar::new(number_of_tests);

    // We create the intersection_tests directory if it does not exist.
    std::fs::create_dir_all("intersection_tests").unwrap();

    (0..number_of_tests).progress_with(loading_bar).for_each(|i| {
        let path = format!("intersection_tests/intersection_cardinality_benchmark_kernel_fused3_{}.tsv", i);

        // If the file already exists we skip it.
        if std::path::Path::new(&path).exists() {
            return;
        }

        let mut file = File::create(path).unwrap();
        file.write_all(b"precision\tbits\texact\tinclusion_exclusion\thll_min\tmle_1\tmle_2\tmle_3\tmle_4\ttime_inclusion_exclusion\ttime_hll_min\ttime_mle_1\ttime_mle_2\ttime_mle_3\ttime_mle_4\n")
            .unwrap();

        let seed = (i + 1).wrapping_mul(234567898765) as u64;
        let mut rng = splitmix64(seed);

        let mut set1 = HashSet::new();
        let mut set2 = HashSet::new();

        let first_set_cardinality = xorshift(rng) % 100_000;
        rng = splitmix64(rng);
        let second_set_cardinality = xorshift(rng) % 100_000;
        rng = splitmix64(rng);
        let first_world_size = xorshift(rng) % 100_000;
        rng = splitmix64(rng);
        let second_world_size = xorshift(rng) % 100_000;
        rng = splitmix64(rng);

        for _ in 0..first_set_cardinality {
            let value = xorshift(rng) % first_world_size;
            set1.insert(value);
            rng = splitmix64(rng);
        }

        for _ in 0..second_set_cardinality {
            let value = xorshift(rng) % second_world_size;
            set2.insert(value);
            rng = splitmix64(rng);
        }

        let exact = intersection(&set1, &set2);

        write_line_set_for_hasher(&set1, &set2, exact, &mut file);
    });
}
