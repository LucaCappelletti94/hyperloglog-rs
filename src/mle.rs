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
struct JointPattern {
    /// How many registers exhibit this exact `(left values, right values)` pattern.
    count: f64,
    /// The surviving inclusion-exclusion terms. Each is `(sign, [(region index, 2^-(P+level))])`.
    /// A term with a region at the saturation cap drops that region (survival 1, constant 0), and
    /// a term that would knock a zero-valued counter below zero is dropped entirely.
    terms: Vec<(f64, Vec<(usize, f64)>)>,
}

/// Tabulates the distinct joint register patterns and precomputes their inclusion-exclusion terms.
///
/// `K = M*N + M + N` is the number of disjoint regions, indexed as: overlap `O_ij` at `i*N + j`,
/// left margin `D^A_i` at `M*N + i`, right margin `D^B_j` at `M*N + M + j`.
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
    use std::collections::HashMap;

    let left_regs: [Vec<u8>; M] =
        core::array::from_fn(|i| lefts[i].registers.iter_registers().collect());
    let right_regs: [Vec<u8>; N] =
        core::array::from_fn(|j| rights[j].registers.iter_registers().collect());

    // q_plus_one is the saturation value; levels above q contribute survival 1 (constant 0).
    let q_plus_one: u8 = (1 << B::NUMBER_OF_BITS) - 1;

    let m_registers = 1_usize << P::EXPONENT;
    let mut counts: HashMap<([u8; M], [u8; N]), f64> = HashMap::new();
    for r in 0..m_registers {
        // Enforce the nesting prior by taking the cumulative max along each chain, so the observed
        // values are monotone even if the inputs are imperfectly nested.
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

    let n_overlap = M * N;
    counts
        .into_iter()
        .map(|((a_pat, b_pat), count)| {
            let mut terms = Vec::new();
            // Inclusion-exclusion over per-counter unit knockdowns: u over the M lefts, v over the
            // N rights. Bit set means that counter is pushed one level down.
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

                    // A region's level is the minimum adjusted value over ALL counters that contain
                    // it. The overlap cell `O_ij` is contained in the left counters `i..M-1` and the
                    // right counters `j..N-1`, the left margin `D^A_i` in the left counters
                    // `i..M-1`, and the right margin `D^B_j` in the right counters `j..N-1`. After a
                    // knockdown the adjusted values are not necessarily monotone, so we take the
                    // suffix minima explicitly rather than assuming the lowest index binds.
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
                            let c = f64::integer_exp2_minus(P::EXPONENT + level as u8);
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
            JointPattern { count, terms }
        })
        .collect()
}

/// Generalized joint MLE over the disjoint-region model, assuming all counters are in register
/// mode. Returns `(overlap[M][N], left_diff[M], right_diff[N])`.
#[allow(clippy::needless_range_loop)]
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

    let patterns = tabulate_joint_patterns::<P, B, R, H, M, N>(lefts, rights);

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

    let relative_error_limit = 10.0_f64.powi(-2) / f64::integer_exp2(P::EXPONENT).sqrt();

    let mut optimizer = JointAdam::new(k);

    for _ in 0_u16..10_000_u16 {
        let (_ll, mut gradients) = joint_ll_and_gradient(&patterns, &phis, k);
        add_marginal_anchor_gradient(&anchors, &phis, &mut gradients);

        optimizer.apply(&mut gradients, &mut phis);

        if gradients.iter().all(|g| g.abs() <= relative_error_limit) {
            break;
        }
    }

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
/// `docs/joint_mle_math.md`, section 5.
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

/// Adam optimizer over a dynamically sized parameter vector, used by the generalized joint MLE
/// where the parameter count `M*N + M + N` is not a compile-time constant. Mirrors [`Adam`].
struct JointAdam {
    first_moments: Vec<f64>,
    second_moments: Vec<f64>,
    time: i32,
    learning_rate: f64,
    first_order_decay_factor: f64,
    second_order_decay_factor: f64,
}

impl JointAdam {
    fn new(dimension: usize) -> Self {
        JointAdam {
            first_moments: vec![0.0; dimension],
            second_moments: vec![0.0; dimension],
            time: 0,
            learning_rate: 0.1,
            first_order_decay_factor: 0.9,
            second_order_decay_factor: 0.999,
        }
    }

    /// Applies one Adam ascent step, overwriting `gradients` with the applied update and adding it
    /// to `phis`.
    fn apply(&mut self, gradients: &mut [f64], phis: &mut [f64]) {
        self.time += 1_i32;
        for (((first_moment, second_moment), gradient), phi) in self
            .first_moments
            .iter_mut()
            .zip(self.second_moments.iter_mut())
            .zip(gradients.iter_mut())
            .zip(phis.iter_mut())
        {
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
        }
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
}
