use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::Path;

trait RedableNumber: Display {
    fn format_with_precision(&self, precision: usize) -> String {
        // Format the float with the specified precision
        let formatted = format!("{:.*}", precision, self);

        // Split into integer and fractional parts
        let mut parts: Vec<&str> = formatted.split('.').collect();
        let integer_part = parts.remove(0);
        let fractional_part = parts.first().unwrap_or(&"");

        // Trim values smaller than five from the end of the fractional part
        let fractional_part = fractional_part.trim_end_matches(['0']); //'2', '3', '4']);

        // We save whether the number is negative, so to remove the minus sign
        // and re-add it at the end
        let is_negative = integer_part.starts_with('-');

        // Remove the minus sign from the integer part
        let integer_part = integer_part.trim_start_matches('-');

        // Add underscores to the integer part
        let integer_with_underscores = integer_part
            .chars()
            .rev()
            .collect::<String>()
            .chars()
            .collect::<Vec<_>>()
            .chunks(3)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("_")
            .chars()
            .rev()
            .collect::<String>();

        let integer_with_underscores = if is_negative {
            format!("-{}", integer_with_underscores)
        } else {
            integer_with_underscores
        };

        // Add underscores to the fractional part
        let fractional_part = fractional_part
            .chars()
            .collect::<Vec<_>>()
            .chunks(3)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("_");

        // Combine the integer part with underscores and the fractional part
        let result = if !fractional_part.is_empty() {
            format!("{}.{fractional_part}", integer_with_underscores)
        } else if precision == 0 {
            integer_with_underscores
        } else {
            format!("{integer_with_underscores}.0")
        };

        // We check whether the string matches for its
        // entirety with the known constants
        let constants = [
            ("0.693_147_180_559_945_309_417_232_121_458_176_568", "LN_2"),
            ("2.302_585_092_994", "LN_10"),
        ];

        for (formatted, constant) in constants.iter() {
            if result.len() < 6 {
                continue;
            }

            // We remove the last character, which might be
            // different solely because of the formatting.
            let result = &result[..result.len() - 1];

            let shorter = if result.len() < formatted.len() {
                &formatted[..result.len()]
            } else {
                &formatted[..formatted.len() - 1]
            };

            if result == shorter {
                return constant.to_string();
            }
        }

        result
    }
}

impl RedableNumber for f64 {}
impl RedableNumber for i32 {}
impl RedableNumber for u32 {}
impl RedableNumber for usize {}

/// Returns the list of precision to generate the weights for.
fn get_precisions() -> Vec<usize> {
    vec![
        #[cfg(feature = "precision_4")]
        4,
        #[cfg(feature = "precision_5")]
        5,
        #[cfg(feature = "precision_6")]
        6,
        #[cfg(feature = "precision_7")]
        7,
        #[cfg(feature = "precision_8")]
        8,
        #[cfg(feature = "precision_9")]
        9,
        #[cfg(feature = "precision_10")]
        10,
        #[cfg(feature = "precision_11")]
        11,
        #[cfg(feature = "precision_12")]
        12,
        #[cfg(feature = "precision_13")]
        13,
        #[cfg(feature = "precision_14")]
        14,
        #[cfg(feature = "precision_15")]
        15,
        #[cfg(feature = "precision_16")]
        16,
        #[cfg(feature = "precision_17")]
        17,
        #[cfg(feature = "precision_18")]
        18,
    ]
}

fn get_smallest_data_type(value: usize) -> &'static str {
    if value <= u8::MAX as usize {
        "u8"
    } else if value <= u16::MAX as usize {
        "u16"
    } else if value <= u32::MAX as usize {
        "u32"
    } else {
        "u64"
    }
}

#[cfg(feature = "plusplus")]
include!("./original_biases.rs");
#[cfg(feature = "plusplus")]
include!("./original_estimates.rs");

#[cfg(feature = "plusplus")]
fn get_sorted_biases_and_estimates(precision: usize) -> (Vec<f64>, Vec<f64>) {
    // Get the biases and estimates for the specified precision
    let biases = BIASES[precision - 4].to_vec();
    let estimates = ESTIMATES[precision - 4].to_vec();

    assert_eq!(biases.len(), estimates.len());

    // We need to sort them in ascending order by estimates, making sure the biases are sorted accordingly
    let data: Vec<(f64, f64)> = biases
        .iter()
        .zip(estimates.iter())
        .map(|(a, b)| (*a, *b))
        .collect();

    // We exclude the data whose estimate (the first value) is lower than the linear count threshold
    // associated with the current precision, as it wil never be used. I am not sure why they ever provided
    // weights for these values in the first place.
    let linear_count_threshold = get_linear_count_threshold(precision);
    let mut data: Vec<(f64, f64)> = data
        .into_iter()
        .filter(|(_, b)| *b >= linear_count_threshold as f64)
        .collect();

    data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    // We convert the data back into biases and estimates
    let biases: Vec<f64> = data.iter().map(|(a, _)| *a).collect();
    let estimates: Vec<f64> = data.iter().map(|(_, b)| *b).collect();

    // We check that both vectors have the same length
    assert_eq!(biases.len(), estimates.len());

    // We check that the estimates vector are sorted
    for window in estimates.windows(2) {
        assert!(
            window[0] <= window[1],
            "Found unsorted estimates: {:?} at precision {}",
            window,
            precision
        );
    }

    (biases, estimates)
}

#[cfg(feature = "integer_plusplus")]
fn get_integer_biases_and_estimates(precision: usize) -> (Vec<i32>, Vec<u32>) {
    let (biases, estimates) = get_sorted_biases_and_estimates(precision);

    // We convert by rounding the values into u32s and i32s respectively
    let mut data: Vec<(i32, u32)> = biases
        .iter()
        .zip(estimates.iter())
        .map(|(a, b)| (*a as i32, *b as u32))
        .collect();

    // We drop the duplicates
    data.dedup_by(|a, b| a.1 == b.1);

    // We now identify whether at some point in the bias series there are all and solely zeros afterwards.
    // In such cases, we keep the first zero and drop the rest.
    let mut first_zero_index: Option<usize> = None;
    for (i, (bias, _estimate)) in data.iter().enumerate() {
        if *bias != 0 {
            first_zero_index = None;
            continue;
        }
        if bias == &0 && first_zero_index.is_none() {
            first_zero_index = Some(i);
        }
    }

    if let Some(first_zero_index) = first_zero_index {
        data.truncate(first_zero_index + 1);
    }

    // We convert the data back into biases and estimates
    let biases: Vec<i32> = data.iter().map(|(a, _)| *a).collect();
    let estimates: Vec<u32> = data.iter().map(|(_, b)| *b).collect();

    // We check that both vectors have the same length
    assert_eq!(biases.len(), estimates.len());

    // We check that the estimates vector are sorted
    assert!(estimates.windows(2).all(|w| w[0] <= w[1]));

    (biases, estimates)
}

#[cfg(feature = "beta")]
include!("./beta.rs");

#[cfg(feature = "beta")]
fn get_beta(precision: usize) -> [f64; 8] {
    BETAS[precision - 4]
}

#[cfg(feature = "precomputed_beta")]
fn beta_horner(number_of_zeros: f64, precision: usize) -> f64 {
    let beta = get_beta(precision);
    let zl = number_of_zeros.ln_1p();
    let mut res = 0.0;
    for i in (1..8).rev() {
        res = res * zl + beta[i];
    }
    res * zl + beta[0] * number_of_zeros
}

#[cfg(feature = "precomputed_beta")]
fn get_unrolled_beta_horner(precision: usize) -> Vec<f64> {
    (0..(1 << precision))
        .map(|x| beta_horner(x as f64, precision))
        .collect()
}

fn get_alpha(number_of_registers: usize) -> f64 {
    // Match the number of registers to the known alpha values
    match number_of_registers {
        16 => 0.673,
        32 => 0.697,
        64 => 0.709,
        _ => 0.7213 / (1.0 + 1.079 / number_of_registers as f64),
    }
}

fn get_linear_count(number_of_registers: usize, threshold: usize) -> usize {
    (number_of_registers as f64 * f64::exp(-(threshold as f64 / number_of_registers as f64)))
        .round() as usize
}

fn get_linear_count_threshold(precision: usize) -> usize {
    let linear_count_threshold: [usize; 15] = [
        10, 20, 40, 80, 220, 400, 900, 1_800, 3_100, 6_500, 11_500, 20_000, 50_000, 120_000,
        350_000,
    ];
    linear_count_threshold[precision - 4]
}

#[cfg(feature = "plusplus")]
fn write_weights(precisions: &[usize]) {
    // For each precision, we generate the biases and estimates
    let mut all_types = Vec::new();
    let mut all_biases = Vec::new();
    let mut all_estimates = Vec::new();

    for precision in precisions.iter().copied() {
        #[cfg(feature = "integer_plusplus")]
        let (biases, estimates, biases_type, estimates_type, resolution) = {
            let (biases, estimates) = get_integer_biases_and_estimates(precision);
            (biases, estimates, "i32", "u32", 0)
        };

        #[cfg(not(feature = "integer_plusplus"))]
        let (biases, estimates, biases_type, estimates_type, resolution) = {
            let (biases, estimates) = get_sorted_biases_and_estimates(precision);
            (biases, estimates, "f64", "f64", 3)
        };

        let number_of_biases = biases.len().format_with_precision(0);

        let weights_type = format!("/// Bias centroid type for precision {precision} for [`PlusPlus`]. \ntype Bias{precision} = [{biases_type}; {number_of_biases}];");

        let biases = format!(
            "/// Biases aligned with estimates centroids for precision {precision} for [`PlusPlus`]. \nconst BIAS_{precision}:  Bias{precision} = [\n{}\n];",
            biases
                .iter()
                .map(|x| format!("    {},", x.format_with_precision(resolution)))
                .collect::<Vec<String>>()
                .join("\n")
        );

        let estimate_type =
            format!("/// Estimates centroid type for precision {precision} for [`PlusPlus`]. \ntype Estimates{precision} = [{estimates_type}; {number_of_biases}];");

        all_types.push(estimate_type);

        let estimates = format!(
            "/// Sorted estimates centroids for precision {precision} for [`PlusPlus`]. \nconst ESTIMATES_{precision}:  Estimates{precision} = [\n{}\n];",
            estimates
                .iter()
                .map(|x| format!("    {},", x.format_with_precision(resolution)))
                .collect::<Vec<String>>()
                .join("\n")
        );

        all_types.push(weights_type);
        all_biases.push(biases);
        all_estimates.push(estimates);
    }

    // Define the output path for the generated code
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let weights_path = Path::new(&out_dir).join("weights.rs");

    // Write the generated code to the file
    let mut weights_file = File::create(weights_path).unwrap();
    weights_file
        .write_all(
            format!(
                "{}\n\n{}\n\n{}\n",
                all_types.join("\n"),
                all_biases.join("\n"),
                all_estimates.join("\n"),
            )
            .as_bytes(),
        )
        .unwrap();
}

#[cfg(feature = "zero_count_correction")]
fn write_linear_count_zeros(precisions: &[usize]) {
    let linear_count_zeros = precisions
        .iter()
        .map(|precision| {
            let count = get_linear_count(1 << precision, get_linear_count_threshold(*precision));
            let data_type = get_smallest_data_type(1 << precision);
            format!("/// Number of zeros for linear count threshold for precision {precision} used in [`HyperLogLog`]. \nconst LINEAR_COUNT_ZEROS_{precision}: {data_type} = {count};",)
        })
        .collect::<Vec<String>>()
        .join("\n");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let linear_count_zeros_path = Path::new(&out_dir).join("linear_count_zeros.rs");
    let mut linear_count_zeros_file = File::create(linear_count_zeros_path).unwrap();
    linear_count_zeros_file
        .write_all(linear_count_zeros.as_bytes())
        .unwrap();
}

fn write_alphas(precisions: &[usize]) {
    let alpha_values = precisions
        .iter()
        .map(|precision| {
            let number_of_registers = 1 << precision;
            let alpha = get_alpha(number_of_registers);
            format!(
                "/// Alpha constants for precision {precision} used in [`HyperLogLog`]. \nconst ALPHA_{precision}: f64 = {};",
                alpha.format_with_precision(12)
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let alpha_values_path = Path::new(&out_dir).join("alpha_values.rs");
    let mut alpha_values_file = File::create(alpha_values_path).unwrap();
    alpha_values_file
        .write_all(alpha_values.as_bytes())
        .unwrap();
}

#[cfg(all(
    not(feature = "std_ln"),
    any(
        all(feature = "beta", not(feature = "precomputed_beta")),
        feature = "plusplus",
    )
))]
fn write_ln_values(precisions: &[usize]) {
    // Since the ln values are needed up to the maximal number of registers, we
    // determine what is the largest number of registers we need to generate the
    // ln values for.
    let maximal_precision = *precisions.iter().max().unwrap();
    let maximal_number_of_registers = 1 + (1 << maximal_precision);
    let formatted_maximal_number_of_registers =
        maximal_number_of_registers.format_with_precision(0);

    let ln_values = format!(
        "use core::f64::consts::{{LN_2, LN_10}};\n\n/// Precomputed natural log values for no-std log computations. \nstatic LN_VALUES: [f64; {formatted_maximal_number_of_registers}] = [\n{}\n];",
        (0..maximal_number_of_registers)
            .map(|x| if x > 0 {
                format!("    {},", (x as f64).ln().format_with_precision(12))
            } else {
                "    f64::NEG_INFINITY,".to_owned()
            })
            .collect::<Vec<String>>()
            .join("\n")
    );

    // Define the output path for the generated code
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let ln_values_path = Path::new(&out_dir).join("ln_values.rs");

    // Write the generated code to the file
    let mut ln_values_file = File::create(ln_values_path).unwrap();
    ln_values_file.write_all(ln_values.as_bytes()).unwrap();
}

#[cfg(all(feature = "beta", not(feature = "precomputed_beta")))]
fn write_beta(precisions: &[usize]) {
    // For each precision, we generate the biases and estimates
    let mut all_betas = Vec::new();

    for precision in precisions.iter().copied() {
        let beta = get_beta(precision);

        let beta = format!(
            "/// Beta factors for the [`LogLogBeta`] approach.\nconst BETA_{precision}: [f64; {}] = [\n{}\n];",
            beta.len(),
            beta.iter()
                .map(|x| format!("    {},", x.format_with_precision(9)))
                .collect::<Vec<String>>()
                .join("\n")
        );

        all_betas.push(beta);
    }

    // Define the output path for the generated code
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let beta_path = Path::new(&out_dir).join("beta.rs");

    // Write the generated code to the file
    let mut beta_file = File::create(beta_path).unwrap();
    beta_file
        .write_all(all_betas.join("\n").as_bytes())
        .unwrap();
}

#[cfg(feature = "precomputed_beta")]
fn write_precomputed_beta(precisions: &[usize]) {
    // For each precision, we generate the biases and estimates
    let mut all_beta_horner = Vec::new();

    for precision in precisions.iter().copied() {
        let beta_horner = get_unrolled_beta_horner(precision);

        let beta = format!(
            "/// Precomputed beta-horner factors for the [`LogLogBeta`] approach.\nstatic BETA_HORNER_{precision}: [f64; {}] = [\n{}\n];",
            beta_horner.len().format_with_precision(0),
            beta_horner
                .iter()
                .map(|x| format!("    {},", x.format_with_precision(9)))
                .collect::<Vec<String>>()
                .join("\n")
        );

        all_beta_horner.push(beta);
    }

    // Define the output path for the generated code
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let beta_horner_path = Path::new(&out_dir).join("beta_horner.rs");

    // Write the generated code to the file
    let mut beta_horner_file = File::create(beta_horner_path).unwrap();
    beta_horner_file
        .write_all(all_beta_horner.join("\n").as_bytes())
        .unwrap();
}

/// Writes out the type aliases for the number of zeros for each precision.
///
/// # Implementative details
/// Given a maximal number of zeros, we can determine the smallest data type
/// that can hold the number of zeros. We then generate the type aliases for
/// each precision.
fn write_number_of_registers(precisions: &[usize]) {
    let number_of_registers = precisions
        .iter()
        .map(|precision| {
            let number_of_registers = 1 << precision;
            let smallest_data_type = get_smallest_data_type(number_of_registers);
            format!("/// Smallest word-like data-type for the number of register used in [`Precision`] trait implementations.\ntype NumberOfRegisters{precision} = {smallest_data_type};",)
        })
        .collect::<Vec<String>>()
        .join("\n");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let number_of_registers_path = Path::new(&out_dir).join("number_of_registers.rs");
    let mut number_of_registers_file = File::create(number_of_registers_path).unwrap();
    number_of_registers_file
        .write_all(number_of_registers.as_bytes())
        .unwrap();
}

fn main() {
    let precisions = get_precisions();

    write_alphas(&precisions);
    write_number_of_registers(&precisions);

    #[cfg(all(
        not(feature = "std_ln"),
        any(
            all(feature = "beta", not(feature = "precomputed_beta")),
            feature = "plusplus"
        )
    ))]
    write_ln_values(&precisions);

    #[cfg(all(feature = "beta", not(feature = "precomputed_beta")))]
    write_beta(&precisions);

    #[cfg(feature = "precomputed_beta")]
    write_precomputed_beta(&precisions);

    #[cfg(feature = "plusplus")]
    write_linear_count_zeros(&precisions);

    #[cfg(feature = "plusplus")]
    write_weights(&precisions);
}
