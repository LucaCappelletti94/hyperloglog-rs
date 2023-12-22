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
use rayon::prelude::*;
use hyperloglog_rs::prelude::*;
use indicatif::ParallelProgressIterator;
use rayon::prelude::IntoParallelIterator;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::BufWriter;

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

fn union(set1: &HashSet<u32>, set2: &HashSet<u32>) -> usize {
    set1.union(set2).count()
}

fn write_line<P: Precision + WordType<BITS>, const BITS: usize>(
    vec1: &Vec<u32>,
    vec2: &Vec<u32>,
    left_cardinality: usize,
    right_cardinality: usize,
    exact_union: usize,
    writer: &mut impl Write,
) -> std::io::Result<()> {
    let hll1: HyperLogLog<P, BITS> = vec1.iter().collect();
    let hll2: HyperLogLog<P, BITS> = vec2.iter().collect();

    let approximation: EstimatedUnionCardinalities<f32> = hll1.estimate_union_and_sets_cardinality(&hll2);
    // let nn_approximation: EstimatedUnionCardinalities<f32> = hll1.second_order_union_adjustment(&hll2);

    let line = format!(
        "{}\t{}\t{}\t{}\t{}\n",
        P::EXPONENT,
        BITS,
        exact_union,
        approximation.get_union_cardinality(),
        ""
        // nn_approximation.get_union_cardinality(),
    );

    writer.write_all(line.as_bytes())
}

fn write_line_set<
    P: Precision + WordType<1> + WordType<2> + WordType<3> + WordType<4> + WordType<5> + WordType<6>,
>(
    vec1: &Vec<u32>,
    vec2: &Vec<u32>,
    left_cardinality: usize,
    right_cardinality: usize,
    exact_union: usize,
    writer: &mut impl Write,
) {
    // write_line::<P, 1>(&vec1, &vec2,  left_cardinality, right_cardinality, exact_union, writer).unwrap();
    // write_line::<P, 2>(&vec1, &vec2,  left_cardinality, right_cardinality, exact_union, writer).unwrap();
    // write_line::<P, 3>(&vec1, &vec2,  left_cardinality, right_cardinality, exact_union, writer).unwrap();
    // write_line::<P, 4>(&vec1, &vec2,  left_cardinality, right_cardinality, exact_union, writer).unwrap();
    // write_line::<P, 5>(&vec1, &vec2,  left_cardinality, right_cardinality, exact_union, writer).unwrap();
    write_line::<P, 6>(&vec1, &vec2,  left_cardinality, right_cardinality, exact_union, writer).unwrap();
}

fn write_line_set_for_hasher(
    vec1: &Vec<u32>,
    vec2: &Vec<u32>,
    left_cardinality: usize,
    right_cardinality: usize,
    exact_union: usize,
    writer: &mut impl Write,
) {
    // write_line_set::<Precision4>(&vec1, &vec2, left_cardinality, right_cardinality, exact_union, writer);
    // write_line_set::<Precision5>(&vec1, &vec2, left_cardinality, right_cardinality, exact_union, writer);
    // write_line_set::<Precision6>(&vec1, &vec2, left_cardinality, right_cardinality, exact_union, writer);
    // write_line_set::<Precision7>(&vec1, &vec2, left_cardinality, right_cardinality, exact_union, writer);
    write_line_set::<Precision8>(&vec1, &vec2, left_cardinality, right_cardinality, exact_union, writer);
    // write_line_set::<Precision9>(&vec1, &vec2, left_cardinality, right_cardinality, exact_union, writer);
    // write_line_set::<Precision10>(&vec1, &vec2, left_cardinality, right_cardinality, exact_union, writer);
    // write_line_set::<Precision11>(&vec1, &vec2, left_cardinality, right_cardinality, exact_union, writer);
    // write_line_set::<Precision12>(&vec1, &vec2, left_cardinality, right_cardinality, exact_union, writer);
    // write_line_set::<Precision13>(&vec1, &vec2, left_cardinality, right_cardinality, exact_union, writer);
    // write_line_set::<Precision14>(&vec1, &vec2, left_cardinality, right_cardinality, exact_union, writer);
    // write_line_set::<Precision15>(&vec1, &vec2, left_cardinality, right_cardinality, exact_union, writer);
    // write_line_set::<Precision16>(&vec1, &vec2, left_cardinality, right_cardinality, exact_union, writer);
    // write_line_set::<Precision17>(&vec1, &vec2, left_cardinality, right_cardinality, exact_union, writer);
}

// #[test]
fn test_union_cardinality_perfs() {

    let number_of_tests = 100_000;
    let progress_bar = indicatif::ProgressBar::new(number_of_tests);

    (0..number_of_tests).into_par_iter().progress_with(progress_bar).for_each(|i|{
        let path = format!("union_test/union_cardinality_benchmark_{}.tsv.gz", i);

        // If the path already exists, we skip it.
        // if std::path::Path::new(&path).exists() {
        //     return;
        // }

        let file = File::create(path).unwrap();

        let mut writer = BufWriter::with_capacity(
            128 * 1024,
            GzEncoder::new(file, Compression::default()),
        );

        writer.write_all(b"precision\tbits\texact\thll\tnn\n")
            .unwrap();
        
        let seed = (i + 1).wrapping_mul(234567898765);
        let mut rng = splitmix64(seed);

        let mut set1: HashSet<u32> = HashSet::new();
        let mut set2: HashSet<u32> = HashSet::new();

        let first_set_cardinality = xorshift(rng) % 1_000_000;
        rng = splitmix64(rng);
        let second_set_cardinality = xorshift(rng) % 1_000_000;
        rng = splitmix64(rng);
        let first_world_size = xorshift(rng) % 1_000_000;
        rng = splitmix64(rng);
        let second_world_size = xorshift(rng) % 1_000_000;
        rng = splitmix64(rng);

        let mut vec1: Vec<u32> = Vec::new();
        let mut vec2: Vec<u32> = Vec::new();

        for _ in 0..first_set_cardinality {
            let value = xorshift(rng) % first_world_size;
            set1.insert(value as u32);
            vec1.push(value as u32);
            rng = splitmix64(rng);
        }

        for _ in 0..second_set_cardinality {
            let value = xorshift(rng) % second_world_size;
            set2.insert(value as u32);
            vec2.push(value as u32);
            rng = splitmix64(rng);
        }

        let exact = union(&set1, &set2);

        write_line_set_for_hasher(
            &vec1, &vec2, set1.len(), set2.len(), exact, &mut writer,
        );

        writer.flush().unwrap();

    });
}
