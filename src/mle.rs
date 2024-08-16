//! Struct marker MLE.

use crate::hll_impl;
use crate::prelude::*;

#[derive(Debug, Clone, Copy, Hash)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
/// A struct representing the Maximum Likelihood Estimation.
pub struct MLE<H, const ERROR: i32 = 2> {
    counter: H,
}

hll_impl!(MLE<LogLogBeta<P, B, R, Hasher>, 2>);
hll_impl!(MLE<LogLogBeta<P, B, R, Hasher>, 3>);
hll_impl!(MLE<PlusPlus<P, B, R, Hasher>, 2>);
hll_impl!(MLE<PlusPlus<P, B, R, Hasher>, 3>);

impl<H, const ERROR: i32> From<H> for MLE<H, ERROR> {
    fn from(counter: H) -> Self {
        Self { counter }
    }
}

impl<const ERROR: i32, H: Named> Named for MLE<H, ERROR> {
    fn name(&self) -> String {
        format!("MLE{}{}", ERROR, self.counter.name())
    }
}

fn mle_union_cardinality<
    P: Precision + PrecisionConstants<F>,
    B: Bits,
    F: FloatNumber,
    Hasher: HasherType,
    H: HyperLogLog<P, B, Hasher> + Estimator<F>,
    const ERROR: i32,
>(
    left: &H,
    right: &H,
    estimate: fn(F, P::NumberOfZeros) -> F,
) -> F {
    let mut left_multiplicities_larger =
        vec![F::ZERO; maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS)];
    let mut left_multiplicities_smaller =
        vec![F::ZERO; maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS)];
    let mut right_multiplicities_larger =
        vec![F::ZERO; maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS)];
    let mut right_multiplicities_smaller =
        vec![F::ZERO; maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS)];
    let mut joint_multiplicities =
        vec![F::ZERO; maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS)];
    let mut union_harmonic_sum = F::ZERO;
    let mut union_zeros = P::NumberOfZeros::ZERO;

    for (left, right) in left.registers().iter_registers_zipped(right.registers()) {
        let larger_register = match left.cmp(&right) {
            core::cmp::Ordering::Less => {
                left_multiplicities_smaller[left as usize] += F::ONE;
                right_multiplicities_larger[right as usize] += F::ONE;
                right
            }
            core::cmp::Ordering::Greater => {
                left_multiplicities_larger[left as usize] += F::ONE;
                right_multiplicities_smaller[right as usize] += F::ONE;
                left
            }
            core::cmp::Ordering::Equal => {
                // If left register is equal to right register
                joint_multiplicities[left as usize] += F::ONE;
                left
            }
        };

        union_harmonic_sum += F::inverse_register(larger_register as i32);
        union_zeros += P::NumberOfZeros::from_bool(larger_register.is_zero());
    }

    // We get the best estimates from HyperLogLog++
    let union_cardinality_estimate = estimate(union_harmonic_sum, union_zeros);

    let left_cardinality_estimate = left.estimate_cardinality();
    let right_cardinality_estimate = right.estimate_cardinality();

    // If the sum of the number of registers equal to zero, i.e.
    // the first value in the multiplicities vectors, is equal
    // to the number of registers, it means that the intersection
    // is empty.

    let number_of_zeros: usize = (left_multiplicities_smaller[0]
        + left_multiplicities_smaller[0]
        + right_multiplicities_smaller[0])
        .to_usize();
    if number_of_zeros == P::NUMBER_OF_REGISTERS {
        return F::ZERO;
    }

    let mut intersection: F =
        left_cardinality_estimate + right_cardinality_estimate - union_cardinality_estimate;
    if intersection < F::ZERO {
        intersection = F::EPSILON;
    }

    let left_difference: F = union_cardinality_estimate - right_cardinality_estimate;
    if left_difference < F::ZERO {
        return F::EPSILON;
    }

    let right_difference: F = union_cardinality_estimate - left_cardinality_estimate;
    if right_difference < F::ZERO {
        return F::EPSILON;
    }

    let relative_error_limit = F::TEN.powi(-ERROR) / F::from_usize(P::NUMBER_OF_REGISTERS).sqrt();
    debug_assert!(intersection >= F::ZERO);
    debug_assert!(left_difference >= F::ZERO);
    debug_assert!(right_difference >= F::ZERO);

    // we introdce the following expressions to simplify the computation
    // of the gradient.
    let x = |phi: F, two_to_minus_register: F| -> F { phi.exp() * two_to_minus_register };

    let yz = |x: F| -> (F, F) {
        let exp_m1 = (-x).exp_m1();
        (F::ONE + exp_m1, -exp_m1)
    };

    // We precompute q and q+1 for reference.
    let q_plus_one: usize = maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS) - 1;
    let q: i32 = q_plus_one as i32 - 1;

    // We initialize the vectors for the Adam optimizer.
    let mut phis = [
        left_difference.ln(),
        right_difference.ln(),
        intersection.ln(),
    ];
    let mut gradients: [F; 3] = [F::ZERO, F::ZERO, F::ZERO];

    let mut optimizer: Adam<F, 3> = Default::default();

    let left_number_of_zeros =
        left_multiplicities_smaller[0] + left_multiplicities_larger[0] + joint_multiplicities[0];
    let right_number_of_zeros =
        right_multiplicities_smaller[0] + right_multiplicities_larger[0] + joint_multiplicities[0];
    let intersection_number_of_zeros =
        right_multiplicities_smaller[0] + left_multiplicities_smaller[0] + joint_multiplicities[0];

    let left_number_of_saturated_registers =
        left_multiplicities_larger[left_multiplicities_larger.len() - 1];
    let right_number_of_saturated_registers =
        right_multiplicities_larger[right_multiplicities_larger.len() - 1];
    let intersection_number_of_saturated_registers =
        joint_multiplicities[joint_multiplicities.len() - 1];

    let two_to_zero: F = F::inverse_register(P::EXPONENT as i32);
    let two_to_minus_q: F = F::inverse_register(P::EXPONENT as i32 + q as i32);

    for _ in 0..10_000 {
        let x_left_0 = x(phis[0], two_to_zero);
        debug_assert!(x_left_0 >= F::ZERO);
        debug_assert!(x_left_0.is_finite());
        let x_right_0 = x(phis[1], two_to_zero);
        debug_assert!(x_right_0 >= F::ZERO);
        debug_assert!(
            x_right_0.is_finite(),
            "x(phis[1]: {}, two_to_zero: {}) = {}",
            phis[1],
            two_to_zero,
            x_right_0
        );
        let x_joint_0 = x(phis[2], two_to_zero);
        debug_assert!(x_joint_0 >= F::ZERO, "x_joint_0: {}", x_joint_0);
        debug_assert!(x_joint_0.is_finite());
        let x_left_q = x(phis[0], two_to_minus_q);
        debug_assert!(x_left_q >= F::ZERO);
        debug_assert!(x_left_q.is_finite());
        let (y_left_q, z_left_q) = yz(x_left_q);
        let x_right_q = x(phis[1], two_to_minus_q);
        debug_assert!(x_right_q >= F::ZERO);
        debug_assert!(x_right_q.is_finite());
        let (y_right_q, z_right_q) = yz(x_right_q);
        let x_joint_q = x(phis[2], two_to_minus_q);
        debug_assert!(x_joint_q >= F::ZERO);
        debug_assert!(x_joint_q.is_finite());
        let (y_joint_q, z_joint_q) = yz(x_joint_q);

        let denominator = F::ONE / (z_joint_q + y_joint_q * z_left_q * z_right_q);

        let xl_yl_q = x_left_q * y_left_q;
        let xr_yr_q = x_right_q * y_right_q;
        let xj_yj_q = x_joint_q * y_joint_q;
        let shared_factor =
            if intersection_number_of_saturated_registers > F::ZERO && y_joint_q > F::EPSILON {
                intersection_number_of_saturated_registers * y_joint_q * denominator
            } else {
                F::ZERO
            };

        gradients[0] = if xl_yl_q > F::EPSILON {
            xl_yl_q * (shared_factor * z_right_q + left_number_of_saturated_registers / z_left_q)
        } else {
            F::ZERO
        };
        gradients[0] -= left_number_of_zeros * x_left_0;

        debug_assert!(gradients[0].is_finite());
        debug_assert!(z_right_q >= F::ZERO);

        gradients[1] = if xr_yr_q > F::EPSILON {
            xr_yr_q * (shared_factor * z_left_q + right_number_of_saturated_registers / z_right_q)
        } else {
            F::ZERO
        };

        gradients[1] -= right_number_of_zeros * x_right_0;

        debug_assert!(
            gradients[1].is_finite(),
            concat!(
                "The gradient is not finite: {}. ",
                "We computed this gradient with the following values: ",
                "xr_yr_q({}) * (shared_factor({}) * z_left_q({}) + right_number_of_saturated_registers({}) / z_right_q({})) - right_number_of_zeros({}) * x_right_0({})",
            ),
            gradients[1],
            xr_yr_q,
            shared_factor,
            z_left_q,
            right_number_of_saturated_registers,
            z_right_q,
            right_number_of_zeros,
            x_right_0,
        );

        gradients[2] = if intersection_number_of_saturated_registers > F::ZERO
            && xj_yj_q > F::EPSILON
            && denominator.is_finite()
        {
            intersection_number_of_saturated_registers
                * xj_yj_q
                * (y_left_q + z_left_q * y_right_q)
                * denominator
        } else {
            F::ZERO
        };
        gradients[2] -= intersection_number_of_zeros * x_joint_0;

        debug_assert!(gradients[2].is_finite());

        (1..q_plus_one as i32).for_each(|register_value| {
            let two_to_minus_register: F =
                F::inverse_register(P::EXPONENT as i32 + register_value);

            let x_left = x(phis[0], two_to_minus_register);
            debug_assert!(x_left >= F::ZERO);
            let x_right = x(phis[1], two_to_minus_register);
            debug_assert!(x_right >= F::ZERO);
            let x_joint = x(phis[2], two_to_minus_register);
            debug_assert!(x_joint >= F::ZERO);
            let (y_left, z_left) = yz(x_left);
            let (y_right, z_right) = yz(x_right);
            let (y_joint, z_joint) = yz(x_joint);

            let joint_k = joint_multiplicities[register_value as usize];
            let left_smaller_k = left_multiplicities_smaller[register_value as usize];
            let left_larger_k = left_multiplicities_larger[register_value as usize];
            let right_smaller_k = right_multiplicities_smaller[register_value as usize];
            let right_larger_k = right_multiplicities_larger[register_value as usize];

            let yj_zl = y_joint * z_left;
            let yjr_zl = yj_zl * y_right;
            let yj_zr = y_joint * z_right;
            let yjl_zr = yj_zr * y_left;
            let yjl = y_joint * y_left;
            let yjr = y_joint * y_right;
            let yj_zlr = yj_zl * z_right;
            let mut zj_plus_yj_zl = z_joint + yj_zl;
            debug_assert!(zj_plus_yj_zl >= F::ZERO);
            if zj_plus_yj_zl < F::EPSILON {
                zj_plus_yj_zl = F::EPSILON;
            }
            let reciprocal_zj_plus_yj_zl = F::ONE / zj_plus_yj_zl;
            let mut zj_plus_yj_zr = z_joint + yj_zr;
            debug_assert!(zj_plus_yj_zr >= F::ZERO);
            if zj_plus_yj_zr < F::EPSILON {
                zj_plus_yj_zr = F::EPSILON;
            }
            let reciprocal_zj_plus_yj_zr = F::ONE / zj_plus_yj_zr;
            let mut zj_plus_yj_zlr = z_joint + yj_zlr;
            debug_assert!(zj_plus_yj_zlr >= F::ZERO);
            if zj_plus_yj_zlr < F::EPSILON {
                zj_plus_yj_zlr = F::EPSILON;
            }
            let reciprocal_zj_plus_yj_zlr = F::ONE / zj_plus_yj_zlr;

            let left_reciprocal = left_smaller_k * (reciprocal_zj_plus_yj_zl * yjl - F::ONE);
            let right_reciprocal = right_smaller_k * (reciprocal_zj_plus_yj_zr * yjr - F::ONE);

            if x_left > F::EPSILON {
                gradients[0] += x_left
                    * (left_reciprocal
                        + joint_k * (yjl_zr * reciprocal_zj_plus_yj_zlr - F::ONE)
                        + left_larger_k * (y_left / z_left - F::ONE));
            }

            debug_assert!(gradients[0].is_finite());

            if x_right > F::EPSILON {
                gradients[1] += x_right
                    * (right_reciprocal
                        + joint_k * (yjr_zl * reciprocal_zj_plus_yj_zlr - F::ONE)
                        + right_larger_k * (y_right / z_right - F::ONE));
            }

            debug_assert!(
                gradients[1].is_finite(),
                concat!(
                    "The gradient is not finite: {}. ",
                    "We computed this gradient with the following values: ",
                    "x_right({}) * (right_reciprocal({}) + joint_k({}) * (yjr_zl({}) * reciprocal_zj_plus_yj_zlr({}) - 1) + right_larger_k({}) * (y_right({}) / z_right({}) - 1)",
                ),
                gradients[1],
                x_right,
                right_reciprocal,
                joint_k,
                yjr_zl,
                reciprocal_zj_plus_yj_zlr,
                right_larger_k,
                y_right,
                z_right,
            );

            if x_joint > F::EPSILON {
                gradients[2] += x_joint
                    * (left_reciprocal
                        + right_reciprocal
                        + joint_k * ((yjl + yjr_zl) * reciprocal_zj_plus_yj_zlr - F::ONE));
            }

            debug_assert!(gradients[2].is_finite());
        });

        // We execute the update of the Adam first and second moments.
        optimizer.apply(&mut gradients, &mut phis);

        // If any of the gradient update, in absolute value, is higher
        if gradients
            .iter()
            .all(|gradient| gradient.abs() <= relative_error_limit)
        {
            break;
        }
    }

    let left_difference = phis[0].exp();
    let right_difference = phis[1].exp();
    let intersection = phis[2].exp();

    left_difference + right_difference + intersection
}

impl<
        const ERROR: i32,
        F: FloatNumber,
        P: Precision,
        B: Bits,
        R: Registers<P, B>,
        Hasher: HasherType,
    > Estimator<F> for MLE<LogLogBeta<P, B, R, Hasher>, ERROR>
where
    P: PrecisionConstants<F>,
    Self: HyperLogLog<P, B, Hasher>,
{
    fn estimate_cardinality(&self) -> F {
        self.counter.estimate_cardinality()
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        true
    }

    fn estimate_union_cardinality(&self, other: &Self) -> F {
        mle_union_cardinality::<P, B, F, Hasher, LogLogBeta<P, B, R, Hasher>, ERROR>(
            &self.counter,
            &other.counter,
            P::beta_estimate,
        )
    }
}

impl<
        const ERROR: i32,
        F: FloatNumber,
        P: Precision,
        B: Bits,
        R: Registers<P, B>,
        Hasher: HasherType,
    > Estimator<F> for MLE<PlusPlus<P, B, R, Hasher>, ERROR>
where
    P: PrecisionConstants<F>,
    Self: HyperLogLog<P, B, Hasher>,
{
    fn estimate_cardinality(&self) -> F {
        self.counter.estimate_cardinality()
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        true
    }

    fn estimate_union_cardinality(&self, other: &Self) -> F {
        mle_union_cardinality::<P, B, F, Hasher, PlusPlus<P, B, R, Hasher>, ERROR>(
            &self.counter,
            &other.counter,
            P::plusplus_estimate,
        )
    }
}

trait Optimizer<F, const N: usize> {
    fn apply(&mut self, gradients: &mut [F; N], phis: &mut [F; N]);
}

struct Adam<F, const N: usize> {
    first_moments: [F; N],
    second_moments: [F; N],
    time: i32,
    learning_rate: F,
    first_order_decay_factor: F,
    second_order_decay_factor: F,
}

impl<F: FloatNumber, const N: usize> Default for Adam<F, N> {
    fn default() -> Self {
        Adam {
            first_moments: [F::ZERO; N],
            second_moments: [F::ZERO; N],
            time: 0,
            learning_rate: F::ONE / F::ONE_THOUSAND,
            first_order_decay_factor: F::from_usize(9) / F::TEN,
            second_order_decay_factor: F::from_usize(999) / F::ONE_THOUSAND,
        }
    }
}

impl<F: FloatNumber, const N: usize> Optimizer<F, N> for Adam<F, N> {
    #[inline(always)]
    fn apply(&mut self, gradients: &mut [F; N], phis: &mut [F; N]) {
        self.time += 1;
        self.first_moments
            .iter_mut()
            .zip(self.second_moments.iter_mut())
            .zip(gradients.iter_mut().zip(phis.iter_mut()))
            .for_each(|((first_moment, second_moment), (gradient, phi))| {
                *first_moment = self.first_order_decay_factor * *first_moment
                    + (F::ONE - self.first_order_decay_factor) * *gradient;
                *second_moment = self.second_order_decay_factor * *second_moment
                    + (F::ONE - self.second_order_decay_factor) * (*gradient).powi(2);
                let adaptative_learning_rate = self.learning_rate
                    * (F::ONE - self.second_order_decay_factor.powi(self.time)).sqrt()
                    / (F::ONE - self.first_order_decay_factor.powi(self.time));
                let second_moment_root = (*second_moment).sqrt();
                *gradient = adaptative_learning_rate * (*first_moment)
                    / if second_moment_root > F::EPSILON {
                        second_moment_root
                    } else {
                        F::EPSILON
                    };
                *phi += *gradient;
            });
    }
}

// Minimal test suite with trivial functions to optimize
#[cfg(test)]
mod tests {
    use super::*;

    // Test function: f(x) = -(x1 - 1)^2 - (x2 + 2)^2
    fn quadratic_function(phis: &[f64; 2]) -> (f64, [f64; 2]) {
        let value = -(phis[0] - 1.0).powi(2) - (phis[1] + 2.0).powi(2);
        let gradients = [-2.0 * (phis[0] - 1.0), -2.0 * (phis[1] + 2.0)];
        (value, gradients)
    }

    #[test]
    fn test_adam_optimizer() {
        let mut phis = [0.0, 0.0]; // Initial guess
        let mut adam = Adam::<f64, 2>::default();

        for _ in 0..10_000 {
            let (value, gradients) = quadratic_function(&phis);
            println!(
                "Current value: {:.6}, phis: [{:.6}, {:.6}]",
                value, phis[0], phis[1]
            );
            adam.apply(&mut gradients.clone(), &mut phis);
        }

        assert!((phis[0] - 1.0).abs() < 1e-4);
        assert!((phis[1] + 2.0).abs() < 1e-4);
    }
}
