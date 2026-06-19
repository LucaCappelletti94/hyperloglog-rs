//! The 2-set union joint MLE (Ertl's joint estimator), with the dynamically-sized Adam optimizer
//! and element-wise array helpers it uses.

use crate::prelude::*;
use crate::utils::{FloatOps, Zero};
use core::cmp::Ordering;
use core::ops::{Add, Mul, Sub};
#[allow(unused_imports)]
use num_traits::Float;

#[allow(clippy::too_many_lines)]
/// Computes the three disjoint regions of the 2-set joint MLE: `[left_difference, right_difference,
/// intersection]`, i.e. `[|A \ B|, |B \ A|, |A intersect B|]`. The union cardinality is their sum.
/// This is the analytic Ertl estimator over the register multiplicity arrays (each Adam iteration is
/// O(2^B), not O(number of registers)), cheap enough that the `M = N = 1` joint sketch dispatches
/// here directly.
///
/// # Arguments
/// * `registers` - Iterator over the `[left, right]` register pairs of the two counters.
/// * `left_cardinality` / `right_cardinality` - Cardinality estimates of the two counters.
/// * `estimate` - Maps a union harmonic sum (and zero-register count) to a union cardinality.
/// * `error_exponent` - The optimizer stops once every gradient is below `10^-error_exponent`
///   scaled by the precision.
pub(crate) fn mle_union_regions<P: Precision, B: Bits, I: ExactSizeIterator<Item = [u8; 2]>>(
    registers: I,
    left_cardinality: f64,
    right_cardinality: f64,
    estimate: impl Fn(f64, u32) -> f64,
    error_exponent: i32,
) -> [f64; 3] {
    debug_assert!(1_usize << B::NUMBER_OF_BITS <= crate::mle::REGISTER_MULTIPLICITIES_CAPACITY);
    // Stack histograms sized to the widest `Bits`; only the first `1 << B::NUMBER_OF_BITS` entries
    // are ever touched, so this is bit-identical to the former `vec![f64::ZERO; 1 << B]`.
    const CAP: usize = crate::mle::REGISTER_MULTIPLICITIES_CAPACITY;
    let mut left_multiplicities_larger = [f64::ZERO; CAP];
    let mut left_multiplicities_smaller = [f64::ZERO; CAP];
    let mut right_multiplicities_larger = [f64::ZERO; CAP];
    let mut right_multiplicities_smaller = [f64::ZERO; CAP];
    let mut joint_multiplicities = [f64::ZERO; CAP];
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

    // If every register of the union is zero, the union is empty. The number of registers is
    // `2^P::EXPONENT` (NOT `2^B::NUMBER_OF_BITS`, which is the number of distinct register *values*
    // and merely sizes the multiplicity arrays): confusing the two made this guard fire spuriously
    // whenever a non-empty union happened to have exactly `2^B` zero registers (e.g. around a union
    // of ~18000 at Precision12/Bits6, where the zero count hovers near 64), returning a union of zero.
    if union_zeros == 1_u32 << P::EXPONENT {
        return [f64::ZERO; 3];
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

    let mut optimizer: ArrayAdam<3> = ArrayAdam::default();

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

    // phis are [ln(left_difference), ln(right_difference), ln(intersection)].
    [phis[0].exp(), phis[1].exp(), phis[2].exp()]
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

/// Fixed-size Adam optimizer used by the 2-set union MLE (`mle_union_cardinality`).
pub(crate) struct ArrayAdam<const N: usize> {
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

impl<const N: usize> Default for ArrayAdam<N> {
    fn default() -> Self {
        ArrayAdam {
            first_moments: [0.0; N],
            second_moments: [0.0; N],
            time: 0,
            learning_rate: 0.1,
            first_order_decay_factor: 0.9,
            second_order_decay_factor: 0.999,
        }
    }
}

impl<const N: usize> ArrayAdam<N> {
    /// Apply the Adam optimizer to the gradients and weights.
    pub(crate) fn apply(&mut self, gradients: &mut [f64; N], phis: &mut [f64; N]) {
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    type Hll = HyperLogLog<Precision12, Bits6>;

    fn splitmix64(mut x: u64) -> u64 {
        x = x.wrapping_add(0x9E3779B97F4A7C15);
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
        x ^ (x >> 31)
    }

    /// Regression for the spurious "empty union" early return. At a union cardinality where the
    /// number of zero registers hovers near `2^B::NUMBER_OF_BITS` (64 at Bits6), the guard used to
    /// compare the union zero count against that wrong constant and collapse the union to zero. With
    /// two sets of ~12111 elements at 50 percent overlap (true union ~18167, zero count near 49), the
    /// register-mode union MLE must stay close to the truth on every seed, never returning zero.
    #[test]
    fn union_mle_does_not_collapse_to_zero_near_64_zero_registers() {
        let card = 12111u64;
        for seed in 1..=256u64 {
            let base = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let shared = card / 2;
            let only_b = card - shared;
            let va: Vec<u64> = (0..card)
                .map(|i| splitmix64(base.wrapping_add(i)))
                .collect();
            let vb: Vec<u64> = (0..shared)
                .map(|i| splitmix64(base.wrapping_add(i)))
                .chain((0..only_b).map(|i| splitmix64(base.wrapping_add(card + i))))
                .collect();

            let mut a = Hll::default();
            for &v in &va {
                a.insert(&v);
            }
            let mut b = Hll::default();
            for &v in &vb {
                b.insert(&v);
            }
            let a = a.into_hll();
            let b = b.into_hll();

            let truth = (card + only_b) as f64;
            let union = a.mle().estimate_union_cardinality(&b.mle());
            assert!(
                (union - truth).abs() / truth < 0.1,
                "seed {seed}: union MLE {union} vs truth {truth} (relative error {:.1}%)",
                100.0 * (union - truth).abs() / truth
            );
        }
    }
}

#[cfg(test)]
mod parity {
    //! Parity harness for the register-multiplicity histogram refactor: a frozen, heap-`Vec`-backed
    //! copy of `mle_union_regions` as it was before the five histograms moved to stack arrays. The
    //! production fn must produce BIT-IDENTICAL output to this reference for every register-pair
    //! configuration (the refactor only changes where the histograms live, never the arithmetic).
    use super::*;

    /// Frozen heap-`Vec` reference (verbatim copy of the pre-refactor `mle_union_regions`).
    #[allow(clippy::too_many_lines)]
    fn mle_union_regions_vec_reference<
        P: Precision,
        B: Bits,
        I: ExactSizeIterator<Item = [u8; 2]>,
    >(
        registers: I,
        left_cardinality: f64,
        right_cardinality: f64,
        estimate: impl Fn(f64, u32) -> f64,
        error_exponent: i32,
    ) -> [f64; 3] {
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

        let union_cardinality = estimate(union_harmonic_sum, union_zeros);

        if union_zeros == 1_u32 << P::EXPONENT {
            return [f64::ZERO; 3];
        }

        let intersection: f64 =
            (left_cardinality + right_cardinality - union_cardinality).max(f64::EPSILON);

        let left_difference: f64 = (union_cardinality - right_cardinality).max(f64::EPSILON);

        let right_difference: f64 = (union_cardinality - left_cardinality).max(f64::EPSILON);

        let relative_error_limit =
            10.0_f64.powi(-error_exponent) / f64::integer_exp2(P::EXPONENT).sqrt();

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

        let q_plus_one: u8 = (1 << B::NUMBER_OF_BITS) - 1;
        let q: u8 = q_plus_one - 1;

        let mut phis = [
            left_difference.ln(),
            right_difference.ln(),
            intersection.ln(),
        ];
        let mut gradients: [f64; 3] = [f64::ZERO, f64::ZERO, f64::ZERO];

        let mut optimizer: ArrayAdam<3> = ArrayAdam::default();

        let zeros_0: [f64; 3] = [
            left_multiplicities_smaller[0]
                + left_multiplicities_larger[0]
                + joint_multiplicities[0],
            right_multiplicities_smaller[0]
                + right_multiplicities_larger[0]
                + joint_multiplicities[0],
            right_multiplicities_smaller[0]
                + left_multiplicities_smaller[0]
                + joint_multiplicities[0],
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
                let zj_plus_yjoint_zlr =
                    z_register[2] + y_register[2] * z_register[0] * z_register[1];
                let reciprocal_zj_plus_yjoint_zlr = 1.0 / zj_plus_yjoint_zlr;

                let left_reciprocal = left_smaller_k
                    * (y_register[2] * y_register[0]
                        / (z_register[2] + y_register[2] * z_register[0])
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

            optimizer.apply(&mut gradients, &mut phis);

            if gradients
                .iter()
                .all(|gradient| gradient.abs() <= relative_error_limit)
            {
                break;
            }
        }

        [phis[0].exp(), phis[1].exp(), phis[2].exp()]
    }

    /// Builds `2^P` deterministic register pairs in `0..2^B`, with deterministic operand
    /// cardinalities and a deterministic union estimator, then checks the production fn against the
    /// reference bit-for-bit on all three returned regions.
    fn check<P: Precision, B: Bits>(seed: u64) -> bool {
        let number_of_registers = 1_usize << P::EXPONENT;
        let register_modulo = 1_u64 << B::NUMBER_OF_BITS;
        let mut state = seed;
        let mut next = || {
            state = state
                .wrapping_mul(0x5851_F42D_4C95_7F2D)
                .wrapping_add(0x1405_7B7E_F767_814F);
            state >> 33
        };
        let mut pairs = alloc::vec::Vec::with_capacity(number_of_registers);
        for _ in 0..number_of_registers {
            let left = next().wrapping_rem(register_modulo) as u8;
            let right = next().wrapping_rem(register_modulo) as u8;
            pairs.push([left, right]);
        }
        // Deterministic, positive operand cardinalities and a deterministic union estimator (both
        // fns receive identical inputs, so the exact formula is immaterial to parity).
        let left_cardinality = 1.0 + (next() % 100_000) as f64;
        let right_cardinality = 1.0 + (next() % 100_000) as f64;
        let estimate = |harmonic_sum: f64, _zeros: u32| {
            P::ALPHA * f64::integer_exp2(P::EXPONENT + P::EXPONENT) / harmonic_sum
        };

        let produced = mle_union_regions::<P, B, _>(
            pairs.iter().copied(),
            left_cardinality,
            right_cardinality,
            &estimate,
            2,
        );
        let reference = mle_union_regions_vec_reference::<P, B, _>(
            pairs.iter().copied(),
            left_cardinality,
            right_cardinality,
            &estimate,
            2,
        );
        (0..3).all(|i| produced[i].to_bits() == reference[i].to_bits())
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig { cases: 192, ..Default::default() })]

        #[test]
        fn union_mle_array_matches_vec_reference_p8_b6(seed in proptest::prelude::any::<u64>()) {
            proptest::prop_assert!(check::<Precision8, Bits6>(seed));
        }

        #[test]
        fn union_mle_array_matches_vec_reference_p8_b4(seed in proptest::prelude::any::<u64>()) {
            proptest::prop_assert!(check::<Precision8, Bits4>(seed));
        }

        #[test]
        fn union_mle_array_matches_vec_reference_p6_b5(seed in proptest::prelude::any::<u64>()) {
            proptest::prop_assert!(check::<Precision6, Bits5>(seed));
        }
    }
}
