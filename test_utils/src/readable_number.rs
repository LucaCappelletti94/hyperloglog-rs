use core::fmt::Display;

pub trait ReadableNumber: Display {
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

impl ReadableNumber for f64 {}
impl ReadableNumber for i32 {}
impl ReadableNumber for u32 {}
impl ReadableNumber for usize {}
