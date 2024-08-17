//! Struct marker MLE.

use crate::hll_impl;
use crate::prelude::*;
use core::cmp::Ordering;
use core::f64;

#[derive(Debug, Clone, Copy, Hash)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
/// A struct representing the Maximum Likelihood Estimation.
pub struct MLE<H, const ERROR: i32 = 2> {
    /// The underlying counter.
    counter: H,
}

hll_impl!(MLE<LogLogBeta<P, B, R, Hasher>, 2>);
hll_impl!(MLE<LogLogBeta<P, B, R, Hasher>, 3>);
hll_impl!(MLE<PlusPlus<P, B, R, Hasher>, 2>);
hll_impl!(MLE<PlusPlus<P, B, R, Hasher>, 3>);

impl<H, const ERROR: i32> From<H> for MLE<H, ERROR> {
    #[inline]
    fn from(counter: H) -> Self {
        Self { counter }
    }
}

#[cfg(feature = "std")]
impl<const ERROR: i32, H: Named> Named for MLE<H, ERROR> {
    #[inline]
    fn name(&self) -> String {
        format!("MLE{}{}", ERROR, self.counter.name())
    }
}

/// Compute the union cardinality using the Maximum Likelihood Estimation.
fn mle_union_cardinality<
    P: Precision,
    B: Bits,
    Hasher: HasherType,
    H: HyperLogLog<P, B, Hasher> + Estimator<f64>,
    const ERROR: i32,
>(
    left: &H,
    right: &H,
    estimate: fn(f64, P::NumberOfRegisters) -> f64,
) -> f64 {
    let mut left_multiplicities_larger =
        vec![f64::ZERO; usize::from(maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS))];
    let mut left_multiplicities_smaller =
        vec![f64::ZERO; usize::from(maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS))];
    let mut right_multiplicities_larger =
        vec![f64::ZERO; usize::from(maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS))];
    let mut right_multiplicities_smaller =
        vec![f64::ZERO; usize::from(maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS))];
    let mut joint_multiplicities =
        vec![f64::ZERO; usize::from(maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS))];
    let mut union_harmonic_sum = f64::ZERO;
    let mut union_zeros = P::NumberOfRegisters::ZERO;

    for (left_register, right_register) in left.registers().iter_registers_zipped(right.registers())
    {
        let larger_register = match left_register.cmp(&right_register) {
            Ordering::Less => {
                left_multiplicities_smaller[usize::from(left_register)] += f64::ONE;
                right_multiplicities_larger[usize::from(right_register)] += f64::ONE;
                right_register
            }
            Ordering::Greater => {
                left_multiplicities_larger[usize::from(left_register)] += f64::ONE;
                right_multiplicities_smaller[usize::from(right_register)] += f64::ONE;
                left_register
            }
            Ordering::Equal => {
                // If left register is equal to right register
                joint_multiplicities[usize::from(left_register)] += f64::ONE;
                left_register
            }
        };

        union_harmonic_sum += f64::integer_exp2_minus(larger_register);
        union_zeros += P::NumberOfRegisters::from_bool(larger_register.is_zero());
    }

    // We get the best estimates from HyperLogLog++
    let union_cardinality_estimate = estimate(union_harmonic_sum, union_zeros);

    let left_cardinality_estimate = left.estimate_cardinality();
    let right_cardinality_estimate = right.estimate_cardinality();

    // If the sum of the number of registers equal to zero, i.e.
    // the first value in the multiplicities vectors, is equal
    // to the number of registers, it means that the intersection
    // is empty.
    if union_zeros == P::NUMBER_OF_REGISTERS {
        return f64::ZERO;
    }

    let intersection: f64 = (left_cardinality_estimate + right_cardinality_estimate
        - union_cardinality_estimate)
        .max(f64::EPSILON);

    let left_difference: f64 =
        (union_cardinality_estimate - right_cardinality_estimate).max(f64::EPSILON);

    let right_difference: f64 =
        (union_cardinality_estimate - left_cardinality_estimate).max(f64::EPSILON);

    let relative_error_limit = 10.0_f64.powi(-ERROR) / f64::integer_exp2(P::EXPONENT).sqrt();

    // we introdce the following expressions to simplify the computation
    // of the gradient.
    let x = |phi: f64, two_to_minus_register: f64| -> f64 { phi.exp() * two_to_minus_register };

    let yz = |x: f64| -> (f64, f64) {
        let exp_m1 = (-x).exp_m1();
        (f64::ONE + exp_m1, -exp_m1)
    };

    // We precompute q and q+1 for reference.
    let q_plus_one: u8 = maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS) - 1;
    let q: u8 = q_plus_one - 1;

    // We initialize the vectors for the Adam optimizer.
    let mut phis = [
        left_difference.ln(),
        right_difference.ln(),
        intersection.ln(),
    ];
    let mut gradients: [f64; 3] = [f64::ZERO, f64::ZERO, f64::ZERO];

    let mut optimizer: Adam<3> = Adam::default();

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

    let two_to_zero: f64 = f64::integer_exp2_minus(P::EXPONENT);
    let two_to_minus_q: f64 = f64::integer_exp2_minus(P::EXPONENT + q);

    for _ in 0_u16..10_000_u16 {
        let x_left_0 = x(phis[0], two_to_zero);
        let x_right_0 = x(phis[1], two_to_zero);
        let x_joint_0 = x(phis[2], two_to_zero);
        let x_left_q = x(phis[0], two_to_minus_q);
        let (y_left_q, z_left_q) = yz(x_left_q);
        let x_right_q = x(phis[1], two_to_minus_q);
        let (y_right_q, z_right_q) = yz(x_right_q);
        let x_joint_q = x(phis[2], two_to_minus_q);
        let (y_joint_q, z_joint_q) = yz(x_joint_q);

        let denominator = f64::ONE / (z_joint_q + y_joint_q * z_left_q * z_right_q);

        let xl_yl_q = x_left_q * y_left_q;
        let xr_yr_q = x_right_q * y_right_q;
        let xj_yjoint_q = x_joint_q * y_joint_q;
        let shared_factor =
            if intersection_number_of_saturated_registers > f64::ZERO && y_joint_q > f64::EPSILON {
                intersection_number_of_saturated_registers * y_joint_q * denominator
            } else {
                f64::ZERO
            };

        gradients[0] = if xl_yl_q > f64::EPSILON {
            xl_yl_q * (shared_factor * z_right_q + left_number_of_saturated_registers / z_left_q)
        } else {
            f64::ZERO
        };
        gradients[0] -= left_number_of_zeros * x_left_0;

        gradients[1] = if xr_yr_q > f64::EPSILON {
            xr_yr_q * (shared_factor * z_left_q + right_number_of_saturated_registers / z_right_q)
        } else {
            f64::ZERO
        };

        gradients[1] -= right_number_of_zeros * x_right_0;

        gradients[2] = if intersection_number_of_saturated_registers > f64::ZERO
            && xj_yjoint_q > f64::EPSILON
            && denominator.is_finite()
        {
            intersection_number_of_saturated_registers
                * xj_yjoint_q
                * (y_left_q + z_left_q * y_right_q)
                * denominator
        } else {
            f64::ZERO
        };
        gradients[2] -= intersection_number_of_zeros * x_joint_0;

        (1..q_plus_one).for_each(|register_value| {
            let two_to_minus_register = f64::integer_exp2_minus(P::EXPONENT + register_value);

            let x_left = x(phis[0], two_to_minus_register);
            let x_right = x(phis[1], two_to_minus_register);
            let x_joint = x(phis[2], two_to_minus_register);
            let (y_left, z_left) = yz(x_left);
            let (y_right, z_right) = yz(x_right);
            let (y_joint, z_joint) = yz(x_joint);

            let joint_k = joint_multiplicities[usize::from(register_value)];
            let left_smaller_k = left_multiplicities_smaller[usize::from(register_value)];
            let left_larger_k = left_multiplicities_larger[usize::from(register_value)];
            let right_smaller_k = right_multiplicities_smaller[usize::from(register_value)];
            let right_larger_k = right_multiplicities_larger[usize::from(register_value)];

            let yjoint_zleft = y_joint * z_left;
            let yjoint_right_zleft = yjoint_zleft * y_right;
            let yjoint_zright = y_joint * z_right;
            let yjointleft_zright = yjoint_zright * y_left;
            let yjointleft = y_joint * y_left;
            let yjointright = y_joint * y_right;
            let yjoint_zlr = yjoint_zleft * z_right;
            let mut zj_plus_yjoint_zleft = z_joint + yjoint_zleft;
            if zj_plus_yjoint_zleft < f64::EPSILON {
                zj_plus_yjoint_zleft = f64::EPSILON;
            }
            let reciprocal_zj_plus_yjoint_zleft = f64::ONE / zj_plus_yjoint_zleft;
            let mut zj_plus_yjoint_zright = z_joint + yjoint_zright;
            if zj_plus_yjoint_zright < f64::EPSILON {
                zj_plus_yjoint_zright = f64::EPSILON;
            }
            let reciprocal_zj_plus_yjoint_zright = f64::ONE / zj_plus_yjoint_zright;
            let mut zj_plus_yjoint_zlr = z_joint + yjoint_zlr;
            if zj_plus_yjoint_zlr < f64::EPSILON {
                zj_plus_yjoint_zlr = f64::EPSILON;
            }
            let reciprocal_zj_plus_yjoint_zlr = f64::ONE / zj_plus_yjoint_zlr;

            let left_reciprocal =
                left_smaller_k * (reciprocal_zj_plus_yjoint_zleft * yjointleft - f64::ONE);
            let right_reciprocal =
                right_smaller_k * (reciprocal_zj_plus_yjoint_zright * yjointright - f64::ONE);

            if x_left > f64::EPSILON {
                gradients[0] += x_left
                    * (left_reciprocal
                        + joint_k * (yjointleft_zright * reciprocal_zj_plus_yjoint_zlr - f64::ONE)
                        + left_larger_k * (y_left / z_left - f64::ONE));
            }

            if x_right > f64::EPSILON {
                gradients[1] += x_right
                    * (right_reciprocal
                        + joint_k
                            * (yjoint_right_zleft * reciprocal_zj_plus_yjoint_zlr - f64::ONE)
                        + right_larger_k * (y_right / z_right - f64::ONE));
            }

            if x_joint > f64::EPSILON {
                gradients[2] += x_joint
                    * (left_reciprocal
                        + right_reciprocal
                        + joint_k
                            * ((yjointleft + yjoint_right_zleft) * reciprocal_zj_plus_yjoint_zlr
                                - f64::ONE));
            }
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

    phis[0].exp() + phis[1].exp() + phis[2].exp()
}

impl<const ERROR: i32, P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> Estimator<f64>
    for MLE<LogLogBeta<P, B, R, Hasher>, ERROR>
where
    Self: HyperLogLog<P, B, Hasher>,
{
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        self.counter.estimate_cardinality()
    }

    #[inline]
    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        true
    }

    #[inline]
    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        mle_union_cardinality::<P, B, Hasher, LogLogBeta<P, B, R, Hasher>, ERROR>(
            &self.counter,
            &other.counter,
            P::beta_estimate,
        )
    }
}

impl<const ERROR: i32, P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> Estimator<f64>
    for MLE<PlusPlus<P, B, R, Hasher>, ERROR>
where
    Self: HyperLogLog<P, B, Hasher>,
{
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        self.counter.estimate_cardinality()
    }

    #[inline]
    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        true
    }

    #[inline]
    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        mle_union_cardinality::<P, B, Hasher, PlusPlus<P, B, R, Hasher>, ERROR>(
            &self.counter,
            &other.counter,
            P::plusplus_estimate,
        )
    }
}

/// Adam optimizer for the Maximum Likelihood Estimation.
struct Adam<const N: usize> {
    /// First moments.
    first_moments: [f64; N],
    /// Second moments.
    second_moments: [f64; N],
    /// Current time.
    time: i32,
    /// Learning rate.
    learning_rate: f64,
    /// First order decay factor.
    first_order_decay_factor: f64,
    /// Second order decay factor.
    second_order_decay_factor: f64,
}

impl<const N: usize> Default for Adam<N> {
    fn default() -> Self {
        Adam {
            first_moments: [0.0; N],
            second_moments: [0.0; N],
            time: 0,
            learning_rate: 0.01,
            first_order_decay_factor: 0.9,
            second_order_decay_factor: 0.999,
        }
    }
}

impl<const N: usize> Adam<N> {
    /// Apply the Adam optimizer to the gradients and weights.
    fn apply(&mut self, gradients: &mut [f64; N], phis: &mut [f64; N]) {
        self.time += 1_i32;
        self.first_moments
            .iter_mut()
            .zip(self.second_moments.iter_mut())
            .zip(gradients.iter_mut().zip(phis.iter_mut()))
            .for_each(|((first_moment, second_moment), (gradient, phi))| {
                *first_moment = self.first_order_decay_factor * *first_moment
                    + (f64::ONE - self.first_order_decay_factor) * *gradient;
                *second_moment = self.second_order_decay_factor * *second_moment
                    + (f64::ONE - self.second_order_decay_factor) * (*gradient).powi(2);
                let adaptative_learning_rate = self.learning_rate
                    * (f64::ONE - self.second_order_decay_factor.powi(self.time)).sqrt()
                    / (f64::ONE - self.first_order_decay_factor.powi(self.time));
                let second_moment_root = (*second_moment).sqrt();
                *gradient = adaptative_learning_rate * (*first_moment)
                    / if second_moment_root > f64::EPSILON {
                        second_moment_root
                    } else {
                        f64::EPSILON
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
        let mut adam = Adam::<2>::default();

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
