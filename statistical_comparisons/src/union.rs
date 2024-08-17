//! Comparative test making sure that the quality of the estimation
//! obtain by the hyperloglog-rs library is equal or better than the
//! competitor libraries.
//! Evaluation of set-like properties across different data structures.
use crate::utils::*;
use hyperloglog_rs::prelude::*;

use core::f64;
use indicatif::ParallelProgressIterator;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use mem_dbg::MemSize;
use mem_dbg::SizeFlags;
use rayon::prelude::*;

pub(super) fn union_comparatively<
    const EXPONENT: usize,
    P: mem_dbg::MemSize
        + Precision
        + ArrayRegister<Bits8>
        + ArrayRegister<Bits6>
        + ArrayRegister<Bits5>
        + ArrayRegister<Bits4>
       
        + ArrayRegister<Bits5>,
>()
where
    <P as ArrayRegister<Bits8>>::ArrayRegister: mem_dbg::MemSize,
    <P as ArrayRegister<Bits6>>::ArrayRegister: mem_dbg::MemSize,
    <P as ArrayRegister<Bits5>>::ArrayRegister: mem_dbg::MemSize,
    <P as ArrayRegister<Bits4>>::ArrayRegister: mem_dbg::MemSize,
    <P as hyperloglog_rs::prelude::Precision>::NumberOfRegisters: mem_dbg::MemSize,
{
    // If there is already a report stored, we can skip the evaluation.
    let path = format!("./statistical_tests_reports/union_{}.csv", P::EXPONENT);

    // If there is already a file at the expected path, we can skip the evaluation.
    let path = std::path::Path::new(&path);

    if path.exists() {
        println!("Skipping evaluation union {}", P::EXPONENT);
        return;
    }

    println!("Running evaluation union {}", P::EXPONENT);

    assert_eq!(P::EXPONENT, EXPONENT);
    let number_of_vectors = 3_00;
    let minimum_number_of_samples = 5;
    let maximum_number_of_samples = 2000;
    let left_random_state = splitmix64(6516781878233_u64);
    let right_random_state = splitmix64(497635734233_u64);

    let progress_bar = ProgressBar::new(number_of_vectors as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
            .unwrap()
            .progress_chars("##-"),
    );

    let (unions, mean_errors, memory_requirements): (Vec<f64>, Vec<Vec<f64>>, Vec<Vec<usize>>) = (0
        ..number_of_vectors)
        .into_par_iter()
        .progress_with(progress_bar)
        .flat_map(|thread_number| {
            let left_random_state = splitmix64(splitmix64(
                left_random_state.wrapping_mul(thread_number + 1),
            ));
            let right_random_state = splitmix64(splitmix64(
                right_random_state.wrapping_mul(thread_number + 1),
            ));
            let mut exact_unions: Vec<f64> = Vec::new();
            let mut all_memory_requirements: Vec<Vec<usize>> = Vec::new();
            let mut all_mean_errors: Vec<Vec<f64>> = Vec::new();

            let mut left_objects = SetLikeObjects::<EXPONENT, P>::all_union();
            let mut right_objects = SetLikeObjects::<EXPONENT, P>::all_union();
            let mut left_iter = iter_random_values(2_000_000, Some(1_000_000), left_random_state);
            let mut right_iter = iter_random_values(2_000_000, Some(1_000_000), right_random_state);

            let mut current_sample_rate = minimum_number_of_samples;

            let mut i = 0;

            loop {
                let mut new_object = false;
                if let Some(left) = left_iter.next() {
                    left_objects.iter_mut().for_each(|object| {
                        <SetLikeObjects<EXPONENT, P> as TestSetLike<u64>>::insert(object, &left)
                    });
                    new_object = true;
                }
                if let Some(right) = right_iter.next() {
                    right_objects.iter_mut().for_each(|object| {
                        <SetLikeObjects<EXPONENT, P> as TestSetLike<u64>>::insert(object, &right)
                    });
                    new_object = true;
                }
                if !new_object {
                    break;
                }

                if i % current_sample_rate == 0 {
                    if current_sample_rate < maximum_number_of_samples {
                        current_sample_rate *= 2;
                    }

                    let (cardinalities, memory_requirements): (Vec<f64>, Vec<usize>) = left_objects
                        .iter()
                        .zip(right_objects.iter())
                        .map(|(left, right)| {
                            let cardinality = left.union(right);
                            let memory_requirement = left
                                .mem_size(SizeFlags::default() | SizeFlags::FOLLOW_REFS)
                                + right.mem_size(SizeFlags::default() | SizeFlags::FOLLOW_REFS);
                            (cardinality, memory_requirement)
                        })
                        .unzip();

                    let mean_errors = cardinalities
                        .iter()
                        .map(|v| (v - cardinalities[0]).abs() / cardinalities[0])
                        .collect();

                    exact_unions.push(cardinalities[0]);
                    all_memory_requirements.push(memory_requirements);
                    all_mean_errors.push(mean_errors);
                }

                i += 1;
            }

            (exact_unions, all_mean_errors, all_memory_requirements)
        })
        .fold(
            || (Vec::new(), Vec::new(), Vec::new()),
            |(mut unions, mut mean_errors, mut memory_requirements), (u, me, mr)| {
                unions.push(u);
                mean_errors.push(me);
                memory_requirements.push(mr);
                (unions, mean_errors, memory_requirements)
            },
        )
        .reduce(
            || (Vec::new(), Vec::new(), Vec::new()),
            |(mut unions, mut mean_errors, mut memory_requirements), (u, me, mr)| {
                unions.extend(u);
                mean_errors.extend(me);
                memory_requirements.extend(mr);
                (unions, mean_errors, memory_requirements)
            },
        );

    let names = SetLikeObjects::<EXPONENT, P>::all_union()
        .iter()
        .map(|object| object.name())
        .collect::<Vec<_>>();

    statistical_report::<P>(&names, unions, mean_errors, memory_requirements, "union");
}
