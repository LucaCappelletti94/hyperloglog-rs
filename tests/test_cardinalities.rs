use hyperloglog_rs::prelude::*;
use indicatif::ProgressIterator;
/// Example file which writes a reference TSV with two random sets and their exact cardinality,
/// and the estimated cardinality using HyperLogLog. The file can be used to benchmark the
/// accuracy of the HyperLogLog algorithm against other implementations. Of course, we need to run this
/// for multiple precisions and number of bits, which we will log as different rows in the TSV.
///
/// The TSV will have the following columns:
///
/// - `precision`: The precision of the HyperLogLog algorithm.
/// - `bits`: The number of bits used by the HyperLogLog algorithm.
/// - `exact`: The exact cardinality between the two sets.
/// - `hll`: The estimated cardinality using HyperLogLog.
/// - `seed`: The seed used to generate the two sets.
/// - `set1`: The first set, with values separated by commas
/// - `set2`: The second set, with values separated by commas
///
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

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

fn write_line<PRECISION: Precision + WordType<BITS>, const BITS: usize>(
    vector: &Vec<u64>,
    set_str: &str,
    exact_cardinality: usize,
    file: &mut File,
) -> std::io::Result<()> {
    let hll: HyperLogLog<PRECISION, BITS> = vector.iter().collect();
    let hll2: HyperLogLogWithMulteplicities<PRECISION, BITS> = hll.clone().into();

    let hll_cardinality = hll.estimate_cardinality();

    let mle_cardinality = hll2.estimate_cardinality_mle::<3>();

    let line = format!(
        "{}\t{}\t{}\t{}\t{}\n",
        PRECISION::EXPONENT,
        BITS,
        exact_cardinality,
        hll_cardinality,
        mle_cardinality,
        //set_str,
    );

    file.write_all(line.as_bytes())
}

fn write_line_set<
    PRECISION: Precision + WordType<1> + WordType<2> + WordType<3> + WordType<4> + WordType<5> + WordType<6>,
>(
    vector: &Vec<u64>,
    set_str: &str,
    exact_cardinality: usize,
    file: &mut File,
) {
    // write_line::<PRECISION, 1>(vector, set_str, exact_cardinality, file).unwrap();
    // write_line::<PRECISION, 2>(vector, set_str, exact_cardinality, file).unwrap();
    // write_line::<PRECISION, 3>(vector, set_str, exact_cardinality, file).unwrap();
    // write_line::<PRECISION, 4>(vector, set_str, exact_cardinality, file).unwrap();
    // write_line::<PRECISION, 5>(vector, set_str, exact_cardinality, file).unwrap();
    write_line::<PRECISION, 6>(vector, set_str, exact_cardinality, file).unwrap();
}

fn write_line_set_for_hasher(
    vector: &Vec<u64>,
    set_str: &str,
    exact_cardinality: usize,
    file: &mut File,
) {
    // write_line_set::<Precision4>(vector, set_str, exact_cardinality, file);
    // write_line_set::<Precision5>(vector, set_str, exact_cardinality, file);
    // write_line_set::<Precision6>(vector, set_str, exact_cardinality, file);
    // write_line_set::<Precision7>(vector, set_str, exact_cardinality, file);
    write_line_set::<Precision8>(vector, set_str, exact_cardinality, file);
    // write_line_set::<Precision9>(vector, set_str, exact_cardinality, file);
    // write_line_set::<Precision10>(vector, set_str, exact_cardinality, file);
    // write_line_set::<Precision11>(vector, set_str, exact_cardinality, file);
    // write_line_set::<Precision12>(vector, set_str, exact_cardinality, file);
    // write_line_set::<Precision13>(vector, set_str, exact_cardinality, file);
    // write_line_set::<Precision14>(vector, set_str, exact_cardinality, file);
    // write_line_set::<Precision15>(vector, set_str, exact_cardinality, file);
    // write_line_set::<Precision16>(vector, set_str, exact_cardinality, file);
    // write_line_set::<Precision17>(vector, set_str, exact_cardinality, file);
    // write_line_set::<Precision18>(vector, set_str, exact_cardinality, file);
}

#[test]
fn test_cardinality_perfs() {
    let mut file = File::create("cardinality_benchmark.tsv").unwrap();
    file.write_all(b"precision\tbits\texact\thll\tmle\n")
        .unwrap();

    // since both the precision and the number of bits are compile time constants, we can
    // not iterate over the precision and bits, but we need to manually change them, making
    // the code a bit verbose:

    // precision 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
    // bits 4, 5, 6

    // For each precision and number of bits, we generate 1000 random sets and write them to the file.
    // We also write the exact cardinality and the estimated cardinality using HyperLogLog.
    for i in (0..1_000_u64).progress() {
        let seed = (i + 1).wrapping_mul(234567898765);
        let mut rng = splitmix64(seed);

        let mut set = HashSet::new();

        let cardinality = xorshift(rng) % 100_000;
        rng = splitmix64(rng);
        let maximal_universe_size = xorshift(rng) % 100_000;
        rng = splitmix64(rng);
        let mut vector = Vec::with_capacity(cardinality as usize);

        for _ in 0..cardinality {
            let value = xorshift(rng) % maximal_universe_size;
            set.insert(value);
            vector.push(value);
            rng = splitmix64(rng);
        }

        let exact = set.len();

        let set_str = set
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",");

        write_line_set_for_hasher(&vector, &set_str, exact, &mut file);
    }
}
