//! In this test suite, we evaluate the error rate we can obtain with the
//! MLE approach and how does it compare to the error rate we obtain with
//! the tradictional HLL approach. We also keep track of the time requirements
//! of the traditional HLL approach and the MLE approach.
//!
//! We generate random vectors and test two different procedures:
//!
//! 1) Estimating the cardinality of a set derived from the random vector.
//! 2) Estimating the cardinality of the intersection of two sets derived from the random vectors.
//!

use hyperloglog_rs::prelude::*;
use indicatif::ParallelProgressIterator;
use indicatif::ProgressBar;
use rand::Rng;
use rand::SeedableRng;
use rayon::prelude::*;
use std::collections::HashSet;

/// Returns a random vector and set of size `size` and with random state `random_state`.
///
/// # Arguments
/// * `size` - The size of the vector and set.
/// * `random_state` - The random state used to generate the random vector and set.
fn get_random_vector(size: usize, random_state: u64) -> HashSet<u64> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(random_state);
    let mut set = HashSet::new();

    // We randomize the maximal size
    let max_size = rng.gen_range(0..size);

    // We randomize the maximal vocabulary entry
    let max_value = rng.gen_range(1..100_000);

    for _ in 0..max_size {
        let value = rng.gen::<u64>() % max_value;
        set.insert(value);
    }
    set
}

fn evaluate_mle_cardinality_estimation<PRECISION: Precision + WordType<BITS>, const BITS: usize>(
    set: &HashSet<u64>,
) -> (f32, f32, f32, f32) {
    let hll = set
        .iter()
        .copied()
        .collect::<HyperLogLog<PRECISION, BITS>>();
    let start_time_hll = std::time::Instant::now();
    let hll_cardinality = hll.estimate_cardinality();
    let hll_time = start_time_hll.elapsed().as_secs_f32();
    let start_time_mle = std::time::Instant::now();
    let mle_hll: &MLE<4, HyperLogLog<PRECISION, BITS>> = hll.as_ref();
    let mle_cardinality = mle_hll.estimate_cardinality();
    let mle_time = start_time_mle.elapsed().as_secs_f32();
    let hll_error = (set.len() as f32 - hll_cardinality).powi(2) / set.len() as f32;
    let mle_error = (set.len() as f32 - mle_cardinality).powi(2) / set.len() as f32;
    (hll_error, mle_error, hll_time, mle_time)
}

fn evaluate_mle_intersection_estimation<
    PRECISION: Precision + WordType<BITS>,
    const BITS: usize,
>(
    left_set: &HashSet<u64>,
    right_set: &HashSet<u64>,
) -> (f32, f32, f32, f32) {
    let left_hll = left_set
        .iter()
        .copied()
        .collect::<HyperLogLog<PRECISION, BITS>>();
    let right_hll = right_set
        .iter()
        .copied()
        .collect::<HyperLogLog<PRECISION, BITS>>();
    let start_time_hll = std::time::Instant::now();
    let hll_cardinality: f32 = left_hll.estimate_intersection_cardinality(&right_hll);
    let hll_time = start_time_hll.elapsed().as_secs_f32();
    let left_mle_hll: MLE<4, HyperLogLog<PRECISION, BITS>> = left_hll.into();
    let right_mle_hll: MLE<4, HyperLogLog<PRECISION, BITS>> = right_hll.into();
    let start_time_mle = std::time::Instant::now();
    let mle_cardinality: f32 = left_mle_hll.estimate_intersection_cardinality(&right_mle_hll);
    let mle_time = start_time_mle.elapsed().as_secs_f32();
    let true_intersection = left_set.intersection(&right_set).count() as f32;
    let hll_error = (true_intersection - hll_cardinality).powi(2);
    let mle_error = (true_intersection - mle_cardinality).powi(2);
    (hll_error, mle_error, hll_time, mle_time)
}

// #[test]
/// Test to evaluate the comparative error rate of the MLE approach and the HLL approach.
// fn test_mle_cardinality_estimation() {
//     let number_of_tests: usize = 100;
//     let size: usize = 100_000;
//     let progress_bar = ProgressBar::new(number_of_tests as u64);

//     let (hll_error, mle_error, error_rate, time_rate) = (0..number_of_tests)
//         .into_par_iter()
//         .progress_with(progress_bar)
//         .map(|i| {
//             let (vector, set) = get_random_vector(size, i as u64);
//             let mut total_hll_error = 0.0;
//             let mut total_mle_error = 0.0;
//             let mut total_error_rate = 0.0;
//             let mut total_time_rate = 0.0;
//             // let (hll_error_04, mle_error_04, hll_time, mle_time) =
//             //     evaluate_mle_cardinality_estimation::<Precision4, 6>(&vector, &set);
//             // total_hll_error += hll_error_04;
//             // total_mle_error += mle_error_04;
//             // total_error_rate += hll_error_04 / mle_error_04;
//             // total_time_rate += hll_time / mle_time;

//             // let (hll_error_05, mle_error_05, hll_time, mle_time) =
//             //     evaluate_mle_cardinality_estimation::<Precision5, 6>(&vector, &set);
//             // total_hll_error += hll_error_05;
//             // total_mle_error += mle_error_05;
//             // total_error_rate += hll_error_05 / mle_error_05;
//             // total_time_rate += hll_time / mle_time;

//             // let (hll_error_06, mle_error_06, hll_time, mle_time) =
//             //     evaluate_mle_cardinality_estimation::<Precision6, 6>(&vector, &set);
//             // total_hll_error += hll_error_06;
//             // total_mle_error += mle_error_06;
//             // total_error_rate += hll_error_06 / mle_error_06;
//             // total_time_rate += hll_time / mle_time;

//             // let (hll_error_07, mle_error_07, hll_time, mle_time) =
//             //     evaluate_mle_cardinality_estimation::<Precision7, 6>(&vector, &set);
//             // total_hll_error += hll_error_07;
//             // total_mle_error += mle_error_07;
//             // total_error_rate += hll_error_07 / mle_error_07;
//             // total_time_rate += hll_time / mle_time;

//             // let (hll_error_08, mle_error_08, hll_time, mle_time) =
//             //     evaluate_mle_cardinality_estimation::<Precision8, 6>(&vector, &set);
//             // total_hll_error += hll_error_08;
//             // total_mle_error += mle_error_08;
//             // total_error_rate += hll_error_08 / mle_error_08;
//             // total_time_rate += hll_time / mle_time;

//             let (hll_error_09, mle_error_09, hll_time, mle_time) =
//                 evaluate_mle_cardinality_estimation::<Precision9, 6>(&vector, &set);
//             total_hll_error += hll_error_09;
//             total_mle_error += mle_error_09;
//             total_error_rate += hll_error_09 / mle_error_09;
//             total_time_rate += hll_time / mle_time;

//             let (hll_error_10, mle_error_10, hll_time, mle_time) =
//                 evaluate_mle_cardinality_estimation::<Precision10, 6>(&vector, &set);
//             total_hll_error += hll_error_10;
//             total_mle_error += mle_error_10;
//             total_error_rate += hll_error_10 / mle_error_10;
//             total_time_rate += hll_time / mle_time;

//             let (hll_error_11, mle_error_11, hll_time, mle_time) =
//                 evaluate_mle_cardinality_estimation::<Precision11, 6>(&vector, &set);
//             total_hll_error += hll_error_11;
//             total_mle_error += mle_error_11;
//             total_error_rate += hll_error_11 / mle_error_11;
//             total_time_rate += hll_time / mle_time;

//             let (hll_error_12, mle_error_12, hll_time, mle_time) =
//                 evaluate_mle_cardinality_estimation::<Precision12, 6>(&vector, &set);
//             total_hll_error += hll_error_12;
//             total_mle_error += mle_error_12;
//             total_error_rate += hll_error_12 / mle_error_12;
//             total_time_rate += hll_time / mle_time;

//             let (hll_error_13, mle_error_13, hll_time, mle_time) =
//                 evaluate_mle_cardinality_estimation::<Precision13, 6>(&vector, &set);
//             total_hll_error += hll_error_13;
//             total_mle_error += mle_error_13;
//             total_error_rate += hll_error_13 / mle_error_13;
//             total_time_rate += hll_time / mle_time;

//             let (hll_error_14, mle_error_14, hll_time, mle_time) =
//                 evaluate_mle_cardinality_estimation::<Precision14, 6>(&vector, &set);
//             total_hll_error += hll_error_14;
//             total_mle_error += mle_error_14;
//             total_error_rate += hll_error_14 / mle_error_14;
//             total_time_rate += hll_time / mle_time;

//             let (hll_error_15, mle_error_15, hll_time, mle_time) =
//                 evaluate_mle_cardinality_estimation::<Precision15, 6>(&vector, &set);
//             total_hll_error += hll_error_15;
//             total_mle_error += mle_error_15;
//             total_error_rate += hll_error_15 / mle_error_15;
//             total_time_rate += hll_time / mle_time;

//             let (hll_error_16, mle_error_16, hll_time, mle_time) =
//                 evaluate_mle_cardinality_estimation::<Precision16, 6>(&vector, &set);
//             total_hll_error += hll_error_16;
//             total_mle_error += mle_error_16;
//             total_error_rate += hll_error_16 / mle_error_16;
//             total_time_rate += hll_time / mle_time;

//             (
//                 total_hll_error,
//                 total_mle_error,
//                 total_error_rate,
//                 total_time_rate,
//             )
//         })
//         .reduce(
//             || (0.0, 0.0, 0.0, 0.0),
//             |a, b| (a.0 + b.0, a.1 + b.1, a.2 + b.2, a.3 + b.3),
//         );

//     println!(
//         "HLL error rate: {}, MLE error rate: {}, Error rate: {}, Time rate: {}",
//         hll_error / number_of_tests as f32,
//         mle_error / number_of_tests as f32,
//         error_rate / number_of_tests as f32,
//         time_rate / number_of_tests as f32
//     );
// }

#[test]
/// Test to evaluate the comparative error rate of the MLE approach and the HLL approach.
fn test_mle_intersection_estimation() {
    let number_of_tests: usize = 10_000;
    let size: usize = 100_000;
    let progress_bar = ProgressBar::new(number_of_tests as u64);

    let (hll_error, mle_error, hll_time, mle_time) = (0..number_of_tests)
        .into_par_iter()
        .progress_with(progress_bar)
        .map(|i| {
            let left_set = get_random_vector(size, i as u64);
            let right_set = get_random_vector(size, (i as u64 + 1).wrapping_mul(2736542));

            let (hll_error, mle_error, hll_time, mle_time) =
                evaluate_mle_intersection_estimation::<Precision12, 6>(&left_set, &right_set);
            (
                hll_error,
                mle_error,
                hll_time,
                mle_time
            )
        })
        .reduce(
            || (0.0, 0.0, 0.0, 0.0),
            |a, b| (a.0 + b.0, a.1 + b.1, a.2 + b.2, a.3 + b.3),
        );

    println!(
        "HLL MSE: {}, MLE MSE: {}, Error rate: {}, Time rate: {}",
        hll_error / number_of_tests as f32,
        mle_error / number_of_tests as f32,
        hll_error / mle_error as f32,
        hll_time / mle_time as f32
    );
}
