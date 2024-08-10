#![cfg(feature = "std")]
//! This test is to evaluate whether the current implementation
//! of the hyper spheres sketch is correct by implementing the
//! necessary traits on the HashSet type and then comparing the
//! results of the hyper spheres sketch of the HyperLogLog with
//! the results of the hyper spheres sketch of the HashSet.
use hyperloglog_rs::prelude::*;
use rand::rngs::StdRng;
use rand::Rng;
use twox_hash::XxHash64;
use rand::SeedableRng;
use std::collections::HashSet;

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
        let maximal_universe_size = rng.gen_range(0..100_000);
        let number_of_elements = rng.gen_range(0..100_000);
        for _ in 0..number_of_elements {
            hyper_sphere.push(rng.gen_range(0..maximal_universe_size.max(1)));
        }
        hyper_spheres.push(hyper_sphere);
    }
    hyper_spheres
}

fn get_random_hyper_spheres_sets<const N: usize>(random_state: u64) -> [HashSet<usize>; N]
where
    [HashSet<usize>; N]: Default,
{
    let hyperspheres = get_random_hyper_spheres(random_state, N);
    let mut hyperspheres_sets: Vec<HashSet<usize>> = Vec::with_capacity(N);
    for hyper_sphere in hyperspheres {
        hyperspheres_sets.push(hyper_sphere.into_iter().collect());
    }

    let mut hyperspheres_sets_array: [HashSet<usize>; N] = Default::default();
    for (i, hyper_sphere_set) in hyperspheres_sets.iter().enumerate() {
        hyperspheres_sets_array[i] = hyper_sphere_set.clone().into();
    }
    hyperspheres_sets_array
}

fn get_random_hyper_spheres_hll<
    Hasher: Default + core::hash::Hasher,
    HA: HyperLogLogArrayTrait<P, B, H, Hasher, N>,
    P: Precision,
    B: Bits,
    H: Copy + HyperLogLogTrait<P, B, Hasher>,
    const N: usize,
>(
    random_state: u64,
) -> HA {
    let hyperspheres = get_random_hyper_spheres(random_state, N);
    let mut hyperspheres_hll: HA = HA::default();
    for (i, hyper_sphere) in hyperspheres.iter().enumerate() {
        for element in hyper_sphere {
            hyperspheres_hll.insert(i, element);
        }
    }
    hyperspheres_hll
}

fn test_hyper_spheres_sketch<Hasher, HA, P, B, H, const N: usize>()
where
    Hasher: Default + core::hash::Hasher,
    HA: HyperLogLogArrayTrait<P, B, H, Hasher, N>,
    P: Precision + PrecisionConstants<f64>,
    B: Bits,
    H: Copy + HyperLogLogTrait<P, B, Hasher> + SetLike<f64>,
    [HashSet<usize>; N]: Default,
{
    let number_of_tests = 100;

    // We iterate over the number of tests.
    for current_test in 0..number_of_tests {
        let random_state = (current_test as u64 + 173845_u64).wrapping_mul(456789);
        let left_sets = get_random_hyper_spheres_sets::<N>(random_state);
        let right_sets = get_random_hyper_spheres_sets::<N>(random_state * 2);
        let left_hll = get_random_hyper_spheres_hll::<Hasher, HA, P, B, H, N>(random_state);
        let right_hll = get_random_hyper_spheres_hll::<Hasher, HA, P, B, H, N>(random_state * 2);

        let (overlap_sets, left_diff_sets, right_diff_sets) =
            HashSet::overlap_and_differences_cardinality_matrices(&left_sets, &right_sets);

        // We can execute some self-consistency checks, namely that
        // the sum of all values in the overlap cardinality matrix
        // has to be equal to the cardinality of the intersection between
        // the two largest sets left and right, which are the last.
        let expected_intersection = left_sets
            .last()
            .unwrap()
            .intersection(&right_sets.last().unwrap())
            .count();

        let overlap_summation = overlap_sets.iter().flatten().sum::<usize>();
        assert_eq!(
            overlap_summation, expected_intersection,
            concat!(
                "The sum of all values in the overlap cardinality matrix ",
                "has to be equal to the cardinality of the intersection ",
                "between the two largest sets left and right, which are ",
                "the last. We expect {:?} but we got {:?} instead."
            ),
            expected_intersection, overlap_summation
        );

        // Furthermore, we can check that the sum of the left difference
        // cardinality vector has to be equal to the cardinality of the
        // difference between the largest set left and the largest set
        // right.
        let expected_left_difference = left_sets
            .last()
            .unwrap()
            .difference(&right_sets.last().unwrap())
            .count();

        let left_diff_summation = left_diff_sets.iter().sum::<usize>();

        assert_eq!(
            left_diff_summation, expected_left_difference,
            concat!(
                "The sum of all values in the left difference cardinality ",
                "vector has to be equal to the cardinality of the difference ",
                "between the largest set left and the largest set right. ",
                "We expect {:?} but we got {:?} instead."
            ),
            expected_left_difference, left_diff_summation
        );

        // Simmetrically, the same must hold for the right difference.
        let expected_right_difference = right_sets
            .last()
            .unwrap()
            .difference(&left_sets.last().unwrap())
            .count();

        let right_diff_summation = right_diff_sets.iter().sum::<usize>();

        assert_eq!(
            right_diff_summation, expected_right_difference,
            concat!(
                "The sum of all values in the right difference cardinality ",
                "vector has to be equal to the cardinality of the difference ",
                "between the largest set right and the largest set left. ",
                "We expect {:?} but we got {:?} instead."
            ),
            expected_right_difference, right_diff_summation
        );

        let (overlap_hll, left_diff_hll, right_diff_hll) =
            left_hll.overlap_and_differences_cardinality_matrices::<f64>(&right_hll);

        let (overlap_normalized_hll, left_diff_normalized_hll, right_diff_normalized_hll) =
            left_hll.normalized_overlap_and_differences_cardinality_matrices::<f64>(&right_hll);

        // We check that none of the values is less than zero, i.e. no
        // negative cardinalities have somehow been computed.
        for i in 0..N {
            for j in 0..N {
                assert!(
                    overlap_normalized_hll[i][j] >= 0.0,
                    concat!(
                        "We expect the overlap cardinality matrix to have ",
                        "non-negative values but we got {:?} instead."
                    ),
                    overlap_normalized_hll
                );
                assert!(
                    overlap_normalized_hll[i][j] <= 1.0,
                    concat!(
                        "We expect the overlap cardinality matrix to have ",
                        "values less than or equal to 1.0 but we got {:?} instead ",
                        "in position ({:?}, {:?})."
                    ),
                    overlap_normalized_hll[i][j],
                    i,
                    j
                );
                assert!(
                    overlap_hll[i][j] >= 0.0,
                    concat!(
                        "We expect the overlap cardinality matrix to have ",
                        "non-negative values but we got {:?} instead."
                    ),
                    overlap_hll
                );
            }
            assert!(
                left_diff_normalized_hll[i] >= 0.0,
                concat!(
                    "We expect the left difference cardinality vector to ",
                    "have non-negative values but we got {:?} instead."
                ),
                left_diff_normalized_hll
            );
            assert!(
                left_diff_normalized_hll[i] <= 1.0,
                concat!(
                    "We expect the left difference cardinality vector to ",
                    "have values less than or equal to 1.0 but we got {:?} instead. ",
                    "This happened in position {:?}."
                ),
                left_diff_normalized_hll[i],
                i
            );

            assert!(
                left_diff_hll[i] >= 0.0,
                concat!(
                    "We expect the left difference cardinality vector to ",
                    "have non-negative values but we got {:?} instead."
                ),
                left_diff_hll
            );
            assert!(
                right_diff_normalized_hll[i] >= 0.0,
                concat!(
                    "We expect the right difference cardinality vector to ",
                    "have non-negative values but we got {:?} instead."
                ),
                right_diff_normalized_hll
            );
            assert!(
                right_diff_normalized_hll[i] <= 1.0,
                concat!(
                    "We expect the right difference cardinality vector to ",
                    "have values less than or equal to 1.0 but we got {:?} instead. ",
                    "This happened in position {:?}."
                ),
                right_diff_normalized_hll[i],
                i
            );

            assert!(
                right_diff_hll[i] >= 0.0,
                concat!(
                    "We expect the right difference cardinality vector to ",
                    "have non-negative values but we got {:?} instead."
                ),
                right_diff_hll
            );
        }
    }
}

macro_rules! test_hyper_spheres_by_precision_and_size {
    ($hasher:ty, $precision:ty, $bits:ty, $($size:expr),+) => {
        $(
            paste::paste! {
                #[test]
                fn [<test_hll_hyper_spheres_sketch_ $hasher:lower _ $precision:lower _ $bits:lower _size_ $size>]() {
                    test_hyper_spheres_sketch::<$hasher, HyperLogLogArray<$precision, $bits, HyperLogLog<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>, $hasher, $size>, $precision, $bits, HyperLogLog<$precision, $bits, <$precision as ArrayRegister<$bits>>::ArrayRegister, $hasher>, $size>();
                }
            }
        )+
    };
}

macro_rules! test_hyper_spheres_by_precision_and_bits {
    ($hasher:ty, $precision:ty, ($($bits:ty),+)) => {
        $(
            test_hyper_spheres_by_precision_and_size!($hasher, $precision, $bits, 3, 4, 5);
        )+
    };
}

macro_rules! test_hyper_spheres_by_precisions {
    ($hasher:ty, $($precision:ty),*) => {
        $(
            test_hyper_spheres_by_precision_and_bits!($hasher, $precision, (Bits4, Bits5, Bits6));
        )*
    };
}

macro_rules! test_hyper_spheres_by_hashers {
    ($($hasher:ty),*) => {
        $(
            test_hyper_spheres_by_precisions!($hasher, Precision4, Precision5, Precision6, Precision7, Precision8, Precision9, Precision10, Precision11, Precision12, Precision13, Precision14, Precision15, Precision16);
        )*
    };
}

test_hyper_spheres_by_hashers!(XxHash64);

fn test_hyper_spheres(
    left: &[Vec<usize>; 3],
    right: &[Vec<usize>; 3],
    expected_overlap_sets: [[usize; 3]; 3],
    expected_left_diff_sets: [usize; 3],
    expected_right_diff_sets: [usize; 3],
) {
    let left_sets: [HashSet<usize>; 3] = [
        left[0].iter().copied().collect::<HashSet<usize>>().into(),
        left[1].iter().copied().collect::<HashSet<usize>>().into(),
        left[2].iter().copied().collect::<HashSet<usize>>().into(),
    ];

    let right_sets: [HashSet<usize>; 3] = [
        right[0].iter().copied().collect::<HashSet<usize>>().into(),
        right[1].iter().copied().collect::<HashSet<usize>>().into(),
        right[2].iter().copied().collect::<HashSet<usize>>().into(),
    ];

    let (overlap_sets, left_diff_sets, right_diff_sets): ([[usize; 3]; 3], [usize; 3], [usize; 3]) =
        HashSet::overlap_and_differences_cardinality_matrices(&left_sets, &right_sets);

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
fn test_hand_picked_hyper_spheres_1() {
    let left = [
        vec![1_usize, 2, 3],
        vec![1, 2, 3, 7],
        vec![1, 2, 3, 4, 5, 7],
    ];

    let right = [vec![1_usize, 2], vec![1, 2, 6, 7], vec![1, 2, 3, 6, 7]];
    let expected_overlap_sets = [[2, 0, 1], [0, 1, 0], [0, 0, 0]];
    let expected_left_diff_sets = [0, 0, 2];
    let expected_right_diff_sets = [0, 1, 0];

    test_hyper_spheres(
        &left,
        &right,
        expected_overlap_sets,
        expected_left_diff_sets,
        expected_right_diff_sets,
    );
}

#[test]
/// Second of several tests to evaluate the correctness of the
/// hyper spheres sketch.
fn test_hand_picked_hyper_spheres_2() {
    let left = [
        vec![1_usize, 2, 3],
        vec![1, 2, 3, 7],
        vec![1, 2, 3, 4, 5, 7],
    ];

    let right = [vec![], vec![], vec![1, 2, 3, 6, 7]];
    let expected_overlap_sets = [[0, 0, 3], [0, 0, 1], [0, 0, 0]];
    let expected_left_diff_sets = [0, 0, 2];
    let expected_right_diff_sets = [0, 0, 1];

    test_hyper_spheres(
        &left,
        &right,
        expected_overlap_sets,
        expected_left_diff_sets,
        expected_right_diff_sets,
    );
}
