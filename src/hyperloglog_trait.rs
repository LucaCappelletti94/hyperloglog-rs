use crate::estimated_union_cardinalities::EstimatedUnionCardinalities;
use crate::precisions::Precision;
use crate::prelude::*;
use crate::utils::FloatNumber;
use core::hash::Hash;

pub trait HyperLogLogTrait<
    P: Precision,
    B: Bits,
    Hasher: core::hash::Hasher + Default + Default = twox_hash::XxHash64,
>:
    Sized + Default + Eq + PartialEq + BitOrAssign<Self> + BitOr<Self, Output = Self> + Send + Sync
{
    type Registers: Registers<P, B>;

    #[inline(always)]
    fn currently_applies_linear_counting<F: FloatNumber>(&self) -> bool
    where
        P: PrecisionConstants<F>,
    {
        Self::linear_count_zeros(self.get_number_of_zero_registers())
    }

    #[inline(always)]
    fn linear_count_zeros<F: FloatNumber>(number_of_zeros: P::NumberOfZeros) -> bool
    where
        P: PrecisionConstants<F>,
    {
        number_of_zeros >= P::LINEAR_COUNT_ZEROS
    }

    #[inline(always)]
    fn get_estimate_from_harmonic_sum<F: FloatNumber>(harmonic_sum: F) -> F
    where
        P: PrecisionConstants<F>,
    {
        // Apply the final scaling factor to obtain the estimate of the cardinality
        P::ALPHA * P::NUMBER_OF_REGISTERS_FLOAT * P::NUMBER_OF_REGISTERS_FLOAT / harmonic_sum
    }

    #[inline(always)]
    fn adjust_harmonic_sum_with_zeros<F: FloatNumber>(
        harmonic_sum: F,
        number_of_zeros: P::NumberOfZeros,
    ) -> F
    where
        P: PrecisionConstants<F>,
    {
        if Self::linear_count_zeros(number_of_zeros) {
            return P::small_correction(number_of_zeros);
        }
        P::adjust_estimate(Self::get_estimate_from_harmonic_sum(harmonic_sum))
    }

    /// Returns a reference to the registers of the HyperLogLog counter.
    fn registers(&self) -> &Self::Registers;

    /// Returns the harmonic sum of the registers.
    fn harmonic_sum<F: FloatNumber>(&self) -> F
    where
        P: PrecisionConstants<F>;

    #[inline(always)]
    /// Estimates the cardinality of the set based on the HLL counter data.
    ///
    /// # Example
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    /// let mut hll = HyperLogLog::<
    ///     Precision9,
    ///     Bits5,
    ///     <Precision9 as ArrayRegister<Bits5>>::ArrayRegister,
    /// >::default();
    /// let elements = vec![1, 2, 3, 4, 5];
    /// for element in &elements {
    ///     hll.insert(element);
    /// }
    /// let estimated_cardinality: f32 = hll.estimate_cardinality();
    /// assert!(
    ///     estimated_cardinality >= elements.len() as f32 * 0.9
    ///         && estimated_cardinality <= elements.len() as f32 * 1.1
    /// );
    /// ```
    ///
    /// # Returns
    /// * `f32` - The estimated cardinality of the set.
    fn estimate_cardinality<F: FloatNumber>(&self) -> F
    where
        P: PrecisionConstants<F>,
    {
        Self::adjust_harmonic_sum_with_zeros(
            self.harmonic_sum(),
            self.get_number_of_zero_registers(),
        )
    }

    fn estimate_cardinality_with_beta<F: FloatNumber>(&self) -> F
    where
        P: PrecisionConstants<F>,
    {
        let number_of_zero_registers = F::from_usize(unsafe {
            self.get_number_of_zero_registers()
                .try_into()
                .unwrap_unchecked()
        });
        P::ALPHA
            * (P::NUMBER_OF_REGISTERS_FLOAT
                * (P::NUMBER_OF_REGISTERS_FLOAT - number_of_zero_registers))
            / (self.harmonic_sum() + P::beta_horner(number_of_zero_registers))
            + F::HALF
    }

    fn estimate_cardinality_with_biases<F: FloatNumber>(&self, estimates: &[F], biases: &[F]) -> F
    where
        P: PrecisionConstants<F>,
    {
        assert_eq!(estimates.len(), biases.len());
        if Self::linear_count_zeros(self.get_number_of_zero_registers()) {
            return P::small_correction(self.get_number_of_zero_registers());
        }

        let estimate = Self::get_estimate_from_harmonic_sum(self.harmonic_sum());

        // Apply the small range correction factor if the raw estimate is below the threshold
        // and there are zero registers in the counter.
        estimate
            - F::from_usize(
                (if P::requires_bias_correction(estimate) {
                    if estimate < estimates[0] {
                        biases[0]
                    } else if estimate >= estimates[estimates.len() - 1] {
                        biases[biases.len() - 1]
                    } else {
                        // Find the partition index of the estimate.
                        let partition_index = estimates.partition_point(|est| *est <= estimate);

                        // Return linear interpolation between raw's neighboring points.
                        let ratio = (estimate - estimates[partition_index - 1])
                            / (estimates[partition_index] - estimates[partition_index - 1]);

                        // Calculate bias.
                        biases[partition_index - 1]
                            + ratio * (biases[partition_index] - biases[partition_index - 1])
                    }
                } else {
                    F::ZERO
                })
                .to_usize(),
            )
    }

    /// Returns cardinality without the application of any correction.
    fn estimate_incorrected_cardinality<F: FloatNumber>(&self) -> F
    where
        P: PrecisionConstants<F>,
    {
        Self::get_estimate_from_harmonic_sum(self.harmonic_sum())
    }

    #[inline(always)]
    /// Returns an estimate of the cardinality of the union of two HyperLogLog counters.
    ///
    /// This method calculates an estimate of the cardinality of the union of two HyperLogLog counters
    /// using the raw estimation values of each counter. It combines the estimation values by iterating
    /// over the register words of both counters and performing necessary calculations.
    ///
    /// # Arguments
    /// * `other`: A reference to the other HyperLogLog counter.
    ///
    /// # Returns
    /// An estimation of the cardinality of the union of the two HyperLogLog counters.
    ///
    /// # Example
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll1.insert(1);
    /// hll1.insert(2);
    ///
    /// let mut hll2 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll2.insert(2);
    /// hll2.insert(3);
    ///
    /// let union_cardinality: f32 = hll1.estimate_union_cardinality(&hll2);
    ///
    /// assert!(union_cardinality >= 3.0 * 0.9 && union_cardinality <= 3.0 * 1.1);
    /// ```
    fn estimate_union_cardinality<F: FloatNumber>(&self, other: &Self) -> F
    where
        P: PrecisionConstants<F>,
    {
        let (harmonic_sum, union_zeros) = self
            .registers()
            .get_harmonic_sum_and_zeros(other.registers());

        Self::adjust_harmonic_sum_with_zeros(harmonic_sum, union_zeros)
    }

    #[cfg(feature = "std")]
    fn estimate_union_cardinality_with_mle<const ERROR: i32, F: FloatNumber>(
        &self,
        other: &Self,
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
            .registers()
            .iter_registers_zipped(other.registers())
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
        let union_cardinality_estimate =
            Self::adjust_harmonic_sum_with_zeros(union_harmonic_sum, union_zeros);

        let left_cardinality_estimate = self.estimate_cardinality();
        let right_cardinality_estimate = other.estimate_cardinality();

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

        let mut optimizer: crate::utils::Adam<F, 3> = crate::utils::Adam::default();

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

    #[inline(always)]
    /// Returns an estimate of the cardinality of the intersection of two HyperLogLog counters.
    ///
    /// This method calculates an estimate of the cardinality of the intersection of two HyperLogLog
    /// counters using the raw estimation values of each counter. It combines the estimation values by
    /// iterating over the register words of both counters and performing necessary calculations.
    ///
    /// # Arguments
    /// * `other`: A reference to the other HyperLogLog counter.
    ///
    /// # Returns
    /// An estimation of the cardinality of the intersection of the two HyperLogLog counters.
    ///
    /// # Example
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    ///
    /// let mut hll2 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll2.insert(&2);
    /// hll2.insert(&3);
    ///
    /// let intersection_cardinality: f32 = hll1.estimate_intersection_cardinality(&hll2);
    ///
    /// assert!(intersection_cardinality >= 1.0 * 0.9 && intersection_cardinality <= 1.0 * 1.1);
    /// ```
    fn estimate_intersection_cardinality<F: FloatNumber>(&self, other: &Self) -> F
    where
        P: PrecisionConstants<F>,
    {
        let self_cardinality = self.estimate_cardinality::<F>();
        let other_cardinality = other.estimate_cardinality::<F>();
        let union_cardinality = self.estimate_union_cardinality::<F>(other);

        // We apply correction to the union cardinality to get the intersection cardinality.
        if self_cardinality + other_cardinality < union_cardinality {
            F::ZERO
        } else {
            self_cardinality + other_cardinality - union_cardinality
        }
    }

    #[inline(always)]
    /// Returns an estimate of the Jaccard index between two HyperLogLog counters.
    ///
    /// The Jaccard index is a measure of similarity between two sets. In the context of HyperLogLog
    /// counters, it represents the ratio of the size of the intersection of the sets represented by
    /// the counters to the size of their union. This method estimates the Jaccard index by utilizing
    /// the cardinality estimation values of the intersection, left set, and right set.
    ///
    /// # Arguments
    /// * `other`: A reference to the other HyperLogLog counter.
    ///
    /// # Returns
    /// An estimation of the Jaccard index between the two HyperLogLog counters.
    ///
    /// # Example
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    /// hll1.insert(&3);
    /// hll1.insert(&4);
    ///
    /// let mut hll2 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll2.insert(&2);
    /// hll2.insert(&3);
    /// hll2.insert(&5);
    /// hll2.insert(&6);
    ///
    /// let jaccard_index: f32 = hll1.estimate_jaccard_index(&hll2);
    ///
    /// let expected = 2.0 / 6.0;
    ///
    /// assert!(jaccard_index >= expected * 0.9 && jaccard_index <= expected * 1.1);
    /// ```
    fn estimate_jaccard_index<F: FloatNumber>(&self, other: &Self) -> F
    where
        P: PrecisionConstants<F>,
    {
        let self_cardinality = self.estimate_cardinality::<F>();
        let other_cardinality = other.estimate_cardinality::<F>();
        let union_cardinality = self.estimate_union_cardinality::<F>(other);

        // We apply correction to the union cardinality to get the intersection cardinality.
        if self_cardinality + other_cardinality < union_cardinality || union_cardinality.is_zero() {
            F::ZERO
        } else {
            (self_cardinality + other_cardinality - union_cardinality) / union_cardinality
        }
    }

    #[inline(always)]
    /// Returns an estimate of the cardinality of the current HyperLogLog counter minus the provided one.
    ///
    /// # Arguments
    /// * `other`: A reference to the other HyperLogLog counter.
    ///
    /// # Example
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    /// hll1.insert(&3);
    /// hll1.insert(&4);
    ///
    /// let mut hll2 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll2.insert(&2);
    /// hll2.insert(&3);
    /// hll2.insert(&5);
    /// hll2.insert(&6);
    ///
    /// let difference_cardinality: f32 = hll1.estimate_difference_cardinality(&hll2);
    ///
    /// assert!(difference_cardinality >= 2.0 * 0.9 && difference_cardinality <= 2.0 * 1.1);
    /// ```
    fn estimate_difference_cardinality<F: FloatNumber>(&self, other: &Self) -> F
    where
        P: PrecisionConstants<F>,
    {
        let union_cardinality = self.estimate_union_cardinality::<F>(other);
        let other_cardinality = other.estimate_cardinality::<F>();
        if union_cardinality < other_cardinality {
            F::ZERO
        } else {
            union_cardinality - other_cardinality
        }
    }

    #[inline(always)]
    /// Returns whether no element was yet added to the HLL counter.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll: HyperLogLog<
    ///     Precision8,
    ///     Bits4,
    ///     <Precision8 as ArrayRegister<Bits4>>::ArrayRegister,
    /// > = HyperLogLog::default();
    ///
    /// assert!(hll.is_empty());
    ///
    /// hll.insert(&1);
    ///
    /// assert!(!hll.is_empty());
    /// ```
    fn is_empty(&self) -> bool {
        P::NUMBER_OF_REGISTERS
            == unsafe {
                self.get_number_of_zero_registers()
                    .try_into()
                    .unwrap_unchecked()
            }
    }

    /// Returns whether the HLL counter is full.
    ///
    /// A counter is considered full when all registers are maximally filled.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll: HyperLogLog<
    ///     Precision4,
    ///     Bits4,
    ///     <Precision4 as ArrayRegister<Bits4>>::ArrayRegister,
    /// > = HyperLogLog::from_registers([u32::MAX; 2]);
    ///
    /// assert!(
    ///     hll.is_full(),
    ///     "1) The counter is not full: {:?}",
    ///     hll.harmonic_sum::<f64>()
    /// );
    ///
    /// let mut hll: HyperLogLog<
    ///     Precision10,
    ///     Bits4,
    ///     <Precision10 as ArrayRegister<Bits4>>::ArrayRegister,
    /// > = HyperLogLog::from_registers([u32::MAX; 128]);
    ///
    /// assert!(
    ///     hll.is_full(),
    ///     "2) The counter is not full: {:?}",
    ///     hll.harmonic_sum::<f64>()
    /// );
    ///
    /// let mut hll: HyperLogLog<
    ///     Precision4,
    ///     Bits4,
    ///     <Precision4 as ArrayRegister<Bits4>>::ArrayRegister,
    /// > = HyperLogLog::from_registers([1; 2]);
    ///
    /// assert!(!hll.is_full());
    ///
    /// let mut hll: HyperLogLog<
    ///     Precision10,
    ///     Bits4,
    ///     <Precision10 as ArrayRegister<Bits4>>::ArrayRegister,
    /// > = HyperLogLog::from_registers([1; 128]);
    ///
    /// assert!(!hll.is_full());
    ///
    /// let mut hll: HyperLogLog<
    ///     Precision10,
    ///     Bits4,
    ///     <Precision10 as ArrayRegister<Bits4>>::ArrayRegister,
    /// > = HyperLogLog::default();
    ///
    /// assert!(!hll.is_full());
    /// ```
    fn is_full(&self) -> bool;

    /// Returns the number of registers with zero values. This value is used for computing a small
    /// correction when estimating the cardinality of a small set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// // Create a new HyperLogLog counter with precision 14 and 5 bits per register.
    /// let mut hll = HyperLogLog::<
    ///     Precision14,
    ///     Bits5,
    ///     <Precision14 as ArrayRegister<Bits5>>::ArrayRegister,
    /// >::default();
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
    fn get_number_of_zero_registers(&self) -> P::NumberOfZeros;

    #[inline(always)]
    /// Returns `true` if the HyperLogLog counter may contain the given element.
    ///
    /// # Arguments
    /// * `rhs` - The element to check.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll: HyperLogLog<
    ///     Precision8,
    ///     Bits6,
    ///     <Precision8 as ArrayRegister<Bits6>>::ArrayRegister,
    /// > = HyperLogLog::default();
    /// assert_eq!(hll.may_contain(&42), false);
    ///
    /// hll.insert(&42);
    /// assert_eq!(hll.may_contain(&42), true);
    /// ```
    fn may_contain<T: Hash>(&self, rhs: &T) -> bool {
        self.get_register(Self::hash_and_index::<T>(rhs).1) > 0
    }

    #[inline(always)]
    /// Returns whether the provided HyperLogLog counter may be fully contained in the current HyperLogLog counter.
    ///
    /// # Arguments
    /// * `rhs` - The HyperLogLog counter to check.
    ///
    /// # Implementative details
    /// We define a counter that fully contains another counter when all of the registers
    /// of the first counter are greater than or equal to the corresponding registers of the second counter.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1: HyperLogLog<
    ///     Precision8,
    ///     Bits6,
    ///     <Precision8 as ArrayRegister<Bits6>>::ArrayRegister,
    /// > = HyperLogLog::default();
    /// let mut hll2: HyperLogLog<
    ///     Precision8,
    ///     Bits6,
    ///     <Precision8 as ArrayRegister<Bits6>>::ArrayRegister,
    /// > = HyperLogLog::default();
    ///
    /// hll1.insert(&42);
    /// hll1.insert(&43);
    /// hll1.insert(&44);
    ///
    /// hll2.insert(&42);
    /// hll2.insert(&43);
    ///
    /// assert_eq!(hll1.may_contain_all(&hll2), true);
    /// assert_eq!(hll2.may_contain_all(&hll1), false);
    ///
    /// hll2.insert(&44);
    ///
    /// assert_eq!(hll1.may_contain_all(&hll2), true);
    /// assert_eq!(hll2.may_contain_all(&hll1), true);
    /// ```
    fn may_contain_all(&self, rhs: &Self) -> bool {
        self.registers()
            .iter_registers_zipped(rhs.registers())
            .all(|(lhs, rhs)| lhs >= rhs)
    }

    /// Insert a value into the HyperLogLog counter.
    ///
    /// # Arguments
    /// * `value` - A reference to the value to be inserted.
    ///
    /// # Returns
    /// Whether the counter has changed after the insertion.
    fn insert<T: Hash>(&mut self, value: T) -> bool;

    #[inline(always)]
    fn hash_and_index<T: core::hash::Hash>(element: &T) -> (u64, usize) {
        let mut hasher = Hasher::default();
        element.hash(&mut hasher);
        let hash = hasher.finish();

        // Calculate the register's index using the highest bits of the hash.
        // The index of the register has to vary from 0 to 2^p - 1, where p is the precision,
        // so we use the highest p bits of the hash.
        let index: usize = hash as usize >> (64 - P::EXPONENT);

        // And we delete the used bits from the hash.
        let mut hash: u64 = hash << P::EXPONENT;

        debug_assert!(
            index < P::NUMBER_OF_REGISTERS,
            "The index {} must be less than the number of registers {}.",
            index,
            P::NUMBER_OF_REGISTERS
        );

        // We need to add ones to the hash to make sure that the
        // the number of zeros we obtain afterwards is never higher
        // than the maximal value that may be represented in a register
        // with BITS bits.
        if B::NUMBER_OF_BITS < 6 {
            hash |= 1 << (64 - ((1 << B::NUMBER_OF_BITS) - 1));
        } else {
            // The only goal of this operation is to guarantee that the
            // number of zeros in the leading position of the hash does
            // not include the zeroes elided by the shift by the EXPONENT,
            // and therefore limits the multeplicity of the number of zeros,
            // which are the values that will be used to calculate the register's
            // values, so as to avoid unused values.
            hash |= 1 << (P::EXPONENT - 1);
        }

        (hash, index)
    }

    /// Return the value of the register at the given index.
    fn get_register(&self, index: usize) -> u32;

    /// Clears out the HyperLogLog counter.
    fn clear(&mut self);

    /// Extend the HyperLogLog counter with the elements from an iterator.
    fn extend<I: IntoIterator<Item = T>, T: Hash>(&mut self, iter: I) {
        for value in iter {
            self.insert(&value);
        }
    }
}
