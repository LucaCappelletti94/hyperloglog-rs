//! Comparative test making sure that the quality of the estimation
//! obtain by the hyperloglog-rs library is equal or better than the
//! competitor libraries.
//! Evaluation of set-like properties across different data structures.
use crate::utils::*;
use core::f64;
use hyperloglog_rs::prelude::*;
use indicatif::ParallelProgressIterator;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use mem_dbg::MemSize;
use mem_dbg::SizeFlags;
use rayon::prelude::*;

pub(super) fn cardinality_comparatively<
    const EXPONENT: usize,
    P: mem_dbg::MemSize
        + Precision
        + ArrayRegister<Bits8>
        + ArrayRegister<Bits6>
        + ArrayRegister<Bits5>
        + ArrayRegister<Bits4>,
>()
where
    <P as ArrayRegister<Bits8>>::ArrayRegister: mem_dbg::MemSize + Words,
    <P as ArrayRegister<Bits6>>::ArrayRegister: mem_dbg::MemSize + Words,
    <P as ArrayRegister<Bits5>>::ArrayRegister: mem_dbg::MemSize + Words,
    <P as ArrayRegister<Bits4>>::ArrayRegister: mem_dbg::MemSize + Words,
    <P as hyperloglog_rs::prelude::Precision>::NumberOfRegisters: mem_dbg::MemSize,
{
    // If there is already a report stored, we can skip the evaluation.
    let path = format!(
        "./statistical_tests_reports/cardinality_{}.csv",
        P::EXPONENT
    );

    // If there is already a file at the expected path, we can skip the evaluation.
    let path = std::path::Path::new(&path);

    if path.exists() {
        println!("Skipping evaluation cardinality {}", P::EXPONENT);
        return;
    }

    println!("Running evaluation cardinality {}", P::EXPONENT);

    assert_eq!(P::EXPONENT, EXPONENT);
    let number_of_vectors = 2000;
    let minimum_sample_interval = 5;
    let maximum_sample_interval = 1000;
    let random_state = splitmix64(9516748163234878233_u64);

    let progress_bar = ProgressBar::new(number_of_vectors as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
            .unwrap()
            .progress_chars("##-"),
    );

    let (exact_cardinalities, mean_errors, memory_requirements): (
        Vec<f64>,
        Vec<Vec<f64>>,
        Vec<Vec<usize>>,
    ) = (0..number_of_vectors)
        .into_par_iter()
        .progress_with(progress_bar)
        .flat_map(|thread_number| {
            let random_state = splitmix64(splitmix64(random_state.wrapping_mul(thread_number + 1)));
            let mut exact_cardinalities = Vec::new();
            let mut all_memory_requirements = Vec::new();
            let mut all_mean_errors = Vec::new();

            let mut all_objects = SetLikeObjects::<EXPONENT, P>::all_cardinalities();

            let mut current_sample_rate = minimum_sample_interval;

            for (i, l) in iter_random_values(2_000_000, None, random_state).enumerate() {
                all_objects.iter_mut().for_each(|object| {
                    <SetLikeObjects<EXPONENT, P> as TestSetLike<u64>>::insert(object, &l)
                });

                if i % current_sample_rate == 0 {
                    if current_sample_rate < maximum_sample_interval {
                        current_sample_rate *= 2;
                    }

                    let (cardinalities, memory_requirements): (Vec<f64>, Vec<usize>) = all_objects
                        .iter()
                        .map(|object| {
                            let cardinality = object.cardinality();
                            let memory_requirement =
                                object.mem_size(SizeFlags::default() | SizeFlags::FOLLOW_REFS);
                            (cardinality, memory_requirement)
                        })
                        .unzip();

                    let mean_errors = cardinalities
                        .iter()
                        .map(|v| (v - cardinalities[0]).abs() / cardinalities[0])
                        .collect();

                    exact_cardinalities.push(cardinalities[0]);
                    all_memory_requirements.push(memory_requirements);
                    all_mean_errors.push(mean_errors);
                }
            }

            (
                exact_cardinalities,
                all_mean_errors,
                all_memory_requirements,
            )
        })
        .fold(
            || (Vec::new(), Vec::new(), Vec::new()),
            |(mut cardinalities, mut mean_errors, mut memory_requirements), (c, me, mr)| {
                cardinalities.push(c);
                mean_errors.push(me);
                memory_requirements.push(mr);
                (cardinalities, mean_errors, memory_requirements)
            },
        )
        .reduce(
            || (Vec::new(), Vec::new(), Vec::new()),
            |(mut cardinalities, mut mean_errors, mut memory_requirements), (c, me, mr)| {
                cardinalities.extend(c);
                mean_errors.extend(me);
                memory_requirements.extend(mr);
                (cardinalities, mean_errors, memory_requirements)
            },
        );

    let names = SetLikeObjects::<EXPONENT, P>::all_cardinalities()
        .iter()
        .map(|object| object.name())
        .collect::<Vec<_>>();

    statistical_report::<P>(
        &names,
        exact_cardinalities,
        mean_errors,
        memory_requirements,
        "cardinality",
    );
}
