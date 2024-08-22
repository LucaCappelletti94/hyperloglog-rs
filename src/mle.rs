//! Struct marker MLE.

use crate::basicloglog::BasicLogLog;
use crate::prelude::*;
use core::cmp::Ordering;
use core::hash::Hash;
use core::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy, Hash, Default, Eq, PartialEq)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
/// A struct representing the Maximum Likelihood Estimation.
pub struct MLE<H, const ERROR: i32 = 2> {
    /// The underlying counter.
    counter: H,
}

impl<X, H: AsMut<X>, const ERROR: i32> AsMut<X> for MLE<H, ERROR> {
    #[inline]
    fn as_mut(&mut self) -> &mut X {
        self.counter.as_mut()
    }
}

impl<X, H: AsRef<X>, const ERROR: i32> AsRef<X> for MLE<H, ERROR> {
    #[inline]
    fn as_ref(&self) -> &X {
        self.counter.as_ref()
    }
}

impl<H: BitOr<Output = H>, const ERROR: i32> BitOr for MLE<H, ERROR> {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            counter: self.counter | rhs.counter,
        }
    }
}

impl<H: BitOrAssign, const ERROR: i32> BitOrAssign for MLE<H, ERROR> {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.counter |= rhs.counter;
    }
}

impl<
        H: HyperLogLog + AsMut<BasicLogLog<H::Precision, H::Bits, H::Registers, H::Hasher>>,
        const ERROR: i32,
    > HyperLogLog for MLE<H, ERROR>
{
    type Registers = H::Registers;
    type Precision = H::Precision;
    type Bits = H::Bits;
    type Hasher = H::Hasher;

    #[inline]
    fn registers(&self) -> &Self::Registers {
        self.counter.registers()
    }

    #[inline]
    fn get_number_of_zero_registers(&self) -> usize {
        self.counter.get_number_of_zero_registers()
    }

    #[inline]
    fn get_register(&self, index: usize) -> u8 {
        self.counter.get_register(index)
    }

    #[inline]
    fn harmonic_sum(&self) -> f64 {
        self.counter.harmonic_sum()
    }

    #[inline]
    fn insert_register_value_and_index(
        &mut self,
        new_register_value: u8,
        index: usize,
    ) -> bool {
        self.counter
            .insert_register_value_and_index(new_register_value, index)
    }

    #[inline]
    fn from_registers(registers: H::Registers) -> Self {
        Self {
            counter: HyperLogLog::from_registers(registers),
        }
    }
}

impl<H, const ERROR: i32> From<H> for MLE<H, ERROR> {
    #[inline]
    fn from(counter: H) -> Self {
        Self { counter }
    }
}

#[cfg(feature = "std")]
impl<const ERROR: i32, H: Named> Named for MLE<H, ERROR>
where
    Self: Default,
{
    #[inline]
    fn name(&self) -> String {
        format!("MLE{}{}", ERROR, self.counter.name())
    }
}

/// Compute the union cardinality using the Maximum Likelihood Estimation.
fn mle_union_cardinality<
    P: Precision,
    B: Bits,
    I: ExactSizeIterator<Item = [u8; 2]>,
    const ERROR: i32,
>(
    registers: I,
    left_cardinality: f64,
    right_cardinality: f64,
    estimate: fn(f64, usize) -> f64,
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
        union_zeros += usize::from(larger_register.is_zero());
    }

    // We get the best estimates from HyperLogLog++
    let union_cardinality = estimate(union_harmonic_sum, union_zeros);

    // If the sum of the number of registers equal to zero, i.e.
    // the first value in the multiplicities vectors, is equal
    // to the number of registers, it means that the intersection
    // is empty.
    if union_zeros == 1 << B::NUMBER_OF_BITS {
        return f64::ZERO;
    }

    let intersection: f64 =
        (left_cardinality + right_cardinality - union_cardinality).max(f64::EPSILON);

    let left_difference: f64 = (union_cardinality - right_cardinality).max(f64::EPSILON);

    let right_difference: f64 = (union_cardinality - left_cardinality).max(f64::EPSILON);

    let relative_error_limit = 10.0_f64.powi(-ERROR) / f64::integer_exp2(P::EXPONENT).sqrt();

    // we introdce the following expressions to simplify the computation
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
                (f64::ONE + exp_m1[0]).max(f64::EPSILON),
                (f64::ONE + exp_m1[1]).max(f64::EPSILON),
                (f64::ONE + exp_m1[2]).max(f64::EPSILON),
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

        let denominator = f64::ONE / (z_q[2] + y_q[2] * z_q[0] * z_q[1]);

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
            let reciprocal_zj_plus_yjoint_zlr = f64::ONE / zj_plus_yjoint_zlr;

            let left_reciprocal = left_smaller_k
                * (y_register[2] * y_register[0] / (z_register[2] + y_register[2] * z_register[0])
                    - f64::ONE);
            let right_reciprocal = right_smaller_k
                * (y_register[2] * y_register[1] / zj_plus_yjoint_zright - f64::ONE);

            let delta = [
                left_reciprocal
                    + joint_k * (yjoint_left_zright * reciprocal_zj_plus_yjoint_zlr - f64::ONE)
                    + left_larger_k * (y_register[0] / z_register[0] - f64::ONE),
                right_reciprocal
                    + joint_k * (yjoint_right_zleft * reciprocal_zj_plus_yjoint_zlr - f64::ONE)
                    + right_larger_k * (y_register[1] / z_register[1] - f64::ONE),
                left_reciprocal
                    + right_reciprocal
                    + joint_k
                        * ((y_register[2] * y_register[0] + yjoint_right_zleft)
                            * reciprocal_zj_plus_yjoint_zlr
                            - f64::ONE),
            ];

            gradients = gradients.ew_add(x_register.ew_mul(delta));
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

impl<const ERROR: i32, H: Correction> Correction for MLE<H, ERROR> {
    fn correction(
        harmonic_sum: f64,
        number_of_zero_registers: usize,
    ) -> f64 {
        H::correction(harmonic_sum, number_of_zero_registers)
    }
}

impl<const ERROR: i32, H> Estimator<f64> for MLE<H, ERROR>
where
    H: Estimator<f64> + Correction + HyperLogLog,
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
    fn estimate_union_cardinality_with_cardinalities(
        &self,
        other: &Self,
        self_cardinality: f64,
        other_cardinality: f64,
    ) -> f64 {
        mle_union_cardinality::<
            <H as HyperLogLog>::Precision,
            <H as HyperLogLog>::Bits,
            <<H as HyperLogLog>::Registers as Registers<
                <H as HyperLogLog>::Precision,
                <H as HyperLogLog>::Bits,
            >>::IterZipped<'_>,
            ERROR,
        >(
            self.counter
                .registers()
                .iter_registers_zipped(other.counter.registers()),
            self_cardinality,
            other_cardinality,
            <H as Correction>::correction,
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
