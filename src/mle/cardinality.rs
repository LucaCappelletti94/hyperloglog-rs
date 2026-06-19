//! Ertl's secant-method single-counter cardinality MLE.

use crate::prelude::*;
use crate::utils::{FloatOps, Zero};
#[allow(unused_imports)]
use num_traits::Float;

#[allow(clippy::too_many_lines)]
/// Single-counter cardinality via Ertl's secant-method Maximum Likelihood Estimation.
///
/// # Arguments
/// * `registers` - Iterator over the counter's register values.
/// * `harmonic_sum` - The counter's harmonic sum (sum of `2^-register`).
/// * `is_full` - Whether the counter is saturated (returns infinity).
/// * `error_exponent` - The secant method stops once the relative step is below
///   `10^-error_exponent` scaled by the precision.
pub(crate) fn mle_cardinality<P: Precision, B: Bits>(
    registers: impl Iterator<Item = u8>,
    harmonic_sum: f64,
    is_full: bool,
    error_exponent: i32,
) -> f64 {
    if is_full {
        return f64::INFINITY;
    }

    let multiplicities_len = 1_usize << B::NUMBER_OF_BITS;
    debug_assert!(multiplicities_len <= crate::mle::REGISTER_MULTIPLICITIES_CAPACITY);
    // Stack histogram sized to the widest `Bits`; only the first `multiplicities_len` entries are
    // ever touched, so this is bit-identical to the former `vec![f64::ZERO; multiplicities_len]`.
    let mut multiplicities = [f64::ZERO; crate::mle::REGISTER_MULTIPLICITIES_CAPACITY];
    let q = multiplicities_len as u32 - 2;

    let mut smallest_register_value: u32 = q;
    let mut largest_register_value: u32 = 0;

    for register in registers {
        let register = u32::from(register);
        if register > 0 {
            smallest_register_value = smallest_register_value.min(register);
        }
        largest_register_value = largest_register_value.max(register);
        multiplicities[register as usize] += 1.0;
    }

    smallest_register_value = smallest_register_value.max(1);
    largest_register_value = largest_register_value.min(q).max(1);

    let number_of_registers = f64::integer_exp2(P::EXPONENT);

    let c =
        multiplicities[multiplicities_len - 1] + multiplicities[largest_register_value as usize];

    let mut g_prev: f64 = 0.0;
    let number_of_zero_registers = multiplicities[0];
    let reciprocal_saturated_registers = multiplicities[multiplicities_len - 1]
        * f64::integer_exp2_minus((multiplicities_len - 1) as u8);

    let harmonic_sum_minus_zero_and_saturated =
        harmonic_sum - (number_of_zero_registers + reciprocal_saturated_registers);

    let a = harmonic_sum_minus_zero_and_saturated + number_of_zero_registers;
    let b = harmonic_sum_minus_zero_and_saturated
        + multiplicities[multiplicities_len - 1] * f64::integer_exp2_minus(q as u8);

    let number_of_non_zero_registers = number_of_registers - number_of_zero_registers;

    let mut x = if b <= 1.5 * a {
        number_of_non_zero_registers / (0.5 * b + a)
    } else {
        (number_of_non_zero_registers / b) * (b / a).ln_1p()
    };

    // We begin the secant method iterations.
    let mut delta_x = x;
    let relative_error_limit = 10.0_f64.powi(-error_exponent) / number_of_registers.sqrt();

    let forty_five_recip = 1.0 / 45.0;
    let four_seventy_two_point_five_recip = 1.0 / 472.5;

    let taylor = |x_first: f64, h: f64| -> f64 { (x_first + h * (1.0 - h)) / (x_first + 1.0 - h) };

    while delta_x > x * relative_error_limit {
        // Equivalent to `2 + floor(log2(x))`, saturating non-positive exponents to 0.
        let k: u32 = 2 + (x.log2().floor().max(0.0) as u32);

        let maximal = largest_register_value.max(k);
        let mut x_first = x * f64::integer_exp2_minus((maximal + 1) as u8);
        let x_second = x_first * x_first;
        let x_forth = x_second * x_second;
        let mut taylor_series_approximation = x_first - x_second / 3.0
            + x_forth * (forty_five_recip - x_second * four_seventy_two_point_five_recip);

        for _ in largest_register_value..k {
            taylor_series_approximation = taylor(x_first, taylor_series_approximation);
            x_first *= 2.0;
        }

        let mut g = c * taylor_series_approximation;

        for register_value in (smallest_register_value..largest_register_value).rev() {
            taylor_series_approximation = taylor(x_first, taylor_series_approximation);
            g += multiplicities[register_value as usize] * taylor_series_approximation;
            x_first *= 2.0;
        }

        g += x * a;

        if g > g_prev && number_of_non_zero_registers >= g {
            delta_x *= (number_of_non_zero_registers - g) / (g - g_prev);
        } else {
            delta_x = 0.0;
        }

        x += delta_x;
        g_prev = g;
    }

    number_of_registers * x
}

#[cfg(test)]
mod parity {
    //! Parity harness for the register-multiplicity histogram refactor: a frozen, heap-`Vec`-backed
    //! copy of `mle_cardinality` as it was before the histogram moved to a stack array. The
    //! production fn must produce BIT-IDENTICAL output to this reference for every register
    //! configuration (the refactor only changes where the histogram lives, never the arithmetic).
    use super::*;
    use crate::prelude::{Bits4, Bits5, Bits6, Precision6, Precision8};

    /// Frozen heap-`Vec` reference (verbatim copy of the pre-refactor `mle_cardinality`).
    #[allow(clippy::too_many_lines)]
    fn mle_cardinality_vec_reference<P: Precision, B: Bits>(
        registers: impl Iterator<Item = u8>,
        harmonic_sum: f64,
        is_full: bool,
        error_exponent: i32,
    ) -> f64 {
        if is_full {
            return f64::INFINITY;
        }

        let multiplicities_len = 1_usize << B::NUMBER_OF_BITS;
        let mut multiplicities = vec![f64::ZERO; multiplicities_len];
        let q = multiplicities_len as u32 - 2;

        let mut smallest_register_value: u32 = q;
        let mut largest_register_value: u32 = 0;

        for register in registers {
            let register = u32::from(register);
            if register > 0 {
                smallest_register_value = smallest_register_value.min(register);
            }
            largest_register_value = largest_register_value.max(register);
            multiplicities[register as usize] += 1.0;
        }

        smallest_register_value = smallest_register_value.max(1);
        largest_register_value = largest_register_value.min(q).max(1);

        let number_of_registers = f64::integer_exp2(P::EXPONENT);

        let c = multiplicities[multiplicities_len - 1]
            + multiplicities[largest_register_value as usize];

        let mut g_prev: f64 = 0.0;
        let number_of_zero_registers = multiplicities[0];
        let reciprocal_saturated_registers = multiplicities[multiplicities_len - 1]
            * f64::integer_exp2_minus((multiplicities_len - 1) as u8);

        let harmonic_sum_minus_zero_and_saturated =
            harmonic_sum - (number_of_zero_registers + reciprocal_saturated_registers);

        let a = harmonic_sum_minus_zero_and_saturated + number_of_zero_registers;
        let b = harmonic_sum_minus_zero_and_saturated
            + multiplicities[multiplicities_len - 1] * f64::integer_exp2_minus(q as u8);

        let number_of_non_zero_registers = number_of_registers - number_of_zero_registers;

        let mut x = if b <= 1.5 * a {
            number_of_non_zero_registers / (0.5 * b + a)
        } else {
            (number_of_non_zero_registers / b) * (b / a).ln_1p()
        };

        let mut delta_x = x;
        let relative_error_limit = 10.0_f64.powi(-error_exponent) / number_of_registers.sqrt();

        let forty_five_recip = 1.0 / 45.0;
        let four_seventy_two_point_five_recip = 1.0 / 472.5;

        let taylor =
            |x_first: f64, h: f64| -> f64 { (x_first + h * (1.0 - h)) / (x_first + 1.0 - h) };

        while delta_x > x * relative_error_limit {
            let k: u32 = 2 + (x.log2().floor().max(0.0) as u32);

            let maximal = largest_register_value.max(k);
            let mut x_first = x * f64::integer_exp2_minus((maximal + 1) as u8);
            let x_second = x_first * x_first;
            let x_forth = x_second * x_second;
            let mut taylor_series_approximation = x_first - x_second / 3.0
                + x_forth * (forty_five_recip - x_second * four_seventy_two_point_five_recip);

            for _ in largest_register_value..k {
                taylor_series_approximation = taylor(x_first, taylor_series_approximation);
                x_first *= 2.0;
            }

            let mut g = c * taylor_series_approximation;

            for register_value in (smallest_register_value..largest_register_value).rev() {
                taylor_series_approximation = taylor(x_first, taylor_series_approximation);
                g += multiplicities[register_value as usize] * taylor_series_approximation;
                x_first *= 2.0;
            }

            g += x * a;

            if g > g_prev && number_of_non_zero_registers >= g {
                delta_x *= (number_of_non_zero_registers - g) / (g - g_prev);
            } else {
                delta_x = 0.0;
            }

            x += delta_x;
            g_prev = g;
        }

        number_of_registers * x
    }

    /// Builds a deterministic register vector of `2^P` values in `0..2^B` from `seed`, and the
    /// matching harmonic sum, then checks the production fn against the reference bit-for-bit.
    fn check<P: Precision, B: Bits>(seed: u64) -> bool {
        let number_of_registers = 1_usize << P::EXPONENT;
        let register_modulo = 1_u64 << B::NUMBER_OF_BITS;
        let mut state = seed;
        let mut registers = alloc::vec::Vec::with_capacity(number_of_registers);
        for _ in 0..number_of_registers {
            state = state
                .wrapping_mul(0x5851_F42D_4C95_7F2D)
                .wrapping_add(0x1405_7B7E_F767_814F);
            registers.push((state >> 33).wrapping_rem(register_modulo) as u8);
        }
        let harmonic_sum: f64 = registers
            .iter()
            .map(|&register| f64::integer_exp2_minus(register))
            .sum();

        let produced = mle_cardinality::<P, B>(registers.iter().copied(), harmonic_sum, false, 2);
        let reference = mle_cardinality_vec_reference::<P, B>(
            registers.iter().copied(),
            harmonic_sum,
            false,
            2,
        );
        produced.to_bits() == reference.to_bits()
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig { cases: 256, ..Default::default() })]

        #[test]
        fn cardinality_mle_array_matches_vec_reference_p8_b6(seed in proptest::prelude::any::<u64>()) {
            proptest::prop_assert!(check::<Precision8, Bits6>(seed));
        }

        #[test]
        fn cardinality_mle_array_matches_vec_reference_p8_b4(seed in proptest::prelude::any::<u64>()) {
            proptest::prop_assert!(check::<Precision8, Bits4>(seed));
        }

        #[test]
        fn cardinality_mle_array_matches_vec_reference_p6_b5(seed in proptest::prelude::any::<u64>()) {
            proptest::prop_assert!(check::<Precision6, Bits5>(seed));
        }
    }
}
