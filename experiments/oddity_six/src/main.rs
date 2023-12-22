//! Evaluation of set-like properties across different data structures.
use hyperloglog_rs::prelude::*;
use indicatif::ProgressIterator;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

use mem_dbg::MemSize;
use mem_dbg::SizeFlags;
use std::collections::HashSet;
use std::hash::Hash;

trait SetLike<I> {
    fn struct_name(&self) -> String;
    fn memory_requirements(&self) -> usize;
    fn insert(&mut self, x: I);
    fn precision(&self) -> usize;
    fn bits(&self) -> usize;
    fn cardinality(&self) -> usize;
}

impl<I: Hash + Eq + PartialEq + MemSize> SetLike<I> for HashSet<I>
where
    Self: MemSize,
{
    fn struct_name(&self) -> String {
        "HashSet".to_string()
    }

    fn memory_requirements(&self) -> usize {
        self.mem_size(SizeFlags::CAPACITY)
    }

    fn insert(&mut self, x: I) {
        self.insert(x);
    }

    fn precision(&self) -> usize {
        0
    }

    fn bits(&self) -> usize {
        0
    }

    fn cardinality(&self) -> usize {
        self.len()
    }
}

impl<I: Hash + Eq + PartialEq, PRECISION: Precision + WordType<BITS>, const BITS: usize> SetLike<I>
    for HyperLogLog<PRECISION, BITS>
{
    fn struct_name(&self) -> String {
        "HyperLogLog".to_string()
    }

    fn memory_requirements(&self) -> usize {
        core::mem::size_of::<Self>()
    }

    fn insert(&mut self, x: I) {
        self.insert(x);
    }

    fn precision(&self) -> usize {
        PRECISION::EXPONENT
    }

    fn bits(&self) -> usize {
        BITS
    }

    fn cardinality(&self) -> usize {
        self.estimate_cardinality() as usize
    }
}

struct TimeRequirements {
    population_time: u128,
    cardinality_time: u128,
}

impl TimeRequirements {
    fn get_column_names() -> Vec<&'static str> {
        vec!["population_time", "cardinality_time"]
    }
}

struct Cardinalities {
    cardinality: usize,
}

impl Cardinalities {
    fn get_column_names() -> Vec<&'static str> {
        vec!["cardinality"]
    }
}

struct SquaredError {
    cardinality: usize,
}

impl SquaredError {
    fn get_column_names() -> Vec<&'static str> {
        vec!["cardinality_squared_error"]
    }

    fn from_cardinalities(exact: &Cardinalities, estimated: &Cardinalities) -> Self {
        Self {
            cardinality: (exact.cardinality as isize - estimated.cardinality as isize).pow(2)
                as usize,
        }
    }
}

fn evaluate<S: SetLike<usize>>(
    mut set_like: S,
    array: &Vec<usize>,
) -> (usize, TimeRequirements, Cardinalities) {
    // First, we populate the set-like data structure with the elements.
    let population_start_time = std::time::Instant::now();

    for a in array.iter() {
        set_like.insert(a.clone());
    }

    let population_time_requirements = population_start_time.elapsed().as_nanos();

    let memory_requirements = set_like.memory_requirements();

    // Now, we evaluate the time requirements for the cardinality estimation.
    let cardinality_start_time = std::time::Instant::now();

    let cardinality = set_like.cardinality();

    let cardinality_time_requirements = cardinality_start_time.elapsed().as_nanos();

    (
        memory_requirements,
        TimeRequirements {
            population_time: population_time_requirements,
            cardinality_time: cardinality_time_requirements,
        },
        Cardinalities { cardinality },
    )
}

fn write_csv_header(csv_writer: &mut csv::Writer<std::fs::File>) {
    let mut row = csv::StringRecord::new();

    row.push_field("struct_name");
    row.push_field("precision");
    row.push_field("bits");
    row.push_field("memory_requirements");

    for column_name in TimeRequirements::get_column_names().iter() {
        row.push_field(column_name);
    }

    for column_name in Cardinalities::get_column_names().iter() {
        row.push_field(column_name);
    }

    for column_name in SquaredError::get_column_names().iter() {
        row.push_field(column_name);
    }

    csv_writer.write_record(&row).unwrap();
}

fn write_out_evaluations<S: SetLike<usize>>(
    set_like: S,
    array: &Vec<usize>,
    exact_cardinalities: &Cardinalities,
    csv_writer: &mut csv::Writer<std::fs::File>,
) {
    let mut row = csv::StringRecord::new();

    row.push_field(&set_like.struct_name());
    row.push_field(&set_like.precision().to_string());
    row.push_field(&set_like.bits().to_string());

    let (memory_requirements, time_requirements, cardinalities) = evaluate(set_like, array);

    row.push_field(&memory_requirements.to_string());

    let squared_error = SquaredError::from_cardinalities(exact_cardinalities, &cardinalities);

    row.push_field(&time_requirements.population_time.to_string());
    row.push_field(&time_requirements.cardinality_time.to_string());

    row.push_field(&cardinalities.cardinality.to_string());

    row.push_field(&squared_error.cardinality.to_string());

    csv_writer.write_record(&row).unwrap();
}

fn evaluate_hyperloglog_rs_per_precision<
    PRECISION: Precision + WordType<1> + WordType<2> + WordType<3> + WordType<4> + WordType<5> + WordType<6>,
>(
    array: &Vec<usize>,
    exact_cardinalities: &Cardinalities,
    csv_writer: &mut csv::Writer<std::fs::File>,
) {
    write_out_evaluations(
        HyperLogLog::<PRECISION, 1>::default(),
        array,
        exact_cardinalities,
        csv_writer,
    );
    write_out_evaluations(
        HyperLogLog::<PRECISION, 2>::default(),
        array,
        exact_cardinalities,
        csv_writer,
    );
    write_out_evaluations(
        HyperLogLog::<PRECISION, 3>::default(),
        array,
        exact_cardinalities,
        csv_writer,
    );
    write_out_evaluations(
        HyperLogLog::<PRECISION, 4>::default(),
        array,
        exact_cardinalities,
        csv_writer,
    );
    write_out_evaluations(
        HyperLogLog::<PRECISION, 5>::default(),
        array,
        exact_cardinalities,
        csv_writer,
    );
    write_out_evaluations(
        HyperLogLog::<PRECISION, 6>::default(),
        array,
        exact_cardinalities,
        csv_writer,
    );
}

fn evaluate_hyperloglog_rs(
    array: &Vec<usize>,
    exact_cardinalities: &Cardinalities,
    csv_writer: &mut csv::Writer<std::fs::File>,
) {
    evaluate_hyperloglog_rs_per_precision::<Precision4>(array, exact_cardinalities, csv_writer);
    evaluate_hyperloglog_rs_per_precision::<Precision5>(array, exact_cardinalities, csv_writer);
    evaluate_hyperloglog_rs_per_precision::<Precision6>(array, exact_cardinalities, csv_writer);
    evaluate_hyperloglog_rs_per_precision::<Precision7>(array, exact_cardinalities, csv_writer);
    evaluate_hyperloglog_rs_per_precision::<Precision8>(array, exact_cardinalities, csv_writer);
    evaluate_hyperloglog_rs_per_precision::<Precision9>(array, exact_cardinalities, csv_writer);
    evaluate_hyperloglog_rs_per_precision::<Precision10>(array, exact_cardinalities, csv_writer);
    evaluate_hyperloglog_rs_per_precision::<Precision11>(array, exact_cardinalities, csv_writer);
    evaluate_hyperloglog_rs_per_precision::<Precision12>(array, exact_cardinalities, csv_writer);
    evaluate_hyperloglog_rs_per_precision::<Precision13>(array, exact_cardinalities, csv_writer);
    evaluate_hyperloglog_rs_per_precision::<Precision14>(array, exact_cardinalities, csv_writer);
    evaluate_hyperloglog_rs_per_precision::<Precision15>(array, exact_cardinalities, csv_writer);
    evaluate_hyperloglog_rs_per_precision::<Precision16>(array, exact_cardinalities, csv_writer);
}

fn populate_vector(min: usize, max: usize, mut rng: StdRng) -> Vec<usize> {
    let current_vector_maximal_size = 1 + rng.gen_range(min..max);
    let mut random_vector = Vec::with_capacity(current_vector_maximal_size);

    for _ in 0..current_vector_maximal_size {
        random_vector.push(rng.gen_range(min..max));
    }

    random_vector
}

fn main() {
    let number_of_vectors = 1000;
    let min = 0;
    let max = 10_000_000;

    let loading_bar = indicatif::ProgressBar::new((number_of_vectors) as u64);

    let mut csv_writer = csv::Writer::from_path("results.csv").unwrap();

    write_csv_header(&mut csv_writer);

    for i in (0..number_of_vectors).progress_with(loading_bar.clone()) {
        // We create a random state for the loop, using as seed
        // the current iteration.
        let random_state = StdRng::seed_from_u64(4654674567556_u64.wrapping_mul(i));

        let vector = populate_vector(min, max, random_state);
        let (_, _, exact_cardinalities) = evaluate(HashSet::new(), &vector);
        evaluate_hyperloglog_rs(&vector, &exact_cardinalities, &mut csv_writer);
        write_out_evaluations(
            HashSet::new(),
            &vector,
            &exact_cardinalities,
            &mut csv_writer,
        );
    }

    csv_writer.flush().unwrap();
}
