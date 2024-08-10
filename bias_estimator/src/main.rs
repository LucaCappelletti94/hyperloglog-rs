//! Estimator for the optimal biases for HyperLogLog++ given a precision and number of bits.
use hyperloglog_rs::prelude::*;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use std::{collections::HashSet, usize};

fn number_of_threads() -> usize {
    std::env::var("RAYON_NUM_THREADS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| rayon::current_num_threads())
}

fn closest<F: FloatNumber>(value: F, values: &[F]) -> usize {
    debug_assert!(!values.is_empty());
    // We check in a debug assert that the values are sorted.
    debug_assert!(values.windows(2).all(|w| w[0] <= w[1]));
    // We can employ a binary search to find the position where
    // the value should be inserted to keep the array sorted, and
    // then we can return the closest centroid.
    let index = values.partition_point(|&x| x <= value);
    if index == 0 {
        0
    } else if index == values.len() {
        values.len() - 1
    } else {
        if (values[index] - value) < (value - values[index - 1]) {
            index
        } else {
            index - 1
        }
    }
}

/// Returns the N centroids of the KMeans clustering of the given iterator.
pub fn kmeans<const N: usize, F: FloatNumber>(
    incorrected_estimates: &mut [(F, F)],
    maximal_number_of_iterations: usize,
) -> [F; N] {
    println!("Clustering the estimates into {} clusters...", N);
    // We shuffle the values to avoid any bias in the initial centroids.
    incorrected_estimates.shuffle(&mut rand::thread_rng());

    // We display a loading bar to show the progress of the clustering, which
    // includes showing the current mean variation of the centroids.
    let loading_bar = indicatif::ProgressBar::new(maximal_number_of_iterations as u64);

    loading_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    // We initialize the centroids with the first N values.
    let mut centroids: [F; N] = [F::ZERO; N];

    for i in 0..N {
        centroids[i] = incorrected_estimates[i].0;
    }

    // We iterate until convergence.
    for iteration in 0..maximal_number_of_iterations {
        // We sort the centroids to make it easier to work with them.
        centroids.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

        // We assign each value to the closest centroid.
        let (sums, counts) = incorrected_estimates
            .par_iter()
            .fold(
                || (vec![F::ZERO; N], vec![F::ZERO; N]),
                |mut acc, value| {
                    let min_index = closest(value.0, &centroids);
                    acc.0[min_index] += value.0;
                    acc.1[min_index] += F::ONE;
                    acc
                },
            )
            .reduce(
                || (vec![F::ZERO; N], vec![F::ZERO; N]),
                |mut acc1, acc2| {
                    for i in 0..N {
                        acc1.0[i] += acc2.0[i];
                        acc1.1[i] += acc2.1[i];
                    }
                    acc1
                },
            );

        // We now compute the mean of the values assigned to each centroid.
        let mut new_centroids: [F; N] = [F::ZERO; N];

        for i in 0..N {
            if counts[i].is_zero() {
                continue;
            }
            new_centroids[i] = sums[i] / counts[i];
        }

        // We compute the new centroids.
        let mut total_variation = F::ZERO;
        for i in 0..N {
            total_variation += (new_centroids[i] - centroids[i]).abs();
            centroids[i] = new_centroids[i];
        }

        let mean_variation = total_variation / F::from_usize(N);

        if mean_variation <= F::from_f64(f64::EPSILON) {
            println!("Converged after {} iterations", iteration);
            break;
        }

        loading_bar.set_message(format!("Mean variation: {:.6}", mean_variation));
        loading_bar.inc(1);
    }

    // We sort the centroids to make it easier to work with them.
    centroids.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

    centroids
}

fn number_of_samples_from_precision<P: Precision>() -> usize {
    match P::EXPONENT {
        4 => 50_000_000,
        5 => 10_000_000,
        6 => 10_000_000,
        7 => 10_000_000,
        8 => 50_000_000,
        9 => 32_000,
        10 => 64_000,
        11 => 128_000,
        12 => 256_000,
        13 => 512_000,
        14 => 1_024_000,
        15 => 2_048_000,
        16 => 4_096_000,
        _ => unreachable!(),
    }
}

pub fn estimate_biases<
    const N: usize,
    P: Precision + PrecisionConstants<F> + ArrayRegister<B>,
    B: Bits,
    H: HasherType,
    F: FloatNumber,
>(
    mut random_state: u64,
) {
    println!(
        "Estimating the biases for precision {}, {} bits and hasher {:?}...",
        P::EXPONENT,
        B::NUMBER_OF_BITS,
        std::any::type_name::<H>()
    );

    let maximal_cardinality_in_estimation = P::ESTIMATES[P::ESTIMATES.len() - 1].to_usize();
    let number_of_samples_to_collect = number_of_samples_from_precision::<P>();

    let mut incorrected_estimates: Vec<(F, F)> = Vec::with_capacity(number_of_samples_to_collect);
    let mut sets =
        vec![HashSet::with_capacity(maximal_cardinality_in_estimation); number_of_threads()];
    let mut hlls = vec![
        HyperLogLog::<P, B, <P as ArrayRegister<B>>::ArrayRegister, H>::default();
        number_of_threads()
    ];

    let loading_bar = indicatif::ProgressBar::new(number_of_samples_to_collect as u64);

    loading_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    println!(
        "Collecting {} samples to estimate biases...",
        number_of_samples_to_collect
    );
    random_state = splitmix64(random_state);
    incorrected_estimates.extend(
        sets.par_iter_mut()
            .zip(hlls.par_iter_mut())
            .enumerate()
            .flat_map(|(thread_number, (set, hll))| {
                let mut random_state = random_state
                    .wrapping_mul((thread_number as u64 + 1).wrapping_mul(46354758698789));
                let mut collected_estimates =
                    Vec::with_capacity(number_of_samples_to_collect / number_of_threads());
                while collected_estimates.len() < number_of_samples_to_collect / number_of_threads()
                {
                    random_state = splitmix64(splitmix64(random_state));
                    for value in iter_random_values(usize::MAX, None, random_state) {
                        hll.insert(value);
                        set.insert(value);

                        // We calculate the cardinality estimate without any bias correction.
                        let incorrected_estimate = hll.estimate_incorrected_cardinality::<F>();

                        if incorrected_estimate > F::FIVE * P::NUMBER_OF_REGISTERS_FLOAT {
                            // If the estimate is larger than 5 * m, we skip the current iteration.
                            break;
                        }

                        // We compute by how much the estimate is off.
                        let error: F = F::from_usize(set.len()) - incorrected_estimate;

                        // And we store it for later use.
                        collected_estimates.push((incorrected_estimate, error));
                    }
                    set.clear();
                    hll.clear();
                }
                collected_estimates
            })
            .collect::<Vec<(F, F)>>(),
    );
    println!("Collected {} samples.", incorrected_estimates.len());

    // We identify the largest and smallest of the collected estimates.
    let (min, max) = incorrected_estimates.par_iter().copied().reduce(
        || (F::INFINITY, F::NEG_INFINITY),
        |(min, max), (estimate, _)| {
            (
                if estimate < min { estimate } else { min },
                if estimate > max { estimate } else { max },
            )
        },
    );

    // We compute the cutoff point for the zeros.
    let bias_correction_upper_cutoff = P::NUMBER_OF_REGISTERS * 5;
    let bias_correction_lower_cutoff = P::small_correction(P::LINEAR_COUNT_ZEROS);

    println!("Min estimate: {:.6}, Max estimate: {:.6}", min, max);
    println!(
        "Bias correction cutoffS: {} and {}",
        bias_correction_lower_cutoff, bias_correction_upper_cutoff
    );
    println!(
        "Original estimates min: {}, max: {}",
        P::ESTIMATES[0],
        P::ESTIMATES[P::ESTIMATES.len() - 1]
    );

    // Now that we have collected enough samples, we can cluster the estimates into N clusters.
    let centroids = kmeans::<N, F>(&mut incorrected_estimates, 3_000);

    // We can now minimize the centroids, meaning we proceed to drop the zero centroids and
    // the duplicates.
    let mut centroids = centroids
        .iter()
        .copied()
        .filter(|&x| !x.is_zero())
        .collect::<Vec<F>>();
    centroids.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    centroids.dedup();

    assert!(centroids.len() > 1);

    let k = centroids.len();

    let (sum, counts) = incorrected_estimates
        .into_par_iter()
        .fold(
            || (vec![F::ZERO; k], vec![F::ZERO; k]),
            |mut acc, (estimate, error)| {
                let min_index = closest(estimate, &centroids);
                acc.0[min_index] += error;
                acc.1[min_index] += F::ONE;
                acc
            },
        )
        .reduce(
            || (vec![F::ZERO; k], vec![F::ZERO; k]),
            |mut acc1, acc2| {
                for i in 0..k {
                    acc1.0[i] += acc2.0[i];
                    acc1.1[i] += acc2.1[i];
                }
                acc1
            },
        );

    let biases = sum
        .iter()
        .zip(counts.iter())
        .map(|(&sum, &count)| {
            -if count.is_zero() {
                F::ZERO
            } else {
                sum / count
            }
        })
        .collect::<Vec<F>>();

    assert_eq!(centroids.len(), biases.len());

    evaluate::<F, P, B, H>(random_state, &centroids, &biases);

    // We store to a json document the biases and centroids.
    // {"biases": [...], "centroids": [...]}
    let biases = biases.iter().map(|x| x.to_f64()).collect::<Vec<f64>>();
    let centroids = centroids.iter().map(|x| x.to_f64()).collect::<Vec<f64>>();
    let json = serde_json::json!({ "biases": biases, "centroids": centroids });
    let json = serde_json::to_string_pretty(&json).unwrap();
    std::fs::write("biases.json", json).unwrap();
}

fn evaluate<
    F: FloatNumber,
    P: Precision + ArrayRegister<B> + PrecisionConstants<F>,
    B: Bits,
    H: HasherType,
>(
    random_state: u64,
    new_estimates: &[F],
    new_biases: &[F],
) {
    println!("Evaluating the new biases...");
    assert_eq!(new_estimates.len(), new_biases.len());

    let random_state = splitmix64(splitmix64(random_state));
    let mut sets = vec![HashSet::new(); number_of_threads()];

    let number_of_samples_to_collect =
        F::from_usize(number_of_samples_from_precision::<P>() / number_of_threads());
    let mut hlls = vec![
        HyperLogLog::<P, B, <P as ArrayRegister<B>>::ArrayRegister, H>::default();
        number_of_threads()
    ];

    let (total_incorrected_error, total_original_corrected_error, total_new_corrected_error) = sets
        .par_iter_mut()
        .zip(hlls.par_iter_mut())
        .enumerate()
        .map(|(thread_number, (set, hll))| {
            let mut random_state = splitmix64(splitmix64(
                random_state.wrapping_mul((thread_number as u64 + 1).wrapping_mul(46354758698789)),
            ));
            let mut total_incorrected_error = F::ZERO;
            let mut total_original_corrected_error = F::ZERO;
            let mut total_new_corrected_error = F::ZERO;
            let mut number_of_collected_estimates = F::ZERO;
            while number_of_collected_estimates < number_of_samples_to_collect {
                random_state = splitmix64(random_state);
                for value in iter_random_values(1_000_000, None, random_state) {
                    hll.insert(&value);
                    set.insert(value);

                    // We calculate the cardinality estimate without any bias correction.
                    let incorrected_estimate = hll.estimate_incorrected_cardinality::<F>();

                    if incorrected_estimate > F::FIVE * P::NUMBER_OF_REGISTERS_FLOAT {
                        // If the estimate is larger than 5 * m, we skip the current iteration.
                        break;
                    }

                    // We calculate the absolute error rate of the incorrected estimate.
                    total_incorrected_error += (F::from_usize(set.len()) - incorrected_estimate)
                        .abs()
                        / F::from_usize(set.len());
                    // We calculate the absolute error rate of the original corrected estimate.
                    total_original_corrected_error +=
                        (F::from_usize(set.len()) - hll.estimate_cardinality()).abs()
                            / F::from_usize(set.len());
                    // We calculate the absolute error rate of the new corrected estimate.
                    total_new_corrected_error += (F::from_usize(set.len())
                        - hll.estimate_cardinality_with_biases(new_estimates, new_biases))
                    .abs()
                        / F::from_usize(set.len());

                    number_of_collected_estimates += F::ONE;
                }
                set.clear();
                hll.clear();
            }
            (
                total_incorrected_error / number_of_collected_estimates,
                total_original_corrected_error / number_of_collected_estimates,
                total_new_corrected_error / number_of_collected_estimates,
            )
        })
        .reduce(
            || (F::ZERO, F::ZERO, F::ZERO),
            |acc1, acc2| (acc1.0 + acc2.0, acc1.1 + acc2.1, acc1.2 + acc2.2),
        );

    let mean_incorrected_error = total_incorrected_error / F::from_usize(number_of_threads());
    let mean_original_corrected_error =
        total_original_corrected_error / F::from_usize(number_of_threads());
    let mean_new_corrected_error = total_new_corrected_error / F::from_usize(number_of_threads());

    println!(
        "Mean incorrected error: {:.6}, {:.4} of original corrected",
        mean_incorrected_error,
        mean_incorrected_error / mean_original_corrected_error
    );
    println!(
        "Mean original corrected error: {:.6}, <={:.4} expected",
        mean_original_corrected_error,
        P::error_rate()
    );
    println!(
        "Mean new corrected error: {:.6}, {:.4} of original corrected",
        mean_new_corrected_error,
        mean_new_corrected_error / mean_original_corrected_error
    );
}

fn main() {
    let random_state = 586354872369_u64;
    estimate_biases::<200, Precision8, Bits6, twox_hash::XxHash64, f64>(random_state);
    // estimate_biases::<200, Precision8, Bits4, wyhash::WyHash, f64>(random_state);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kmeans() {
        let mut incorrected_estimates = vec![
            (1.0, 0.0),
            (1.1, 0.0),
            (0.9, 0.0),
            (10.0, 0.0),
            (10.1, 0.0),
            (9.9, 0.0),
            (100.0, 0.0),
            (100.1, 0.0),
            (99.9, 0.0),
        ];
        let centroids = kmeans::<3, f64>(&mut incorrected_estimates, 100);
        assert_eq!(centroids, [1.0, 10.0, 100.0]);
    }
}
