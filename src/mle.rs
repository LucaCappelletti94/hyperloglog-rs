//! Maximum Likelihood Estimation of the union cardinality (Ertl's joint estimator).
//!
//! This provides [`HyperLogLog::estimate_union_cardinality_mle`], an alternative to the default
//! union estimator that maximizes the joint likelihood of the two counters' register
//! multiplicities via an Adam optimizer. It operates on the HyperLogLog register
//! representation; hash-list operands are materialized into registers first.
//!
//! Only the joint (union) estimator is implemented. A single-counter Maximum Likelihood
//! cardinality estimator is not provided; `estimate_cardinality` continues to use the
//! HyperLogLog++ corrected estimate.

use crate::correction_coefficients::{
    HYPERLOGLOG_CORRECTION_BIAS, HYPERLOGLOG_CORRECTION_CARDINALITIES,
};
use crate::hyperloglog::correct_cardinality;
use crate::prelude::*;
use crate::utils::{FloatOps, Zero};
use core::cmp::Ordering;
use core::ops::{Add, Mul, Sub};

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperLogLog<P, B, R, H> {
    /// Returns the union cardinality estimated with the joint Maximum Likelihood Estimation.
    ///
    /// # Implementative details
    /// The estimator operates on the HyperLogLog register multiplicities, so if either operand
    /// is still a hash list it is materialized into a fully-fledged HyperLogLog first. The
    /// likelihood of the left difference, right difference and intersection is maximized jointly
    /// with an Adam optimizer; the union estimate is their sum.
    #[inline]
    pub fn estimate_union_cardinality_mle(&self, other: &Self) -> f64 {
        if self.is_hash_list() || other.is_hash_list() {
            let mut left = self.clone();
            let mut right = other.clone();
            if left.is_hash_list() {
                left.convert_hash_list_to_hyperloglog().unwrap();
            }
            if right.is_hash_list() {
                right.convert_hash_list_to_hyperloglog().unwrap();
            }
            return left.mle_union_from_registers(&right);
        }

        self.mle_union_from_registers(other)
    }

    /// Joint MLE union estimate assuming both counters are in HyperLogLog (register) mode.
    fn mle_union_from_registers(&self, other: &Self) -> f64 {
        // Maps a union harmonic sum to the HyperLogLog++ corrected cardinality, exactly as the
        // default register-based union estimator does.
        let estimate = |harmonic_sum: f64, _zeros: u32| {
            correct_cardinality::<P, B>(
                P::ALPHA * f64::integer_exp2(P::EXPONENT + P::EXPONENT) / harmonic_sum,
                &HYPERLOGLOG_CORRECTION_CARDINALITIES[P::EXPONENT as usize - 4]
                    [B::NUMBER_OF_BITS as usize - 4],
                &HYPERLOGLOG_CORRECTION_BIAS[P::EXPONENT as usize - 4]
                    [B::NUMBER_OF_BITS as usize - 4],
            )
        };

        mle_union_cardinality::<P, B, _>(
            self.registers.iter_registers_zipped(&other.registers),
            self.estimate_cardinality(),
            other.estimate_cardinality(),
            estimate,
            2,
        )
    }
}

#[allow(clippy::too_many_lines)]
/// Computes the union cardinality using the Maximum Likelihood Estimation.
///
/// # Arguments
/// * `registers` - Iterator over the `[left, right]` register pairs of the two counters.
/// * `left_cardinality` / `right_cardinality` - Cardinality estimates of the two counters.
/// * `estimate` - Maps a union harmonic sum (and zero-register count) to a union cardinality.
/// * `error_exponent` - The optimizer stops once every gradient is below `10^-error_exponent`
///   scaled by the precision.
fn mle_union_cardinality<P: Precision, B: Bits, I: ExactSizeIterator<Item = [u8; 2]>>(
    registers: I,
    left_cardinality: f64,
    right_cardinality: f64,
    estimate: impl Fn(f64, u32) -> f64,
    error_exponent: i32,
) -> f64 {
    let mut left_multiplicities_larger = vec![f64::ZERO; 1 << B::NUMBER_OF_BITS];
    let mut left_multiplicities_smaller = vec![f64::ZERO; 1 << B::NUMBER_OF_BITS];
    let mut right_multiplicities_larger = vec![f64::ZERO; 1 << B::NUMBER_OF_BITS];
    let mut right_multiplicities_smaller = vec![f64::ZERO; 1 << B::NUMBER_OF_BITS];
    let mut joint_multiplicities = vec![f64::ZERO; 1 << B::NUMBER_OF_BITS];
    let mut union_harmonic_sum = f64::ZERO;
    let mut union_zeros = 0;

    for [left_register, right_register] in registers {
        let cmp = left_register.cmp(&right_register);

        let larger_register = if cmp == Ordering::Greater || cmp == Ordering::Equal {
            left_register
        } else {
            right_register
        };
        let left_register = usize::from(left_register);
        let right_register = usize::from(right_register);
        left_multiplicities_smaller[left_register] += f64::from(cmp == Ordering::Less);
        right_multiplicities_larger[right_register] += f64::from(cmp == Ordering::Less);
        left_multiplicities_larger[left_register] += f64::from(cmp == Ordering::Greater);
        right_multiplicities_smaller[right_register] += f64::from(cmp == Ordering::Greater);
        joint_multiplicities[left_register] += f64::from(cmp == Ordering::Equal);

        union_harmonic_sum += f64::integer_exp2_minus(larger_register);
        union_zeros += u32::from(larger_register.is_zero());
    }

    // We get the best estimates from HyperLogLog++
    let union_cardinality = estimate(union_harmonic_sum, union_zeros);

    // If the number of registers equal to zero in the union is equal to the number of
    // registers, the union is empty.
    if union_zeros == 1 << B::NUMBER_OF_BITS {
        return f64::ZERO;
    }

    let intersection: f64 =
        (left_cardinality + right_cardinality - union_cardinality).max(f64::EPSILON);

    let left_difference: f64 = (union_cardinality - right_cardinality).max(f64::EPSILON);

    let right_difference: f64 = (union_cardinality - left_cardinality).max(f64::EPSILON);

    let relative_error_limit =
        10.0_f64.powi(-error_exponent) / f64::integer_exp2(P::EXPONENT).sqrt();

    // we introduce the following expressions to simplify the computation
    // of the gradient.
    let x = |phi: [f64; 3], two_to_minus_register: f64| -> [f64; 3] {
        [
            (phi[0].exp() * two_to_minus_register).max(f64::EPSILON),
            (phi[1].exp() * two_to_minus_register).max(f64::EPSILON),
            (phi[2].exp() * two_to_minus_register).max(f64::EPSILON),
        ]
    };

    let yz = |x: [f64; 3]| -> ([f64; 3], [f64; 3]) {
        let exp_m1 = [(-x[0]).exp_m1(), (-x[1]).exp_m1(), (-x[2]).exp_m1()];

        (
            [
                (1.0 + exp_m1[0]).max(f64::EPSILON),
                (1.0 + exp_m1[1]).max(f64::EPSILON),
                (1.0 + exp_m1[2]).max(f64::EPSILON),
            ],
            [-exp_m1[0], -exp_m1[1], -exp_m1[2]],
        )
    };

    // We precompute q and q+1 for reference.
    let q_plus_one: u8 = (1 << B::NUMBER_OF_BITS) - 1;
    let q: u8 = q_plus_one - 1;

    // We initialize the vectors for the Adam optimizer.
    let mut phis = [
        left_difference.ln(),
        right_difference.ln(),
        intersection.ln(),
    ];
    let mut gradients: [f64; 3] = [f64::ZERO, f64::ZERO, f64::ZERO];

    let mut optimizer: Adam<3> = Adam::default();

    let zeros_0: [f64; 3] = [
        left_multiplicities_smaller[0] + left_multiplicities_larger[0] + joint_multiplicities[0],
        right_multiplicities_smaller[0] + right_multiplicities_larger[0] + joint_multiplicities[0],
        right_multiplicities_smaller[0] + left_multiplicities_smaller[0] + joint_multiplicities[0],
    ];

    let zeros_q: [f64; 3] = [
        left_multiplicities_larger[usize::from(q_plus_one)],
        right_multiplicities_larger[usize::from(q_plus_one)],
        joint_multiplicities[usize::from(q_plus_one)],
    ];

    let two_to_zero: f64 = f64::integer_exp2_minus(P::EXPONENT);
    let two_to_minus_q: f64 = f64::integer_exp2_minus(P::EXPONENT + q);

    for _ in 0_u16..10_000_u16 {
        let x_0 = x(phis, two_to_zero);
        let x_q = x(phis, two_to_minus_q);
        let (y_q, z_q) = yz(x_q);

        let denominator = 1.0 / (z_q[2] + y_q[2] * z_q[0] * z_q[1]);

        let y_q_saturated = y_q.ew_mul(zeros_q[2]);

        gradients[0] = y_q_saturated[2] * denominator * z_q[1] + zeros_q[0] / z_q[0];

        gradients[1] = y_q_saturated[2] * denominator * z_q[0] + zeros_q[1] / z_q[1];

        gradients[2] = y_q_saturated[0] * denominator + y_q_saturated[1] * z_q[0];
        gradients = (gradients.ew_mul(y_q.ew_mul(z_q))).ew_sub(zeros_0.ew_mul(x_0));

        (1..q_plus_one).for_each(|register_value| {
            let two_to_minus_register = f64::integer_exp2_minus(P::EXPONENT + register_value);

            let x_register = x(phis, two_to_minus_register);
            let (y_register, z_register) = yz(x_register);

            let joint_k = joint_multiplicities[usize::from(register_value)];
            let left_smaller_k = left_multiplicities_smaller[usize::from(register_value)];
            let left_larger_k = left_multiplicities_larger[usize::from(register_value)];
            let right_smaller_k = right_multiplicities_smaller[usize::from(register_value)];
            let right_larger_k = right_multiplicities_larger[usize::from(register_value)];

            let yjoint_right_zleft = y_register[2] * z_register[0] * y_register[1];
            let yjoint_left_zright = y_register[2] * z_register[1] * y_register[0];
            let zj_plus_yjoint_zright = z_register[2] + y_register[2] * z_register[1];
            let zj_plus_yjoint_zlr = z_register[2] + y_register[2] * z_register[0] * z_register[1];
            let reciprocal_zj_plus_yjoint_zlr = 1.0 / zj_plus_yjoint_zlr;

            let left_reciprocal = left_smaller_k
                * (y_register[2] * y_register[0] / (z_register[2] + y_register[2] * z_register[0])
                    - 1.0);
            let right_reciprocal =
                right_smaller_k * (y_register[2] * y_register[1] / zj_plus_yjoint_zright - 1.0);

            let delta = [
                left_reciprocal
                    + joint_k * (yjoint_left_zright * reciprocal_zj_plus_yjoint_zlr - 1.0)
                    + left_larger_k * (y_register[0] / z_register[0] - 1.0),
                right_reciprocal
                    + joint_k * (yjoint_right_zleft * reciprocal_zj_plus_yjoint_zlr - 1.0)
                    + right_larger_k * (y_register[1] / z_register[1] - 1.0),
                left_reciprocal
                    + right_reciprocal
                    + joint_k
                        * ((y_register[2] * y_register[0] + yjoint_right_zleft)
                            * reciprocal_zj_plus_yjoint_zlr
                            - 1.0),
            ];

            gradients = gradients.ew_add(x_register.ew_mul(delta));
        });

        // We execute the update of the Adam first and second moments.
        optimizer.apply(&mut gradients, &mut phis);

        // If every gradient update is, in absolute value, below the error limit, we stop.
        if gradients
            .iter()
            .all(|gradient| gradient.abs() <= relative_error_limit)
        {
            break;
        }
    }

    phis[0].exp() + phis[1].exp() + phis[2].exp()
}

/// Trait for element-wise multiplication.
trait ElementWiseMultiplication<Rhs = Self> {
    /// Element-wise multiplication.
    fn ew_mul(self, other: Rhs) -> Self;
}

impl<const N: usize, T: Default + Copy + Mul<T, Output = T>> ElementWiseMultiplication for [T; N] {
    #[inline]
    fn ew_mul(self, other: Self) -> Self {
        let mut result = [T::default(); N];
        for i in 0..N {
            result[i] = self[i] * other[i];
        }
        result
    }
}

impl<const N: usize, T: Default + Copy + Mul<T, Output = T>> ElementWiseMultiplication<T>
    for [T; N]
{
    #[inline]
    fn ew_mul(self, other: T) -> Self {
        let mut result = [T::default(); N];
        for i in 0..N {
            result[i] = self[i] * other;
        }
        result
    }
}

/// Trait for element-wise subtraction.
trait ElementWiseSubtraction {
    /// Element-wise subtraction.
    fn ew_sub(self, other: Self) -> Self;
}

impl<const N: usize, T: Default + Copy + Sub<T, Output = T>> ElementWiseSubtraction for [T; N] {
    #[inline]
    fn ew_sub(self, other: Self) -> Self {
        let mut result = [T::default(); N];
        for i in 0..N {
            result[i] = self[i] - other[i];
        }
        result
    }
}

/// Trait for element-wise addition.
trait ElementWiseAddition {
    /// Element-wise addition.
    fn ew_add(self, other: Self) -> Self;
}

impl<const N: usize, T: Default + Copy + Add<T, Output = T>> ElementWiseAddition for [T; N] {
    #[inline]
    fn ew_add(self, other: Self) -> Self {
        let mut result = [T::default(); N];
        for i in 0..N {
            result[i] = self[i] + other[i];
        }
        result
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
            learning_rate: 0.1,
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
                    + (1.0 - self.first_order_decay_factor) * *gradient;
                *second_moment = self.second_order_decay_factor * *second_moment
                    + (1.0 - self.second_order_decay_factor) * (*gradient).powi(2);
                let adaptative_learning_rate = self.learning_rate
                    * (1.0 - self.second_order_decay_factor.powi(self.time)).sqrt()
                    / (1.0 - self.first_order_decay_factor.powi(self.time));
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
            let (_value, gradients) = quadratic_function(&phis);
            adam.apply(&mut gradients.clone(), &mut phis);
        }

        assert!((phis[0] - 1.0).abs() < 1e-4);
        assert!((phis[1] + 2.0).abs() < 1e-4);
    }
}
