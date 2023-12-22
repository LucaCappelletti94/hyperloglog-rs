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
    union_cardinality: usize,
    approximation_left_cardinality: f32,
    approximation_right_cardinality: f32,
    approximation_union_cardinality: f32,
    approximation_left_difference: f32,
    approximation_right_difference: f32,
    approximation_symmetric_difference: f32,
    approximation_intersection_cardinality: f32,
    left_number_of_zeros: usize,
    right_number_of_zeros: usize,
}

impl Sample {
    fn from_vecs<P: Precision + WordType<BITS>, const BITS: usize>(
        left: &[u32],
        right: &[u32],
    ) -> Self {
        let hll1: HyperLogLog<P, BITS> = left.iter().collect();
        let hll2: HyperLogLog<P, BITS> = right.iter().collect();
        let set1: HashSet<u32> = left.iter().copied().collect();
        let set2: HashSet<u32> = right.iter().copied().collect();
        let euc: EstimatedUnionCardinalities<f32> = hll1.estimate_union_and_sets_cardinality(&hll2);

        Sample {
            // left_cardinality: set1.len(),
            // right_cardinality: set2.len(),
            union_cardinality: set1.union(&set2).count(),
            approximation_left_cardinality: euc.get_left_cardinality(),
            approximation_right_cardinality: euc.get_right_cardinality(),
            approximation_union_cardinality: euc.get_union_cardinality(),
            approximation_intersection_cardinality: euc.get_intersection_cardinality(),
            approximation_left_difference: euc.get_left_difference_cardinality(),
            approximation_right_difference: euc.get_right_difference_cardinality(),
            approximation_symmetric_difference: euc.get_symmetric_difference_cardinality(),
            left_number_of_zeros: hll1.get_number_of_zero_registers(),
            right_number_of_zeros: hll2.get_number_of_zero_registers(),
        }
    }

    fn as_array(&self) -> [f32; 9] {
        [
            self.approximation_left_cardinality as f32,
            self.approximation_right_cardinality as f32,
            self.approximation_union_cardinality as f32,
            self.approximation_left_difference as f32,
            self.approximation_right_difference as f32,
            self.approximation_symmetric_difference as f32,
            self.approximation_intersection_cardinality as f32,
            self.left_number_of_zeros as f32,
            self.right_number_of_zeros as f32,
        ]
    }

    fn feature_names() -> [&'static str; 9] {
        [
            "approximation left cardinality",
            "approximation right cardinality",
            "approximation union cardinality",
            "approximation left difference",
            "approximation right difference",
            "approximation symmetric difference",
            "approximation intersection cardinality",
            "left number of zeros",
            "right number of zeros",
        ]
    }

    fn get_prediction_squared_error(&self, prediction: f32) -> f32 {
        ((self.union_cardinality as f32 - prediction) / self.union_cardinality as f32).powi(2)
    }

    fn get_prediction_squared_error_derivative(&self, prediction: f32) -> f32 {
        -2.0 * (self.union_cardinality as f32 - prediction) / self.union_cardinality as f32
    }

    fn get_hyperloglog_squared_error(&self) -> f32 {
        self.get_prediction_squared_error(self.approximation_union_cardinality as f32)
    }
}

fn generate_sample<P: Precision + WordType<BITS>, const BITS: usize>(
    mut random_state: u64,
) -> Sample {
    let first_set_cardinality = xorshift(random_state) % 1_000_000;
    random_state = splitmix64(random_state);
    let second_set_cardinality = xorshift(random_state) % 1_000_000;
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

    Sample::from_vecs::<P, BITS>(vec1.as_slice(), vec2.as_slice())
}

struct EpochHistory {
    total_model_squared_error: f32,
    number_of_samples: usize,
    better: usize,
    total_hyperloglog_squared_error: f32,
}

impl Default for EpochHistory {
    fn default() -> Self {
        Self {
            total_model_squared_error: 0.0,
            number_of_samples: 0,
            better: 0,
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
            better: self.better + other.better,
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
            better: (model_squared_error < hyperloglog_squared_error) as usize,
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
        self.better as f32 / self.number_of_samples as f32
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
    feature_mask: [bool; N],
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
            feature_mask: [true; N],
        }
    }

    fn random_nuked(random_state: u64, feature_to_nuke: usize) -> Self {
        let mut model = Self::random(random_state);
        model.feature_mask[feature_to_nuke] = false;
        model
    }
}

impl Dense<9> {
    fn predict(&self, sample: &Sample) -> ([f32; 9], f32) {
        let mut sample_values: [f32; 9] = sample.as_array();
        sample_values
            .iter_mut()
            .zip(self.feature_mask.iter())
            .for_each(|(sample_value, feature_mask)| {
                if !feature_mask {
                    *sample_value = 0.0;
                }
            });

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
            [f32; 9],
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
                || ([0.0; 9], 0.0, EpochHistory::default()),
                |(mut total_weights_gradient, total_bias_gradient, history): (
                    [f32; 9],
                    f32,
                    EpochHistory,
                ),
                 (partial_weights_gradient, partial_bias_gradient, partial_history): (
                    [f32; 9],
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

        let mut test_samples = vec![Sample::default(); number_of_samples];

        // We create a loading bar for the sample generation
        let samples_progress_bar =
            multi_progress_bar.add(ProgressBar::new(number_of_samples as u64));
        samples_progress_bar.set_style(style.clone());

        test_samples
            .par_iter_mut()
            .enumerate()
            .progress_with(samples_progress_bar)
            .for_each(|(i, sample)| {
                *sample = generate_sample::<P, BITS>(splitmix64(
                    random_state.wrapping_mul(i as u64 + 1),
                ));
            });

        let histories = (0..number_of_epochs)
            .map(|_| {
                random_state = splitmix64(random_state);

                let mut train_samples = vec![Sample::default(); number_of_samples];

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

                random_state = splitmix64(random_state);

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

                epoch_history
            })
            .collect();

        let evaluate_history = self.evaluate(&test_samples);

        // We update the progress bar with the latest epoch history metadata.
        println!("{:?} {}", self.feature_mask, evaluate_history.to_csv_line());

        multi_progress_bar.clear().unwrap();
        histories
    }

    fn get_weights(&self) -> &[f32; 9] {
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
    let number_of_samples = 1_000;
    let repetitions_per_batch = 10_000;
    let random_state = 64376587;

    for feature_to_nuke in 0..9 {
        let mut model = Dense::<9>::random_nuked(random_state, feature_to_nuke);
        model.train::<Precision8, 6>(
            number_of_epochs,
            repetitions_per_batch,
            number_of_samples,
            "weights.json",
            random_state,
        );
    }

    // println!("{:?}", Sample::feature_names());
    // println!("{}", model.get_weights_and_bias_as_json(),);
}
