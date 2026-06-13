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

    /// Returns the cardinality estimated with the single-counter Maximum Likelihood Estimation.
    ///
    /// # Implementative details
    /// This is Ertl's secant-method maximum-likelihood estimator over the register
    /// multiplicities. It operates on the HyperLogLog register representation, so a hash-list
    /// operand is materialized into registers first. It is provided for completeness and
    /// comparison: it is less accurate, and substantially slower, than the default
    /// [`HyperLogLog::estimate_cardinality`] (HyperLogLog++ corrected) estimate.
    #[inline]
    pub fn estimate_cardinality_mle(&self) -> f64 {
        if self.is_hash_list() {
            let mut counter = self.clone();
            counter.convert_hash_list_to_hyperloglog().unwrap();
            return counter.estimate_cardinality_mle();
        }

        mle_cardinality::<P, B>(
            self.registers.iter_registers(),
            self.harmonic_sum,
            self.is_full(),
            2,
        )
    }

    /// Generalized joint Maximum Likelihood Estimation of the disjoint-cell cardinalities of the
    /// hypersphere sketch for `M` nested left counters and `N` nested right counters.
    ///
    /// Given `lefts = [A_0 subset ... subset A_{M-1}]` and `rights = [B_0 subset ... subset
    /// B_{N-1}]`, this jointly estimates, in a single optimization over the disjoint-region model,
    /// all `M*N + M + N` non-negative cell cardinalities:
    /// * `overlap[i][j] = |L_i intersect R_j|`, the exclusive overlap grid, where `L_i = A_i \
    ///   A_{i-1}` and `R_j = B_j \ B_{j-1}` are the left/right shells.
    /// * `left_diff[i] = |L_i \ B_{N-1}|` and `right_diff[j] = |R_j \ A_{M-1}|`, the margins.
    ///
    /// Because the parameters are the disjoint regions themselves (optimized in log-space), the
    /// returned cells are non-negative and globally consistent by construction. At `M = N = 1`
    /// this reduces to the three-region model of [`HyperLogLog::estimate_union_cardinality_mle`].
    ///
    /// # Implementative details
    /// Any hash-list operand is materialized into registers first. The optimization is warm-started
    /// from the pairwise sketch and refined with an Adam optimizer driven by the exact forward-mode
    /// gradient of the joint per-register log-likelihood. See `docs/joint_mle_math.md`.
    #[inline]
    pub fn joint_sketch_mle<const M: usize, const N: usize>(
        lefts: &[Self; M],
        rights: &[Self; N],
    ) -> ([[f64; N]; M], [f64; M], [f64; N]) {
        let materialize = |counter: &Self| -> Self {
            if counter.is_hash_list() {
                let mut counter = counter.clone();
                counter.convert_hash_list_to_hyperloglog().unwrap();
                counter
            } else {
                counter.clone()
            }
        };
        let lefts: [Self; M] = core::array::from_fn(|i| materialize(&lefts[i]));
        let rights: [Self; N] = core::array::from_fn(|j| materialize(&rights[j]));

        joint_sketch_mle_from_registers::<P, B, R, H, M, N>(&lefts, &rights)
    }

    /// Same as [`HyperLogLog::joint_sketch_mle`] but with a caller-chosen `optimizer` driving the
    /// refinement (generic composition). Compose optimizers with [`Chain`], for example
    /// `Chain<Adam, Lbfgs>`, or implement [`JointOptimizer`] for a custom strategy. The default
    /// method uses `Chain<Adam, Lbfgs>`. Example:
    /// `Counter::joint_sketch_mle_with::<Lbfgs>(&lefts, &rights)` (`M`/`N` inferred from the arrays).
    #[inline]
    pub fn joint_sketch_mle_with<O: JointOptimizer, const M: usize, const N: usize>(
        lefts: &[Self; M],
        rights: &[Self; N],
    ) -> ([[f64; N]; M], [f64; M], [f64; N]) {
        let materialize = |counter: &Self| -> Self {
            if counter.is_hash_list() {
                let mut counter = counter.clone();
                counter.convert_hash_list_to_hyperloglog().unwrap();
                counter
            } else {
                counter.clone()
            }
        };
        let lefts: [Self; M] = core::array::from_fn(|i| materialize(&lefts[i]));
        let rights: [Self; N] = core::array::from_fn(|j| materialize(&rights[j]));

        joint_sketch_mle_from_registers_with::<P, B, R, H, O, M, N>(&lefts, &rights)
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

    phis[0].exp() + phis[1].exp() + phis[2].exp()
}

#[allow(clippy::too_many_lines)]
/// Single-counter cardinality via Ertl's secant-method Maximum Likelihood Estimation.
///
/// # Arguments
/// * `registers` - Iterator over the counter's register values.
/// * `harmonic_sum` - The counter's harmonic sum (sum of `2^-register`).
/// * `is_full` - Whether the counter is saturated (returns infinity).
/// * `error_exponent` - The secant method stops once the relative step is below
///   `10^-error_exponent` scaled by the precision.
fn mle_cardinality<P: Precision, B: Bits>(
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

/// One distinct observed joint register pattern and everything needed to evaluate its
/// per-register log-likelihood contribution under the inclusion-exclusion model.
///
/// See `docs/joint_mle_math.md`. Each pattern contributes `count * ln(P_reg)` to the joint
/// log-likelihood, where `P_reg = sum over terms of sign * exp(-sum over (region, c) of
/// e^{phi_region} * c)` and `c = 2^-(P + level)` is the level constant baked in here.
#[cfg(test)]
struct JointPattern {
    /// How many registers exhibit this exact `(left values, right values)` pattern.
    count: f64,
    /// The surviving inclusion-exclusion terms. Each is `(sign, [(region index, 2^-(P+level))])`.
    /// A term with a region at the saturation cap drops that region (survival 1, constant 0), and
    /// a term that would knock a zero-valued counter below zero is dropped entirely.
    terms: Vec<(f64, Vec<(usize, f64)>)>,
}

/// Builds the `2^(M+N)` inclusion-exclusion terms for a single observed register pattern
/// `(a_pat, b_pat)` (sorted left/right values). This is the exact-but-exponential reference
/// evaluation. Each term is `(sign, [(region index, 2^-(P+level))])`; a region at the saturation
/// cap is dropped (survival 1), and a term knocking a zero-valued counter below zero is dropped.
#[cfg(test)]
fn build_pattern_terms<const M: usize, const N: usize>(
    a_pat: &[u8; M],
    b_pat: &[u8; N],
    p_exponent: u8,
    q_plus_one: u8,
) -> Vec<(f64, Vec<(usize, f64)>)> {
    let n_overlap = M * N;
    let mut terms = Vec::new();
    // Inclusion-exclusion over per-counter unit knockdowns: u over the M lefts, v over the N
    // rights. Bit set means that counter is pushed one level down.
    for u in 0u32..(1u32 << M) {
        for v in 0u32..(1u32 << N) {
            // Adjusted (knocked-down) observed values; a knockdown below zero kills the term.
            let mut al = [0i16; M];
            let mut killed = false;
            for i in 0..M {
                let adjusted = i16::from(a_pat[i]) - i16::from((u >> i) & 1 == 1);
                if adjusted < 0 {
                    killed = true;
                }
                al[i] = adjusted;
            }
            let mut ar = [0i16; N];
            for j in 0..N {
                let adjusted = i16::from(b_pat[j]) - i16::from((v >> j) & 1 == 1);
                if adjusted < 0 {
                    killed = true;
                }
                ar[j] = adjusted;
            }
            if killed {
                continue;
            }

            let sign = if (u.count_ones() + v.count_ones()) % 2 == 0 {
                1.0
            } else {
                -1.0
            };

            // A region's level is the minimum adjusted value over ALL counters that contain it.
            // The overlap cell `O_ij` is contained in the left counters `i..M-1` and the right
            // counters `j..N-1`, the left margin `D^A_i` in the left counters `i..M-1`, and the
            // right margin `D^B_j` in the right counters `j..N-1`. After a knockdown the adjusted
            // values are not necessarily monotone, so we take the suffix minima explicitly rather
            // than assuming the lowest index binds.
            let mut suffix_min_al = [0i16; M];
            let mut running = i16::MAX;
            for i in (0..M).rev() {
                running = running.min(al[i]);
                suffix_min_al[i] = running;
            }
            let mut suffix_min_ar = [0i16; N];
            running = i16::MAX;
            for j in (0..N).rev() {
                running = running.min(ar[j]);
                suffix_min_ar[j] = running;
            }

            // A region at the saturation cap contributes survival 1 (skipped).
            let mut regions: Vec<(usize, f64)> = Vec::new();
            let mut push_region = |idx: usize, level: i16| {
                if level < i16::from(q_plus_one) {
                    let c = f64::integer_exp2_minus(p_exponent + level as u8);
                    regions.push((idx, c));
                }
            };
            for i in 0..M {
                for j in 0..N {
                    push_region(i * N + j, suffix_min_al[i].min(suffix_min_ar[j]));
                }
            }
            for i in 0..M {
                push_region(n_overlap + i, suffix_min_al[i]);
            }
            for j in 0..N {
                push_region(n_overlap + M + j, suffix_min_ar[j]);
            }

            terms.push((sign, regions));
        }
    }
    terms
}

/// Tabulates the distinct joint register value patterns and their multiplicities (the cheap part
/// of pattern accounting, shared by the polynomial and reference paths). Nesting is enforced by a
/// cumulative max along each chain so the observed values are monotone.
///
/// `K = M*N + M + N` is the number of disjoint regions, indexed as: overlap `O_ij` at `i*N + j`,
/// left margin `D^A_i` at `M*N + i`, right margin `D^B_j` at `M*N + M + j`.
fn tabulate_joint_value_patterns<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
    const M: usize,
    const N: usize,
>(
    lefts: &[HyperLogLog<P, B, R, H>; M],
    rights: &[HyperLogLog<P, B, R, H>; N],
) -> Vec<([u8; M], [u8; N], f64)> {
    use std::collections::HashMap;

    let left_regs: [Vec<u8>; M] =
        core::array::from_fn(|i| lefts[i].registers.iter_registers().collect());
    let right_regs: [Vec<u8>; N] =
        core::array::from_fn(|j| rights[j].registers.iter_registers().collect());

    let m_registers = 1_usize << P::EXPONENT;
    let mut counts: HashMap<([u8; M], [u8; N]), f64> = HashMap::new();
    for r in 0..m_registers {
        let mut a_pat = [0u8; M];
        let mut acc = 0u8;
        for i in 0..M {
            acc = acc.max(left_regs[i][r]);
            a_pat[i] = acc;
        }
        let mut b_pat = [0u8; N];
        acc = 0u8;
        for j in 0..N {
            acc = acc.max(right_regs[j][r]);
            b_pat[j] = acc;
        }
        *counts.entry((a_pat, b_pat)).or_insert(0.0) += 1.0;
    }

    counts
        .into_iter()
        .map(|((a_pat, b_pat), count)| (a_pat, b_pat, count))
        .collect()
}

/// Tabulates the distinct joint register patterns and precomputes their inclusion-exclusion terms.
/// This is the exact-but-exponential `2^(M+N)` reference path, retained as the oracle that the
/// polynomial path is validated against.
#[cfg(test)]
fn tabulate_joint_patterns<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
    const M: usize,
    const N: usize,
>(
    lefts: &[HyperLogLog<P, B, R, H>; M],
    rights: &[HyperLogLog<P, B, R, H>; N],
) -> Vec<JointPattern> {
    // q_plus_one is the saturation value; levels above q contribute survival 1 (constant 0).
    let q_plus_one: u8 = (1 << B::NUMBER_OF_BITS) - 1;
    tabulate_joint_value_patterns::<P, B, R, H, M, N>(lefts, rights)
        .into_iter()
        .map(|(a_pat, b_pat, count)| JointPattern {
            count,
            terms: build_pattern_terms::<M, N>(&a_pat, &b_pat, P::EXPONENT, q_plus_one),
        })
        .collect()
}

/// Generalized joint MLE over the disjoint-region model, assuming all counters are in register
/// mode. Returns `(overlap[M][N], left_diff[M], right_diff[N])`. Uses the polynomial per-pattern
/// log-likelihood gradient and the default optimizer `Chain<Adam, Lbfgs>`: an Adam warmup (whose
/// momentum escapes poor local optima) followed by L-BFGS for fast final convergence. In the
/// `experiment_optimizers` comparison this matches a long Adam run's accuracy while being several
/// times faster, and beats plain L-BFGS on accuracy.
fn joint_sketch_mle_from_registers<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
    const M: usize,
    const N: usize,
>(
    lefts: &[HyperLogLog<P, B, R, H>; M],
    rights: &[HyperLogLog<P, B, R, H>; N],
) -> ([[f64; N]; M], [f64; M], [f64; N]) {
    joint_sketch_mle_from_registers_with::<P, B, R, H, Chain<Adam, Lbfgs>, M, N>(lefts, rights)
}

/// Generalized joint MLE with a caller-chosen optimizer type (compile-time generic composition). The
/// default path [`joint_sketch_mle_from_registers`] uses `Chain<Adam, Lbfgs>`.
fn joint_sketch_mle_from_registers_with<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
    O: JointOptimizer,
    const M: usize,
    const N: usize,
>(
    lefts: &[HyperLogLog<P, B, R, H>; M],
    rights: &[HyperLogLog<P, B, R, H>; N],
) -> ([[f64; N]; M], [f64; M], [f64; N]) {
    let value_patterns = tabulate_joint_value_patterns::<P, B, R, H, M, N>(lefts, rights);
    let p_exponent = P::EXPONENT;
    let q_plus_one: u8 = (1 << B::NUMBER_OF_BITS) - 1;

    joint_sketch_mle_core::<P, B, R, H, O, M, N>(lefts, rights, |phis, gradient| {
        let ephi: Vec<f64> = phis.iter().map(|phi| phi.exp()).collect();
        let mut log_likelihood = f64::ZERO;
        for (a_pat, b_pat, count) in &value_patterns {
            log_likelihood += joint_pattern_ll_and_gradient_poly::<M, N>(
                a_pat, b_pat, &ephi, p_exponent, q_plus_one, *count, gradient,
            );
        }
        log_likelihood
    })
}

/// Runs the warm-started, marginal-anchored optimization of the disjoint-region model with the
/// chosen `optimizer` and returns the cell matrices. The log-likelihood term is supplied by
/// `log_likelihood_gradient`, which receives the current `phis` and a pre-zeroed gradient buffer,
/// returns the log-likelihood value, and accumulates its ascent gradient into the buffer. Production
/// passes the polynomial evaluation; tests pass the exponential `2^(M+N)` oracle for cross-validation.
/// The objective being maximized is the MAP log-posterior (log-likelihood plus the marginal-anchor
/// log-prior).
#[allow(clippy::needless_range_loop)]
fn joint_sketch_mle_core<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    H: HasherType,
    O: JointOptimizer,
    const M: usize,
    const N: usize,
>(
    lefts: &[HyperLogLog<P, B, R, H>; M],
    rights: &[HyperLogLog<P, B, R, H>; N],
    mut log_likelihood_gradient: impl FnMut(&[f64], &mut [f64]) -> f64,
) -> ([[f64; N]; M], [f64; M], [f64; N]) {
    let n_overlap = M * N;
    let k = n_overlap + M + N;

    // Warm start from the pairwise hypersphere sketch: its differential overlaps and margin
    // differences are exactly the disjoint regions we optimize.
    let (overlap0, left0, right0) =
        <HyperLogLog<P, B, R, H> as HyperSpheresSketch<f64>>::overlap_and_differences_cardinality_matrices(
            lefts, rights,
        );

    let mut phis = vec![f64::ZERO; k];
    for i in 0..M {
        for j in 0..N {
            phis[i * N + j] = overlap0[i][j].max(f64::EPSILON).ln();
        }
    }
    for i in 0..M {
        phis[n_overlap + i] = left0[i].max(f64::EPSILON).ln();
    }
    for j in 0..N {
        phis[n_overlap + M + j] = right0[j].max(f64::EPSILON).ln();
    }

    // Marginal anchors. The deep overlap cells (contained only in the largest counters) are weakly
    // identified at high load: `x = n * 2^-(P + level)` is negligible at the high register levels
    // those counters reach, so the register likelihood barely constrains them and the free MLE
    // inflates them. We anchor each counter's cumulative cardinality to its HyperLogLog++ estimate
    // (the most reliable single-counter estimate) with a Gaussian log-space prior, which pins the
    // cell sums at every nesting level while the register likelihood still distributes mass among
    // the cells. See docs/joint_mle_math.md.
    let mut anchors: Vec<(Vec<usize>, f64, f64)> = Vec::with_capacity(M + N);
    // A HyperLogLog++ relative error of ~1.04/sqrt(m) corresponds, in log-space, to a Gaussian of
    // that standard deviation, hence a precision (weight) of m / 1.04^2.
    let m_registers = f64::integer_exp2(P::EXPONENT);
    let anchor_weight = m_registers / 1.04_f64.powi(2);
    for i in 0..M {
        let mut regions = Vec::new();
        for ii in 0..=i {
            for j in 0..N {
                regions.push(ii * N + j);
            }
            regions.push(n_overlap + ii);
        }
        anchors.push((
            regions,
            lefts[i].estimate_cardinality().max(f64::EPSILON).ln(),
            anchor_weight,
        ));
    }
    for j in 0..N {
        let mut regions = Vec::new();
        for jj in 0..=j {
            for i in 0..M {
                regions.push(i * N + jj);
            }
            regions.push(n_overlap + M + jj);
        }
        anchors.push((
            regions,
            rights[j].estimate_cardinality().max(f64::EPSILON).ln(),
            anchor_weight,
        ));
    }

    // The expected statistical error scales like 1/sqrt(m), so the optimizer stops once the
    // parameter step falls below that scale (the convergence threshold Ertl uses for the 2-set
    // joint MLE).
    let step_tolerance = 10.0_f64.powi(-2) / f64::integer_exp2(P::EXPONENT).sqrt();

    // The MAP objective: log-likelihood plus the marginal-anchor log-prior, with its ascent gradient.
    let mut objective = |phis: &[f64], gradient: &mut [f64]| {
        let log_likelihood = log_likelihood_gradient(phis, gradient);
        add_marginal_anchor_gradient(&anchors, phis, gradient);
        let mut log_prior = f64::ZERO;
        for (regions, log_estimate, weight) in &anchors {
            let sum: f64 = regions.iter().map(|&rho| phis[rho].exp()).sum();
            let residual = sum.max(f64::EPSILON).ln() - log_estimate;
            log_prior -= 0.5 * weight * residual * residual;
        }
        log_likelihood + log_prior
    };
    let phis = O::maximize(phis, objective, step_tolerance);

    let mut overlap = [[f64::ZERO; N]; M];
    for i in 0..M {
        for j in 0..N {
            overlap[i][j] = phis[i * N + j].exp();
        }
    }
    let mut left_diff = [f64::ZERO; M];
    for i in 0..M {
        left_diff[i] = phis[n_overlap + i].exp();
    }
    let mut right_diff = [f64::ZERO; N];
    for j in 0..N {
        right_diff[j] = phis[n_overlap + M + j].exp();
    }

    (overlap, left_diff, right_diff)
}

/// Evaluates the joint log-likelihood and its exact gradient at `phis` (log-space region
/// cardinalities) over the tabulated register patterns. `k = M*N + M + N` is the region count.
///
/// Forward-mode differentiation: each per-register inclusion-exclusion term is log-linear in the
/// `phis`, so `d/dphi_rho` of `sign * exp(-sum_x)` is `-x_rho * sign * exp(-sum_x)`. The
/// log-likelihood gradient follows by the quotient `d ln P_reg = dP_reg / P_reg`. See
/// `docs/joint_mle_math.md`, section 5. Retained as the test-only oracle for the polynomial path.
#[cfg(test)]
fn joint_ll_and_gradient(patterns: &[JointPattern], phis: &[f64], k: usize) -> (f64, Vec<f64>) {
    let ephi: Vec<f64> = phis.iter().map(|phi| phi.exp()).collect();
    let mut gradient = vec![f64::ZERO; k];
    let mut log_likelihood = f64::ZERO;

    for pattern in patterns {
        let mut p_value = f64::ZERO;
        let mut p_gradient = vec![f64::ZERO; k];
        for (sign, regions) in &pattern.terms {
            let mut sum_x = f64::ZERO;
            for &(rho, c) in regions {
                sum_x += ephi[rho] * c;
            }
            let signed_exp = sign * (-sum_x).exp();
            p_value += signed_exp;
            for &(rho, c) in regions {
                p_gradient[rho] -= signed_exp * ephi[rho] * c;
            }
        }
        let p_value = p_value.max(f64::EPSILON);
        let inverse = 1.0 / p_value;
        log_likelihood += pattern.count * p_value.ln();
        for rho in 0..k {
            gradient[rho] += pattern.count * p_gradient[rho] * inverse;
        }
    }

    (log_likelihood, gradient)
}

/// Polynomial per-pattern log-likelihood: the level-factor / block-collapse evaluation that
/// reproduces [`build_pattern_terms`] exactly in `O((M+N)^2 + M*N)` instead of `O(2^(M+N))`.
///
/// `ephi[rho] = n_rho = exp(phi_rho)`. The likelihood factorizes as `P_reg = exp(base) * prod_w
/// Q_w`: a CDF base (empty-above-ceiling) times one achievement factor per distinct observed value
/// `w`. Each counter block (the contiguous run of counters sharing value `w`) collapses to its
/// smallest index because nesting makes "contained in `A_l`" monotone. See `docs/joint_mle_math.md`.
///
/// The cancellation-free value-only reference (production optimizes via the gradient form below);
/// used by the likelihood and finite-difference cross-checks.
#[cfg(test)]
fn joint_pattern_ll_poly<const M: usize, const N: usize>(
    a_pat: &[u8; M],
    b_pat: &[u8; N],
    ephi: &[f64],
    p_exponent: u8,
    q_plus_one: u8,
) -> f64 {
    let n_overlap = M * N;
    let q = q_plus_one - 1;

    // x_rho at level k, and y_rho(w) = P(region empty at level w). The saturated top bucket
    // (w = q+1) uses level q, matching the existing register model.
    let x = |rho: usize, level: u8| ephi[rho] * f64::integer_exp2_minus(p_exponent + level);
    let y = |rho: usize, w: u8| (-x(rho, w.min(q))).exp();

    // CDF base: -sum over regions of x_rho(ceil_rho). Saturated ceilings (>= q+1) contribute 0.
    let mut ln_p = 0.0_f64;
    for i in 0..M {
        for j in 0..N {
            let ceiling = a_pat[i].min(b_pat[j]);
            if ceiling < q_plus_one {
                ln_p -= x(i * N + j, ceiling);
            }
        }
    }
    for i in 0..M {
        if a_pat[i] < q_plus_one {
            ln_p -= x(n_overlap + i, a_pat[i]);
        }
    }
    for j in 0..N {
        if b_pat[j] < q_plus_one {
            ln_p -= x(n_overlap + M + j, b_pat[j]);
        }
    }

    // Achievement factor Q_w for every value w that some counter attains.
    for w in 1..=q_plus_one {
        let left_p = (0..M).find(|&i| a_pat[i] == w);
        let right_r = (0..N).find(|&j| b_pat[j] == w);
        if left_p.is_none() && right_r.is_none() {
            continue;
        }

        // PL = P(smallest left counter at value w not hit); PR symmetric; PLR = both missed.
        let product_left = |p: usize| {
            let mut product = y(n_overlap + p, w);
            for j in 0..N {
                if b_pat[j] >= w {
                    product *= y(p * N + j, w);
                }
            }
            product
        };
        let product_right = |r: usize| {
            let mut product = y(n_overlap + M + r, w);
            for i in 0..M {
                if a_pat[i] >= w {
                    product *= y(i * N + r, w);
                }
            }
            product
        };

        let q_w = match (left_p, right_r) {
            (Some(p), Some(r)) => {
                let pl = product_left(p);
                let pr = product_right(r);
                // Union product: rows of p, plus column r excluding the shared cell O_{p,r}.
                let mut plr = y(n_overlap + p, w) * y(n_overlap + M + r, w);
                for j in 0..N {
                    if b_pat[j] >= w {
                        plr *= y(p * N + j, w);
                    }
                }
                for i in 0..M {
                    if a_pat[i] >= w && i != p {
                        plr *= y(i * N + r, w);
                    }
                }
                1.0 - pl - pr + plr
            }
            (Some(p), None) => 1.0 - product_left(p),
            (None, Some(r)) => 1.0 - product_right(r),
            (None, None) => unreachable!(),
        };

        ln_p += q_w.max(f64::MIN_POSITIVE).ln();
    }

    ln_p
}

/// Polynomial per-pattern log-likelihood and its exact gradient. Accumulates `count * d ln P_reg /
/// d phi_rho` into `gradient` and returns `count * ln P_reg`. The gradient is the closed-form
/// derivative of the level-factor form in [`joint_pattern_ll_poly`]: `d base / d phi_rho =
/// -x_rho(ceil_rho)` (one term per region), and `d ln Q_w / d phi_rho = (1/Q_w) * dQ_w` built from
/// `d(prod y)/d phi_rho = prod * (-x_rho(w))` for the regions in that product.
fn joint_pattern_ll_and_gradient_poly<const M: usize, const N: usize>(
    a_pat: &[u8; M],
    b_pat: &[u8; N],
    ephi: &[f64],
    p_exponent: u8,
    q_plus_one: u8,
    count: f64,
    gradient: &mut [f64],
) -> f64 {
    let n_overlap = M * N;
    let q = q_plus_one - 1;

    let x = |rho: usize, level: u8| ephi[rho] * f64::integer_exp2_minus(p_exponent + level);
    let y = |rho: usize, w: u8| (-x(rho, w.min(q))).exp();

    let mut ln_p = 0.0_f64;

    // CDF base and its gradient: each region contributes -x_rho(ceil_rho) to both.
    let mut base_region = |rho: usize, ceiling: u8| {
        if ceiling < q_plus_one {
            let xv = x(rho, ceiling);
            ln_p -= xv;
            gradient[rho] += count * (-xv);
        }
    };
    for i in 0..M {
        for j in 0..N {
            base_region(i * N + j, a_pat[i].min(b_pat[j]));
        }
    }
    for i in 0..M {
        base_region(n_overlap + i, a_pat[i]);
    }
    for j in 0..N {
        base_region(n_overlap + M + j, b_pat[j]);
    }

    // Achievement factors and their gradients.
    for w in 1..=q_plus_one {
        let left_p = (0..M).find(|&i| a_pat[i] == w);
        let right_r = (0..N).find(|&j| b_pat[j] == w);
        if left_p.is_none() && right_r.is_none() {
            continue;
        }

        // Hitter region indices for the smallest left/right counters at value w.
        let left_hitters = |p: usize| -> Vec<usize> {
            let mut hitters = vec![n_overlap + p];
            for j in 0..N {
                if b_pat[j] >= w {
                    hitters.push(p * N + j);
                }
            }
            hitters
        };
        let right_hitters = |r: usize| -> Vec<usize> {
            let mut hitters = vec![n_overlap + M + r];
            for i in 0..M {
                if a_pat[i] >= w {
                    hitters.push(i * N + r);
                }
            }
            hitters
        };

        let l_hit = left_p.map(left_hitters).unwrap_or_default();
        let r_hit = right_r.map(right_hitters).unwrap_or_default();
        let pl: f64 = l_hit.iter().map(|&rho| y(rho, w)).product();
        let pr: f64 = r_hit.iter().map(|&rho| y(rho, w)).product();
        let mut plr = pl;
        for &rho in &r_hit {
            if !l_hit.contains(&rho) {
                plr *= y(rho, w);
            }
        }

        let both = left_p.is_some() && right_r.is_some();
        let q_w = if both {
            1.0 - pl - pr + plr
        } else if left_p.is_some() {
            1.0 - pl
        } else {
            1.0 - pr
        };
        ln_p += q_w.max(f64::MIN_POSITIVE).ln();

        // dQ_w/dphi_rho = [in L]*PL + [in R]*PR - [in L or R]*PLR, times x_rho(w); then /Q_w.
        let inverse = count / q_w.max(f64::MIN_POSITIVE);
        let mut accumulate = |rho: usize| {
            let in_l = l_hit.contains(&rho);
            let in_r = r_hit.contains(&rho);
            let coefficient = if both {
                f64::from(u8::from(in_l)) * pl + f64::from(u8::from(in_r)) * pr - plr
            } else if left_p.is_some() {
                pl
            } else {
                pr
            };
            gradient[rho] += inverse * coefficient * x(rho, w.min(q));
        };
        for &rho in &l_hit {
            accumulate(rho);
        }
        for &rho in &r_hit {
            if !l_hit.contains(&rho) {
                accumulate(rho);
            }
        }
    }

    count * ln_p
}

/// Adds the gradient of the marginal-anchor log-prior to `gradient` (which already holds the
/// log-likelihood gradient), forming the gradient of the MAP objective being maximized.
///
/// Each anchor is `(region indices summing to a counter, ln of that counter's HLL++ estimate,
/// weight)`. The prior is `-(weight/2) * (ln(sum n_rho) - ln(estimate))^2`, whose derivative with
/// respect to `phi_rho` (for `rho` in the counter) is `-weight * (ln S - ln estimate) * n_rho / S`,
/// with `S = sum over the counter of n_rho` and `n_rho = e^{phi_rho}`.
fn add_marginal_anchor_gradient(
    anchors: &[(Vec<usize>, f64, f64)],
    phis: &[f64],
    gradient: &mut [f64],
) {
    for (regions, log_estimate, weight) in anchors {
        let sum: f64 = regions.iter().map(|&rho| phis[rho].exp()).sum();
        let residual = sum.max(f64::EPSILON).ln() - log_estimate;
        let factor = -weight * residual / sum.max(f64::EPSILON);
        for &rho in regions {
            gradient[rho] += factor * phis[rho].exp();
        }
    }
}

/// A maximizer of a smooth objective, used to refine the joint-MLE warm start. The optimizer is
/// chosen at compile time by type (`O::maximize(..)`), not as a runtime value: each implementor is a
/// zero-sized marker with its hyperparameters as fixed `const`s. The `objective` closure returns the
/// value to MAXIMIZE and fills its ascent gradient into a pre-zeroed buffer; `step_tolerance` is the
/// convergence scale (the expected statistical error, `~1/sqrt(m)`).
pub trait JointOptimizer {
    /// Maximizes `objective` starting from `init`, returning the best point found.
    fn maximize<F: FnMut(&[f64], &mut [f64]) -> f64>(
        init: Vec<f64>,
        objective: F,
        step_tolerance: f64,
    ) -> Vec<f64>;
}

/// Limited-memory BFGS (the quasi-Newton method Ertl uses for the 2-set joint MLE). Converges in
/// tens of iterations from a good warm start and self-terminates on the step size, but as a greedy
/// descent method it converges to the nearest local optimum, which on multi-modal instances can be
/// worse than the optimum a momentum method reaches. Cheap, so it pairs well as the polishing stage
/// of a [`Chain`].
pub struct Lbfgs;

impl Lbfgs {
    /// Number of `(s, y)` correction pairs retained.
    const MEMORY: usize = 8;
    /// Hard iteration cap (a backstop; convergence is normally by step size).
    const MAX_ITERATIONS: usize = 1000;
}

impl JointOptimizer for Lbfgs {
    fn maximize<F: FnMut(&[f64], &mut [f64]) -> f64>(
        mut x: Vec<f64>,
        mut objective: F,
        step_tolerance: f64,
    ) -> Vec<f64> {
        let n = x.len();
        let memory = Self::MEMORY;
        let max_iterations = Self::MAX_ITERATIONS;
        let dot = |a: &[f64], b: &[f64]| -> f64 { a.iter().zip(b).map(|(u, v)| u * v).sum() };

        // `objective` accumulates into a pre-zeroed buffer, so we clear before every evaluation.
        // We minimize `f = -objective`, so `gradient` below is the gradient of `f`.
        let mut gradient = vec![f64::ZERO; n];
        let mut f_value = -objective(&x, &mut gradient);
        for g in &mut gradient {
            *g = -*g;
        }

        let mut s_history: Vec<Vec<f64>> = Vec::new();
        let mut y_history: Vec<Vec<f64>> = Vec::new();
        let mut rho_history: Vec<f64> = Vec::new();

        let mut ascent_gradient = vec![f64::ZERO; n];

        for _ in 0..max_iterations {
            // Two-loop recursion: direction = -H * gradient, with H the implicit inverse Hessian.
            let mut q = gradient.clone();
            let mut alphas = vec![f64::ZERO; s_history.len()];
            for i in (0..s_history.len()).rev() {
                let alpha = rho_history[i] * dot(&s_history[i], &q);
                alphas[i] = alpha;
                for j in 0..n {
                    q[j] -= alpha * y_history[i][j];
                }
            }
            let gamma = if let Some(last) = s_history.len().checked_sub(1) {
                let yy = dot(&y_history[last], &y_history[last]).max(f64::EPSILON);
                dot(&s_history[last], &y_history[last]) / yy
            } else {
                1.0
            };
            for q_value in &mut q {
                *q_value *= gamma;
            }
            for i in 0..s_history.len() {
                let beta = rho_history[i] * dot(&y_history[i], &q);
                for j in 0..n {
                    q[j] += (alphas[i] - beta) * s_history[i][j];
                }
            }
            let mut direction: Vec<f64> = q.iter().map(|v| -v).collect();

            // Fall back to steepest descent if the quasi-Newton direction is not a descent direction.
            let mut slope = dot(&gradient, &direction);
            if slope >= 0.0 {
                direction = gradient.iter().map(|g| -g).collect();
                slope = dot(&gradient, &direction);
            }

            // Backtracking Armijo line search on `f`.
            let c1 = 1e-4;
            let mut step = 1.0;
            let mut x_new = x.clone();
            let mut f_new = f_value;
            let mut succeeded = false;
            for _ in 0..40 {
                for j in 0..n {
                    x_new[j] = x[j] + step * direction[j];
                }
                for g in &mut ascent_gradient {
                    *g = f64::ZERO;
                }
                f_new = -objective(&x_new, &mut ascent_gradient);
                if f_new.is_finite() && f_new <= f_value + c1 * step * slope {
                    succeeded = true;
                    break;
                }
                step *= 0.5;
            }
            if !succeeded {
                break;
            }

            let mut step_inf_norm = f64::ZERO;
            let mut s = vec![f64::ZERO; n];
            let mut y = vec![f64::ZERO; n];
            for j in 0..n {
                s[j] = x_new[j] - x[j];
                // ascent_gradient holds the ascent gradient at x_new; negate for f.
                y[j] = -ascent_gradient[j] - gradient[j];
                step_inf_norm = step_inf_norm.max(s[j].abs());
                x[j] = x_new[j];
                gradient[j] = -ascent_gradient[j];
            }
            f_value = f_new;

            let curvature = dot(&s, &y);
            if curvature > 1e-10 {
                s_history.push(s);
                y_history.push(y);
                rho_history.push(1.0 / curvature);
                if s_history.len() > memory {
                    s_history.remove(0);
                    y_history.remove(0);
                    rho_history.remove(0);
                }
            }

            if step_inf_norm <= step_tolerance {
                break;
            }
        }

        x
    }
}

/// Adam (adaptive first-order). Its momentum lets it escape poor local optima that a greedy
/// descent method settles into, at the cost of many small steps that never shrink near a flat
/// optimum. Used here for a fixed budget (returning the best point seen), typically as the
/// basin-escaping warmup stage of a [`Chain`].
pub struct Adam;

impl Adam {
    /// Fixed iteration budget (enough to escape poor basins as a [`Chain`] warmup).
    const ITERATIONS: usize = 500;
    /// Fixed step size.
    const LEARNING_RATE: f64 = 0.1;
}

impl JointOptimizer for Adam {
    fn maximize<F: FnMut(&[f64], &mut [f64]) -> f64>(
        mut x: Vec<f64>,
        mut objective: F,
        _step_tolerance: f64,
    ) -> Vec<f64> {
        let n = x.len();
        let (mut first_moment, mut second_moment) = (vec![0.0; n], vec![0.0; n]);
        let mut gradient = vec![0.0; n];
        let (beta1, beta2) = (0.9_f64, 0.999_f64);
        let mut best_value = f64::NEG_INFINITY;
        let mut best_x = x.clone();
        for t in 1..=Self::ITERATIONS as i32 {
            for g in &mut gradient {
                *g = 0.0;
            }
            let value = objective(&x, &mut gradient);
            if value > best_value {
                best_value = value;
                best_x.copy_from_slice(&x);
            }
            let bias = (1.0 - beta2.powi(t)).sqrt() / (1.0 - beta1.powi(t));
            for i in 0..n {
                first_moment[i] = beta1 * first_moment[i] + (1.0 - beta1) * gradient[i];
                second_moment[i] =
                    beta2 * second_moment[i] + (1.0 - beta2) * gradient[i] * gradient[i];
                x[i] += Self::LEARNING_RATE * bias * first_moment[i]
                    / second_moment[i].sqrt().max(f64::EPSILON);
            }
        }
        best_x
    }
}

/// RMSProp (adaptive first-order, no momentum). Cheaper per step than Adam, also basin-escaping
/// in practice. Runs a fixed budget and returns the best point seen. (Dominated by [`Adam`] in the
/// comparison harness; kept for completeness.)
pub struct RmsProp;

impl RmsProp {
    /// Fixed iteration budget.
    const ITERATIONS: usize = 500;
    /// Fixed step size.
    const LEARNING_RATE: f64 = 0.1;
}

impl JointOptimizer for RmsProp {
    fn maximize<F: FnMut(&[f64], &mut [f64]) -> f64>(
        mut x: Vec<f64>,
        mut objective: F,
        _step_tolerance: f64,
    ) -> Vec<f64> {
        let n = x.len();
        let mut mean_square = vec![0.0; n];
        let mut gradient = vec![0.0; n];
        let mut best_value = f64::NEG_INFINITY;
        let mut best_x = x.clone();
        for _ in 0..Self::ITERATIONS {
            for g in &mut gradient {
                *g = 0.0;
            }
            let value = objective(&x, &mut gradient);
            if value > best_value {
                best_value = value;
                best_x.copy_from_slice(&x);
            }
            for i in 0..n {
                mean_square[i] = 0.9 * mean_square[i] + 0.1 * gradient[i] * gradient[i];
                x[i] += Self::LEARNING_RATE * gradient[i] / (mean_square[i].sqrt() + 1e-8);
            }
        }
        best_x
    }
}

/// Sequential composition of two optimizers: run `A` from the initial point, then `B` from where it
/// stopped. `Chain<Adam, Lbfgs>` is the recommended (and default) estimator path: an Adam warmup
/// escapes poor basins, then L-BFGS converges quickly to the optimum within the good basin. A
/// zero-sized type selected purely at compile time, e.g. `joint_sketch_mle_with::<Chain<Adam, Lbfgs>>`.
pub struct Chain<A, B>(core::marker::PhantomData<(A, B)>);

impl<A: JointOptimizer, B: JointOptimizer> JointOptimizer for Chain<A, B> {
    fn maximize<F: FnMut(&[f64], &mut [f64]) -> f64>(
        init: Vec<f64>,
        mut objective: F,
        step_tolerance: f64,
    ) -> Vec<f64> {
        let intermediate = A::maximize(init, &mut objective, step_tolerance);
        B::maximize(intermediate, objective, step_tolerance)
    }
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
struct ArrayAdam<const N: usize> {
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
        let mut adam = ArrayAdam::<2>::default();

        for _ in 0..10_000 {
            let (_value, gradients) = quadratic_function(&phis);
            adam.apply(&mut gradients.clone(), &mut phis);
        }

        assert!((phis[0] - 1.0).abs() < 1e-4);
        assert!((phis[1] + 2.0).abs() < 1e-4);
    }

    /// Every `JointOptimizer` must maximize a smooth concave objective (here a negated quadratic)
    /// to its optimum, mirroring how the joint MLE refines the warm start.
    #[cfg(feature = "mle")]
    #[test]
    fn test_optimizers_maximize_quadratic() {
        // Maximize -((x0-1)^2 + (x1+2)^2 + (x2-3)^2), with maximum at (1, -2, 3).
        fn check<O: JointOptimizer>(tolerance: f64) {
            let eval = |x: &[f64], grad: &mut [f64]| -> f64 {
                grad[0] = -2.0 * (x[0] - 1.0);
                grad[1] = -2.0 * (x[1] + 2.0);
                grad[2] = -2.0 * (x[2] - 3.0);
                -((x[0] - 1.0).powi(2) + (x[1] + 2.0).powi(2) + (x[2] - 3.0).powi(2))
            };
            let result = O::maximize(vec![0.0, 0.0, 0.0], eval, 1e-12);
            assert!((result[0] - 1.0).abs() < tolerance, "{result:?}");
            assert!((result[1] + 2.0).abs() < tolerance, "{result:?}");
            assert!((result[2] - 3.0).abs() < tolerance, "{result:?}");
        }
        check::<Lbfgs>(1e-5);
        check::<Adam>(1e-2);
        check::<RmsProp>(1e-2);
        check::<Chain<Adam, Lbfgs>>(1e-5);
    }

    /// The analytic forward-mode gradient of the joint log-likelihood must match a central
    /// finite-difference estimate at every coordinate, on real tabulated register patterns.
    #[cfg(feature = "mle")]
    #[test]
    fn test_joint_ll_gradient_matches_finite_differences() {
        type Counter =
            HyperLogLog<
                crate::prelude::Precision8,
                crate::prelude::Bits6,
                <crate::prelude::Precision8 as crate::prelude::PackedRegister<
                    crate::prelude::Bits6,
                >>::Array,
                twox_hash::XxHash64,
            >;

        let insert = |hll: &mut Counter, start: u64, count: u64| {
            for v in start..start + count {
                hll.insert(&v);
            }
        };
        let mut a0 = Counter::default();
        insert(&mut a0, 0, 4_000);
        insert(&mut a0, 4_000, 2_500);
        insert(&mut a0, 10_000, 2_200);
        let mut a1 = a0.clone();
        insert(&mut a1, 12_000, 1_800);
        insert(&mut a1, 14_000, 3_000);
        insert(&mut a1, 20_000, 1_500);
        let mut b0 = Counter::default();
        insert(&mut b0, 0, 4_000);
        insert(&mut b0, 12_000, 1_800);
        insert(&mut b0, 30_000, 2_000);
        let mut b1 = b0.clone();
        insert(&mut b1, 4_000, 2_500);
        insert(&mut b1, 14_000, 3_000);
        insert(&mut b1, 40_000, 2_800);

        let patterns = tabulate_joint_patterns::<_, _, _, _, 2, 2>(&[a0, a1], &[b0, b1]);
        let k = 2 * 2 + 2 + 2;

        // A non-trivial, non-degenerate evaluation point.
        let phis: Vec<f64> = (0..k)
            .map(|i| 8.0 + 0.3 * (i as f64) - 0.05 * (i * i) as f64)
            .collect();

        let (_ll, grad) = joint_ll_and_gradient(&patterns, &phis, k);

        let h = 1e-5;
        for rho in 0..k {
            let mut plus = phis.clone();
            let mut minus = phis.clone();
            plus[rho] += h;
            minus[rho] -= h;
            let (ll_plus, _) = joint_ll_and_gradient(&patterns, &plus, k);
            let (ll_minus, _) = joint_ll_and_gradient(&patterns, &minus, k);
            let fd = (ll_plus - ll_minus) / (2.0 * h);
            let scale = grad[rho].abs().max(fd.abs()).max(1.0);
            assert!(
                (grad[rho] - fd).abs() / scale < 1e-3,
                "gradient[{rho}] = {} but finite difference = {fd}",
                grad[rho],
            );
        }
    }

    /// Evaluates `ln P_reg` for one pattern via the exponential `2^(M+N)` reference (oracle).
    #[cfg(feature = "mle")]
    fn oracle_pattern_ln_p_reg<const M: usize, const N: usize>(
        a_pat: &[u8; M],
        b_pat: &[u8; N],
        ephi: &[f64],
        p_exponent: u8,
        q_plus_one: u8,
    ) -> f64 {
        let terms = build_pattern_terms::<M, N>(a_pat, b_pat, p_exponent, q_plus_one);
        let mut p = 0.0_f64;
        for (sign, regions) in &terms {
            let mut sum_x = 0.0;
            for &(rho, c) in regions {
                sum_x += ephi[rho] * c;
            }
            p += sign * (-sum_x).exp();
        }
        p.max(f64::EPSILON).ln()
    }

    /// Draws a sorted (monotone) register pattern of length `L` with values in `0..=q_plus_one`.
    #[cfg(feature = "mle")]
    fn random_monotone_pattern<const L: usize>(state: &mut u64, q_plus_one: u8) -> [u8; L] {
        let mut pattern: [u8; L] = core::array::from_fn(|_| {
            *state = splitmix64(*state);
            (*state % (u64::from(q_plus_one) + 1)) as u8
        });
        pattern.sort_unstable();
        pattern
    }

    /// Draws region cardinalities `n_rho = exp(phi)` with `phi` uniform in `[ln 2, ln 40]`, the
    /// regime where the signed `2^(M+N)` oracle is numerically reliable (matching the Python
    /// cross-check). The polynomial form is cancellation-free at any scale.
    #[cfg(feature = "mle")]
    fn random_ephi(state: &mut u64, k: usize) -> Vec<f64> {
        (0..k)
            .map(|_| {
                *state = splitmix64(*state);
                let u = (*state >> 11) as f64 / (1u64 << 53) as f64;
                (2.0_f64.ln() + (40.0_f64.ln() - 2.0_f64.ln()) * u).exp()
            })
            .collect()
    }

    /// The polynomial per-pattern log-likelihood must match the exponential `2^(M+N)` oracle to
    /// floating-point tolerance across random monotone patterns (covering ties, zeros, saturation)
    /// and random region cardinalities, for several `(M, N)` including rectangular shapes.
    #[cfg(feature = "mle")]
    fn check_poly_likelihood_matches_oracle<const M: usize, const N: usize>(
        seed: u64,
        p_exponent: u8,
        q_plus_one: u8,
    ) {
        let k = M * N + M + N;
        let mut state = seed;
        let mut tested = 0;
        for _ in 0..1500 {
            let a_pat = random_monotone_pattern::<M>(&mut state, q_plus_one);
            let b_pat = random_monotone_pattern::<N>(&mut state, q_plus_one);
            let ephi = random_ephi(&mut state, k);

            let oracle =
                oracle_pattern_ln_p_reg::<M, N>(&a_pat, &b_pat, &ephi, p_exponent, q_plus_one);
            // Skip patterns where the signed oracle sum loses precision to catastrophic
            // cancellation (small P_reg from tiny x at high levels); the polynomial form is
            // cancellation-free, so the oracle is the limiting factor here, not the poly.
            if oracle < -18.0 {
                continue;
            }
            let poly = joint_pattern_ll_poly::<M, N>(&a_pat, &b_pat, &ephi, p_exponent, q_plus_one);
            assert!(
                (oracle - poly).abs() < 1e-7,
                "M={M} N={N} a={a_pat:?} b={b_pat:?}: oracle ln={oracle} poly ln={poly}"
            );
            tested += 1;
        }
        assert!(tested > 20, "too few non-degenerate trials: {tested}");
    }

    #[cfg(feature = "mle")]
    #[test]
    fn test_poly_pattern_likelihood_matches_oracle() {
        // Small p_exponent keeps x in a range where the signed oracle is numerically reliable; the
        // per-pattern algorithm is independent of p_exponent (it only scales x), so this fully
        // validates correctness. q_plus_one = 7 exercises the saturation boundary.
        for &q_plus_one in &[7u8, 15u8] {
            let p = 4u8;
            check_poly_likelihood_matches_oracle::<1, 1>(0x1111, p, q_plus_one);
            check_poly_likelihood_matches_oracle::<2, 2>(0x2222, p, q_plus_one);
            check_poly_likelihood_matches_oracle::<3, 2>(0x3232, p, q_plus_one);
            check_poly_likelihood_matches_oracle::<2, 3>(0x2323, p, q_plus_one);
            check_poly_likelihood_matches_oracle::<3, 3>(0x3333, p, q_plus_one);
        }

        // Explicit edge cases: all-zero, all-saturated, all-equal mid, a single zero.
        let p = 4u8;
        let q1 = 7u8;
        let k = 2 * 2 + 2 + 2;
        let ephi: Vec<f64> = (0..k).map(|i| (1.0 + 0.5 * i as f64).exp()).collect();
        for (a, b) in [
            ([0u8, 0], [0u8, 0]),
            ([7, 7], [7, 7]),
            ([3, 3], [3, 3]),
            ([0, 3], [0, 5]),
            ([0, 7], [2, 7]),
        ] {
            let oracle = oracle_pattern_ln_p_reg::<2, 2>(&a, &b, &ephi, p, q1);
            let poly = joint_pattern_ll_poly::<2, 2>(&a, &b, &ephi, p, q1);
            assert!(
                (oracle - poly).abs() < 1e-7,
                "edge a={a:?} b={b:?}: oracle={oracle} poly={poly}"
            );
        }
    }

    /// Gradient of `ln P_reg` for one pattern via the exponential `2^(M+N)` oracle.
    #[cfg(feature = "mle")]
    fn oracle_pattern_gradient<const M: usize, const N: usize>(
        a_pat: &[u8; M],
        b_pat: &[u8; N],
        phis: &[f64],
        p_exponent: u8,
        q_plus_one: u8,
        k: usize,
    ) -> Vec<f64> {
        let pattern = JointPattern {
            count: 1.0,
            terms: build_pattern_terms::<M, N>(a_pat, b_pat, p_exponent, q_plus_one),
        };
        joint_ll_and_gradient(&[pattern], phis, k).1
    }

    /// The polynomial per-pattern gradient must match both a central finite difference of the
    /// polynomial likelihood and the exponential oracle gradient, across random patterns.
    #[cfg(feature = "mle")]
    fn check_poly_gradient_matches_oracle_and_fd<const M: usize, const N: usize>(
        seed: u64,
        p_exponent: u8,
        q_plus_one: u8,
    ) {
        let k = M * N + M + N;
        let mut state = seed;
        let mut tested = 0;
        for _ in 0..1500 {
            let a_pat = random_monotone_pattern::<M>(&mut state, q_plus_one);
            let b_pat = random_monotone_pattern::<N>(&mut state, q_plus_one);
            let ephi = random_ephi(&mut state, k);
            let phis: Vec<f64> = ephi.iter().map(|e| e.ln()).collect();

            let ll = joint_pattern_ll_poly::<M, N>(&a_pat, &b_pat, &ephi, p_exponent, q_plus_one);
            if ll < -18.0 {
                continue;
            }

            let mut grad = vec![0.0_f64; k];
            let ret = joint_pattern_ll_and_gradient_poly::<M, N>(
                &a_pat, &b_pat, &ephi, p_exponent, q_plus_one, 1.0, &mut grad,
            );
            assert!(
                (ret - ll).abs() < 1e-9,
                "returned ll {ret} != value-fn ll {ll}"
            );

            // Finite-difference cross-check (perturb in phi-space).
            let h = 1e-6_f64;
            for rho in 0..k {
                let mut ep = ephi.clone();
                let mut em = ephi.clone();
                ep[rho] = ephi[rho] * h.exp();
                em[rho] = ephi[rho] * (-h).exp();
                let lp = joint_pattern_ll_poly::<M, N>(&a_pat, &b_pat, &ep, p_exponent, q_plus_one);
                let lm = joint_pattern_ll_poly::<M, N>(&a_pat, &b_pat, &em, p_exponent, q_plus_one);
                let fd = (lp - lm) / (2.0 * h);
                let scale = grad[rho].abs().max(fd.abs()).max(1.0);
                assert!(
                    (grad[rho] - fd).abs() / scale < 1e-4,
                    "FD mismatch M={M} N={N} a={a_pat:?} b={b_pat:?} rho={rho}: grad={} fd={fd}",
                    grad[rho]
                );
            }

            // Oracle gradient cross-check.
            let ograd =
                oracle_pattern_gradient::<M, N>(&a_pat, &b_pat, &phis, p_exponent, q_plus_one, k);
            for rho in 0..k {
                let scale = grad[rho].abs().max(ograd[rho].abs()).max(1.0);
                assert!(
                    (grad[rho] - ograd[rho]).abs() / scale < 1e-6,
                    "oracle mismatch M={M} N={N} a={a_pat:?} b={b_pat:?} rho={rho}: poly={} oracle={}",
                    grad[rho],
                    ograd[rho]
                );
            }
            tested += 1;
        }
        assert!(tested > 20, "too few non-degenerate trials: {tested}");
    }

    #[cfg(feature = "mle")]
    #[test]
    fn test_poly_pattern_gradient_matches_oracle_and_fd() {
        for &q_plus_one in &[7u8, 15u8] {
            let p = 4u8;
            check_poly_gradient_matches_oracle_and_fd::<1, 1>(0xA1A1, p, q_plus_one);
            check_poly_gradient_matches_oracle_and_fd::<2, 2>(0xB2B2, p, q_plus_one);
            check_poly_gradient_matches_oracle_and_fd::<3, 2>(0xC3C2, p, q_plus_one);
            check_poly_gradient_matches_oracle_and_fd::<2, 3>(0xD2D3, p, q_plus_one);
            check_poly_gradient_matches_oracle_and_fd::<3, 3>(0xE3E3, p, q_plus_one);
        }
    }

    /// The full estimator driven by the polynomial gradient must converge to the same cell matrices
    /// as the same optimization driven by the exponential `2^(M+N)` oracle gradient, confirming the
    /// production rewrite is faithful end to end.
    #[cfg(feature = "mle")]
    fn check_full_estimator_poly_vs_oracle<const M: usize, const N: usize>(unit: u64) {
        type Counter = HyperLogLog<
            crate::prelude::Precision10,
            crate::prelude::Bits6,
            <crate::prelude::Precision10 as crate::prelude::PackedRegister<
                crate::prelude::Bits6,
            >>::Array,
            twox_hash::XxHash64,
        >;
        let build = |ranges: &[(u64, u64)]| -> Counter {
            let mut hll = Counter::default();
            for &(start, count) in ranges {
                for v in start..start + count {
                    hll.insert(&v);
                }
            }
            hll
        };
        // Disjoint integer ranges, one per region; nested counters built from them.
        let mut cursor = 0u64;
        let mut ranges_o = [[(0u64, 0u64); N]; M];
        for i in 0..M {
            for j in 0..N {
                let count = unit * (2 + ((i * 5 + j * 3) % 4) as u64);
                ranges_o[i][j] = (cursor, count);
                cursor += count;
            }
        }
        let mut ranges_da = [(0u64, 0u64); M];
        for i in 0..M {
            let count = unit * (1 + (i % 2) as u64);
            ranges_da[i] = (cursor, count);
            cursor += count;
        }
        let mut ranges_db = [(0u64, 0u64); N];
        for j in 0..N {
            let count = unit * (1 + (j % 3) as u64);
            ranges_db[j] = (cursor, count);
            cursor += count;
        }
        let lefts: [Counter; M] = core::array::from_fn(|i| {
            let mut ranges = Vec::new();
            for ii in 0..=i {
                for j in 0..N {
                    ranges.push(ranges_o[ii][j]);
                }
                ranges.push(ranges_da[ii]);
            }
            build(&ranges)
        });
        let rights: [Counter; N] = core::array::from_fn(|j| {
            let mut ranges = Vec::new();
            for jj in 0..=j {
                for i in 0..M {
                    ranges.push(ranges_o[i][jj]);
                }
                ranges.push(ranges_db[jj]);
            }
            build(&ranges)
        });

        let p_exponent = crate::prelude::Precision10::EXPONENT;
        let q_plus_one: u8 = (1 << crate::prelude::Bits6::NUMBER_OF_BITS) - 1;
        let k = M * N + M + N;

        // Production path (polynomial gradient). Both paths use the same optimizer (Lbfgs) so the
        // comparison isolates the gradient source.
        let value_patterns = tabulate_joint_value_patterns::<_, _, _, _, M, N>(&lefts, &rights);
        let (ov_poly, l_poly, r_poly) =
            joint_sketch_mle_core::<_, _, _, _, Lbfgs, M, N>(&lefts, &rights, |phis, gradient| {
                let ephi: Vec<f64> = phis.iter().map(|phi| phi.exp()).collect();
                let mut log_likelihood = 0.0;
                for (a_pat, b_pat, count) in &value_patterns {
                    log_likelihood += joint_pattern_ll_and_gradient_poly::<M, N>(
                        a_pat, b_pat, &ephi, p_exponent, q_plus_one, *count, gradient,
                    );
                }
                log_likelihood
            });

        // Oracle path (exponential gradient).
        let oracle_patterns = tabulate_joint_patterns::<_, _, _, _, M, N>(&lefts, &rights);
        let (ov_oracle, l_oracle, r_oracle) =
            joint_sketch_mle_core::<_, _, _, _, Lbfgs, M, N>(&lefts, &rights, |phis, gradient| {
                let (ll, g) = joint_ll_and_gradient(&oracle_patterns, phis, k);
                for (slot, value) in gradient.iter_mut().zip(g) {
                    *slot += value;
                }
                ll
            });

        // The per-pattern gradients agree to ~1e-7, but the two paths sum patterns in different
        // (HashMap) orders and the oracle carries ~1e-7 cancellation error, which compound over the
        // optimization along weakly-identified directions. A 0.1% end-to-end agreement still
        // confirms the rewrite is faithful; a real bug would diverge grossly (as the tight
        // per-pattern gradient test would already catch).
        let close = |a: f64, b: f64| (a - b).abs() <= 1e-3 * a.abs().max(b.abs()) + 1.0;
        for i in 0..M {
            for j in 0..N {
                assert!(
                    close(ov_poly[i][j], ov_oracle[i][j]),
                    "M={M} N={N} overlap[{i}][{j}]: poly={} oracle={}",
                    ov_poly[i][j],
                    ov_oracle[i][j]
                );
            }
        }
        for i in 0..M {
            assert!(close(l_poly[i], l_oracle[i]), "M={M} N={N} left[{i}]");
        }
        for j in 0..N {
            assert!(close(r_poly[j], r_oracle[j]), "M={M} N={N} right[{j}]");
        }
    }

    #[cfg(feature = "mle")]
    #[test]
    fn test_poly_joint_sketch_matches_oracle() {
        check_full_estimator_poly_vs_oracle::<1, 1>(20_000);
        check_full_estimator_poly_vs_oracle::<2, 2>(8_000);
        check_full_estimator_poly_vs_oracle::<3, 2>(5_000);
    }

    /// Experiment (ignored by default): compares how fast different optimizers drive the per-cell
    /// error down from the warm start, to see whether the Adam iteration budget can be cut.
    /// Run with: `cargo test --release --features mle --lib experiment_optimizers -- --ignored --nocapture`.
    #[cfg(feature = "mle")]
    fn experiment_optimizers<const M: usize, const N: usize>(unit: u64) {
        type Counter =
            HyperLogLog<
                crate::prelude::Precision8,
                crate::prelude::Bits6,
                <crate::prelude::Precision8 as crate::prelude::PackedRegister<
                    crate::prelude::Bits6,
                >>::Array,
                twox_hash::XxHash64,
            >;
        let n_overlap = M * N;
        let k = n_overlap + M + N;

        // Varied-cell partition (matching joint_matrix_bench), register mode. `exact[rho]` holds
        // each region's true cardinality for the per-cell error.
        let mut exact = vec![0.0_f64; k];
        let mut cursor = 0u64;
        let mut ro = [[(0u64, 0u64); N]; M];
        for i in 0..M {
            for j in 0..N {
                let count = unit * (2 + ((i * 7 + j * 3) % 5) as u64);
                ro[i][j] = (cursor, count);
                cursor += count;
                exact[i * N + j] = count as f64;
            }
        }
        let mut rda = [(0u64, 0u64); M];
        for i in 0..M {
            let count = unit * (1 + (i % 3) as u64);
            rda[i] = (cursor, count);
            cursor += count;
            exact[n_overlap + i] = count as f64;
        }
        let mut rdb = [(0u64, 0u64); N];
        for j in 0..N {
            let count = unit * (1 + (j % 4) as u64);
            rdb[j] = (cursor, count);
            cursor += count;
            exact[n_overlap + M + j] = count as f64;
        }
        let total_union: f64 = exact.iter().sum();
        let build = |ranges: &[(u64, u64)]| -> Counter {
            let mut hll = Counter::default();
            for &(start, count) in ranges {
                for v in start..start + count {
                    hll.insert(&v);
                }
            }
            hll
        };
        let lefts: [Counter; M] = core::array::from_fn(|i| {
            let mut ranges = Vec::new();
            for ii in 0..=i {
                for j in 0..N {
                    ranges.push(ro[ii][j]);
                }
                ranges.push(rda[ii]);
            }
            build(&ranges)
        });
        let rights: [Counter; N] = core::array::from_fn(|j| {
            let mut ranges = Vec::new();
            for jj in 0..=j {
                for i in 0..M {
                    ranges.push(ro[i][jj]);
                }
                ranges.push(rdb[jj]);
            }
            build(&ranges)
        });

        // Warm start and anchors (mirroring joint_sketch_mle_core).
        let (overlap0, left0, right0) =
            <Counter as HyperSpheresSketch<f64>>::overlap_and_differences_cardinality_matrices(
                &lefts, &rights,
            );
        let mut init = vec![0.0; k];
        for i in 0..M {
            for j in 0..N {
                init[i * N + j] = overlap0[i][j].max(f64::EPSILON).ln();
            }
        }
        for i in 0..M {
            init[n_overlap + i] = left0[i].max(f64::EPSILON).ln();
        }
        for j in 0..N {
            init[n_overlap + M + j] = right0[j].max(f64::EPSILON).ln();
        }
        let anchor_weight =
            f64::integer_exp2(crate::prelude::Precision8::EXPONENT) / 1.04_f64.powi(2);
        let mut anchors: Vec<(Vec<usize>, f64, f64)> = Vec::new();
        for i in 0..M {
            let mut regions = Vec::new();
            for ii in 0..=i {
                for j in 0..N {
                    regions.push(ii * N + j);
                }
                regions.push(n_overlap + ii);
            }
            anchors.push((
                regions,
                lefts[i].estimate_cardinality().max(f64::EPSILON).ln(),
                anchor_weight,
            ));
        }
        for j in 0..N {
            let mut regions = Vec::new();
            for jj in 0..=j {
                for i in 0..M {
                    regions.push(i * N + jj);
                }
                regions.push(n_overlap + M + jj);
            }
            anchors.push((
                regions,
                rights[j].estimate_cardinality().max(f64::EPSILON).ln(),
                anchor_weight,
            ));
        }

        let value_patterns = tabulate_joint_value_patterns::<_, _, _, _, M, N>(&lefts, &rights);
        let p_exponent = crate::prelude::Precision8::EXPONENT;
        let q_plus_one: u8 = (1 << crate::prelude::Bits6::NUMBER_OF_BITS) - 1;

        let cell_err = |phis: &[f64]| -> f64 {
            let mut e = 0.0;
            for rho in 0..k {
                e += (phis[rho].exp() - exact[rho]).abs() / total_union;
            }
            e / k as f64
        };
        // The MAP objective being maximized (log-likelihood plus marginal-anchor log-prior), value
        // only (for reporting).
        let objective = |phis: &[f64]| -> f64 {
            let ephi: Vec<f64> = phis.iter().map(|p| p.exp()).collect();
            let mut scratch = vec![0.0; k];
            let mut value = 0.0;
            for (a_pat, b_pat, count) in &value_patterns {
                value += joint_pattern_ll_and_gradient_poly::<M, N>(
                    a_pat,
                    b_pat,
                    &ephi,
                    p_exponent,
                    q_plus_one,
                    *count,
                    &mut scratch,
                );
            }
            for (regions, log_estimate, weight) in &anchors {
                let sum: f64 = regions.iter().map(|&r| ephi[r]).sum();
                let residual = sum.max(f64::EPSILON).ln() - log_estimate;
                value -= 0.5 * weight * residual * residual;
            }
            value
        };
        // The same objective with its ascent gradient (the closure each optimizer drives).
        let mut map_objective = |phis: &[f64], gradient: &mut [f64]| -> f64 {
            let ephi: Vec<f64> = phis.iter().map(|p| p.exp()).collect();
            let mut value = 0.0;
            for (a_pat, b_pat, count) in &value_patterns {
                value += joint_pattern_ll_and_gradient_poly::<M, N>(
                    a_pat, b_pat, &ephi, p_exponent, q_plus_one, *count, gradient,
                );
            }
            add_marginal_anchor_gradient(&anchors, phis, gradient);
            for (regions, log_estimate, weight) in &anchors {
                let sum: f64 = regions.iter().map(|&r| ephi[r]).sum();
                let residual = sum.max(f64::EPSILON).ln() - log_estimate;
                value -= 0.5 * weight * residual * residual;
            }
            value
        };

        let step_tolerance = 1e-2 / (1u64 << 8) as f64;
        println!(
            "\n=== M={M} N={N} P8 unit={unit} (higher obj = better fit; lower cell_err = more accurate) ==="
        );
        // Each optimizer is selected at compile time by type and run explicitly (the trait is no
        // longer object-safe).
        macro_rules! run {
            ($name:expr, $optimizer:ty) => {{
                let start = std::time::Instant::now();
                let result = <$optimizer as JointOptimizer>::maximize(
                    init.clone(),
                    &mut map_objective,
                    step_tolerance,
                );
                let elapsed_ms = start.elapsed().as_secs_f64() * 1e3;
                println!(
                    "{:<20} cell_err={:>6.3}%  obj={:>10.2}  time={:>8.2}ms",
                    $name,
                    100.0 * cell_err(&result),
                    objective(&result),
                    elapsed_ms
                );
            }};
        }
        run!("lbfgs", Lbfgs);
        run!("adam", Adam);
        run!("rmsprop", RmsProp);
        run!("adam+lbfgs", Chain<Adam, Lbfgs>);
        run!("rmsprop+lbfgs", Chain<RmsProp, Lbfgs>);
    }

    #[cfg(feature = "mle")]
    #[test]
    #[ignore]
    fn experiment_optimizers_run() {
        experiment_optimizers::<4, 4>(256);
        experiment_optimizers::<5, 5>(256);
    }
}
