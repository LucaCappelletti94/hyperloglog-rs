use std::fs::File;
use std::io::Write;
use std::path::Path;

fn get_alpha(number_of_registers: usize) -> f32 {
    // Match the number of registers to the known alpha values
    match number_of_registers {
        16 => 0.673,
        32 => 0.697,
        64 => 0.709,
        _ => 0.7213 / (1.0 + 1.079 / number_of_registers as f32),
    }
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
    } else {
        format!("{integer_with_underscores}.0")
    };

    match result.as_str() {
        "0.693_147" => "core::f32::consts::LN_2".to_owned(),
        "2.302_585" => "core::f32::consts::LN_10".to_owned(),
        _ => result,
    }
}

fn main() {
    let minimum_precision = 4;
    let maximal_precision = 16;
    let maximal_number_of_registers = (1 << maximal_precision) + 1;

    // For each precision, we generate the linear counting corrections
    let log_values = format!(
        "pub(crate) static LOG_VALUES: [f32; {maximal_number_of_registers}] = [\n{}\n];",
        (0..maximal_number_of_registers)
            .map(|x| if x > 0 {
                format!("    {},", format_float_with_underscores((x as f64).ln(), 6))
            } else {
                "    f32::NEG_INFINITY,".to_owned()
            })
            .collect::<Vec<String>>()
            .join("\n")
    );

    let number_of_precisions = maximal_precision - minimum_precision + 1;
    let alpha_values = format!(
        "pub(crate) static ALPHA_VALUES: [f32; {number_of_precisions}] = [\n{}\n];",
        (minimum_precision..=maximal_precision)
            .map(|x| format!("    {},", get_alpha(1 << x)))
            .collect::<Vec<String>>()
            .join("\n"),
    );

    // Define the output path for the generated code
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let log_values_path = Path::new(&out_dir).join("log_values.rs");
    let alpha_values_path = Path::new(&out_dir).join("alpha_values.rs");

    // Write the generated code to the file
    let mut file = File::create(log_values_path).unwrap();
    file.write_all(log_values.as_bytes()).unwrap();

    let mut alpha_values_file = File::create(alpha_values_path).unwrap();
    alpha_values_file
        .write_all(alpha_values.as_bytes())
        .unwrap();
}
