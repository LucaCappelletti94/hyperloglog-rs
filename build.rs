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

fn precompute_linear_counting(precision: u8) -> Vec<f32> {
    let number_of_registers = 1 << precision;
    let mut small_corrections = vec![0_f32; number_of_registers];
    let mut i = 0;
    // We can skip the last value in the small range correction array, because it is always 0.
    while i < number_of_registers - 1 {
        small_corrections[i] = (number_of_registers as f64
            * ((number_of_registers as f64 / (i as f64 + 1.0)).ln()))
            as f32;
        i += 1;
    }
    small_corrections
}

fn main() {
    let minimum_precision = 4;
    let maximal_precision = 16;

    // For each precision, we generate the linear counting corrections
    let linear_counting_corrections = (minimum_precision..=maximal_precision)
        .map(|x| {
            format!(
                "pub const LINEAR_COUNTING_CORRECTIONS_{precision}: [f32; {number_of_registers}] = [\n{corrections}\n];",
                precision = x,
                number_of_registers = 1 << x,
                corrections = precompute_linear_counting(x)
                    .iter()
                    .map(|x| format!("    {:.2},", x))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let number_of_precisions = maximal_precision - minimum_precision + 1;
    let alpha_values = format!(
        "pub const ALPHA_VALUES: [f32; {number_of_precisions}] = [\n{}\n];",
        (minimum_precision..=maximal_precision)
            .map(|x| format!("    {},", get_alpha(1 << x)))
            .collect::<Vec<String>>()
            .join("\n"),
    );

    // Define the output path for the generated code
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let linear_counting_corrections_path =
        Path::new(&out_dir).join("linear_counting_corrections.rs");
    let alpha_values_path = Path::new(&out_dir).join("alpha_values.rs");

    // Write the generated code to the file
    let mut file = File::create(&linear_counting_corrections_path).unwrap();
    file.write_all(linear_counting_corrections.as_bytes())
        .unwrap();

    let mut alpha_values_file = File::create(&alpha_values_path).unwrap();
    alpha_values_file
        .write_all(alpha_values.as_bytes())
        .unwrap();
}
