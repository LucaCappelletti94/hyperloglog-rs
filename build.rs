use std::fs::File;
use std::io::Write;
use std::path::Path;

fn get_biases_and_estimates(precision: usize) -> (Vec<i32>, Vec<u32>) {
    // We load the 'original_biases.json' and 'original_estimates.json' files
    let original_biases = include_str!("original_biases.json");
    let original_estimates = include_str!("original_estimates.json");

    // Parse the JSON files
    let original_biases: Vec<Vec<f64>> = serde_json::from_str(original_biases).unwrap();
    let original_estimates: Vec<Vec<f64>> = serde_json::from_str(original_estimates).unwrap();

    // Get the biases and estimates for the specified precision
    let biases = original_biases[precision - 4].clone();
    let estimates = original_estimates[precision - 4].clone();

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
    let data: Vec<(f64, f64)> = data
        .into_iter()
        .filter(|(_, b)| *b >= linear_count_threshold as f64)
        .collect();

    // We convert by rounding the values into u32s and i32s respectively
    let mut data: Vec<(i32, u32)> = data
        .into_iter()
        .map(|(bias, estimate)| (bias.round() as i32, estimate.round() as u32))
        .collect();

    // We need to sort them in ascending order by estimates, and then drop the duplicates
    data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    data.dedup_by(|a, b| a.1 == b.1);

    // We now identify whether at some point in the bias series there are all and solely zeros afterwards.
    // In such cases, we keep the first zero and drop the rest.
    let mut first_zero_index: Option<usize> = None;
    for (i, (bias, _estimate)) in data.iter().enumerate() {
        if *bias != 0 {
            first_zero_index = None;
            continue;
        }
        if bias == &0 && first_zero_index.is_none(){
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

fn get_beta(precision: usize) -> Vec<f64> {
    let beta = include_str!("beta.json");

    // Parse the JSON files
    let beta: Vec<Vec<f64>> = serde_json::from_str(beta).unwrap();

    // Get the biases and estimates for the specified precision
    let beta = beta[precision - 4].clone();

    beta
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

fn get_linear_count_zeros() -> [usize; 15] {
    let mut linear_count_zeros = [0; 15];
    for i in 0..15 {
        let exponent = i + 4;
        let number_of_registers = 1 << exponent;
        linear_count_zeros[i] =
            get_linear_count(number_of_registers, get_linear_count_threshold(exponent));
    }
    linear_count_zeros
}

fn format_float_with_underscores(value: f64, precision: usize) -> String {
    // Format the float with the specified precision
    let formatted = format!("{:.*}", precision, value);

    // Split into integer and fractional parts
    let mut parts: Vec<&str> = formatted.split('.').collect();
    let integer_part = parts.remove(0);
    let fractional_part = parts.first().unwrap_or(&"");

    // Trim zeros from the fractional part
    let fractional_part = fractional_part.trim_end_matches('0');

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

    match result.as_str() {
        "0.693_147" => "core::f64::consts::LN_2".to_owned(),
        "2.302_585" => "core::f64::consts::LN_10".to_owned(),
        _ => result,
    }
}

fn write_bias_and_estimate_files(minimum_precision: usize, maximal_precision: usize) {
    // For each precision, we generate the biases and estimates
    let mut all_types = Vec::new();
    let mut all_biases = Vec::new();
    let mut all_estimates = Vec::new();
    let mut all_betas = Vec::new();

    for precision in minimum_precision..=maximal_precision {
        let (biases, estimates) = get_biases_and_estimates(precision);
        let beta = get_beta(precision);
        let number_of_biases = biases.len();

        for data_type in ["f32", "f64"] {
            let capitalized_data_type = data_type.to_uppercase();

            let beta = format!(
                "const BETA_{capitalized_data_type}_{precision}: [{data_type}; 8] = [\n{}\n];",
                beta.iter()
                    .map(|x| format!("    {},", format_float_with_underscores(*x, 6)))
                    .collect::<Vec<String>>()
                    .join("\n")
            );

            all_betas.push(beta);
        }

        let weights_type = format!("type Bias{precision} = [i32; {number_of_biases}];");

        let biases = format!(
            "const BIAS_{precision}:  Bias{precision} = [\n{}\n];",
            biases
                .iter()
                .map(|x| format!("    {},", format_float_with_underscores(*x as f64, 0)))
                .collect::<Vec<String>>()
                .join("\n")
        );

        let estimate_type = format!("type Estimates{precision} = [u32; {number_of_biases}];");

        all_types.push(estimate_type);

        let estimates = format!(
            "const ESTIMATES_{precision}:  Estimates{precision} = [\n{}\n];",
            estimates
                .iter()
                .map(|x| format!("    {},", format_float_with_underscores(*x as f64, 0)))
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
                "{}\n\n{}\n\n{}\n\n{}\n",
                all_types.join("\n"),
                all_biases.join("\n"),
                all_estimates.join("\n"),
                all_betas.join("\n"),
            )
            .as_bytes(),
        )
        .unwrap();
}

fn main() {
    let minimum_precision = 4;
    let maximal_precision = 18;
    let maximal_number_of_registers = (1 << maximal_precision) + 1;

    // For each precision, we generate the linear counting corrections
    let log_values = format!(
        "static LOG_VALUES: [f64; {maximal_number_of_registers}] = [\n{}\n];",
        (0..maximal_number_of_registers)
            .map(|x| if x > 0 {
                format!("    {},", format_float_with_underscores((x as f64).ln(), 6))
            } else {
                "    f64::NEG_INFINITY,".to_owned()
            })
            .collect::<Vec<String>>()
            .join("\n")
    );

    let number_of_precisions = maximal_precision - minimum_precision + 1;
    let alpha_values = format!(
        "const ALPHA_VALUES: [f64; {number_of_precisions}] = [\n{}\n];",
        (minimum_precision..=maximal_precision)
            .map(|x| format!("    {},", get_alpha(1 << x)))
            .collect::<Vec<String>>()
            .join("\n"),
    );

    let linear_count_zeros = format!(
        "const LINEAR_COUNT_ZEROS: [usize; {number_of_precisions}] = [\n{}\n];",
        get_linear_count_zeros()
            .iter()
            .map(|x| format!("    {},", x))
            .collect::<Vec<String>>()
            .join("\n"),
    );

    // Define the output path for the generated code
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let log_values_path = Path::new(&out_dir).join("log_values.rs");
    let alpha_values_path = Path::new(&out_dir).join("alpha_values.rs");
    let linear_count_zeros_path = Path::new(&out_dir).join("linear_count_zeros.rs");

    // Write the generated code to the file
    let mut log_values_file = File::create(log_values_path).unwrap();
    log_values_file.write_all(log_values.as_bytes()).unwrap();

    let mut alpha_values_file = File::create(alpha_values_path).unwrap();
    alpha_values_file
        .write_all(alpha_values.as_bytes())
        .unwrap();

    let mut linear_count_zeros_file = File::create(linear_count_zeros_path).unwrap();
    linear_count_zeros_file
        .write_all(linear_count_zeros.as_bytes())
        .unwrap();

    write_bias_and_estimate_files(minimum_precision, maximal_precision);
}
