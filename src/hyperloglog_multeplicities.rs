use crate::array_default::{ArrayDefault, ArrayIter};
use crate::precisions::{Precision, WordType};
use crate::prelude::*;
use core::hash::Hash;

/// A HyperLogLog counter with multeplicities.
///
/// # Implementation details
/// This struct differs from the traditional HyperLogLog counter in that it stores the multeplicities
/// of the registers. This allows us to speed up significantly the computation of the cardinality of
/// the counter, as we do not need to compute the harmonic mean of the registers but we can instead
/// use the multiplities instead, reducing by a large amount the sums we need to compute.
///
/// For instance, for a counter with 2^14 registers, we need to compute the harmonic mean of 2^14
/// registers, i.e. 16384 registers. With the multeplicities, we only need to compute the sum of the
/// multeplicities, which is much smaller, and at most equal to 52 when you use 6 bits per register.
///
/// That being said, when memory is an extreme concern, you may want to use the traditional HyperLogLog
/// as this struct contains the multeplicities vector, which in the example case we considered above
/// would be adding u16 * 52 = 104 bytes to the size of the counter.
///
/// Additionally, note that while one may expect to obtain better accuracy by executing less sums,
/// we do not observe any statistically significant difference in the accuracy of the counter when
/// using the multeplicities instead of the registers in our tests.
///
/// Note that this struct DOES NOT provide any other faster operation other than the estimation of the
/// cardinality of the counter. All other operations, such as the union of two counters, are fast as
/// they are implemented using the traditional HyperLogLog counter.
///
pub struct HyperLogLogWithMulteplicities<PRECISION: Precision + WordType<BITS>, const BITS: usize> {
    pub(crate) words: PRECISION::Words,
    pub(crate) multeplicities: PRECISION::Registermulteplicities,
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize>
    From<HyperLogLogWithMulteplicities<PRECISION, BITS>> for HyperLogLog<PRECISION, BITS>
{
    fn from(hll: HyperLogLogWithMulteplicities<PRECISION, BITS>) -> Self {
        Self::from_words(hll.get_words())
    }
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize>
    HyperLogLogWithMulteplicities<PRECISION, BITS>
{
    #[inline(always)]
    /// Returns the estimated cardinality of the counter using the MLE approach.
    ///
    /// # References
    /// The paper describing the MLE approach is [New cardinality estimation algorithms for HyperLogLog sketches](http://oertl.github.io/hyperloglog-sketch-estimation-paper/paper/paper.pdf).
    ///
    /// # Differences with traditional method
    /// The traditional method for estimating the cardinality of a HyperLogLog counter is to use the harmonic mean of the registers,
    /// and then execute a correction step as described in the HLL++ paper. The MLE approach instead uses a maximum likelihood estimator
    /// to estimate the cardinality of the counter. The MLE approach is statistically equivalent to the traditional approach, but it is
    /// slower, with the advantage of not requiring the rather large parameters tables, so it works even for values for which we do not
    /// have the correction tables.
    ///
    /// It is this library's author opinion that in most cases you should use the HLL++ method, as it is faster and achieves a
    /// similar accuracy in our tests.
    ///
    /// # Implementation details
    /// This method makes
    ///
    pub fn estimate_cardinality_mle(&self) -> f32 {
        Self::estimate_cardinality_from_multeplicities_mle(&self.multeplicities)
    }

    pub fn estimate_cardinality_from_multeplicities_mle(
        multeplicities: &PRECISION::Registermulteplicities,
    ) -> f32 {
        // If the multeplicity associated to the last register
        // is equal to the number of registers, we return infinity.
        if multeplicities.last().unwrap().convert() == PRECISION::NUMBER_OF_REGISTERS {
            return f32::INFINITY;
        }

        let q = multeplicities.len() - 2;

        let smallest_register_value = multeplicities.first_non_zero_index().unwrap().get_max(1);
        let largest_register_value = multeplicities.last_non_zero_index().unwrap().get_min(q);

        debug_assert!(smallest_register_value > 0);
        debug_assert!(
            largest_register_value > 0,
            concat!(
                "The largest register value should be greater than 0. ",
                "The multiplicities are: {:?}."
            ),
            multeplicities
        );

        let mut raw_estimate = 0.0;

        for k in (smallest_register_value..=largest_register_value).rev() {
            raw_estimate = 0.5 * raw_estimate + multeplicities[k].convert() as f32;
        }

        let two_to_minus_smallest_register: i32 = (127 - smallest_register_value as i32) << 23;
        raw_estimate *= f32::from_le_bytes(two_to_minus_smallest_register.to_le_bytes());

        let c = multeplicities.last().unwrap().convert() as f32
            + multeplicities[largest_register_value].convert() as f32;

        let mut g_prev = 0.0;
        let a = raw_estimate + multeplicities[0].convert() as f32;

        let two_to_minus_q: i32 = (127 - q as i32) << 23;
        let b = raw_estimate
            + multeplicities.last().unwrap().convert() as f32
                * f32::from_le_bytes(two_to_minus_q.to_le_bytes());

        let number_of_non_zero_registers =
            (PRECISION::NUMBER_OF_REGISTERS as f32) - (multeplicities[0].convert() as f32);

        let mut x = if b <= 1.5 * a {
            number_of_non_zero_registers / (0.5 * b + a)
        } else {
            (number_of_non_zero_registers / b) * (b / a).ln_1p()
        };

        // We begin the secant method iterations.
        let mut delta_x = x;
        let relative_error_limit = 1e-2 / (PRECISION::NUMBER_OF_REGISTERS as f32).sqrt();
        while delta_x > x * relative_error_limit {
            // In the C++ implementation they call frexp.
            let kappa_minus_one: usize = x.log2().floor() as usize;

            // We compute the terms for the Taylor series.
            let maximal: usize = (largest_register_value + 1).max(kappa_minus_one + 2);
            let two_to_minus_maximal: i32 = (127 - maximal as i32) << 23;
            let mut x_first = x * f32::from_le_bytes(two_to_minus_maximal.to_le_bytes());
            let x_second = x_first * x_first;
            let x_forth = x_second * x_second;
            let mut taylor_series_approximation =
                x_first - x_second / 3.0 + x_forth * (1.0 / 45.0 - x_second / 472.5);

            // If kappa - 1 is smaller than the maximal register value
            for _k in (largest_register_value..=kappa_minus_one).rev() {
                let taylor_series_approximation_prime = 1.0 - taylor_series_approximation;
                taylor_series_approximation = (x_first
                    + taylor_series_approximation * taylor_series_approximation_prime)
                    / (x_first + taylor_series_approximation_prime);

                // And we double the x first:
                x_first *= 2.0;
            }

            let mut g = c * taylor_series_approximation;

            for k in (smallest_register_value..=largest_register_value.saturating_sub(1)).rev() {
                let taylor_series_approximation_prime = 1.0 - taylor_series_approximation;
                taylor_series_approximation = (x_first
                    + taylor_series_approximation * taylor_series_approximation_prime)
                    / (x_first + taylor_series_approximation_prime);
                g += multeplicities[k].convert() as f32 * taylor_series_approximation;
                x_first *= 2.0;
            }

            g += x * a;

            if g > g_prev && number_of_non_zero_registers >= g {
                delta_x *= (number_of_non_zero_registers - g) / (g - g_prev);
            } else {
                delta_x = 0.0;
            };

            x += delta_x;
            g_prev = g;
        }

        PRECISION::NUMBER_OF_REGISTERS as f32 * x
    }

    /// Returns the number of registers in the counter.
    ///
    /// # Implementation details
    /// This function is overriding the estimate_cardinality function of the HyperLogLogTrait trait
    /// as we can compute the cardinality of the counter using the multeplicities instead of the
    /// registers. This is much faster as we do not need to compute the harmonic mean of the registers.
    fn estimate_cardinality_from_multeplicities(
        multeplicities: &PRECISION::Registermulteplicities,
    ) -> f32 {
        if multeplicities[0] > PRECISION::NumberOfZeros::ZERO {
            let low_range_correction =
                PRECISION::SMALL_CORRECTIONS[multeplicities[0].convert() - 1];
            if low_range_correction <= Self::LINEAR_COUNT_THRESHOLD {
                return low_range_correction;
            }
        }

        let mut raw_estimate = 0.0;

        for (current_register, multeplicity) in multeplicities.iter_elements().enumerate() {
            let two_to_minus_register: i32 = (127 - current_register as i32) << 23;
            raw_estimate += (multeplicity.convert() as f32)
                * f32::from_le_bytes(two_to_minus_register.to_le_bytes());
        }

        Self::adjust_estimate(raw_estimate)
    }

    /// Returns estimated intersection cardinality object based on MLE joint cardinality estimation.
    ///
    /// # References
    /// The paper describing this method is [New cardinality estimation algorithms for HyperLogLog sketches](http://oertl.github.io/hyperloglog-sketch-estimation-paper/paper/paper.pdf).
    ///
    /// # Examples
    /// We start by checking that the estimates for the left cardinality, right cardinality and their intersection cardinality
    /// are correct. We begin with the trivial case of disjointed counters.
    ///
    /// ```rust
    /// use hyperloglog_rs::prelude::*;
    /// use std::collections::HashSet;
    ///
    /// let vec1 = vec![1, 2, 3, 4, 5, 5, 5, 6, 7, 8];
    /// let vec2 = vec![9, 10, 11, 12, 13, 13, 13, 14, 14, 15, 15, 16, 16];
    ///
    /// let set1 = vec1.iter().collect::<HashSet<_>>();
    /// let set2 = vec2.iter().collect::<HashSet<_>>();
    /// let left_difference_true = set1.difference(&set2).count() as f64;
    /// let right_difference_true = set2.difference(&set1).count() as f64;
    ///
    /// assert!(set1.is_disjoint(&set2));
    ///
    /// let mut hll1 = HyperLogLogWithMulteplicities::<Precision6, 6>::new();
    /// let mut hll2 = HyperLogLogWithMulteplicities::<Precision6, 6>::new();
    ///
    /// for &elem in &vec1 {
    ///     hll1.insert(elem);
    /// }
    ///
    /// for &elem in &vec2 {
    ///     hll2.insert(elem);
    /// }
    ///
    /// let euc = hll1.joint_cardinality_estimation::<4>(&hll2);
    ///
    /// let left_difference = euc.get_left_difference_cardinality();
    /// let right_difference = euc.get_right_difference_cardinality();
    /// let intersection_cardinality = euc.get_intersection_cardinality();
    ///
    /// assert!(
    ///     left_difference < left_difference_true * 1.2,
    ///     concat!(
    ///         "Mistaken left difference. ",
    ///         "Obtained: {}, Expected not more than: {}. ",
    ///     ),
    ///     left_difference, left_difference_true * 1.2,
    /// );
    ///
    /// assert!(
    ///     left_difference > left_difference_true * 0.8,
    ///     concat!(
    ///         "Mistaken left difference. ",
    ///         "Obtained: {}, Expected not less than: {}.",
    ///     ),
    ///     left_difference, left_difference_true * 0.8,
    /// );
    ///
    /// assert!(
    ///     right_difference < right_difference_true * 1.2,
    ///     concat!(
    ///         "Mistaken right difference cardinality. ",
    ///         "Obtained: {}, Expected not more than: {}.",
    ///     ),
    ///     right_difference, right_difference_true * 1.2,
    /// );
    ///
    /// assert!(
    ///     right_difference > right_difference_true * 0.8,
    ///     concat!(
    ///         "Mistaken right difference cardinality. ",
    ///         "Obtained: {}, Expected not less than: {}.",
    ///     ),
    ///     right_difference, right_difference_true * 0.8,
    /// );
    ///
    /// assert!(
    ///     intersection_cardinality < 1.0,
    ///     concat!(
    ///         "We expected the intersection cardinality to be around 0. ",
    ///         "Obtained: {}, Expected not more than: {}.",
    ///     ),
    ///     intersection_cardinality, 1.0,
    /// );
    ///
    /// ```
    ///
    /// Now we test with an actual couple of sets that have a non-empty intersection.
    ///
    /// ```rust
    /// use hyperloglog_rs::prelude::*;
    /// use std::collections::HashSet;
    ///
    /// let vec1 = vec![1, 2, 3, 4, 5, 5, 5, 6, 7, 8];
    /// let vec2 = vec![9, 10, 11, 12, 13, 13, 13, 14, 14, 15, 15, 16, 16, 1, 2, 3, 4, 5, 6];
    ///
    /// let set1 = vec1.iter().collect::<HashSet<_>>();
    /// let set2 = vec2.iter().collect::<HashSet<_>>();
    /// let left_difference_true = set1.difference(&set2).count() as f64;
    /// let right_difference_true = set2.difference(&set1).count() as f64;
    ///
    /// assert!(!set1.is_disjoint(&set2));
    ///
    /// let intersection_cardinality = set1.intersection(&set2).count();
    ///
    /// let mut hll1 = HyperLogLogWithMulteplicities::<Precision6, 6>::new();
    /// let mut hll2 = HyperLogLogWithMulteplicities::<Precision6, 6>::new();
    ///
    /// for &elem in &vec1 {
    ///    hll1.insert(elem);
    /// }
    ///
    /// for &elem in &vec2 {
    ///    hll2.insert(elem);
    /// }
    ///
    /// let euc = hll1.joint_cardinality_estimation::<4>(&hll2);
    ///
    /// let left_difference = euc.get_left_difference_cardinality();
    /// let right_difference = euc.get_right_difference_cardinality();
    /// let intersection_cardinality = euc.get_intersection_cardinality();
    ///
    /// assert!(
    ///     left_difference < left_difference_true * 1.2,
    ///     concat!(
    ///         "Mistaken left difference. ",
    ///         "Obtained: {}, Expected not more than: {}. ",
    ///     ),
    ///     left_difference, left_difference_true * 1.2,
    /// );
    ///
    /// assert!(
    ///     left_difference > left_difference_true * 0.8,
    ///     concat!(
    ///         "Mistaken left difference. ",
    ///         "Obtained: {}, Expected not less than: {}.",
    ///     ),
    ///     left_difference, left_difference_true * 0.8,
    /// );
    ///
    /// assert!(
    ///     right_difference < right_difference_true * 1.2,
    ///     concat!(
    ///         "Mistaken right difference cardinality. ",
    ///         "Obtained: {}, Expected not more than: {}.",
    ///     ),
    ///     right_difference, right_difference_true * 1.2,
    /// );
    ///
    /// assert!(
    ///     right_difference > right_difference_true * 0.8,
    ///     concat!(
    ///         "Mistaken right difference cardinality. ",
    ///         "Obtained: {}, Expected not less than: {}.",
    ///     ),
    ///     right_difference, right_difference_true * 0.8,
    /// );
    ///
    /// assert!(
    ///     intersection_cardinality < intersection_cardinality as f64 * 1.2,
    ///     concat!(
    ///         "We expected the intersection cardinality to be around the actual cardinality of the set. ",
    ///         "Obtained: {}, Expected not more than: {}.",
    ///     ),
    ///     intersection_cardinality, intersection_cardinality as f64 * 1.2,
    /// );
    ///
    /// assert!(
    ///     intersection_cardinality > intersection_cardinality as f64 * 0.8,
    ///     concat!(
    ///         "We expected the intersection cardinality to be around the actual cardinality of the set. ",
    ///         "Obtained: {}, Expected not less than: {}.",
    ///     ),
    ///     intersection_cardinality, intersection_cardinality as f64 * 0.8,
    /// );
    ///
    /// ```
    ///
    pub fn joint_cardinality_estimation<const ERROR: i32>(
        &self,
        other: &Self,
    ) -> EstimatedUnionCardinalities<f64> {
        let mut left_multeplicities_larger = PRECISION::Registermulteplicities::default_array();
        let mut left_multeplicities_smaller = PRECISION::Registermulteplicities::default_array();
        let mut right_multeplicities_larger = PRECISION::Registermulteplicities::default_array();
        let mut right_multeplicities_smaller = PRECISION::Registermulteplicities::default_array();
        let mut joint_multeplicities = PRECISION::Registermulteplicities::default_array();

        // First, we populate the vectors of multiplities
        self.get_registers()
            .into_iter_elements()
            .zip(other.get_registers().into_iter_elements())
            .for_each(|(left_register, right_register)| {
                match left_register.cmp(&right_register) {
                    std::cmp::Ordering::Less => {
                        left_multeplicities_smaller[left_register as usize] +=
                            PRECISION::NumberOfZeros::ONE;
                        right_multeplicities_larger[right_register as usize] +=
                            PRECISION::NumberOfZeros::ONE;
                    }
                    std::cmp::Ordering::Greater => {
                        left_multeplicities_larger[left_register as usize] +=
                            PRECISION::NumberOfZeros::ONE;
                        right_multeplicities_smaller[right_register as usize] +=
                            PRECISION::NumberOfZeros::ONE;
                    }
                    std::cmp::Ordering::Equal => {
                        // If left register is equal to right register
                        joint_multeplicities[left_register as usize] +=
                            PRECISION::NumberOfZeros::ONE;
                    }
                }
            });

        // We compute the cardinality for the left and right HLL
        // using the MLE version of the cardinality estimation.
        let left_cardinality = self.estimate_cardinality_mle() as f64;
        let right_cardinality = other.estimate_cardinality_mle() as f64;

        // If the sum of the number of registers equal to zero, i.e.
        // the first value in the multeplicities vectors, is equal
        // to the number of registers, it means that the intersection
        // is empty.
        if left_multeplicities_smaller[0].convert()
            + joint_multeplicities[0].convert()
            + right_multeplicities_smaller[0].convert()
            == PRECISION::NUMBER_OF_REGISTERS
        {
            return EstimatedUnionCardinalities::from((left_cardinality, right_cardinality, 0.0));
        }

        // Otherwise, we compute the multeplicities vector obtained
        // by summing the left larger multeplicities, the right
        // larger multeplicities and the joint multeplicities.
        let mut multeplicities = PRECISION::Registermulteplicities::default_array();

        multeplicities
            .iter_elements_mut()
            .zip(
                left_multeplicities_larger
                    .into_iter_elements()
                    .zip(right_multeplicities_larger.into_iter_elements())
                    .zip(joint_multeplicities.into_iter_elements()),
            )
            .for_each(|(multiplicity, ((left_larger, right_larger), joint))| {
                *multiplicity = left_larger + right_larger + joint;
            });

        // We compute the cardinality associated to the multeplicities vector that we have just defined.
        let union_cardinality =
            Self::estimate_cardinality_from_multeplicities_mle(&multeplicities) as f64;

        let symmetrical_difference = left_cardinality + right_cardinality - union_cardinality;
        let left_difference = union_cardinality - right_cardinality;
        let right_difference = union_cardinality - left_cardinality;

        let relative_error_limit =
            10.0_f64.powi(-ERROR) / (PRECISION::NUMBER_OF_REGISTERS as f64).sqrt();

        // we introdce the following expressions to simplify the computation
        // of the gradient.
        let x = |phi: f64, k: usize| -> f64 {
            phi.exp() / (PRECISION::NUMBER_OF_REGISTERS as f64 * (k as f64).exp2())
        };
        let log2 = 2.0_f64.ln();
        let y = |phi: f64, k: usize| -> f64 {
            let x = x(phi, k);
            if x < log2 {
                1.0 + (-x).exp_m1()
            } else {
                (-x).exp()
            }
            .min(1.0)
            .max(0.0)
        };
        let z = |phi: f64, k: usize| -> f64 {
            let x = x(phi, k);
            if x < log2 {
                -(-x).exp_m1()
            } else {
                1.0 - (-x).exp()
            }
            .min(1.0)
            .max(0.0)
        };

        // We precompute q and q+1 for reference.
        let q_plus_one = self.multeplicities.len() - 1;
        let q = q_plus_one - 1;

        let hlls_gradient = |phi_joint: f64,
                             first_phi: f64,
                             second_phi: f64,
                             smaller: &PRECISION::Registermulteplicities,
                             larger: &PRECISION::Registermulteplicities|
         -> f64 {
            (1..=q)
                .map(|k| {
                    (smaller[k].convert() as f64)
                        * (y(phi_joint, k) * x(first_phi, k) * y(first_phi, k))
                        / (z(phi_joint, k) + y(phi_joint, k) * z(first_phi, k))
                        + (joint_multeplicities[k].convert() as f64)
                            * (y(phi_joint, k)
                                * x(first_phi, k)
                                * y(first_phi, k)
                                * z(second_phi, k))
                            / (z(phi_joint, k)
                                + y(phi_joint, k) * z(first_phi, k) * z(second_phi, k))
                        + (larger[k].convert() as f64) * x(first_phi, k) * y(first_phi, k)
                            / z(first_phi, k)
                })
                .sum::<f64>()
                - (0..=q)
                    .map(|k| {
                        (smaller[k] + joint_multeplicities[k] + larger[k]).convert() as f64
                            * x(first_phi, k)
                    })
                    .sum::<f64>()
                + (joint_multeplicities[q_plus_one].convert() as f64)
                    * (y(phi_joint, q) * x(first_phi, q) * y(first_phi, q) * z(second_phi, q))
                    / (z(phi_joint, q) + y(phi_joint, q) * z(first_phi, q) * z(second_phi, q))
                + (larger[q_plus_one].convert() as f64) * (x(first_phi, q) * y(first_phi, q))
                    / z(first_phi, q)
        };

        let mut iteration = 0;

        // We initialize the vectors for the Adam optimizer.
        let mut first_moment = [0.0; 3];
        let mut second_moment = [0.0; 3];
        let mut phis_old = [0.0; 3];
        let mut phis = [
            left_difference.max(1.0).ln(),
            right_difference.max(1.0).ln(),
            symmetrical_difference.max(1.0).ln(),
        ];
        let first_order_decay_factor = 0.9;
        let second_order_decay_factor = 0.999;
        let learning_rate = 0.001;
        let convergence_patience = 10;
        let mut patience = 0;

        loop {
            let left_phi_gradient: f64 = hlls_gradient(
                phis[2],
                phis[0],
                phis[1],
                &left_multeplicities_smaller,
                &left_multeplicities_larger,
            );

            let right_phi_gradient: f64 = hlls_gradient(
                phis[2],
                phis[1],
                phis[0],
                &right_multeplicities_smaller,
                &right_multeplicities_larger,
            );

            let joint_phi_gradient: f64 = {
                (1..=q)
                    .map(|k| {
                        (left_multeplicities_smaller[k].convert() as f64)
                            * (x(phis[2], k) * y(phis[2], k) * y(phis[0], k))
                            / (z(phis[2], k) + y(phis[2], k) * z(phis[0], k))
                            + (joint_multeplicities[k].convert() as f64)
                                * (y(phis[2], k)
                                    * x(phis[2], k)
                                    * (y(phis[0], k) + z(phis[0], k) * y(phis[1], k)))
                                / (z(phis[2], k) + y(phis[2], k) * z(phis[0], k) * z(phis[1], k))
                            + (right_multeplicities_smaller[k].convert() as f64)
                                * (x(phis[2], k) * y(phis[2], k) * y(phis[1], k)
                                    / (z(phis[2], k) + y(phis[2], k) * z(phis[1], k)))
                    })
                    .sum::<f64>()
                    - (0..=q)
                        .map(|k| {
                            (left_multeplicities_smaller[k]
                                + joint_multeplicities[k]
                                + right_multeplicities_smaller[k])
                                .convert() as f64
                                * x(phis[2], k)
                        })
                        .sum::<f64>()
                    + (joint_multeplicities[q_plus_one].convert() as f64)
                        * (x(phis[2], q)
                            * y(phis[2], q)
                            * (y(phis[0], q) + z(phis[0], q) * y(phis[1], q)))
                        / (z(phis[2], q) + y(phis[2], q) * z(phis[0], q) * z(phis[1], q))
            };

            let gradients = [left_phi_gradient, right_phi_gradient, joint_phi_gradient];

            // We execute the update of the Adam first and second moments.

            first_moment
                .iter_mut()
                .zip(second_moment.iter_mut())
                .zip(phis.iter_mut())
                .zip(gradients.iter())
                .for_each(|(((first_moment, second_moment), phi), gradient)| {
                    *first_moment = first_order_decay_factor * *first_moment
                        + (1.0 - first_order_decay_factor) * *gradient;
                    *second_moment = second_order_decay_factor * *second_moment
                        + (1.0 - second_order_decay_factor) * (*gradient).powi(2);
                    let adaptative_learning_rate = learning_rate
                        * (1.0 - second_order_decay_factor.powi(iteration + 1)).sqrt()
                        / (1.0 - first_order_decay_factor.powi(iteration + 1));
                    *phi += adaptative_learning_rate * (*first_moment)
                        / (*second_moment).sqrt().max(f64::EPSILON)
                });

            if (phis[0] - phis_old[0])
                .abs()
                .max((phis[1] - phis_old[1]).abs())
                .max((phis[2] - phis_old[2]).abs())
                <= relative_error_limit
            {
                patience += 1;
                if patience >= convergence_patience {
                    break;
                }
            } else {
                patience = 0;
            }

            iteration += 1;

            phis_old = phis;
        }

        let left_difference = phis[0].exp();
        let right_difference = phis[1].exp();
        let intersection = phis[2].exp();

        EstimatedUnionCardinalities::from((
            left_difference + intersection,
            right_difference + intersection,
            left_difference + right_difference + intersection,
        ))
    }
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize> From<HyperLogLog<PRECISION, BITS>>
    for HyperLogLogWithMulteplicities<PRECISION, BITS>
{
    fn from(hll: HyperLogLog<PRECISION, BITS>) -> Self {
        Self::from_words(hll.get_words())
    }
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize> HyperLogLogTrait<PRECISION, BITS>
    for HyperLogLogWithMulteplicities<PRECISION, BITS>
{
    fn new() -> Self {
        let mut multeplicities = PRECISION::Registermulteplicities::default_array();

        multeplicities[0] = PRECISION::NumberOfZeros::reverse(PRECISION::NUMBER_OF_REGISTERS);

        Self {
            words: PRECISION::Words::default_array(),
            multeplicities,
        }
    }

    /// Create a new HyperLogLog counter from an array of registers.
    ///
    /// # Arguments
    ///
    /// * `registers` - An array of u32 registers to use for the HyperLogLog counter.
    ///
    /// # Returns
    ///
    /// A new HyperLogLog counter initialized with the given registers.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let registers = [0_u32; 1 << 4];
    /// let hll = HyperLogLogWithMulteplicities::<Precision4, 6>::from_registers(&registers);
    /// assert_eq!(hll.len(), 1 << 4);
    /// ```
    fn from_registers(registers: &[u32]) -> Self {
        debug_assert!(
            registers.len() == PRECISION::NUMBER_OF_REGISTERS,
            "We expect {} registers, but got {}",
            PRECISION::NUMBER_OF_REGISTERS,
            registers.len()
        );
        let mut words = PRECISION::Words::default_array();
        let mut multeplicities = PRECISION::Registermulteplicities::default_array();
        words
            .iter_elements_mut()
            .zip(registers.chunks(Self::NUMBER_OF_REGISTERS_IN_WORD))
            .for_each(|(word, word_registers)| {
                for (i, register) in word_registers.iter().copied().enumerate() {
                    debug_assert!(
                        register <= Self::LOWER_REGISTER_MASK,
                        "Register value {} is too large for the given number of bits {}",
                        register,
                        BITS
                    );
                    multeplicities[register as usize] += PRECISION::NumberOfZeros::ONE;
                    *word |= register << (i * BITS);
                }
            });

        Self {
            words,
            multeplicities,
        }
    }

    /// Create a new HyperLogLog counter from an array of words.
    ///
    /// # Arguments
    /// * `words` - An array of u64 words to use for the HyperLogLog counter.
    ///
    /// # Returns
    /// A new HyperLogLog counter initialized with the given words.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let words = [0_u32; 4];
    /// let hll = HyperLogLogWithMulteplicities::<Precision4, 6>::from_words(&words);
    /// assert_eq!(hll.len(), 16);
    /// ```
    fn from_words(words: &PRECISION::Words) -> Self {
        let mut multeplicities = PRECISION::Registermulteplicities::default_array();

        words.iter_elements().for_each(|word| {
            (0..Self::NUMBER_OF_REGISTERS_IN_WORD).for_each(|i| {
                let register = (word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                multeplicities[register as usize] += PRECISION::NumberOfZeros::ONE;
            });
        });

        multeplicities[0] -=
            PRECISION::NumberOfZeros::reverse(Self::get_number_of_padding_registers());

        Self {
            words: *words,
            multeplicities,
        }
    }

    #[inline(always)]
    /// Returns the number of registers in the counter.
    ///
    /// # Implementation details
    /// This function is overriding the estimate_cardinality function of the HyperLogLogTrait trait
    /// as we can compute the cardinality of the counter using the multeplicities instead of the
    /// registers. This is much faster as we do not need to compute the harmonic mean of the registers.
    fn estimate_cardinality(&self) -> f32 {
        Self::estimate_cardinality_from_multeplicities(&self.multeplicities)
    }

    /// Returns a reference to the words vector.
    fn get_words(&self) -> &PRECISION::Words {
        &self.words
    }

    #[inline(always)]
    /// Returns the number of registers with zero values. This value is used for computing a small
    /// correction when estimating the cardinality of a small set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// // Create a new HyperLogLog counter with precision 14 and 5 bits per register.
    /// let mut hll = HyperLogLogWithMulteplicities::<Precision14, 5>::new();
    ///
    /// // Add some elements to the counter.
    /// hll.insert(&1);
    /// hll.insert(&2);
    /// hll.insert(&3);
    ///
    /// // Get the number of zero registers.
    /// let number_of_zero_registers = hll.get_number_of_zero_registers();
    ///
    /// assert_eq!(number_of_zero_registers, 16381);
    /// ```
    fn get_number_of_zero_registers(&self) -> usize {
        self.multeplicities[0].convert()
    }

    #[inline(always)]
    /// Adds an element to the HyperLogLog counter.
    ///
    /// # Arguments
    /// * `rhs` - The element to add.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll = HyperLogLogWithMulteplicities::<Precision10, 6>::new();
    ///
    /// hll.insert("Hello");
    /// hll.insert("World");
    ///
    /// assert!(hll.estimate_cardinality() >= 2.0);
    /// ```
    ///
    /// # Performance
    ///
    /// The performance of this function depends on the size of the HyperLogLog counter (`N`), the number
    /// of distinct elements in the input, and the hash function used to hash elements. For a given value of `N`,
    /// the function has an average time complexity of O(1) and a worst-case time complexity of O(log N).
    /// However, the actual time complexity may vary depending on the distribution of the hashed elements.
    ///
    /// # Errors
    ///
    /// This function does not return any errors.
    fn insert<T: Hash>(&mut self, rhs: T) {
        let (mut hash, index) = self.get_hash_and_index::<T>(&rhs);

        // We need to add ones to the hash to make sure that the
        // the number of zeros we obtain afterwards is never higher
        // than the maximal value that may be represented in a register
        // with BITS bits.
        if BITS < 6 {
            hash |= 1 << (64 - ((1 << BITS) - 1));
        } else {
            hash |= 1 << (PRECISION::EXPONENT - 1);
        }

        // Count leading zeros.
        let number_of_zeros: u32 = 1 + hash.leading_zeros();

        debug_assert!(
            number_of_zeros < (1 << BITS),
            concat!(
                "The number of leading zeros {} must be less than the number of bits {}. ",
                "You have obtained this values starting from the hash {:064b} and the precision {}."
            ),
            number_of_zeros,
            1 << BITS,
            hash,
            PRECISION::EXPONENT
        );

        // Calculate the position of the register in the internal buffer array.
        let word_position = index / Self::NUMBER_OF_REGISTERS_IN_WORD;
        let register_position_in_u32 = index - word_position * Self::NUMBER_OF_REGISTERS_IN_WORD;

        debug_assert!(
            word_position < self.words.len(),
            concat!(
                "The word_position {} must be less than the number of words {}. ",
                "You have obtained this values starting from the index {} and the number of registers in word {}. ",
                "We currently have {} registers. Currently using precision {} and number of bits {}."
            ),
            word_position,
            self.words.len(),
            index,
            Self::NUMBER_OF_REGISTERS_IN_WORD,
            PRECISION::NUMBER_OF_REGISTERS,
            PRECISION::EXPONENT,
            BITS
        );

        // Extract the current value of the register at `index`.
        let register_value: u32 = (self.words[word_position] >> (register_position_in_u32 * BITS))
            & Self::LOWER_REGISTER_MASK;

        // Otherwise, update the register using a bit mask.
        if number_of_zeros > register_value {
            debug_assert!(
                self.multeplicities[register_value as usize] > PRECISION::NumberOfZeros::ZERO,
            );

            self.multeplicities[register_value as usize] -= PRECISION::NumberOfZeros::ONE;
            self.multeplicities[number_of_zeros as usize] += PRECISION::NumberOfZeros::ONE;

            self.words[word_position] &=
                !(Self::LOWER_REGISTER_MASK << (register_position_in_u32 * BITS));
            self.words[word_position] |= number_of_zeros << (register_position_in_u32 * BITS);

            // We check that the word we have edited maintains that the padding bits are all zeros
            // and have not been manipulated in any way. If these bits were manipulated, it would mean
            // that we have a bug in the code.
            debug_assert!(
                self.words[word_position] & Self::PADDING_BITS_MASK == 0,
                concat!(
                    "The padding bits of the word {} must be all zeros. ",
                    "We have obtained {} instead."
                ),
                self.words[word_position],
                self.words[word_position] & Self::PADDING_BITS_MASK
            );
        }
    }
}
