//! Fuzzing harness to verify that the three methods of sketching produce correct and consistent results.
//!
//! Specifically, we will test the following three properties:
//! * That the estimated exclusive overlap cardinalities are correct for the given two vector sets
//! * That the estimated exclusive differences cardinalities are correct for the given two vector sets
//! * That the method that produces at once the overlap and difference cardinalities is consistent with the two previous methods
#![no_main]
#![feature(generic_const_exprs)]

use arbitrary::Arbitrary;
use hyperloglog_rs::prelude::*;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct FuzzCase {
    left_sets: Vec<Vec<u32>>,
    right_sets: Vec<Vec<u32>>,
}

const BITS: usize = 6;
const PRECISION: usize = 10;
const N: usize = 2;

fuzz_target!(|data: FuzzCase| {
    // First, we check whether the provided fuzzed data is valid for the tests:
    // * The vectors must have at least 2 sub-vectors each
    // * The two sets of vectors must have the same length - since it is unlikely that
    //   the fuzzer will, by chance, generate two sets of vectors with the same length,
    //   we will pad the shorter set with empty vectors.
    // * The vectors must have exactly length equal to N, where N is the number of
    //   HyperLogLog counters in the HyperLogLogArray. We will pad the vectors with
    //   empty vectors if necessary, or truncate them if they are too long.
    // * After that, we will make sure that each i-th vector within each of the two sets
    //   contains all elements of the (i-1)-th vector.

    // The vectors must have at least 2 sub-vectors each:
    if data.left_sets.len() < 2 && data.right_sets.len() < 2 {
        return;
    }

    // First, we pad the shorter set with empty vectors:
    let left_sets = if data.left_sets.len() > data.right_sets.len() {
        data.left_sets.clone()
    } else {
        let mut left_sets = data.left_sets.clone();
        left_sets.resize(data.right_sets.len(), vec![]);
        left_sets
    };

    let right_sets = if data.right_sets.len() > data.left_sets.len() {
        data.right_sets.clone()
    } else {
        let mut right_sets = data.right_sets.clone();
        right_sets.resize(data.left_sets.len(), vec![]);
        right_sets
    };

    // Then, we make sure both sets of vectors have exactly length equal to N:

    let left_sets = if left_sets.len() > N {
        left_sets[..N].to_vec()
    } else {
        let mut left_sets = left_sets;
        left_sets.resize(N, vec![]);
        left_sets
    };

    let right_sets = if right_sets.len() > N {
        right_sets[..N].to_vec()
    } else {
        let mut right_sets = right_sets;
        right_sets.resize(N, vec![]);
        right_sets
    };

    // Then we insert all elements of the (i-1)-th vector into the i-th vector:

    let left_sets = left_sets
        .iter()
        .enumerate()
        .map(|(i, x)| {
            if i == 0 {
                let mut x = x.clone();
                x.sort();
                x.dedup();
                x
            } else {
                let mut x = x.clone();
                x.extend(left_sets[i - 1].clone());
                x.sort();
                x.dedup();
                x
            }
        })
        .collect::<Vec<Vec<u32>>>();

    let right_sets = right_sets
        .iter()
        .enumerate()
        .map(|(i, x)| {
            if i == 0 {
                let mut x = x.clone();
                x.sort();
                x.dedup();
                x
            } else {
                let mut x = x.clone();
                x.extend(right_sets[i - 1].clone());
                x.sort();
                x.dedup();
                x
            }
        })
        .collect::<Vec<Vec<u32>>>();


    // Now, we can create the HyperLogLogArray:

    let left_array = HyperLogLogArray::<PRECISION, BITS, 2>::from(left_sets.as_slice());
    let right_array = HyperLogLogArray::<PRECISION, BITS, 2>::from(right_sets.as_slice());

    // We start with the first property: that the estimated exclusive overlap cardinalities
    // are correct for the given two vector sets:

    let overlap_cardinalities: [[f32; N]; N] =
        left_array.estimate_overlap_cardinalities(&right_array);

    // Secondly, we compute the estimated exclusive differences cardinalities:

    let left_difference_cardinalities: [f32; N] =
        left_array.estimated_difference_cardinality_vector(&right_array[N-1]);
    let right_difference_cardinalities: [f32; N] =
        right_array.estimated_difference_cardinality_vector(&left_array[N-1]);

    // Thirdly, we compute the estimated exclusive overlap and difference cardinalities at once:

    let (
        at_once_overlap_cardinalities,
        at_once_left_difference_cardinalities,
        at_once_right_difference_cardinalities,
    ): ([[f32; N]; N], [f32; N], [f32; N]) =
        left_array.estimated_overlap_and_differences_cardinality_matrices(&right_array);

    // To be extremely clear in the way we test this property, we will use the following
    // specific case for N==2, when we will implement the other cases in the fuzz harness
    // we will also test them separately.
    if N == 2 {
        // The first cell in the overlaps cardinalities matrix should be equal to the
        // intersection of the first vector in the left set and the first vector in theù
        // right set:
        let expected_intersection_cardinality = left_sets[0]
            .iter()
            .filter(|&x| right_sets[0].contains(x))
            .count() as f32;

        assert!(
            (overlap_cardinalities[0][0] - expected_intersection_cardinality).abs() < 0.1,
            concat!(
                "The estimated overlap cardinality of the first vector in the left set and the first vector in the right set is incorrect. ",
                "Expected: {}, got: {}. ",
                "The cardinality of the first vector ({:?}) in the left set is {}, ",
                "the cardinality of the first vector ({:?}) in the right set is {}. ",
                "The estimated cardinality of the first vector in the left set is {}, ",
                "the estimated cardinality of the first vector in the right set is {}"
            ),
            expected_intersection_cardinality,
            overlap_cardinalities[0][0],
            left_sets[0],
            left_sets[0].len(),
            right_sets[0],
            right_sets[0].len(),
            left_array[0].estimate_cardinality(),
            right_array[0].estimate_cardinality()
        );

        // The value in the position (0, 1) of the overlaps cardinalities matrix should be
        // equal to the intersection of the first vector in the left set and the second
        // vector in the right set, minus the intersection of the first vector in the left
        // set and the first vector in the right set:
        let expected_exclusive_overlaps_cardinality = left_sets[0]
            .iter()
            .filter(|&x| right_sets[1].contains(x))
            .count() as f32
            - expected_intersection_cardinality;

        assert!(
            (overlap_cardinalities[0][1] - expected_exclusive_overlaps_cardinality).abs() < 0.1,
            "The estimated exclusive overlap cardinality of the first vector in the left set and the second vector in the right set is incorrect"
        );

        // Similarly, the value in the position (1, 0) of the overlaps cardinalities matrix
        // should be equal to the intersection of the second vector in the left set and the
        // first vector in the right set, minus the intersection of the first vector in the
        // left set and the first vector in the right set:
        let expected_exclusive_overlaps_cardinality = left_sets[1]
            .iter()
            .filter(|&x| right_sets[0].contains(x))
            .count() as f32
            - expected_intersection_cardinality;

        assert!(
            (overlap_cardinalities[1][0] - expected_exclusive_overlaps_cardinality).abs() < 0.1,
            "The estimated exclusive overlap cardinality of the second vector in the left set and the first vector in the right set is incorrect"
        );

        // Finally, the value in the position (1, 1) of the overlaps cardinalities matrix
        // should be equal to the intersection of the second vector in the left set and the
        // second vector in the right set, minus the intersection of the first vector in the
        // left set and the first vector in the right set, minus the intersection of the
        // first vector in the left set and the second vector in the right set, plus the
        // intersection of the first vector in the left set and the first vector in the
        // right set:
        let expected_exclusive_overlaps_cardinality = left_sets[1]
            .iter()
            .filter(|&x| right_sets[1].contains(x))
            .count() as f32
            - expected_intersection_cardinality
            - expected_exclusive_overlaps_cardinality
            + expected_intersection_cardinality;

        assert!(
            (overlap_cardinalities[1][1] - expected_exclusive_overlaps_cardinality).abs() < 0.1,
            concat!(
                "The estimated exclusive overlap cardinality of the second vector in the left set and the second vector in the right set is incorrect. ",
                "Expected: {}, got: {}. ",
                "The cardinality of the second vector ({:?}) in the left set is {}, ",
                "the cardinality of the second vector ({:?}) in the right set is {}. ",
                "The estimated cardinality of the second vector in the left set is {}, ",
                "the estimated cardinality of the second vector in the right set is {}"
            ),
            expected_exclusive_overlaps_cardinality,
            overlap_cardinalities[1][1],
            left_sets[1],
            left_sets[1].len(),
            right_sets[1],
            right_sets[1].len(),
            left_array[1].estimate_cardinality(),
            right_array[1].estimate_cardinality()
        );

        // ================================================================================

        // Now, we test the second property: that the estimated exclusive differences
        // cardinalities are correct for the given two vector sets:

        // The first cell in the left difference cardinalities vector should be equal to the
        // difference between the first vector in the left set and the last vector, i.e. the largest one
        // in the right set:
        let expected_difference_left_cardinality = left_sets[0]
            .iter()
            .filter(|&x| !right_sets[N - 1].contains(x))
            .count() as f32;

        assert!(
            (left_difference_cardinalities[0] - expected_difference_left_cardinality).abs() < 0.1,
            concat!(
                "The estimated difference cardinality of the first vector in the left set and the last vector in the right set is incorrect",
                "Expected: {}, got: {}. ",
                "The cardinality of the first vector ({:?}) in the left set is {}, ",
                "the cardinality of the last vector ({:?}) in the right set is {}. ",
                "The estimated cardinality of the first vector in the left set is {}, ",
                "the estimated cardinality of the last vector in the right set is {}"
            ),
            expected_difference_left_cardinality,
            left_difference_cardinalities[0],
            left_sets[0],
            left_sets[0].len(),
            right_sets[N - 1],
            right_sets[N - 1].len(),
            left_array[0].estimate_cardinality(),
            right_array[N-1].estimate_cardinality()
        );

        // The first cell in the right difference cardinalities vector should be equal to the
        // difference between the first vector in the right set and the last vector, i.e. the largest one
        // in the left set:

        let expected_difference_right_cardinality = right_sets[0]
            .iter()
            .filter(|&x| !left_sets[N - 1].contains(x))
            .count() as f32;

        assert!(
            (right_difference_cardinalities[0] - expected_difference_right_cardinality).abs() < 0.1,
            concat!(
                "The estimated difference cardinality of the first vector in the right set and the last vector in the left set is incorrect. ",
                "Expected: {}, got: {}. ",
                "The cardinality of the first vector ({:?}) in the right set is {}, ",
                "the cardinality of the last vector ({:?}) in the left set is {}. ",
                "The estimated cardinality of the first vector in the right set is {}, ",
                "the estimated cardinality of the last vector in the left set is {}"
            ),
            expected_difference_right_cardinality,
            right_difference_cardinalities[0],
            right_sets[0],
            right_sets[0].len(),
            left_sets[N - 1],
            left_sets[N - 1].len(),
            right_array[0].estimate_cardinality(),
            left_array[N-1].estimate_cardinality()
        );

        // The second cell in the left difference cardinalities vector should be equal to the
        // difference between the second vector in the left set and the last vector, i.e. the largest one
        // in the right set, minus the difference between the first vector in the left set and the last vector, i.e. the largest one
        // in the right set:

        let expected_difference_left_cardinality = left_sets[1]
            .iter()
            .filter(|&x| !right_sets[N - 1].contains(x))
            .count() as f32
            - expected_difference_left_cardinality;

        assert!(
            (left_difference_cardinalities[1] - expected_difference_left_cardinality).abs() < 0.1,
            concat!(
                "The estimated difference cardinality of the second vector in the left set and the last vector in the right set is incorrect. ",
                "Expected: {}, got: {}. ",
                "The cardinality of the second vector ({:?}) in the left set is {}, ",
                "the cardinality of the last vector ({:?}) in the right set is {}. ",
                "The estimated cardinality of the second vector in the left set is {}, ",
                "the estimated cardinality of the last vector in the right set is {}"
            ),
            expected_difference_left_cardinality,
            left_difference_cardinalities[1],
            left_sets[1],
            left_sets[1].len(),
            right_sets[N - 1],
            right_sets[N - 1].len(),
            left_array[1].estimate_cardinality(),
            right_array[N-1].estimate_cardinality()
        );

        // The second cell in the right difference cardinalities vector should be equal to the
        // difference between the second vector in the right set and the last vector, i.e. the largest one
        // in the left set, minus the difference between the first vector in the right set and the last vector, i.e. the largest one
        // in the left set:

        let expected_difference_right_cardinality = right_sets[1]
            .iter()
            .filter(|&x| !left_sets[N - 1].contains(x))
            .count() as f32
            - expected_difference_right_cardinality;

        assert!(
            (right_difference_cardinalities[1] - expected_difference_right_cardinality).abs() < 0.1,
            concat!(
                "The estimated difference cardinality of the second vector in the right set and the last vector in the left set is incorrect. ",
                "Expected: {}, got: {}. ",
                "The cardinality of the second vector ({:?}) in the right set is {}, ",
                "the cardinality of the last vector ({:?}) in the left set is {}. ",
                "The estimated cardinality of the second vector in the right set is {}, ",
                "the estimated cardinality of the last vector in the left set is {}"
            ),
            expected_difference_right_cardinality,
            right_difference_cardinalities[1],
            right_sets[1],
            right_sets[1].len(),
            left_sets[N - 1],
            left_sets[N - 1].len(),
            right_array[1].estimate_cardinality(),
            left_array[N-1].estimate_cardinality()
        );

        // ================================================================================

        // Finally, we test the third property: that the method that produces at once the
        // overlap and difference cardinalities is consistent with the two previous methods.
        // We expect that the two methods match cell-by-cell within an epsilon.

        for i in 0..N {
            assert!(
                (at_once_left_difference_cardinalities[i] - left_difference_cardinalities[i]).abs()
                    < 0.1,
                "The estimated difference cardinality of the {}-th vector in the left set is inconsistent between the two methods",
                i
            );
            assert!(
                (at_once_right_difference_cardinalities[i]
                    - right_difference_cardinalities[i])
                    .abs()
                    < 0.1,
                "The estimated difference cardinality of the {}-th vector in the right set is inconsistent between the two methods",
                i
            );

            for j in 0..N {
                assert!(
                    (at_once_overlap_cardinalities[i][j] - overlap_cardinalities[i][j]).abs() < 0.1,
                    concat!(
                        "The estimated overlap cardinality of the {}-th vector in the left ",
                        "set and the {}-th vector in the right set is inconsistent between the two methods. ",
                        "Expected: {}, got: {}"
                    ),
                    i, j,
                    overlap_cardinalities[i][j],
                    at_once_overlap_cardinalities[i][j]
                );
            }
        }
    } else {
        unimplemented!("N != 2 is not yet implemented");
    }
});
