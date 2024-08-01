use crate::optimizers::*;
use crate::prelude::*;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MLE<const ERROR: i32, H> {
    inner: H,
}

impl<const ERROR: i32, P: Precision + WordType<BITS>, const BITS: usize> AsRef<MLE<ERROR, Self>>
    for HyperLogLog<P, BITS>
{
    fn as_ref(&self) -> &MLE<ERROR, Self> {
        unsafe { core::mem::transmute(self) }
    }
}

impl<const ERROR: i32, P: Precision + WordType<BITS>, const BITS: usize, const N: usize>
    AsRef<[MLE<ERROR, HyperLogLog<P, BITS>>; N]> for HyperLogLogArray<P, BITS, N>
{
    fn as_ref(&self) -> &[MLE<ERROR, HyperLogLog<P, BITS>>; N] {
        unsafe { core::mem::transmute(self) }
    }
}

impl<const ERROR: i32, P: Precision + WordType<BITS>, const BITS: usize> AsRef<HyperLogLog<P, BITS>>
    for MLE<ERROR, HyperLogLog<P, BITS>>
{
    fn as_ref(&self) -> &HyperLogLog<P, BITS> {
        &self.inner
    }
}

impl<const ERROR: i32, P: Precision + WordType<BITS>, const BITS: usize, const N: usize>
    AsRef<[HyperLogLog<P, BITS>; N]> for MLE<ERROR, HyperLogLogArray<P, BITS, N>>
{
    fn as_ref(&self) -> &[HyperLogLog<P, BITS>; N] {
        unsafe { core::mem::transmute(self) }
    }
}

impl<const ERROR: i32, P: Precision + WordType<BITS>, const BITS: usize> From<MLE<ERROR, Self>>
    for HyperLogLog<P, BITS>
{
    fn from(mle: MLE<ERROR, Self>) -> Self {
        mle.inner
    }
}

impl<const ERROR: i32, P: Precision + WordType<BITS>, const BITS: usize> From<HyperLogLog<P, BITS>>
    for MLE<ERROR, HyperLogLog<P, BITS>>
{
    fn from(hll: HyperLogLog<P, BITS>) -> Self {
        Self { inner: hll }
    }
}

impl<const ERROR: i32, P: Precision + WordType<BITS>, const BITS: usize> AsRef<MLE<ERROR, Self>>
    for HyperLogLogWithMultiplicities<P, BITS>
{
    fn as_ref(&self) -> &MLE<ERROR, Self> {
        unsafe { core::mem::transmute(self) }
    }
}

impl<const ERROR: i32, P: Precision + WordType<BITS>, const BITS: usize>
    AsRef<HyperLogLogWithMultiplicities<P, BITS>>
    for MLE<ERROR, HyperLogLogWithMultiplicities<P, BITS>>
{
    fn as_ref(&self) -> &HyperLogLogWithMultiplicities<P, BITS> {
        &self.inner
    }
}

impl<const ERROR: i32, P: Precision + WordType<BITS>, const BITS: usize> From<MLE<ERROR, Self>>
    for HyperLogLogWithMultiplicities<P, BITS>
{
    fn from(mle: MLE<ERROR, Self>) -> Self {
        mle.inner
    }
}

impl<const ERROR: i32, P: Precision + WordType<BITS>, const BITS: usize>
    From<HyperLogLogWithMultiplicities<P, BITS>>
    for MLE<ERROR, HyperLogLogWithMultiplicities<P, BITS>>
{
    fn from(hll: HyperLogLogWithMultiplicities<P, BITS>) -> Self {
        Self { inner: hll }
    }
}

pub trait JointEstimation<P: Precision + WordType<BITS>, const BITS: usize>
where
    Self: HyperLogLogTrait<P, BITS>,
{
    fn estimate_cardinality_from_multiplicities_using_mle<const ERROR: i32>(
        multiplicities: &P::RegisterMultiplicities,
    ) -> f32 {
        // If the multeplicity associated to the last register
        // is equal to the number of registers, we return infinity.
        let number_of_saturated_registers: usize = multiplicities.last().unwrap().convert();
        if number_of_saturated_registers == P::NUMBER_OF_REGISTERS {
            return f32::INFINITY;
        }

        let q = multiplicities.len() - 2;

        let smallest_register_value = multiplicities.first_non_zero_index().unwrap().get_max(1);
        let largest_register_value = multiplicities.last_non_zero_index().unwrap().get_min(q);

        debug_assert!(smallest_register_value > 0);
        debug_assert!(
            largest_register_value > 0,
            concat!(
                "The largest register value should be greater than 0. ",
                "The multiplicities are: {:?}."
            ),
            multiplicities
        );

        let mut raw_estimate = 0.0;

        for k in (smallest_register_value..=largest_register_value).rev() {
            let register_multeplicity: f32 = multiplicities[k].convert();
            raw_estimate = 0.5_f32 * raw_estimate + register_multeplicity;
        }

        let two_to_minus_smallest_register: i32 = (127 - smallest_register_value as i32) << 23;
        raw_estimate *= f32::from_le_bytes(two_to_minus_smallest_register.to_le_bytes());

        let c: f32 =
            (*multiplicities.last().unwrap() + multiplicities[largest_register_value]).convert();

        let mut g_prev: f32 = 0.0;
        let number_of_zero_registers: f32 = multiplicities[0].convert();
        let a: f32 = raw_estimate + number_of_zero_registers;

        let two_to_minus_q: i32 = (127 - q as i32) << 23;
        let b: f32 = raw_estimate
            + number_of_saturated_registers as f32
                * f32::from_le_bytes(two_to_minus_q.to_le_bytes());

        let number_of_non_zero_registers: f32 =
            P::NUMBER_OF_REGISTERS as f32 - number_of_zero_registers;

        let mut x = if b <= 1.5 * a {
            number_of_non_zero_registers / (0.5 * b + a)
        } else {
            (number_of_non_zero_registers / b) * (b / a).ln_1p()
        };

        // We begin the secant method iterations.
        let mut delta_x = x;
        let relative_error_limit = 10.0_f32.powi(-ERROR) / (P::NUMBER_OF_REGISTERS as f32).sqrt();

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

            let mut g: f32 = c * taylor_series_approximation;

            for k in (smallest_register_value..=largest_register_value.saturating_sub(1)).rev() {
                let taylor_series_approximation_prime: f32 = 1.0 - taylor_series_approximation;
                taylor_series_approximation = (x_first
                    + taylor_series_approximation * taylor_series_approximation_prime)
                    / (x_first + taylor_series_approximation_prime);
                let register_multeplicity: f32 = multiplicities[k].convert();
                g += register_multeplicity * taylor_series_approximation;
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

        P::NUMBER_OF_REGISTERS as f32 * x
    }

    #[inline]
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
    /// let left_difference_true = set1.difference(&set2).count() as f32;
    /// let right_difference_true = set2.difference(&set1).count() as f32;
    ///
    /// assert!(set1.is_disjoint(&set2));
    ///
    /// let mut hll1 = HyperLogLogWithMultiplicities::<Precision6, 6>::default();
    /// let mut hll2 = HyperLogLogWithMultiplicities::<Precision6, 6>::default();
    ///
    /// for &elem in &vec1 {
    ///     hll1.insert(elem);
    /// }
    ///
    /// for &elem in &vec2 {
    ///     hll2.insert(elem);
    /// }
    ///
    /// let euc = hll1.joint_cardinality_estimation::<f32, 4>(&hll2);
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
    /// let left_difference_true = set1.difference(&set2).count() as f32;
    /// let right_difference_true = set2.difference(&set1).count() as f32;
    ///
    /// assert!(!set1.is_disjoint(&set2));
    ///
    /// let intersection_cardinality = set1.intersection(&set2).count();
    ///
    /// let mut hll1 = HyperLogLogWithMultiplicities::<Precision6, 6>::default();
    /// let mut hll2 = HyperLogLogWithMultiplicities::<Precision6, 6>::default();
    ///
    /// for &elem in &vec1 {
    ///    hll1.insert(elem);
    /// }
    ///
    /// for &elem in &vec2 {
    ///    hll2.insert(elem);
    /// }
    ///
    /// let euc = hll1.joint_cardinality_estimation::<f32, 4>(&hll2);
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
    ///     intersection_cardinality < intersection_cardinality as f32 * 1.2,
    ///     concat!(
    ///         "We expected the intersection cardinality to be around the actual cardinality of the set. ",
    ///         "Obtained: {}, Expected not more than: {}.",
    ///     ),
    ///     intersection_cardinality, intersection_cardinality as f32 * 1.2,
    /// );
    ///
    /// assert!(
    ///     intersection_cardinality > intersection_cardinality as f32 * 0.8,
    ///     concat!(
    ///         "We expected the intersection cardinality to be around the actual cardinality of the set. ",
    ///         "Obtained: {}, Expected not less than: {}.",
    ///     ),
    ///     intersection_cardinality, intersection_cardinality as f32 * 0.8,
    /// );
    ///
    /// ```
    ///
    fn joint_cardinality_estimation<F: Default + Primitive<f32> + MaxMin, const ERROR: i32>(
        &self,
        other: &Self,
    ) -> EstimatedUnionCardinalities<F> {
        let mut left_multiplicities_larger = P::RegisterMultiplicities::default_array();
        let mut left_multiplicities_smaller = P::RegisterMultiplicities::default_array();
        let mut right_multiplicities_larger = P::RegisterMultiplicities::default_array();
        let mut right_multiplicities_smaller = P::RegisterMultiplicities::default_array();
        let mut joint_multiplicities = P::RegisterMultiplicities::default_array();

        let mut raw_union_estimate = 0.0;
        let mut raw_left_estimate = 0.0;
        let mut raw_right_estimate = 0.0;
        let mut union_zeros = 0;

        // First, we populate the vectors of multiplities
        for (left_word, right_word) in self
            .get_words()
            .iter_elements()
            .copied()
            .zip(other.get_words().iter_elements().copied())
        {
            let mut union_partial: f32 = 0.0;
            let mut left_partial: f32 = 0.0;
            let mut right_partial: f32 = 0.0;
            for i in 0..Self::NUMBER_OF_REGISTERS_IN_WORD {
                let left_register = (left_word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;
                let right_register = (right_word >> (i * BITS)) & Self::LOWER_REGISTER_MASK;

                let maximal_register = (left_register).max(right_register);
                union_partial += f32::from_le_bytes(((127 - maximal_register) << 23).to_le_bytes());
                left_partial += f32::from_le_bytes(((127 - left_register) << 23).to_le_bytes());
                right_partial += f32::from_le_bytes(((127 - right_register) << 23).to_le_bytes());
                union_zeros += (maximal_register == 0) as usize;

                // We compute the fractional multiplicities for the left and right HLL
                match left_register.cmp(&right_register) {
                    core::cmp::Ordering::Less => {
                        left_multiplicities_smaller[left_register as usize] +=
                            P::NumberOfZeros::ONE;
                        right_multiplicities_larger[right_register as usize] +=
                            P::NumberOfZeros::ONE;
                    }
                    core::cmp::Ordering::Greater => {
                        left_multiplicities_larger[left_register as usize] += P::NumberOfZeros::ONE;
                        right_multiplicities_smaller[right_register as usize] +=
                            P::NumberOfZeros::ONE;
                    }
                    core::cmp::Ordering::Equal => {
                        // If left register is equal to right register
                        joint_multiplicities[left_register as usize] += P::NumberOfZeros::ONE;
                    }
                }
            }
            raw_union_estimate += union_partial;
            raw_left_estimate += left_partial;
            raw_right_estimate += right_partial;
        }

        union_zeros -= Self::get_number_of_padding_registers();

        // We need to subtract the padding registers from the raw estimates
        // as for each such register we are adding a one.
        raw_union_estimate -= Self::get_number_of_padding_registers() as f32;
        raw_left_estimate -= Self::get_number_of_padding_registers() as f32;
        raw_right_estimate -= Self::get_number_of_padding_registers() as f32;

        joint_multiplicities[0] -=
            P::NumberOfZeros::reverse(Self::get_number_of_padding_registers());

        // We get the best estimates from HyperLogLog++
        let mut union_cardinality_estimate =
            Self::adjust_estimate_with_zeros(raw_union_estimate, union_zeros);

        let left_cardinality_estimate = Self::adjust_estimate_with_zeros(
            raw_left_estimate,
            self.get_number_of_zero_registers(),
        );

        let right_cardinality_estimate = Self::adjust_estimate_with_zeros(
            raw_right_estimate,
            other.get_number_of_zero_registers(),
        );

        union_cardinality_estimate = union_cardinality_estimate
            .get_min(left_cardinality_estimate + right_cardinality_estimate);

        // If the sum of the number of registers equal to zero, i.e.
        // the first value in the multiplicities vectors, is equal
        // to the number of registers, it means that the intersection
        // is empty.
        let left_difference_number_of_zeros: usize = left_multiplicities_smaller[0].convert();
        let joint_number_of_zeros: usize = joint_multiplicities[0].convert();
        let right_difference_number_of_zeros: usize = right_multiplicities_smaller[0].convert();

        let number_of_zeros: usize = left_difference_number_of_zeros
            + joint_number_of_zeros
            + right_difference_number_of_zeros;
        if number_of_zeros == P::NUMBER_OF_REGISTERS {
            return EstimatedUnionCardinalities::from((
                F::reverse(left_cardinality_estimate),
                F::reverse(right_cardinality_estimate),
                F::reverse(0.0_f32),
            ));
        }

        let intersection: f32 =
            left_cardinality_estimate + right_cardinality_estimate - union_cardinality_estimate;
        let left_difference: f32 = union_cardinality_estimate - right_cardinality_estimate;
        let right_difference: f32 = union_cardinality_estimate - left_cardinality_estimate;

        let relative_error_limit = 10.0_f32.powi(-ERROR) / (P::NUMBER_OF_REGISTERS as f32).sqrt();

        // let reciprocal_registers = 1.0 / P::NUMBER_OF_REGISTERS as f32;

        let exponent: i32 = 127 - P::EXPONENT as i32;

        // we introdce the following expressions to simplify the computation
        // of the gradient.
        let x = |phi: f32, two_to_minus_register: f32| -> f32 { phi.exp() * two_to_minus_register };

        let yz = |x: f32| -> (f32, f32) {
            let exp_m1 = (-x).exp_m1();
            (1.0 + exp_m1, -exp_m1)
        };

        // We precompute q and q+1 for reference.
        let q_plus_one: usize = joint_multiplicities.len() - 1;
        let q: i32 = q_plus_one as i32 - 1;
        let float_joint_multiplicities: P::FloatMultiplicities =
            joint_multiplicities.convert_array();

        // We initialize the vectors for the Adam optimizer.
        let mut phis = [
            left_difference.max(1.0).ln(),
            right_difference.max(1.0).ln(),
            intersection.max(1.0).ln(),
        ];
        let mut gradients: [f32; 3] = [0.0, 0.0, 0.0];

        let mut optimizer: Adam<f32, 3> = Adam::default();

        let float_left_multiplicities_smaller: P::FloatMultiplicities =
            left_multiplicities_smaller.convert_array();
        let float_left_multiplicities_larger: P::FloatMultiplicities =
            left_multiplicities_larger.convert_array();

        let float_right_multiplicities_smaller: P::FloatMultiplicities =
            right_multiplicities_smaller.convert_array();
        let float_right_multiplicities_larger: P::FloatMultiplicities =
            right_multiplicities_larger.convert_array();

        let left_number_of_zeros = float_left_multiplicities_smaller[0]
            + float_left_multiplicities_larger[0]
            + float_joint_multiplicities[0];
        let right_number_of_zeros = float_right_multiplicities_smaller[0]
            + float_right_multiplicities_larger[0]
            + float_joint_multiplicities[0];
        let intersection_number_of_zeros: f32 = float_right_multiplicities_smaller[0]
            + float_left_multiplicities_smaller[0]
            + float_joint_multiplicities[0];

        let left_number_of_saturated_registers = float_left_multiplicities_larger[q_plus_one];
        let right_number_of_saturated_registers = float_right_multiplicities_larger[q_plus_one];
        let intersection_number_of_saturated_registers = float_joint_multiplicities[q_plus_one];

        let two_to_zero: f32 = f32::from_le_bytes((exponent << 23).to_le_bytes());
        let two_to_minus_q: f32 = f32::from_le_bytes(((exponent - q) << 23).to_le_bytes());

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

            let denominator = 1.0 / (z_joint_q + y_joint_q * z_left_q * z_right_q);

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

            (1..q_plus_one as i32).for_each(|k| {
                let two_to_minus_register: f32 =
                    f32::from_le_bytes(((exponent - k) << 23).to_le_bytes());

                let x_left = x(phis[0], two_to_minus_register);
                let x_right = x(phis[1], two_to_minus_register);
                let x_joint = x(phis[2], two_to_minus_register);
                let (y_left, z_left) = yz(x_left);
                let (y_right, z_right) = yz(x_right);
                let (y_joint, z_joint) = yz(x_joint);

                let joint_k = float_joint_multiplicities[k as usize];
                let left_smaller_k = float_left_multiplicities_smaller[k as usize];
                let left_larger_k = float_left_multiplicities_larger[k as usize];
                let right_smaller_k = float_right_multiplicities_smaller[k as usize];
                let right_larger_k = float_right_multiplicities_larger[k as usize];

                let yj_zl = y_joint * z_left;
                let yjr_zl = yj_zl * y_right;
                let yj_zr = y_joint * z_right;
                let yjl_zr = yj_zr * y_left;
                let yjl = y_joint * y_left;
                let yjr = y_joint * y_right;
                let yj_zlr = yj_zl * z_right;
                let zj_plus_yj_zl = z_joint + yj_zl;
                let reciprocal_zj_plus_yj_zl = 1.0 / zj_plus_yj_zl;
                let zj_plus_yj_zr = z_joint + yj_zr;
                let reciprocal_zj_plus_yj_zr = 1.0 / zj_plus_yj_zr;
                let zj_plus_yj_zlr = z_joint + yj_zlr;
                let reciprocal_zj_plus_yj_zlr = 1.0 / zj_plus_yj_zlr;

                let left_reciprocal = left_smaller_k * (reciprocal_zj_plus_yj_zl * yjl - 1.0);
                let right_reciprocal = right_smaller_k * (reciprocal_zj_plus_yj_zr * yjr - 1.0);

                gradients[0] += x_left
                    * (left_reciprocal
                        + joint_k * (yjl_zr * reciprocal_zj_plus_yj_zlr - 1.0)
                        + left_larger_k * (y_left / z_left - 1.0));

                gradients[1] += x_right
                    * (right_reciprocal
                        + joint_k * (yjr_zl * reciprocal_zj_plus_yj_zlr - 1.0)
                        + right_larger_k * (y_right / z_right - 1.0));

                gradients[2] += x_joint
                    * (left_reciprocal
                        + right_reciprocal
                        + joint_k * ((yjl + yjr_zl) * reciprocal_zj_plus_yj_zlr - 1.0));
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

        EstimatedUnionCardinalities::from((
            F::reverse(left_difference + intersection),
            F::reverse(right_difference + intersection),
            F::reverse(left_difference + right_difference + intersection),
        ))
    }
}

impl<const ERROR: i32, P: Precision + WordType<BITS>, const BITS: usize> HyperLogLogTrait<P, BITS>
    for MLE<ERROR, HyperLogLog<P, BITS>>
{
    fn get_number_of_zero_registers(&self) -> usize {
        self.inner.get_number_of_zero_registers()
    }

    fn get_words(&self) -> &P::Words {
        self.inner.get_words()
    }

    fn estimate_cardinality(&self) -> f32 {
        let mut multeplicities = P::RegisterMultiplicities::default_array();

        self.inner
            .get_registers()
            .into_iter_elements()
            .for_each(|register| {
                multeplicities[register as usize] += P::NumberOfZeros::ONE;
            });

        Self::estimate_cardinality_from_multiplicities_using_mle::<ERROR>(&multeplicities)
    }

    fn estimate_union_and_sets_cardinality<F: Primitive<f32> + MaxMin>(
        &self,
        other: &Self,
    ) -> EstimatedUnionCardinalities<F> {
        self.joint_cardinality_estimation::<F, ERROR>(other)
    }
}

impl<const ERROR: i32, P: Precision + WordType<BITS>, const BITS: usize> HyperLogLogTrait<P, BITS>
    for MLE<ERROR, HyperLogLogWithMultiplicities<P, BITS>>
{
    fn get_number_of_zero_registers(&self) -> usize {
        self.inner.get_number_of_zero_registers()
    }

    fn get_words(&self) -> &P::Words {
        self.inner.get_words()
    }

    fn estimate_cardinality(&self) -> f32 {
        Self::estimate_cardinality_from_multiplicities_using_mle::<ERROR>(
            &self.inner.multiplicities,
        )
    }

    fn estimate_union_and_sets_cardinality<F: Primitive<f32> + MaxMin>(
        &self,
        other: &Self,
    ) -> EstimatedUnionCardinalities<F> {
        self.joint_cardinality_estimation::<F, ERROR>(other)
    }
}

impl<T, P: Precision + WordType<BITS>, const BITS: usize> JointEstimation<P, BITS> for T where
    T: HyperLogLogTrait<P, BITS>
{
}

impl<const ERROR: i32, P: Precision + WordType<BITS>, const BITS: usize, F: Primitive<f32>>
    SetLike<F> for MLE<ERROR, HyperLogLog<P, BITS>>
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
        self.as_ref().get_cardinality()
    }
}

impl<F: Primitive<f32>, const ERROR: i32, P: Precision + WordType<BITS>, const BITS: usize>
    HyperSpheresSketch<F> for MLE<ERROR, HyperLogLog<P, BITS>>
{
}
