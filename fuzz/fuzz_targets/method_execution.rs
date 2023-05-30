//! Fuzzing harness to test whether the cardinality estimation works as expected.
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

const BITS: usize = 4;
const PRECISION: usize = 4;
const N: usize = 2;

fuzz_target!(|data: FuzzCase| {
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

    // Now, we can create the HyperLogLogArray. In order to be able to detect
    // potential banal cases that involve hash collisions we need to populate
    // each counter in each array one by one, and test whether the i-th value
    // may be already contained in the counter. If so, we stop the test early
    // as it is a known limitation of the data structure and it is to be expected.

    let mut left_array: HyperLogLogArray<PRECISION, BITS, N> = HyperLogLogArray::new();
    let mut right_array: HyperLogLogArray<PRECISION, BITS, N> = HyperLogLogArray::new();

    for (i, set) in left_sets.iter().enumerate() {
        for item in set.iter() {
            left_array[i].insert(item);
        }
    }

    for (i, set) in right_sets.iter().enumerate() {
        for item in set.iter() {
            right_array[i].insert(item);
        }
    }

    // We start with the first property: that the estimated exclusive overlap cardinalities
    // are correct for the given two vector sets:

    let overlap_cardinalities: [[f32; N]; N] =
        left_array.estimate_overlap_cardinalities(&right_array);

    // Secondly, we compute the estimated exclusive differences cardinalities:

    let left_difference_cardinalities: [f32; N] =
        left_array.estimated_difference_cardinality_vector(&right_array[N - 1]);
    let right_difference_cardinalities: [f32; N] =
        right_array.estimated_difference_cardinality_vector(&left_array[N - 1]);

    // Thirdly, we compute the estimated exclusive overlap and difference cardinalities at once:

    let (
        at_once_overlap_cardinalities,
        at_once_left_difference_cardinalities,
        at_once_right_difference_cardinalities,
    ): ([[f32; N]; N], [f32; N], [f32; N]) =
        left_array.estimated_overlap_and_differences_cardinality_matrices(&right_array);
});
