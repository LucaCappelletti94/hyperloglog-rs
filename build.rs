use std::fs::File;
use std::io::Write;
use std::path::Path;
use test_utils::prelude::ReadableNumber;

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

fn get_alpha(number_of_registers: usize) -> f64 {
    // Match the number of registers to the known alpha values
    match number_of_registers {
        16 => 0.673,
        32 => 0.697,
        64 => 0.709,
        _ => 0.7213 / (1.0 + 1.079 / number_of_registers as f64),
    }
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

fn main() {
    let precisions = get_precisions();

    write_alphas(&precisions);
}
