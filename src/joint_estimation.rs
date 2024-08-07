use core::hash::Hash;

use crate::{prelude::*, utils::FloatNumber};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MLE<const ERROR: i32, H> {
    inner: H,
}

impl<const ERROR: i32, P: Precision, B: Bits, R: Registers<P, B>> AsRef<MLE<ERROR, Self>>
    for HyperLogLog<P, B, R>
{
    fn as_ref(&self) -> &MLE<ERROR, Self> {
        unsafe { core::mem::transmute(self) }
    }
}

impl<const ERROR: i32, H> AsRef<H> for MLE<ERROR, H> {
    fn as_ref(&self) -> &H {
        &self.inner
    }
}

impl<const ERROR: i32, P: Precision, B: Bits, R: Registers<P, B>> From<MLE<ERROR, Self>>
    for HyperLogLog<P, B, R>
{
    fn from(mle: MLE<ERROR, Self>) -> Self {
        mle.inner
    }
}

impl<const ERROR: i32, H> From<H> for MLE<ERROR, H> {
    fn from(hll: H) -> Self {
        Self { inner: hll }
    }
}

impl<const ERROR: i32, H: FromIterator<A>, A: Hash> core::iter::FromIterator<A> for MLE<ERROR, H> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        Self {
            inner: H::from_iter(iter),
        }
    }
}

impl<const ERROR: i32, H> MLE<ERROR, H> {
    /// Changes the error of the MLE.
    pub fn change_error<const NEW_ERROR: i32>(self) -> MLE<NEW_ERROR, H> {
        MLE { inner: self.inner }
    }
}

impl<const ERROR: i32, P: Precision, B: Bits, H: HyperLogLogTrait<P, B>> HyperLogLogTrait<P, B>
    for MLE<ERROR, H>
{
    type IterRegisters<'a> =
        <H as HyperLogLogTrait<P, B>>::IterRegisters<'a> where Self: 'a;

    fn get_number_of_zero_registers(&self) -> P::NumberOfZeros {
        self.inner.get_number_of_zero_registers()
    }

    fn iter_registers(&self) -> Self::IterRegisters<'_> {
        self.inner.iter_registers()
    }

    fn harmonic_sum<F: FloatNumber>(&self) -> F
    where
        P: PrecisionConstants<F>,
    {
        self.inner.harmonic_sum()
    }

    fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    fn estimate_cardinality<F: FloatNumber>(&self) -> F
    where
        P: PrecisionConstants<F>,
    {
        if self.is_full() {
            return F::INFINITY;
        }

        let mut multiplicities: Vec<F> =
            vec![F::ZERO; maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS)];
        let q = maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS) as u32 - 2;

        let mut smallest_register_value: u32 = q;
        let mut largest_register_value: u32 = 0;

        self.iter_registers().for_each(|register| {
            if register > 0 {
                smallest_register_value = smallest_register_value.min(register);
            }
            largest_register_value = largest_register_value.max(register);
            multiplicities[register as usize] += F::ONE;
        });

        smallest_register_value = smallest_register_value.max(1);
        largest_register_value = largest_register_value.min(q).max(1);

        debug_assert!(smallest_register_value > 0);
        debug_assert!(q > 0);
        debug_assert!(
            largest_register_value > 0,
            concat!(
                "The largest register value should be greater than 0. ",
                "The multiplicities are: {:?}."
            ),
            multiplicities
        );
        debug_assert!(
            largest_register_value <= q,
            concat!(
                "The largest register value should be smaller than q. ",
                "The multiplicities are: {:?}."
            ),
            multiplicities
        );

        let c = multiplicities[multiplicities.len() - 1]
            + multiplicities[largest_register_value as usize];

        let mut g_prev: F = F::ZERO;
        let number_of_zero_registers = multiplicities[0];
        let reciprocal_saturated_registers = multiplicities[multiplicities.len() - 1]
            * F::inverse_register((multiplicities.len() - 1) as i32);

        let harmonic_sum_minus_zero_and_saturated_registers: F =
            self.harmonic_sum() - (number_of_zero_registers + reciprocal_saturated_registers);

        let a: F = harmonic_sum_minus_zero_and_saturated_registers + number_of_zero_registers;
        let b: F = harmonic_sum_minus_zero_and_saturated_registers
            + multiplicities[multiplicities.len() - 1] * F::inverse_register(q as i32);

        let number_of_non_zero_registers: F =
            F::from_usize(P::NUMBER_OF_REGISTERS) - number_of_zero_registers;

        let mut x = if b <= F::THREE / F::TWO * a {
            number_of_non_zero_registers / (F::HALF * b + a)
        } else {
            (number_of_non_zero_registers / b) * (b / a).ln_1p()
        };

        // We begin the secant method iterations.
        let mut delta_x = x;
        let relative_error_limit = F::TEN.powi(-ERROR) / P::NUMBER_OF_REGISTERS_FLOAT.sqrt();

        let forty_five_recip: F = F::ONE / F::from_usize(45);
        let four_seventy_two_point_five_recip: F = F::ONE / (F::from_usize(472) + F::HALF);

        let taylor =
            |x_first: F, h: F| -> F { (x_first + h * (F::ONE - h)) / (x_first + F::ONE - h) };

        while delta_x > x * relative_error_limit {
            // In the C++ implementation they call frexp.
            let k: u32 = 2 + x.log2().floor().to_usize() as u32;

            // We compute the terms for the Taylor series.
            let maximal: u32 = if largest_register_value > k {
                largest_register_value
            } else {
                k
            };
            let mut x_first = x * F::inverse_register((maximal + 1) as i32);
            let x_second = x_first * x_first;
            let x_forth = x_second * x_second;
            let mut taylor_series_approximation = x_first - x_second / F::THREE
                + x_forth * (forty_five_recip - x_second * four_seventy_two_point_five_recip);

            // If k - 1 is smaller than the maximal register value
            for _ in largest_register_value..k {
                taylor_series_approximation = taylor(x_first, taylor_series_approximation);

                // And we double the x first:
                x_first *= F::TWO;
            }

            let mut g: F = c * taylor_series_approximation;

            for register_value in (smallest_register_value..largest_register_value).rev() {
                taylor_series_approximation = taylor(x_first, taylor_series_approximation);
                g += multiplicities[register_value as usize] * taylor_series_approximation;
                x_first *= F::TWO;
            }

            g += x * a;

            if g > g_prev && number_of_non_zero_registers >= g {
                delta_x *= (number_of_non_zero_registers - g) / (g - g_prev);
            } else {
                delta_x = F::ZERO;
            };

            x += delta_x;
            g_prev = g;
        }

        P::NUMBER_OF_REGISTERS_FLOAT * x
    }

    fn estimate_union_and_sets_cardinality<F: FloatNumber, Rhs: HyperLogLogTrait<P, B>>(
        &self,
        other: &Rhs,
    ) -> EstimatedUnionCardinalities<F>
    where
        P: PrecisionConstants<F>,
    {
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

        let (union_harmonic_sum, union_zeros) = self
            .iter_registers()
            .zip(other.iter_registers())
            .map(|(left, right)| {
                match left.cmp(&right) {
                    core::cmp::Ordering::Less => {
                        left_multiplicities_smaller[left as usize] += F::ONE;
                        right_multiplicities_larger[right as usize] += F::ONE;
                    }
                    core::cmp::Ordering::Greater => {
                        left_multiplicities_larger[left as usize] += F::ONE;
                        right_multiplicities_smaller[right as usize] += F::ONE;
                    }
                    core::cmp::Ordering::Equal => {
                        // If left register is equal to right register
                        joint_multiplicities[left as usize] += F::ONE;
                    }
                }

                let larger_register = if left > right { left } else { right };

                (
                    F::inverse_register(larger_register as i32),
                    P::NumberOfZeros::from_bool(larger_register.is_zero()),
                )
            })
            .fold(
                (F::ZERO, P::NumberOfZeros::ZERO),
                |(union_harmonic_sum, union_zeros), (union_harmonic_sum_tmp, union_zeros_tmp)| {
                    (
                        union_harmonic_sum + union_harmonic_sum_tmp,
                        union_zeros + union_zeros_tmp,
                    )
                },
            );

        // We get the best estimates from HyperLogLog++
        let mut union_cardinality_estimate =
            Self::adjust_estimate_with_zeros(union_harmonic_sum, union_zeros);

        let left_cardinality_estimate = self.inner.estimate_cardinality();

        let right_cardinality_estimate = Rhs::adjust_estimate_with_zeros(
            other.harmonic_sum(),
            other.get_number_of_zero_registers(),
        );

        if left_cardinality_estimate + right_cardinality_estimate < union_cardinality_estimate {
            union_cardinality_estimate = left_cardinality_estimate + right_cardinality_estimate;
        }

        // If the sum of the number of registers equal to zero, i.e.
        // the first value in the multiplicities vectors, is equal
        // to the number of registers, it means that the intersection
        // is empty.

        let number_of_zeros: usize = (left_multiplicities_smaller[0]
            + left_multiplicities_smaller[0]
            + right_multiplicities_smaller[0])
            .to_usize();
        if number_of_zeros == P::NUMBER_OF_REGISTERS {
            return EstimatedUnionCardinalities::from((
                left_cardinality_estimate,
                right_cardinality_estimate,
                F::ZERO,
            ));
        }

        let mut intersection: F =
            left_cardinality_estimate + right_cardinality_estimate - union_cardinality_estimate;
        let mut left_difference: F = union_cardinality_estimate - right_cardinality_estimate;
        let mut right_difference: F = union_cardinality_estimate - left_cardinality_estimate;

        if intersection < F::ONE {
            intersection = F::ONE;
        }

        if left_difference < F::ONE {
            left_difference = F::ONE;
        }

        if right_difference < F::ONE {
            right_difference = F::ONE;
        }

        let relative_error_limit = F::TEN.powi(-ERROR) / P::NUMBER_OF_REGISTERS_FLOAT.sqrt();

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

        let mut optimizer: Adam<F, 3> = Adam::default();

        let left_number_of_zeros = left_multiplicities_smaller[0]
            + left_multiplicities_larger[0]
            + joint_multiplicities[0];
        let right_number_of_zeros = right_multiplicities_smaller[0]
            + right_multiplicities_larger[0]
            + joint_multiplicities[0];
        let intersection_number_of_zeros = right_multiplicities_smaller[0]
            + left_multiplicities_smaller[0]
            + joint_multiplicities[0];

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
            let x_right_0 = x(phis[1], two_to_zero);
            let x_joint_0 = x(phis[2], two_to_zero);
            let x_left_q = x(phis[0], two_to_minus_q);
            let (y_left_q, z_left_q) = yz(x_left_q);
            let x_right_q = x(phis[1], two_to_minus_q);
            let (y_right_q, z_right_q) = yz(x_right_q);
            let x_joint_q = x(phis[2], two_to_minus_q);
            let (y_joint_q, z_joint_q) = yz(x_joint_q);

            let denominator = F::ONE / (z_joint_q + y_joint_q * z_left_q * z_right_q);

            let xl_yl_q = x_left_q * y_left_q;
            let xr_yr_q = x_right_q * y_right_q;
            let xj_yj_q = x_joint_q * y_joint_q;
            let shared_factor =
                intersection_number_of_saturated_registers * y_joint_q * denominator;

            gradients[0] = xl_yl_q
                * (shared_factor * z_right_q + left_number_of_saturated_registers / z_left_q)
                - left_number_of_zeros * x_left_0;

            gradients[1] = xr_yr_q
                * (shared_factor * z_left_q + right_number_of_saturated_registers / z_right_q)
                - right_number_of_zeros * x_right_0;

            gradients[2] = intersection_number_of_saturated_registers
                * xj_yj_q
                * (y_left_q + z_left_q * y_right_q)
                * denominator
                - intersection_number_of_zeros * x_joint_0;

            (1..q_plus_one as i32).for_each(|register_value| {
                let two_to_minus_register: F =
                    F::inverse_register(P::EXPONENT as i32 + register_value as i32);

                let x_left = x(phis[0], two_to_minus_register);
                let x_right = x(phis[1], two_to_minus_register);
                let x_joint = x(phis[2], two_to_minus_register);
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
                let zj_plus_yj_zl = z_joint + yj_zl;
                let reciprocal_zj_plus_yj_zl = F::ONE / zj_plus_yj_zl;
                let zj_plus_yj_zr = z_joint + yj_zr;
                let reciprocal_zj_plus_yj_zr = F::ONE / zj_plus_yj_zr;
                let zj_plus_yj_zlr = z_joint + yj_zlr;
                let reciprocal_zj_plus_yj_zlr = F::ONE / zj_plus_yj_zlr;

                let left_reciprocal = left_smaller_k * (reciprocal_zj_plus_yj_zl * yjl - F::ONE);
                let right_reciprocal = right_smaller_k * (reciprocal_zj_plus_yj_zr * yjr - F::ONE);

                gradients[0] += x_left
                    * (left_reciprocal
                        + joint_k * (yjl_zr * reciprocal_zj_plus_yj_zlr - F::ONE)
                        + left_larger_k * (y_left / z_left - F::ONE));

                gradients[1] += x_right
                    * (right_reciprocal
                        + joint_k * (yjr_zl * reciprocal_zj_plus_yj_zlr - F::ONE)
                        + right_larger_k * (y_right / z_right - F::ONE));

                gradients[2] += x_joint
                    * (left_reciprocal
                        + right_reciprocal
                        + joint_k * ((yjl + yjr_zl) * reciprocal_zj_plus_yj_zlr - F::ONE));
            });

            // We execute the update of the Adam first and second moments.
            optimizer.apply(&mut gradients, &mut phis);

            // If any of the gradient update, in absolute value, is higher
            if gradients.iter().map(|gradient| gradient.abs()).sum::<F>() <= relative_error_limit {
                break;
            }
        }

        let left_difference = phis[0].exp();
        let right_difference = phis[1].exp();
        let intersection = phis[2].exp();

        let left_cardinality = left_difference + intersection;
        let right_cardinality = right_difference + intersection;
        let union = left_difference + right_difference + intersection;

        EstimatedUnionCardinalities::with_correction(left_cardinality, right_cardinality, union)
    }

    fn insert<T: Hash>(&mut self, value: T) -> bool {
        self.inner.insert(value)
    }

    fn get_register(&self, index: usize) -> u32 {
        self.inner.get_register(index)
    }
}

impl<
        const ERROR: i32,
        P: Precision + PrecisionConstants<F>,
        B: Bits,
        R: Registers<P, B>,
        F: FloatNumber,
    > SetLike<F> for MLE<ERROR, HyperLogLog<P, B, R>>
{
    fn get_estimated_union_cardinality(
        &self,
        _self_cardinality: F,
        other: &Self,
        _other_cardinality: F,
    ) -> EstimatedUnionCardinalities<F> {
        self.estimate_union_and_sets_cardinality(other)
    }

    fn get_cardinality(&self) -> F {
        self.estimate_cardinality()
    }
}

impl<const ERROR: i32, H: core::ops::BitOr<Output = H>> core::ops::BitOr for MLE<ERROR, H> {
    type Output = MLE<ERROR, H>;

    fn bitor(self, rhs: Self) -> Self::Output {
        MLE {
            inner: self.inner | rhs.inner,
        }
    }
}

impl<const ERROR: i32, H: core::ops::BitOrAssign> core::ops::BitOrAssign for MLE<ERROR, H> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.inner |= rhs.inner;
    }
}

impl<const ERROR: i32, H: core::ops::BitAnd<Output = H>> core::ops::BitAnd for MLE<ERROR, H> {
    type Output = MLE<ERROR, H>;

    fn bitand(self, rhs: Self) -> Self::Output {
        MLE {
            inner: self.inner & rhs.inner,
        }
    }
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

impl<F: FloatNumber, const N: usize> Adam<F, N> {
    #[inline(always)]
    pub fn apply(&mut self, gradients: &mut [F; N], phis: &mut [F; N]) {
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
