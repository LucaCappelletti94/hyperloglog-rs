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

fn old_intersection_hll<
    PRECISION: Precision + WordType<BITS>,
    const BITS: usize,
>(
    set1: &Vec<u64>,
    set2: &Vec<u64>,
) -> f32 {
    let hll1: HyperLogLog<PRECISION, BITS> = set1.iter().collect();
    let hll2: HyperLogLog<PRECISION, BITS> = set2.iter().collect();

    hll1.estimate_intersection_cardinality(&hll2)
}

fn intersection_with_set_estimation<
    PRECISION: Precision + WordType<BITS>,
    const BITS: usize,
>(
    set1: &Vec<u64>,
    set2: &Vec<u64>,
) -> f32 {
    let hll1: HyperLogLog<PRECISION, BITS> = set1.iter().collect();
    let hll2: HyperLogLog<PRECISION, BITS> = set2.iter().collect();

    (hll1 & hll2).estimate_cardinality()
}

fn write_line<PRECISION: Precision + WordType<BITS>, const BITS: usize>(
    set1: &Vec<u64>,
    set2: &Vec<u64>,
    set1_str: &str,
    set2_str: &str,
    exact_intersection: usize,
    file: &mut File,
) -> std::io::Result<()> {
    let old_hll = old_intersection_hll::<PRECISION, BITS>(&set1, &set2);
    let intersection_hll = intersection_with_set_estimation::<PRECISION, BITS>(&set1, &set2); 

    let line = format!(
        "{}\t{}\t{}\t{}\t{}\n",
        PRECISION::EXPONENT,
        BITS,
        exact_intersection,
        old_hll,
        intersection_hll
        // set1_str,
        // set2_str
    );

    file.write_all(line.as_bytes())
}

fn write_line_set<
    PRECISION: Precision + WordType<1> + WordType<2> + WordType<3> + WordType<4> + WordType<5> + WordType<6>,
>(
    set1: &Vec<u64>,
    set2: &Vec<u64>,
    set1_str: &str,
    set2_str: &str,
    exact_intersection: usize,
    file: &mut File,
) {
    write_line::<PRECISION, 1>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file)
        .unwrap();
    write_line::<PRECISION, 2>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file)
        .unwrap();
    write_line::<PRECISION, 3>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file)
        .unwrap();
    write_line::<PRECISION, 4>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file)
        .unwrap();
    write_line::<PRECISION, 5>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file)
        .unwrap();
    write_line::<PRECISION, 6>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file)
        .unwrap();
}

fn write_line_set_for_hasher(
    set1: &Vec<u64>,
    set2: &Vec<u64>,
    set1_str: &str,
    set2_str: &str,
    exact_intersection: usize,
    file: &mut File,
) {
    write_line_set::<Precision4>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file);
    write_line_set::<Precision5>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file);
    write_line_set::<Precision6>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file);
    write_line_set::<Precision7>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file);
    write_line_set::<Precision8>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file);
    write_line_set::<Precision9>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file);
    write_line_set::<Precision10>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file);
    write_line_set::<Precision11>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file);
    write_line_set::<Precision12>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file);
    write_line_set::<Precision13>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file);
    write_line_set::<Precision14>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file);
    write_line_set::<Precision15>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file);
    write_line_set::<Precision16>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file);
    write_line_set::<Precision17>(&set1, &set2, &set1_str, &set2_str, exact_intersection, file);
}

#[test]
fn test_intersection_cardinality_perfs() {
    let mut file = File::create("intersection_cardinality_benchmark.tsv").unwrap();
    file.write_all(b"precision\tbits\texact\tunion_exclusion\tintersection_set\n")
        .unwrap();

    // since both the precision and the number of bits are compile time constants, we can
    // not iterate over the precision and bits, but we need to manually change them, making
    // the code a bit verbose:

    // precision 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
    // bits 4, 5, 6

    // For each precision and number of bits, we generate 1000 random sets and write them to the file.
    // We also write the exact intersection similarity and the estimated intersection similarity using HyperLogLog.
    for i in (0..2_000_u64).progress() {
        let seed = (i + 1).wrapping_mul(234567898765);
        let mut rng = splitmix64(seed);

        let mut set1 = HashSet::new();
        let mut set2 = HashSet::new();
        let mut vec1 = Vec::new();
        let mut vec2 = Vec::new();

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
            vec1.push(value);
            rng = splitmix64(rng);
        }

        for _ in 0..second_set_cardinality {
            let value = xorshift(rng) % second_world_size;
            set2.insert(value);
            vec2.push(value);
            rng = splitmix64(rng);
        }

        let exact = intersection(&set1, &set2);

        let set1_str = set1
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let set2_str = set2
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",");

        write_line_set_for_hasher(
            &vec1, &vec2, &set1_str, &set2_str, exact, &mut file,
        );
        // write_line_set_for_hasher::<SipHasher24>(
        //     &set1, &set2, &set1_str, &set2_str, exact, &mut file,
        // );
        // write_line_set_for_hasher::<MetroHasher>(
        //     &set1, &set2, &set1_str, &set2_str, exact, &mut file,
        // );
        // write_line_set_for_hasher::<HighwayHasher>(
        //     &set1, &set2, &set1_str, &set2_str, exact, &mut file,
        // );
    }
}
