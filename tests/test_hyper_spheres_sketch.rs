//! This test is to evaluate whether the current implementation
//! of the hyper spheres sketch is correct by implementing the
//! necessary traits on the HashSet type and then comparing the
//! results of the hyper spheres sketch of the HyperLogLog with
//! the results of the hyper spheres sketch of the HashSet.
//!
use hyperloglog_rs::prelude::*;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::collections::HashSet;

/// We need to create a wrapper around HashSet
/// because of the orphan rule and all that.
#[derive(Debug, Clone, Default)]
struct HashSetWrapper<T>(HashSet<T>);

impl<T> From<HashSet<T>> for HashSetWrapper<T> {
    fn from(set: HashSet<T>) -> Self {
        Self(set)
    }
}

impl SetLike<usize> for HashSetWrapper<usize> {
    fn intersection_size(&self, other: &Self) -> usize {
        self.0.intersection(&other.0).count()
    }

    fn difference_size(&self, other: &Self) -> usize {
        self.0.difference(&other.0).count()
    }
}

impl<T> HyperSpheresSketch for HashSetWrapper<T> {}

/// Returns a vector of vectors of usize representing HyperSpheres.
///
/// Implementative details
/// ----------------------
/// Each i-th HyperSphere in the vector contains all the indices of the
/// HyperSpheres that are contained in the i-th HyperSphere.
fn get_random_hyper_spheres(random_state: u64, number_of_hyper_spheres: usize) -> Vec<Vec<usize>> {
    let mut rng = StdRng::seed_from_u64(random_state);
    let mut hyper_spheres: Vec<Vec<usize>> = Vec::with_capacity(number_of_hyper_spheres);
    for _ in 0..number_of_hyper_spheres {
        let mut hyper_sphere: Vec<usize> = if hyper_spheres.is_empty() {
            Vec::with_capacity(100)
        } else {
            hyper_spheres.last().unwrap().clone()
        };
        for _ in 0..10_000 {
            hyper_sphere.push(rng.gen_range(0..1_000_000));
        }
        hyper_spheres.push(hyper_sphere);
    }
    hyper_spheres
}

fn get_random_hyper_spheres_sets<const N: usize>(random_state: u64) -> [HashSetWrapper<usize>; N]
where
    [HashSetWrapper<usize>; N]: Default,
{
    let hyperspheres = get_random_hyper_spheres(random_state, N);
    let mut hyperspheres_sets: Vec<HashSet<usize>> = Vec::with_capacity(N);
    for hyper_sphere in hyperspheres {
        hyperspheres_sets.push(hyper_sphere.into_iter().collect());
    }

    let mut hyperspheres_sets_array: [HashSetWrapper<usize>; N] = Default::default();
    for (i, hyper_sphere_set) in hyperspheres_sets.iter().enumerate() {
        hyperspheres_sets_array[i] = hyper_sphere_set.clone().into();
    }
    hyperspheres_sets_array
}

fn get_random_hyper_spheres_hll<const N: usize>(
    random_state: u64,
) -> HyperLogLogArray<Precision8, 6, N> {
    let hyperspheres = get_random_hyper_spheres(random_state, N);
    let mut hyperspheres_hll: HyperLogLogArray<Precision8, 6, N> = HyperLogLogArray::new();
    for (i, hyper_sphere) in hyperspheres.iter().enumerate() {
        for element in hyper_sphere {
            hyperspheres_hll[i].insert(element)
        }
    }
    hyperspheres_hll
}

#[test]
fn test_hyper_spheres_sketch() {
    let number_of_tests = 100;

    // We run multiple MSE to have an estimate of how much the
    // HyperLogLog approximation is off when compared to the
    // exact one based on HashSets.
    let mut overlaps_squared_errors = Vec::with_capacity(number_of_tests);
    let mut left_diff_squared_errors = Vec::with_capacity(number_of_tests);
    let mut right_diff_squared_errors = Vec::with_capacity(number_of_tests);

    // We iterate over the number of tests.
    for current_test in 0..number_of_tests {
        let random_state = current_test as u64 + 173845_u64;
        let left_sets = get_random_hyper_spheres_sets::<5>(random_state);
        let right_sets = get_random_hyper_spheres_sets::<5>(random_state * 2);
        let left_hll = get_random_hyper_spheres_hll::<5>(random_state);
        let right_hll = get_random_hyper_spheres_hll::<5>(random_state * 2);

        let (overlap_sets, left_diff_sets, right_diff_sets) =
            HashSetWrapper::overlap_and_differences_cardinality_matrices(&left_sets, &right_sets);
        let (overlap_hll, left_diff_hll, right_diff_hll) =
            left_hll.overlap_and_differences_cardinality_matrices::<f32>(&right_hll);

        let mut overlap_squared_error = 0.0_f32;
        let mut left_diff_squared_error = 0.0_f32;
        let mut right_diff_squared_error = 0.0_f32;
        for i in 0..5 {
            for j in 0..5 {
                overlap_squared_error += (overlap_sets[i][j] as f32 - overlap_hll[i][j]).powi(2);
            }
            left_diff_squared_error += (left_diff_sets[i] as f32 - left_diff_hll[i]).powi(2);
            right_diff_squared_error += (right_diff_sets[i] as f32 - right_diff_hll[i]).powi(2);
        }

        overlaps_squared_errors.push(overlap_squared_error / 25.0_f32);
        left_diff_squared_errors.push(left_diff_squared_error / 5.0_f32);
        right_diff_squared_errors.push(right_diff_squared_error / 5.0_f32);
    }

    let mean_overlaps_squared_error =
        overlaps_squared_errors.iter().sum::<f32>() / overlaps_squared_errors.len() as f32;
    let mean_left_diff_squared_error =
        left_diff_squared_errors.iter().sum::<f32>() / left_diff_squared_errors.len() as f32;
    let mean_right_diff_squared_error =
        right_diff_squared_errors.iter().sum::<f32>() / right_diff_squared_errors.len() as f32;

    let std_overlaps_squared_error = (overlaps_squared_errors
        .iter()
        .map(|x| (x - mean_overlaps_squared_error).powi(2))
        .sum::<f32>()
        / overlaps_squared_errors.len() as f32)
        .sqrt();

    let std_left_diff_squared_error = (left_diff_squared_errors
        .iter()
        .map(|x| (x - mean_left_diff_squared_error).powi(2))
        .sum::<f32>()
        / left_diff_squared_errors.len() as f32)
        .sqrt();

    let std_right_diff_squared_error = (right_diff_squared_errors
        .iter()
        .map(|x| (x - mean_right_diff_squared_error).powi(2))
        .sum::<f32>()
        / right_diff_squared_errors.len() as f32)
        .sqrt();

    println!(
        "The mean squared error of the overlap cardinality matrix is {} with a standard deviation of {}.",
        mean_overlaps_squared_error, std_overlaps_squared_error
    );
    println!(
        "The mean squared error of the left difference cardinality vector is {} with a standard deviation of {}.",
        mean_left_diff_squared_error, std_left_diff_squared_error
    );
    println!(
        "The mean squared error of the right difference cardinality vector is {} with a standard deviation of {}.",
        mean_right_diff_squared_error, std_right_diff_squared_error
    );
}

#[test]
/// First of several tests to evaluate the correctness of the
/// hyper spheres sketch.
fn test_hand_picked_hyper_spheres_1() {
    let left = [
        vec![1_usize, 2, 3],
        vec![1, 2, 3, 7],
        vec![1, 2, 3, 4, 5, 7],
    ];

    let right = [vec![1_usize, 2], vec![1, 2, 6, 7], vec![1, 2, 3, 6, 7]];

    let left_sets: [HashSetWrapper<usize>; 3] = [
        left[0].iter().copied().collect::<HashSet<usize>>().into(),
        left[1].iter().copied().collect::<HashSet<usize>>().into(),
        left[2].iter().copied().collect::<HashSet<usize>>().into(),
    ];

    let right_sets: [HashSetWrapper<usize>; 3] = [
        right[0].iter().copied().collect::<HashSet<usize>>().into(),
        right[1].iter().copied().collect::<HashSet<usize>>().into(),
        right[2].iter().copied().collect::<HashSet<usize>>().into(),
    ];

    let (overlap_sets, left_diff_sets, right_diff_sets) =
        HashSetWrapper::overlap_and_differences_cardinality_matrices(&left_sets, &right_sets);

    let expected_overlap_sets = [[2, 0, 1], [0, 1, 0], [0, 0, 0]];

    let expected_left_diff_sets = [0, 0, 2];

    let expected_right_diff_sets = [0, 1, 0];

    assert_eq!(
        overlap_sets, expected_overlap_sets,
        concat!(
            "We expect the overlap cardinality matrix to be {:?} ",
            "but we got {:?} instead."
        ),
        expected_overlap_sets, overlap_sets
    );
    assert_eq!(
        left_diff_sets, expected_left_diff_sets,
        concat!(
            "We expect the left difference cardinality vector to be ",
            "{:?} and the right difference cardinality vector to be ",
            "{:?} but we got {:?} and {:?} respectively."
        ),
        expected_left_diff_sets, expected_right_diff_sets, left_diff_sets, right_diff_sets
    );
    assert_eq!(
        right_diff_sets, expected_right_diff_sets,
        concat!(
            "We expect the left difference cardinality vector to be ",
            "{:?} and the right difference cardinality vector to be ",
            "{:?} but we got {:?} and {:?} respectively."
        ),
        expected_left_diff_sets, expected_right_diff_sets, left_diff_sets, right_diff_sets
    );
}


#[test]
/// First of several tests to evaluate the correctness of the
/// hyper spheres sketch.
fn test_hand_picked_hyper_spheres_2() {
    let left = [
        vec![1_usize, 2, 3],
        vec![1, 2, 3, 7],
        vec![1, 2, 3, 4, 5, 7],
    ];

    let right = [vec![1_usize, 2], vec![1, 2, 6, 7], vec![1, 2, 3, 6, 7]];

    let left_sets: [HashSetWrapper<usize>; 3] = [
        left[0].iter().copied().collect::<HashSet<usize>>().into(),
        left[1].iter().copied().collect::<HashSet<usize>>().into(),
        left[2].iter().copied().collect::<HashSet<usize>>().into(),
    ];

    let right_sets: [HashSetWrapper<usize>; 3] = [
        right[0].iter().copied().collect::<HashSet<usize>>().into(),
        right[1].iter().copied().collect::<HashSet<usize>>().into(),
        right[2].iter().copied().collect::<HashSet<usize>>().into(),
    ];

    let (overlap_sets, left_diff_sets, right_diff_sets) =
        HashSetWrapper::overlap_and_differences_cardinality_matrices(&left_sets, &right_sets);

    let expected_overlap_sets = [[2, 0, 1], [0, 1, 0], [0, 0, 0]];

    let expected_left_diff_sets = [0, 0, 2];

    let expected_right_diff_sets = [0, 1, 0];

    assert_eq!(
        overlap_sets, expected_overlap_sets,
        concat!(
            "We expect the overlap cardinality matrix to be {:?} ",
            "but we got {:?} instead."
        ),
        expected_overlap_sets, overlap_sets
    );
    assert_eq!(
        left_diff_sets, expected_left_diff_sets,
        concat!(
            "We expect the left difference cardinality vector to be ",
            "{:?} and the right difference cardinality vector to be ",
            "{:?} but we got {:?} and {:?} respectively."
        ),
        expected_left_diff_sets, expected_right_diff_sets, left_diff_sets, right_diff_sets
    );
    assert_eq!(
        right_diff_sets, expected_right_diff_sets,
        concat!(
            "We expect the left difference cardinality vector to be ",
            "{:?} and the right difference cardinality vector to be ",
            "{:?} but we got {:?} and {:?} respectively."
        ),
        expected_left_diff_sets, expected_right_diff_sets, left_diff_sets, right_diff_sets
    );
}
