/// Example file which writes a reference TSV with two random sets and their exact union similarity,
/// and the estimated union similarity using HyperLogLog. The file can be used to benchmark the
/// accuracy of the HyperLogLog algorithm against other implementations. Of course, we need to run this
/// for multiple precisions and number of bits, which we will log as different rows in the TSV.
///
/// The TSV will have the following columns:
///
/// - `precision`: The precision of the HyperLogLog algorithm.
/// - `bits`: The number of bits used by the HyperLogLog algorithm.
/// - `exact`: The exact union similarity between the two sets.
/// - `hll`: The estimated union similarity using HyperLogLog.
/// - `seed`: The seed used to generate the two sets.
/// - `set1`: The first set, with values separated by commas
/// - `set2`: The second set, with values separated by commas
///
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

use hyperloglog_rs::prelude::*;
use indicatif::ProgressIterator;

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

fn union(set1: &HashSet<u64>, set2: &HashSet<u64>) -> usize {
    set1.union(set2).count()
}

fn recent_union_hll<PRECISION: Precision + WordType<BITS>, const BITS: usize>(
    vec1: &Vec<u64>,
    vec2: &Vec<u64>,
) -> f32 {
    let hll1: HyperLogLog<PRECISION, BITS> = vec1.iter().collect();
    let hll2: HyperLogLog<PRECISION, BITS> = vec2.iter().collect();

    hll1.estimate_union_cardinality(&hll2)
}

fn original_union_hll<PRECISION: Precision + WordType<BITS>, const BITS: usize>(
    vec1: &Vec<u64>,
    vec2: &Vec<u64>,
) -> f32 {
    let hll1: HyperLogLog<PRECISION, BITS> = vec1.iter().collect();
    let hll2: HyperLogLog<PRECISION, BITS> = vec2.iter().collect();

    hll1.estimate_union_and_sets_cardinality_old(&hll2)
        .get_union_cardinality()
}

fn write_line<PRECISION: Precision + WordType<BITS>, const BITS: usize>(
    vec1: &Vec<u64>,
    vec2: &Vec<u64>,
    set1_str: &str,
    set2_str: &str,
    exact_union: usize,
    file: &mut File,
) -> std::io::Result<()> {
    let old_hll = original_union_hll::<PRECISION, BITS>(&vec1, &vec2);
    let recent_hll = recent_union_hll::<PRECISION, BITS>(&vec1, &vec2);

    let line = format!(
        "{}\t{}\t{}\t{}\t{}\n",
        PRECISION::EXPONENT,
        BITS,
        exact_union,
        old_hll,
        recent_hll,
        // set1_str,
        // set2_str
    );

    file.write_all(line.as_bytes())
}

fn write_line_set<
    PRECISION: Precision + WordType<1> + WordType<2> + WordType<3> + WordType<4> + WordType<5> + WordType<6>,
>(
    vec1: &Vec<u64>,
    vec2: &Vec<u64>,
    set1_str: &str,
    set2_str: &str,
    exact_union: usize,
    file: &mut File,
) {
    write_line::<PRECISION, 1>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file).unwrap();
    write_line::<PRECISION, 2>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file).unwrap();
    write_line::<PRECISION, 3>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file).unwrap();
    write_line::<PRECISION, 4>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file).unwrap();
    write_line::<PRECISION, 5>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file).unwrap();
    write_line::<PRECISION, 6>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file).unwrap();
}

fn write_line_set_for_hasher(
    vec1: &Vec<u64>,
    vec2: &Vec<u64>,
    set1_str: &str,
    set2_str: &str,
    exact_union: usize,
    file: &mut File,
) {
    write_line_set::<Precision4>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file);
    write_line_set::<Precision5>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file);
    write_line_set::<Precision6>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file);
    write_line_set::<Precision7>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file);
    write_line_set::<Precision8>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file);
    write_line_set::<Precision9>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file);
    write_line_set::<Precision10>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file);
    write_line_set::<Precision11>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file);
    write_line_set::<Precision12>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file);
    write_line_set::<Precision13>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file);
    write_line_set::<Precision14>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file);
    write_line_set::<Precision15>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file);
    write_line_set::<Precision16>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file);
    write_line_set::<Precision17>(&vec1, &vec2, &set1_str, &set2_str, exact_union, file);
}

//#[test]
fn test_union_cardinality_perfs() {
    let mut file = File::create("union_cardinality_benchmark.tsv").unwrap();
    file.write_all(b"precision\tbits\texact\told\trecent\thash_name\n")
        .unwrap();

    // since both the precision and the number of bits are compile time constants, we can
    // not iterate over the precision and bits, but we need to manually change them, making
    // the code a bit verbose:

    // precision 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
    // bits 4, 5, 6

    // For each precision and number of bits, we generate 1000 random sets and write them to the file.
    // We also write the exact union similarity and the estimated union similarity using HyperLogLog.
    for i in (0..2_000_u64).progress() {
        let seed = (i + 1).wrapping_mul(234567898765);
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

        let mut vec1 = Vec::new();
        let mut vec2 = Vec::new();

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

        let exact = union(&set1, &set2);

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
        //     &vec1, &vec2, &set1_str, &set2_str, exact, &mut file,
        // );
        // write_line_set_for_hasher::<MetroHasher>(
        //     &vec1, &vec2, &set1_str, &set2_str, exact, &mut file,
        // );
        // write_line_set_for_hasher::<HighwayHasher>(
        //     &vec1, &vec2, &set1_str, &set2_str, exact, &mut file,
        // );
    }
}
