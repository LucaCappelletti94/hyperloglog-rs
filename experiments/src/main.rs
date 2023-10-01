//! Experiment to tune the 'ingredients' of the union of the HyperLogLog.
//!
//!
use core::ops::Add;
use hyperloglog_rs::prelude::*;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use rayon::prelude::*;
use serde_json;
use std::collections::HashSet;
use vec_rand::{random_f32, splitmix64, xorshift};

struct Sample {
    union_cardinality: usize,
    approximation_left_cardinality: usize,
    approximation_right_cardinality: usize,
    approximation_union_cardinality: usize,
    left_number_of_zeros_rate: f32,
    right_number_of_zeros_rate: f32,
}

impl Sample {
    fn from_vecs<PRECISION: Precision + WordType<BITS>, const BITS: usize>(
        left: &[u32],
        right: &[u32],
    ) -> Self {
        let hll1: HyperLogLog<PRECISION, BITS> = left.iter().collect();
        let hll2: HyperLogLog<PRECISION, BITS> = right.iter().collect();
        let set1: HashSet<u32> = left.iter().copied().collect();
        let set2: HashSet<u32> = right.iter().copied().collect();

        Sample {
            // left_cardinality: set1.len(),
            // right_cardinality: set2.len(),
            union_cardinality: set1.union(&set2).count(),
            approximation_left_cardinality: hll1.estimate_cardinality().round() as usize,
            approximation_right_cardinality: hll2.estimate_cardinality().round() as usize,
            approximation_union_cardinality: hll1
                .estimate_union_and_sets_cardinality(&hll2)
                .get_union_cardinality(),
            left_number_of_zeros_rate: hll1.get_number_of_zero_registers() as f32
                / PRECISION::NUMBER_OF_REGISTERS as f32,
            right_number_of_zeros_rate: hll2.get_number_of_zero_registers() as f32
                / PRECISION::NUMBER_OF_REGISTERS as f32,
        }
    }

    fn as_array(&self) -> [f32; 5] {
        [
            self.approximation_left_cardinality as f32,
            self.approximation_right_cardinality as f32,
            self.approximation_union_cardinality as f32,
            self.left_number_of_zeros_rate,
            self.right_number_of_zeros_rate,
        ]
    }

    fn get_prediction_squared_error(&self, prediction: f32) -> f32 {
        (self.union_cardinality as f32 - prediction).powi(2)
    }

    fn get_prediction_squared_error_derivative(&self, prediction: f32) -> f32 {
        2.0 * (self.union_cardinality as f32 - prediction)
    }

    fn get_hyperloglog_squared_error(&self) -> f32 {
        self.get_prediction_squared_error(self.approximation_union_cardinality as f32)
    }
}

fn generate_sample<PRECISION: Precision + WordType<BITS>, const BITS: usize>(
    mut random_state: u64,
) -> Sample {
    let first_set_cardinality = xorshift(random_state) % 100_000;
    random_state = splitmix64(random_state);
    let second_set_cardinality = xorshift(random_state) % 100_000;
    random_state = splitmix64(random_state);
    let first_world_size = xorshift(random_state) % 100_000;
    random_state = splitmix64(random_state);
    let second_world_size = xorshift(random_state) % 100_000;
    random_state = splitmix64(random_state);

    let mut vec1: Vec<u32> = Vec::with_capacity(first_set_cardinality as usize);
    let mut vec2: Vec<u32> = Vec::with_capacity(second_set_cardinality as usize);

    for _ in 0..first_set_cardinality {
        let value = if first_world_size > 0 {
            xorshift(random_state) % first_world_size
        } else {
            0
        };
        vec1.push(value as u32);
        random_state = splitmix64(random_state);
    }

    for _ in 0..second_set_cardinality {
        let value = if second_world_size > 0 {
            xorshift(random_state) % second_world_size
        } else {
            0
        };
        vec2.push(value as u32);
        random_state = splitmix64(random_state);
    }

    Sample::from_vecs::<PRECISION, BITS>(vec1.as_slice(), vec2.as_slice())
}

fn generate_samples<PRECISION: Precision + WordType<BITS>, const BITS: usize>(
    number_of_samples: usize,
    random_state: u64,
) -> Vec<Sample> {
    let mut samples = Vec::with_capacity(number_of_samples);

    (0..number_of_samples)
        .into_par_iter()
        .map(|i| {
            generate_sample::<PRECISION, BITS>(splitmix64(random_state.wrapping_mul(i as u64 + 1)))
        })
        .collect_into_vec(&mut samples);

    samples
}

struct Adam<const N: usize> {
    first_moments: [f32; N],
    second_moments: [f32; N],
    time: i32,
    learning_rate: f32,
    first_order_decay_factor: f32,
    second_order_decay_factor: f32,
}

impl<const N: usize> Default for Adam<N> {
    fn default() -> Self {
        Adam {
            first_moments: [0.0; N],
            second_moments: [0.0; N],
            time: 0,
            learning_rate: 0.001,
            first_order_decay_factor: 0.9,
            second_order_decay_factor: 0.999,
        }
    }
}

impl<const N: usize> Adam<N> {
    fn update(&mut self, mut gradients: [f32; N]) -> [f32; N] {
        self.first_moments
            .iter_mut()
            .zip(self.second_moments.iter_mut())
            .zip(gradients.iter_mut())
            .for_each(|((first_moment, second_moment), gradient)| {
                *first_moment = self.first_order_decay_factor * *first_moment
                    + (1.0 - self.first_order_decay_factor) * *gradient;
                *second_moment = self.second_order_decay_factor * *second_moment
                    + (1.0 - self.second_order_decay_factor) * *gradient * *gradient;
                *gradient = self.learning_rate * *first_moment
                    / (1.0 - self.first_order_decay_factor.powi(self.time + 1)).sqrt()
                    / (*second_moment
                        / (1.0 - self.second_order_decay_factor.powi(self.time + 1)).sqrt()
                        + 1e-8);
            });
        gradients
    }
}

struct EpochHistory {
    total_model_squared_error: f32,
    number_of_samples: usize,
    total_hyperloglog_squared_error: f32,
}

impl Default for EpochHistory {
    fn default() -> Self {
        Self {
            total_model_squared_error: 0.0,
            number_of_samples: 0,
            total_hyperloglog_squared_error: 0.0,
        }
    }
}

impl Add<Self> for EpochHistory {
    type Output = Self;

    fn add(self, other: EpochHistory) -> EpochHistory {
        Self {
            total_model_squared_error: self.total_model_squared_error
                + other.total_model_squared_error,
            number_of_samples: self.number_of_samples + other.number_of_samples,
            total_hyperloglog_squared_error: self.total_hyperloglog_squared_error
                + other.total_hyperloglog_squared_error,
        }
    }
}

impl EpochHistory {
    fn new(model_squared_error: f32, hyperloglog_squared_error: f32) -> Self {
        assert!(model_squared_error.is_finite());
        assert!(hyperloglog_squared_error.is_finite());
        Self {
            total_model_squared_error: model_squared_error,
            number_of_samples: 1,
            total_hyperloglog_squared_error: hyperloglog_squared_error,
        }
    }

    fn get_mean_squared_error(&self) -> f32 {
        self.total_model_squared_error / self.number_of_samples as f32
    }

    fn get_hyperloglog_mean_squared_error(&self) -> f32 {
        self.total_hyperloglog_squared_error / self.number_of_samples as f32
    }

    /// Rate of the model error divided by the HyperLogLog error.
    ///
    /// When we get an error ratio less than one, we will have model dominance.
    fn get_error_rate(&self) -> f32 {
        self.total_model_squared_error / self.total_hyperloglog_squared_error
    }

    fn get_csv_header() -> &'static str {
        "mse\tmse_hll\trate\n"
    }

    fn to_csv_line(&self) -> String {
        format!(
            "{}\t{}\t{}\n",
            self.get_mean_squared_error(),
            self.get_hyperloglog_mean_squared_error(),
            self.get_error_rate()
        )
    }
}

struct Dense<const N: usize> {
    weights: [f32; N],
    optimizer: Adam<N>,
}

/// Initialize the model with weights and bias in the range (-sqrt(6) / sqrt(k), +sqrt(6) / sqrt(k))
///
/// # Implementative details
/// The square root of 6 is roughly: 2.45
pub(crate) fn get_random_weight(random_state: u64, dimension_squared_root: f32) -> f32 {
    (2.0 * random_f32(splitmix64(random_state)) - 1.0) * (2.45 as f32) / dimension_squared_root
}

impl<const N: usize> Dense<N> {
    fn random(random_state: u64) -> Self {
        let mut weights = [0.0; N];
        let dimension_squared_root = (N as f32).sqrt();
        weights
            .iter_mut()
            .enumerate()
            .for_each(|(i, weight): (usize, &mut f32)| {
                *weight = get_random_weight(
                    random_state.wrapping_mul(i as u64 + 1),
                    dimension_squared_root,
                );
            });
        Self {
            weights,
            optimizer: Adam::default(),
        }
    }
}

impl Dense<5> {
    fn predict(&self, sample: &Sample) -> ([f32; 5], f32) {
        let sample_values: [f32; 5] = sample.as_array();
        let prediction = sample_values
            .iter()
            .zip(self.weights.iter())
            .map(|(sample, weight)| sample * weight)
            .sum::<f32>();
        (sample_values, prediction)
    }

    fn train_single_epoch(&mut self, samples: &[Sample]) -> EpochHistory {
        let (mut total_gradient, history): ([f32; 5], EpochHistory) = samples
            .par_iter()
            .map(|sample: &Sample| {
                let (mut input, prediction) = self.predict(sample);
                let squared_error = sample.get_prediction_squared_error(prediction);
                let squared_error_derivative = sample.get_prediction_squared_error_derivative(prediction);
                input.iter_mut().for_each(|input_value: &mut f32| {
                    *input_value *= squared_error_derivative;
                });
                (
                    input,
                    EpochHistory::new(squared_error, sample.get_hyperloglog_squared_error()),
                )
            })
            .reduce(
                || ([0.0; 5], EpochHistory::default()),
                |(mut total_weights_gradient, history): ([f32; 5], EpochHistory), (partial_weights_gradient, partial_history): ([f32; 5], EpochHistory)| {
                    total_weights_gradient
                        .iter_mut()
                        .zip(partial_weights_gradient.into_iter())
                        .for_each(|(total_weight_gradient, partial_weight_gradient)| {
                            *total_weight_gradient += partial_weight_gradient;
                        });
                    (total_weights_gradient, history + partial_history)
                },
            );

        // We divide the total gradient by the number of samples to get the mean gradient.
        total_gradient.iter_mut().for_each(|gradient: &mut f32| {
            *gradient /= samples.len() as f32;
        });

        let adam_gradient = self.optimizer.update(total_gradient);

        self.weights
            .iter_mut()
            .zip(adam_gradient.into_iter())
            .for_each(|(weight, gradient): (&mut f32, f32)| {
                *weight -= gradient;
            });

        history
    }

    fn train<PRECISION: Precision + WordType<BITS>, const BITS: usize>(
        &mut self,
        number_of_epochs: usize,
        number_of_samples: usize,
        mut random_state: u64,
    ) -> Vec<EpochHistory> {
        // We use indicatif to create an extensive loading bar that can display
        // the data from the latest epoch history metadata.
        let progress_bar = ProgressBar::new(number_of_epochs as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .progress_chars("##-"),
        );

        // We display the empty loading bar.
        progress_bar.set_message(EpochHistory::get_csv_header());

        (0..number_of_epochs)
            .map(|_| {
                random_state = splitmix64(random_state);
                let samples = generate_samples::<PRECISION, BITS>(number_of_samples, random_state);
                let epoch_history = self.train_single_epoch(&samples);

                // We update the progress bar with the latest epoch history metadata.
                progress_bar.set_message(&epoch_history.to_csv_line());

                // We increment the progress bar.
                progress_bar.inc(1);

                epoch_history
            })
            .collect()
    }

    fn get_weights(&self) -> &[f32; 5] {
        &self.weights
    }

    fn get_weights_as_json(&self) -> String {
        serde_json::to_string(&self.weights).unwrap()
    }
}

fn main() {
    let number_of_epochs = 100;
    let number_of_samples = 100_000;
    let random_state = 453465175128736;

    let mut model = Dense::<5>::random(random_state);
    model.train::<Precision6, 6>(number_of_epochs, number_of_samples, random_state);
}
