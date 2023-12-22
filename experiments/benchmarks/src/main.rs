//! Evaluation of set-like properties across different data structures.
use core::hash::BuildHasher;
use hyperloglog_rs::prelude::*;
use hyperloglogplus::HyperLogLog as TabacHyperLogLog;
use hyperloglogplus::HyperLogLogPF as TabacHyperLogLogPF;
use hyperloglogplus::HyperLogLogPlus as TabacHyperLogLogPlus;
use indicatif::ProgressIterator;
use mem_dbg::*;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::hash::Hasher;

use std::collections::hash_map::RandomState;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::hash::Hash;
use streaming_algorithms::HyperLogLog as SAHyperLogLog;

trait SetLike<I> {
    fn struct_name(&self) -> String;
    fn memory_requirements(&self) -> usize;
    fn insert(&mut self, x: I);
    fn cardinality(&self) -> usize;
    fn intersection_cardinality(&self, other: &Self) -> usize;
    fn union_cardinality(&self, other: &Self) -> usize;
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

    fn cardinality(&self) -> usize {
        self.len()
    }

    fn intersection_cardinality(&self, other: &Self) -> usize {
        self.intersection(other).count()
    }

    fn union_cardinality(&self, other: &Self) -> usize {
        self.union(other).count()
    }
}

impl<I: Hash + Eq + PartialEq + Ord> SetLike<I> for BTreeSet<I> {
    fn struct_name(&self) -> String {
        "BTreeSet".to_string()
    }

    fn memory_requirements(&self) -> usize {
        unimplemented!("BTreeSet does not implement a method to compute the memory requirements.")
    }

    fn insert(&mut self, x: I) {
        self.insert(x);
    }

    fn cardinality(&self) -> usize {
        self.len()
    }

    fn intersection_cardinality(&self, other: &Self) -> usize {
        self.intersection(other).count()
    }

    fn union_cardinality(&self, other: &Self) -> usize {
        self.union(other).count()
    }
}

impl<I: Hash + Eq + PartialEq, P: Precision + WordType<BITS>, const BITS: usize> SetLike<I>
    for HyperLogLog<P, BITS>
{
    fn struct_name(&self) -> String {
        format!("HyperLogLog<{}, {}>", P::EXPONENT, BITS)
    }

    fn memory_requirements(&self) -> usize {
        core::mem::size_of::<Self>()
    }

    fn insert(&mut self, x: I) {
        self.insert(x);
    }

    fn cardinality(&self) -> usize {
        self.estimate_cardinality() as usize
    }

    fn intersection_cardinality(&self, other: &Self) -> usize {
        self.estimate_intersection_cardinality::<f32>(other) as usize
    }

    fn union_cardinality(&self, other: &Self) -> usize {
        self.estimate_union_cardinality(other) as usize
    }
}

impl<I: Hash + Eq + PartialEq + MemSize, B: BuildHasher + MemSize> SetLike<I>
    for TabacHyperLogLogPF<I, B>
where
    Self: Clone,
{
    fn struct_name(&self) -> String {
        format!("TabacHyperLogLogPF<{}>", self.precision())
    }

    fn memory_requirements(&self) -> usize {
        self.mem_size(SizeFlags::CAPACITY)
    }

    fn insert(&mut self, x: I) {
        TabacHyperLogLog::insert(self, &x);
    }

    fn cardinality(&self) -> usize {
        let mut copy = self.clone();
        copy.count() as usize
    }

    fn intersection_cardinality(&self, other: &Self) -> usize {
        // We use the inclusion-exclusion principle to compute the intersection cardinality.
        // |A ∩ B| = |A| + |B| - |A ∪ B|
        let union_cardinality = self.union_cardinality(other);
        self.cardinality() + other.cardinality() - union_cardinality
    }

    fn union_cardinality(&self, other: &Self) -> usize {
        let mut copy = self.clone();
        copy.merge(other).unwrap();
        copy.cardinality()
    }
}

impl<I: Hash + Eq + PartialEq + MemSize, B: BuildHasher + MemSize> SetLike<I>
    for TabacHyperLogLogPlus<I, B>
where
    Self: Clone,
{
    fn struct_name(&self) -> String {
        format!("TabacHyperLogLogPlus<{}>", self.precision())
    }

    fn memory_requirements(&self) -> usize {
        self.mem_size(SizeFlags::CAPACITY)
    }

    fn insert(&mut self, x: I) {
        TabacHyperLogLog::insert(self, &x);
    }

    fn cardinality(&self) -> usize {
        let mut copy = self.clone();
        copy.count() as usize
    }

    fn intersection_cardinality(&self, other: &Self) -> usize {
        // We use the inclusion-exclusion principle to compute the intersection cardinality.
        // |A ∩ B| = |A| + |B| - |A ∪ B|
        let union_cardinality = self.union_cardinality(other);
        (self.cardinality() + other.cardinality()).saturating_sub(union_cardinality)
    }

    fn union_cardinality(&self, other: &Self) -> usize {
        let mut copy = self.clone();
        copy.merge(other).unwrap();
        copy.cardinality()
    }
}

impl<I: Hash + Eq + PartialEq + MemSize> SetLike<I> for SAHyperLogLog<I> {
    fn struct_name(&self) -> String {
        format!("SAHyperLogLog<{}>", self.get_precision())
    }

    fn memory_requirements(&self) -> usize {
        self.mem_size(SizeFlags::CAPACITY)
    }

    fn insert(&mut self, x: I) {
        self.push(&x);
    }

    fn cardinality(&self) -> usize {
        self.len() as usize
    }

    fn intersection_cardinality(&self, other: &Self) -> usize {
        let mut copy = self.clone();
        copy.intersect(other);
        copy.len() as usize
    }

    fn union_cardinality(&self, other: &Self) -> usize {
        let mut copy = self.clone();
        copy.union(other);
        copy.len() as usize
    }
}

struct TimeRequirements {
    population_time: u128,
    cardinality_time: u128,
    intersection_cardinality_time: u128,
    union_cardinality_time: u128,
}

impl TimeRequirements {
    fn get_column_names() -> Vec<&'static str> {
        vec![
            "population_time",
            "cardinality_time",
            "intersection_cardinality_time",
            "union_cardinality_time",
        ]
    }
}

struct Cardinalities {
    left_cardinality: usize,
    right_cardinality: usize,
    intersection_cardinality: usize,
    union_cardinality: usize,
}

impl Cardinalities {
    fn get_column_names() -> Vec<&'static str> {
        vec![
            "left_cardinality",
            "right_cardinality",
            "intersection_cardinality",
            "union_cardinality",
        ]
    }
}

struct SquaredError {
    left_cardinality: usize,
    right_cardinality: usize,
    intersection_cardinality: usize,
    union_cardinality: usize,
}

impl SquaredError {
    fn get_column_names() -> Vec<&'static str> {
        vec![
            "left_cardinality_squared_error",
            "right_cardinality_squared_error",
            "intersection_cardinality_squared_error",
            "union_cardinality_squared_error",
        ]
    }

    fn from_cardinalities(exact: &Cardinalities, estimated: &Cardinalities) -> Self {
        Self {
            left_cardinality: (exact.left_cardinality as isize
                - estimated.left_cardinality as isize)
                .pow(2) as usize,
            right_cardinality: (exact.right_cardinality as isize
                - estimated.right_cardinality as isize)
                .pow(2) as usize,
            intersection_cardinality: (exact.intersection_cardinality as isize
                - estimated.intersection_cardinality as isize)
                .pow(2) as usize,
            union_cardinality: (exact.union_cardinality as isize
                - estimated.union_cardinality as isize)
                .pow(2) as usize,
        }
    }
}

fn evaluate<S: SetLike<usize>>(
    mut left_set_like: S,
    mut right_set_like: S,
    left: &Vec<usize>,
    right: &Vec<usize>,
) -> (usize, TimeRequirements, Cardinalities) {
    // First, we populate the set-like data structure with the elements.
    let population_start_time = std::time::Instant::now();

    for l in left.iter() {
        left_set_like.insert(l.clone());
    }

    for r in right.iter() {
        right_set_like.insert(r.clone());
    }

    let population_time_requirements = population_start_time.elapsed().as_nanos();

    let memory_requirements =
        left_set_like.memory_requirements() + right_set_like.memory_requirements();

    // Now, we evaluate the time requirements for the cardinality estimation.
    let cardinality_start_time = std::time::Instant::now();

    let left_cardinality = left_set_like.cardinality();
    let right_cardinality = right_set_like.cardinality();

    let cardinality_time_requirements = cardinality_start_time.elapsed().as_nanos();

    // Now, we evaluate the time requirements for the intersection cardinality estimation.
    let intersection_cardinality_start_time = std::time::Instant::now();

    let intersection_cardinality = left_set_like.intersection_cardinality(&right_set_like);

    let intersection_cardinality_time_requirements =
        intersection_cardinality_start_time.elapsed().as_nanos();

    // Now, we evaluate the time requirements for the union cardinality estimation.

    let union_cardinality_start_time = std::time::Instant::now();

    let union_cardinality = left_set_like.union_cardinality(&right_set_like);

    let union_cardinality_time_requirements = union_cardinality_start_time.elapsed().as_nanos();

    (
        memory_requirements,
        TimeRequirements {
            population_time: population_time_requirements,
            cardinality_time: cardinality_time_requirements,
            intersection_cardinality_time: intersection_cardinality_time_requirements,
            union_cardinality_time: union_cardinality_time_requirements,
        },
        Cardinalities {
            left_cardinality,
            right_cardinality,
            intersection_cardinality,
            union_cardinality,
        },
    )
}

fn write_csv_header(csv_writer: &mut csv::Writer<std::fs::File>) {
    let mut row = csv::StringRecord::new();

    row.push_field("struct_name");
    row.push_field("memory_requirements");

    for column_name in TimeRequirements::get_column_names().iter() {
        row.push_field(column_name);
    }

    for column_name in Cardinalities::get_column_names().iter() {
        row.push_field(column_name);
    }

    // We rewrite the exact cardinalities.
    for column_name in Cardinalities::get_column_names().iter() {
        row.push_field(format!("exact_{}", column_name).as_str());
    }

    for column_name in SquaredError::get_column_names().iter() {
        row.push_field(column_name);
    }

    csv_writer.write_record(&row).unwrap();
}

fn write_out_evaluations<S: SetLike<usize>>(
    left_set_like: S,
    right_set_like: S,
    left: &Vec<usize>,
    right: &Vec<usize>,
    exact_cardinalities: &Cardinalities,
    csv_writer: &mut csv::Writer<std::fs::File>,
) {
    let mut row = csv::StringRecord::new();

    row.push_field(&left_set_like.struct_name());

    let (memory_requirements, time_requirements, cardinalities) =
        evaluate(left_set_like, right_set_like, left, right);

    let squared_error = SquaredError::from_cardinalities(exact_cardinalities, &cardinalities);

    row.push_field(&memory_requirements.to_string());

    row.push_field(&time_requirements.population_time.to_string());
    row.push_field(&time_requirements.cardinality_time.to_string());
    row.push_field(&time_requirements.intersection_cardinality_time.to_string());
    row.push_field(&time_requirements.union_cardinality_time.to_string());

    row.push_field(&cardinalities.left_cardinality.to_string());
    row.push_field(&cardinalities.right_cardinality.to_string());
    row.push_field(&cardinalities.intersection_cardinality.to_string());
    row.push_field(&cardinalities.union_cardinality.to_string());

    row.push_field(&exact_cardinalities.left_cardinality.to_string());
    row.push_field(&exact_cardinalities.right_cardinality.to_string());
    row.push_field(&exact_cardinalities.intersection_cardinality.to_string());
    row.push_field(&exact_cardinalities.union_cardinality.to_string());

    row.push_field(&squared_error.left_cardinality.to_string());
    row.push_field(&squared_error.right_cardinality.to_string());
    row.push_field(&squared_error.intersection_cardinality.to_string());
    row.push_field(&squared_error.union_cardinality.to_string());

    csv_writer.write_record(&row).unwrap();
}

fn evaluate_hyperloglog_rs_per_precision<P: Precision + WordType<4> + WordType<5> + WordType<6>>(
    left: &Vec<usize>,
    right: &Vec<usize>,
    exact_cardinalities: &Cardinalities,
    csv_writer: &mut csv::Writer<std::fs::File>,
) {
    write_out_evaluations(
        HyperLogLog::<P, 4>::default(),
        HyperLogLog::<P, 4>::default(),
        left,
        right,
        exact_cardinalities,
        csv_writer,
    );
    write_out_evaluations(
        HyperLogLog::<P, 5>::default(),
        HyperLogLog::<P, 5>::default(),
        left,
        right,
        exact_cardinalities,
        csv_writer,
    );
    write_out_evaluations(
        HyperLogLog::<P, 6>::default(),
        HyperLogLog::<P, 6>::default(),
        left,
        right,
        exact_cardinalities,
        csv_writer,
    );
}

fn evaluate_hyperloglog_rs(
    left: &Vec<usize>,
    right: &Vec<usize>,
    exact_cardinalities: &Cardinalities,
    csv_writer: &mut csv::Writer<std::fs::File>,
) {
    evaluate_hyperloglog_rs_per_precision::<Precision4>(
        left,
        right,
        exact_cardinalities,
        csv_writer,
    );
    evaluate_hyperloglog_rs_per_precision::<Precision5>(
        left,
        right,
        exact_cardinalities,
        csv_writer,
    );
    evaluate_hyperloglog_rs_per_precision::<Precision6>(
        left,
        right,
        exact_cardinalities,
        csv_writer,
    );
    evaluate_hyperloglog_rs_per_precision::<Precision7>(
        left,
        right,
        exact_cardinalities,
        csv_writer,
    );
    evaluate_hyperloglog_rs_per_precision::<Precision8>(
        left,
        right,
        exact_cardinalities,
        csv_writer,
    );
    evaluate_hyperloglog_rs_per_precision::<Precision9>(
        left,
        right,
        exact_cardinalities,
        csv_writer,
    );
    evaluate_hyperloglog_rs_per_precision::<Precision10>(
        left,
        right,
        exact_cardinalities,
        csv_writer,
    );
    evaluate_hyperloglog_rs_per_precision::<Precision11>(
        left,
        right,
        exact_cardinalities,
        csv_writer,
    );
    evaluate_hyperloglog_rs_per_precision::<Precision12>(
        left,
        right,
        exact_cardinalities,
        csv_writer,
    );
    evaluate_hyperloglog_rs_per_precision::<Precision13>(
        left,
        right,
        exact_cardinalities,
        csv_writer,
    );
    evaluate_hyperloglog_rs_per_precision::<Precision14>(
        left,
        right,
        exact_cardinalities,
        csv_writer,
    );
    evaluate_hyperloglog_rs_per_precision::<Precision15>(
        left,
        right,
        exact_cardinalities,
        csv_writer,
    );
    evaluate_hyperloglog_rs_per_precision::<Precision16>(
        left,
        right,
        exact_cardinalities,
        csv_writer,
    );
}

fn evaluate_hyperloglog_streaming_algorithms(
    left: &Vec<usize>,
    right: &Vec<usize>,
    exact_cardinalities: &Cardinalities,
    csv_writer: &mut csv::Writer<std::fs::File>,
) {
    for precision in 4..=16 {
        write_out_evaluations(
            SAHyperLogLog::new_with_precision(precision),
            SAHyperLogLog::new_with_precision(precision),
            left,
            right,
            exact_cardinalities,
            csv_writer,
        );
    }
}

fn evaluate_hyperloglog_tabac_original(
    left: &Vec<usize>,
    right: &Vec<usize>,
    exact_cardinalities: &Cardinalities,
    csv_writer: &mut csv::Writer<std::fs::File>,
) {
    let random_state = RandomState::new();

    for precision in 4..=16 {
        write_out_evaluations(
            TabacHyperLogLogPF::new(precision, random_state.clone()).unwrap(),
            TabacHyperLogLogPF::new(precision, random_state.clone()).unwrap(),
            left,
            right,
            exact_cardinalities,
            csv_writer,
        );
    }
}

fn evaluate_hyperloglog_tabac_plus(
    left: &Vec<usize>,
    right: &Vec<usize>,
    exact_cardinalities: &Cardinalities,
    csv_writer: &mut csv::Writer<std::fs::File>,
) {
    let random_state = RandomState::new();

    for precision in 4..=16 {
        write_out_evaluations(
            TabacHyperLogLogPlus::new(precision, random_state.clone()).unwrap(),
            TabacHyperLogLogPlus::new(precision, random_state.clone()).unwrap(),
            left,
            right,
            exact_cardinalities,
            csv_writer,
        );
    }
}

fn populate_vector_of_vectors(
    number_of_vectors: usize,
    size: usize,
    min: usize,
    max: usize,
    random_state: RandomState,
) -> Vec<Vec<usize>> {
    let mut rng = StdRng::seed_from_u64(random_state.build_hasher().finish());
    let mut vector_of_vectors = Vec::with_capacity(number_of_vectors);

    for i in 0..number_of_vectors {
        let current_vector_maximal_size = (size * (i + 1)) / number_of_vectors;
        let mut random_vector = Vec::with_capacity(current_vector_maximal_size);

        for _ in 0..current_vector_maximal_size {
            random_vector.push(rng.gen_range(min..max));
        }

        vector_of_vectors.push(random_vector);
    }

    vector_of_vectors
}

fn main() {
    let number_of_vectors = 100;
    let size = 10_000_000;
    let min = 0;
    let max = 1_000_000;

    let random_state = RandomState::new();
    let vectors = populate_vector_of_vectors(number_of_vectors, size, min, max, random_state);

    let loading_bar = indicatif::ProgressBar::new((number_of_vectors * number_of_vectors) as u64);

    let mut csv_writer = csv::Writer::from_path("results.csv").unwrap();

    write_csv_header(&mut csv_writer);

    for (left, right) in vectors
        .iter()
        .flat_map(|l| vectors.iter().map(move |r| (l, r)))
        .progress_with(loading_bar)
    {
        let (_, _, exact_cardinalities) = evaluate(HashSet::new(), HashSet::new(), left, right);
        evaluate_hyperloglog_rs(left, right, &exact_cardinalities, &mut csv_writer);
        write_out_evaluations(
            HashSet::new(),
            HashSet::new(),
            left,
            right,
            &exact_cardinalities,
            &mut csv_writer,
        );
        evaluate_hyperloglog_streaming_algorithms(
            left,
            right,
            &exact_cardinalities,
            &mut csv_writer,
        );
        evaluate_hyperloglog_tabac_original(left, right, &exact_cardinalities, &mut csv_writer);
        evaluate_hyperloglog_tabac_plus(left, right, &exact_cardinalities, &mut csv_writer);
    }

    csv_writer.flush().unwrap();
}
