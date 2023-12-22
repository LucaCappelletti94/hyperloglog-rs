//! Experiment to tune the 'ingredients' of the union of the HyperLogLog.
//!
//!
use core::ops::Add;
use hyperloglog_rs::prelude::*;
use indicatif::ParallelProgressIterator;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use rand::prelude::*;
use rayon::prelude::*;
use serde_json;
use std::collections::HashMap;
use std::collections::HashSet;
use vec_rand::{random_f32, splitmix64, xorshift};

#[derive(Default, Debug, Clone, Copy)]
struct Sample {
    cardinality: usize,
    approximation: f32,
    number_of_zeros: usize,
    number_of_zeros_rate: f32,
}

impl Sample {
    fn from_vec<P: Precision + WordType<BITS>, const BITS: usize>(values: &[u32]) -> Self {
        let hll: HyperLogLog<P, BITS> = values.iter().collect();
        let set: HashSet<u32> = values.iter().copied().collect();

        Sample {
            cardinality: set.len(),
            approximation: hll.estimate_cardinality(),
            number_of_zeros: hll.get_number_of_zero_registers(),
            number_of_zeros_rate: hll.get_number_of_zero_registers() as f32
                / P::NUMBER_OF_REGISTERS as f32,
        }
    }

    fn as_array(&self) -> [f32; 3] {
        [
            self.approximation,
            self.number_of_zeros as f32,
            self.number_of_zeros_rate,
        ]
    }

    fn feature_names() -> [&'static str; 3] {
        ["approximation", "number_of_zeros", "number_of_zeros_rate"]
    }

    fn get_prediction_squared_error(&self, prediction: f32) -> f32 {
        (self.cardinality as f32 - prediction).powi(2)
    }

    fn get_prediction_squared_error_derivative(&self, prediction: f32) -> f32 {
        -2.0 * (self.cardinality as f32 - prediction)
    }

    fn get_hyperloglog_squared_error(&self) -> f32 {
        self.get_prediction_squared_error(self.approximation)
    }
}

fn generate_sample<P: Precision + WordType<BITS>, const BITS: usize>(
    mut random_state: u64,
) -> Sample {
    random_state = splitmix64(random_state);
    let set_cardinality = splitmix64(random_state) % 100_000;
    random_state = splitmix64(random_state);
    let world_size = splitmix64(random_state) % 100_000;
    random_state = splitmix64(random_state);

    let mut vec1: Vec<u32> = Vec::with_capacity(set_cardinality as usize);

    for _ in 0..set_cardinality {
        let value = if world_size > 0 {
            xorshift(random_state) % world_size
        } else {
            0
        };
        vec1.push(value as u32);
        random_state = splitmix64(random_state);
    }

    Sample::from_vec::<P, BITS>(vec1.as_slice())
}

struct EpochHistory {
    total_model_squared_error: f32,
    number_of_samples: usize,
    better_count: usize,
    total_hyperloglog_squared_error: f32,
}

impl Default for EpochHistory {
    fn default() -> Self {
        Self {
            total_model_squared_error: 0.0,
            number_of_samples: 0,
            better_count: 0,
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
            better_count: self.better_count + other.better_count,
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
            better_count: (model_squared_error < hyperloglog_squared_error) as usize,
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

    fn get_better_rate(&self) -> f32 {
        self.better_count as f32 / self.number_of_samples as f32
    }

    fn get_csv_header() -> &'static str {
        "mse\tmse_hll\trate\tbetter_rate"
    }

    fn to_csv_line(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            self.get_mean_squared_error(),
            self.get_hyperloglog_mean_squared_error(),
            self.get_error_rate(),
            self.get_better_rate()
        )
    }
}

struct Dense<const N: usize> {
    weights: [f32; N],
    bias: f32,
    weights_optimizer: Adam<N>,
    bias_optimizer: Adam<1>,
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
            bias: 0.0,
            weights_optimizer: Adam::default(),
            bias_optimizer: Adam::default(),
        }
    }
}

impl Dense<3> {
    fn predict(&self, sample: &Sample) -> ([f32; 3], f32) {
        let sample_values: [f32; 3] = sample.as_array();
        let prediction = (sample_values
            .iter()
            .zip(self.weights.iter())
            .map(|(sample, weight)| sample * weight)
            .sum::<f32>()
            + self.bias)
            .max(0.0);
        (sample_values, prediction)
    }

    fn evaluate(&self, samples: &[Sample]) -> EpochHistory {
        samples
            .par_iter()
            .map(|sample| {
                let (input, prediction) = self.predict(sample);
                let squared_error = sample.get_prediction_squared_error(prediction);
                let hyperloglog_squared_error = sample.get_hyperloglog_squared_error();
                EpochHistory::new(squared_error, hyperloglog_squared_error)
            })
            .reduce(|| EpochHistory::default(), |a, b| a + b)
    }

    fn train_single_epoch(&mut self, samples: &[Sample]) -> EpochHistory {
        let (mut total_weights_gradient, mut total_bias_gradient, history): (
            [f32; 3],
            f32,
            EpochHistory,
        ) = samples
            .par_iter()
            .map(|sample: &Sample| {
                let (mut input, prediction) = self.predict(sample);
                let squared_error = sample.get_prediction_squared_error(prediction);
                let squared_error_derivative =
                    sample.get_prediction_squared_error_derivative(prediction);
                input.iter_mut().for_each(|input_value: &mut f32| {
                    *input_value *= squared_error_derivative;
                });
                (
                    input,
                    squared_error_derivative,
                    EpochHistory::new(squared_error, sample.get_hyperloglog_squared_error()),
                )
            })
            .reduce(
                || ([0.0; 3], 0.0, EpochHistory::default()),
                |(mut total_weights_gradient, total_bias_gradient, history): (
                    [f32; 3],
                    f32,
                    EpochHistory,
                ),
                 (partial_weights_gradient, partial_bias_gradient, partial_history): (
                    [f32; 3],
                    f32,
                    EpochHistory,
                )| {
                    total_weights_gradient
                        .iter_mut()
                        .zip(partial_weights_gradient.into_iter())
                        .for_each(|(total_weight_gradient, partial_weight_gradient)| {
                            *total_weight_gradient += partial_weight_gradient;
                        });
                    (
                        total_weights_gradient,
                        total_bias_gradient + partial_bias_gradient,
                        history + partial_history,
                    )
                },
            );

        // We divide the total gradient by the number of samples to get the mean gradient.
        total_weights_gradient
            .iter_mut()
            .for_each(|gradient: &mut f32| {
                *gradient /= samples.len() as f32;
            });

        total_bias_gradient /= samples.len() as f32;

        let adam_weights_gradient = self.weights_optimizer.update(total_weights_gradient);
        let adam_bias_gradient = self.bias_optimizer.update([total_bias_gradient]);

        let norm = (self.bias.powi(2) + self.weights.iter().map(|x| x.powi(2)).sum::<f32>())
            .sqrt()
            .max(f32::EPSILON);

        self.weights
            .iter_mut()
            .zip(adam_weights_gradient.into_iter())
            .for_each(|(weight, gradient): (&mut f32, f32)| {
                *weight -= gradient / norm;
            });

        self.bias -= adam_bias_gradient[0] / norm;

        history
    }

    /// Train the model for a given number of epochs.
    ///
    /// # Arguments
    /// * `number_of_epochs` - The number of epochs to train the model.
    /// * `repetitions_per_batch` - The number of batches to train the model per epoch.
    /// * `number_of_samples` - The number of samples to generate per epoch.
    /// * `weights_path` - The path to the file where the weights will be saved.
    /// * `random_state` - The random state used to generate the samples.
    ///
    fn train<P: Precision + WordType<BITS>, const BITS: usize>(
        &mut self,
        number_of_epochs: usize,
        repetitions_per_batch: usize,
        number_of_samples: usize,
        weights_path: &str,
        mut random_state: u64,
    ) -> Vec<EpochHistory> {
        // We create a multiprogress bar which will contain the epochs progress bar,
        // the samples creation progress bar and the batches progress bar.
        let multi_progress_bar = indicatif::MultiProgress::new();

        // We use indicatif to create an extensive loading bar that can display
        // the data from the latest epoch history metadata.
        let style = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-");
        let epochs_progress_bar = multi_progress_bar.add(ProgressBar::new(number_of_epochs as u64));
        epochs_progress_bar.set_style(style.clone());

        // We display the empty loading bar.
        epochs_progress_bar.set_message(EpochHistory::get_csv_header());

        let histories = (0..number_of_epochs)
            .map(|_| {
                random_state = splitmix64(random_state);

                let mut train_samples = vec![Sample::default(); number_of_samples];
                let mut test_samples = vec![Sample::default(); number_of_samples];

                // We create a loading bar for the sample generation
                let samples_progress_bar =
                    multi_progress_bar.add(ProgressBar::new(number_of_samples as u64));
                samples_progress_bar.set_style(style.clone());

                train_samples
                    .par_iter_mut()
                    .enumerate()
                    .progress_with(samples_progress_bar)
                    .for_each(|(i, sample)| {
                        *sample = generate_sample::<P, BITS>(splitmix64(
                            random_state.wrapping_mul(i as u64 + 1),
                        ));
                    });

                // We create a loading bar for the sample generation
                let samples_progress_bar =
                    multi_progress_bar.add(ProgressBar::new(number_of_samples as u64));
                samples_progress_bar.set_style(style.clone());
                random_state = splitmix64(random_state);

                test_samples
                    .par_iter_mut()
                    .enumerate()
                    .progress_with(samples_progress_bar)
                    .for_each(|(i, sample)| {
                        *sample = generate_sample::<P, BITS>(splitmix64(
                            random_state.wrapping_mul(i as u64 + 1),
                        ));
                    });

                let mut epoch_history = EpochHistory::default();

                // We create a loading bar for the batches.
                let batches_progress_bar =
                    multi_progress_bar.add(ProgressBar::new((repetitions_per_batch) as u64));

                batches_progress_bar.set_style(style.clone());

                (0..repetitions_per_batch).for_each(|_| {
                    epoch_history = self.train_single_epoch(&train_samples);
                    batches_progress_bar.set_message(epoch_history.to_csv_line());
                    batches_progress_bar.inc(1);

                    // We shuffle the samples
                    train_samples.shuffle(&mut thread_rng());
                });

                let evaluate_history = self.evaluate(&test_samples);

                // We update the progress bar with the latest epoch history metadata.
                epochs_progress_bar.set_message(evaluate_history.to_csv_line());

                // We increment the progress bar.
                epochs_progress_bar.inc(1);

                // We write out the current weights to the file
                // using the JSON format.
                let weights_and_bias = self.get_weights_and_bias_as_json();
                std::fs::write(weights_path, weights_and_bias).unwrap();

                evaluate_history
            })
            .collect();
        multi_progress_bar.clear().unwrap();
        histories
    }

    fn get_weights(&self) -> &[f32; 3] {
        &self.weights
    }

    /// Returns the weights and model bias as a JSON string.
    fn get_weights_and_bias_as_json(&self) -> String {
        let mut dictionary = HashMap::new();
        dictionary.insert("weights".to_string(), self.weights.to_vec());
        dictionary.insert("bias".to_string(), vec![self.bias]);
        serde_json::to_string(&dictionary).unwrap()
    }
}

fn main() {
    let number_of_epochs = 10;
    let number_of_samples = 50_000;
    let repetitions_per_batch = 10_000;
    let random_state = splitmix64(64376723754376523);

    let mut model = Dense::<3>::random(random_state);
    // let mut model = Dense::<6>::known_local_minima();
    model.train::<Precision8, 6>(
        number_of_epochs,
        repetitions_per_batch,
        number_of_samples,
        "weights.json",
        random_state,
    );

    println!("{:?}", Sample::feature_names());
    println!("{}", model.get_weights_and_bias_as_json(),);
}
