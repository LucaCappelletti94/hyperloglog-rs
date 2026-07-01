//! The 2-set union joint MLE (Ertl's joint estimator), maximized by a small Levenberg-damped Newton
//! loop over the analytic score and exact analytic Hessian, with the element-wise array helpers the
//! score uses.

use crate::prelude::*;
use crate::utils::{FloatOps, Zero};
use core::cmp::Ordering;
use core::ops::{Add, Mul, Sub};

/// Asymptotic relative covariance, in log (phi) space, of the 2-set union MLE regions
/// `[left_difference, right_difference, intersection]`, from the observed Fisher information at the
/// optimum. The diagonal entries are the squared relative standard errors of the three regions, and
/// the matrix propagates to linear (cardinality) space via
/// `Cov_linear = diag(regions) * Cov * diag(regions)`.
///
/// It is the inverse of the observed Fisher information `-Hessian(log L)` evaluated at the converged
/// `phis = ln(regions)`, with the log-likelihood score (the same [`TwoSetStats::score`] the solver
/// ascends) finite-differenced in each coordinate.
///
/// Returns a diagonal fallback of large variances when the information matrix is singular (a region
/// pinned at its floor is unidentified, so its relative error is effectively unbounded).
#[allow(clippy::too_many_lines)]
pub(crate) fn union_region_relative_covariance<
    P: Precision,
    B: Bits,
    I: ExactSizeIterator<Item = [u8; 2]>,
>(
    registers: I,
    regions: [f64; 3],
) -> [[f64; 3]; 3] {
    let stats = TwoSetStats::build::<P, B, I>(registers);

    let phi_star = [
        regions[0].natural_log(),
        regions[1].natural_log(),
        regions[2].natural_log(),
    ];

    // Observed Fisher information: minus the Hessian of the log-likelihood, i.e. minus the Jacobian of
    // the score, finite-differenced (central) in each phi coordinate at the optimum.
    let step = 1.0e-4;
    let mut information = [[0.0_f64; 3]; 3];
    for j in 0..3 {
        let mut phi_plus = phi_star;
        let mut phi_minus = phi_star;
        phi_plus[j] += step;
        phi_minus[j] -= step;
        let score_plus = stats.score(phi_plus);
        let score_minus = stats.score(phi_minus);
        for (i, info_row) in information.iter_mut().enumerate() {
            // -d(score_i)/d(phi_j); symmetrized below.
            info_row[j] = -(score_plus[i] - score_minus[i]) / (2.0 * step);
        }
    }
    let mut symmetric = [[0.0_f64; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            symmetric[i][j] = 0.5 * (information[i][j] + information[j][i]);
        }
    }

    invert_symmetric_3x3(symmetric)
}

/// Inverts a symmetric 3x3 matrix. Returns a large-variance diagonal fallback when the matrix is
/// (near) singular, signalling an unidentified parameter rather than producing a nonsensical inverse.
fn invert_symmetric_3x3(matrix: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let [a, b, c] = matrix[0];
    let [_, d, e] = matrix[1];
    let [_, _, f] = matrix[2];
    // Cofactors of the symmetric matrix [[a,b,c],[b,d,e],[c,e,f]].
    let co_a = d * f - e * e;
    let co_b = c * e - b * f;
    let co_c = b * e - c * d;
    let determinant = a * co_a + b * co_b + c * co_c;
    if !(FloatOps::abs(determinant) > 1.0e-300) {
        let big = 1.0e12;
        return [[big, 0.0, 0.0], [0.0, big, 0.0], [0.0, 0.0, big]];
    }
    let inv = 1.0 / determinant;
    let co_d = a * f - c * c;
    let co_e = b * c - a * e;
    let co_f = a * d - b * b;
    [
        [co_a * inv, co_b * inv, co_c * inv],
        [co_b * inv, co_d * inv, co_e * inv],
        [co_c * inv, co_e * inv, co_f * inv],
    ]
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

/// The 2-set register multiplicity statistics that drive the union-MLE log-likelihood, built in one
/// pass over the `[left, right]` register pairs. The [`score`](TwoSetStats::score) is the ascent
/// gradient the damped-Newton loop follows, the [`hessian`](TwoSetStats::hessian) is its exact analytic
/// Hessian, and the [`log_likelihood`](TwoSetStats::log_likelihood) is the objective the step-accept
/// check compares. The covariance estimator reuses the same `score`.
struct TwoSetStats {
    left_multiplicities_larger: [f64; crate::mle::REGISTER_MULTIPLICITIES_CAPACITY],
    left_multiplicities_smaller: [f64; crate::mle::REGISTER_MULTIPLICITIES_CAPACITY],
    right_multiplicities_larger: [f64; crate::mle::REGISTER_MULTIPLICITIES_CAPACITY],
    right_multiplicities_smaller: [f64; crate::mle::REGISTER_MULTIPLICITIES_CAPACITY],
    joint_multiplicities: [f64; crate::mle::REGISTER_MULTIPLICITIES_CAPACITY],
    zeros_0: [f64; 3],
    zeros_q: [f64; 3],
    p_exponent: u8,
    q_plus_one: u8,
}

impl TwoSetStats {
    /// Builds the multiplicity arrays from the register pairs, plus the zero and saturation boundary
    /// counts used by the score.
    fn build<P: Precision, B: Bits, I: ExactSizeIterator<Item = [u8; 2]>>(registers: I) -> Self {
        const CAP: usize = crate::mle::REGISTER_MULTIPLICITIES_CAPACITY;
        let mut left_multiplicities_larger = [f64::ZERO; CAP];
        let mut left_multiplicities_smaller = [f64::ZERO; CAP];
        let mut right_multiplicities_larger = [f64::ZERO; CAP];
        let mut right_multiplicities_smaller = [f64::ZERO; CAP];
        let mut joint_multiplicities = [f64::ZERO; CAP];
        for [left_register, right_register] in registers {
            let cmp = left_register.cmp(&right_register);
            let left_register = usize::from(left_register);
            let right_register = usize::from(right_register);
            left_multiplicities_smaller[left_register] += f64::from(cmp == Ordering::Less);
            right_multiplicities_larger[right_register] += f64::from(cmp == Ordering::Less);
            left_multiplicities_larger[left_register] += f64::from(cmp == Ordering::Greater);
            right_multiplicities_smaller[right_register] += f64::from(cmp == Ordering::Greater);
            joint_multiplicities[left_register] += f64::from(cmp == Ordering::Equal);
        }
        let q_plus_one: u8 = (1 << B::NUMBER_OF_BITS) - 1;
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
        Self {
            left_multiplicities_larger,
            left_multiplicities_smaller,
            right_multiplicities_larger,
            right_multiplicities_smaller,
            joint_multiplicities,
            zeros_0,
            zeros_q,
            p_exponent: P::EXPONENT,
            q_plus_one,
        }
    }

    /// Builds the multiplicity stats from an already-accumulated `(left value, right value)` count
    /// histogram (the damped path builds the histogram once to drive the exact log-likelihood, so the
    /// 1D bins for the score and Hessian are derived from it here rather than swept a second time).
    fn from_pattern_counts<P: Precision, B: Bits>(
        pattern_counts: &[[f64; crate::mle::REGISTER_MULTIPLICITIES_CAPACITY];
             crate::mle::REGISTER_MULTIPLICITIES_CAPACITY],
    ) -> Self {
        const CAP: usize = crate::mle::REGISTER_MULTIPLICITIES_CAPACITY;
        let mut left_multiplicities_larger = [f64::ZERO; CAP];
        let mut left_multiplicities_smaller = [f64::ZERO; CAP];
        let mut right_multiplicities_larger = [f64::ZERO; CAP];
        let mut right_multiplicities_smaller = [f64::ZERO; CAP];
        let mut joint_multiplicities = [f64::ZERO; CAP];
        for (a, row) in pattern_counts.iter().enumerate() {
            for (b, &count) in row.iter().enumerate() {
                if count == 0.0 {
                    continue;
                }
                match a.cmp(&b) {
                    Ordering::Less => {
                        left_multiplicities_smaller[a] += count;
                        right_multiplicities_larger[b] += count;
                    }
                    Ordering::Greater => {
                        left_multiplicities_larger[a] += count;
                        right_multiplicities_smaller[b] += count;
                    }
                    Ordering::Equal => {
                        joint_multiplicities[a] += count;
                    }
                }
            }
        }
        let q_plus_one: u8 = (1 << B::NUMBER_OF_BITS) - 1;
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
        Self {
            left_multiplicities_larger,
            left_multiplicities_smaller,
            right_multiplicities_larger,
            right_multiplicities_smaller,
            joint_multiplicities,
            zeros_0,
            zeros_q,
            p_exponent: P::EXPONENT,
            q_plus_one,
        }
    }

    /// The log-likelihood score (ascent gradient) at `phis = [ln left_diff, ln right_diff, ln
    /// intersection]`. Bit-identical to the expression `mle_union_regions` ascends.
    fn score(&self, phis: [f64; 3]) -> [f64; 3] {
        let q: u8 = self.q_plus_one - 1;
        let two_to_zero: f64 = f64::integer_exp2_minus(self.p_exponent);
        let two_to_minus_q: f64 = f64::integer_exp2_minus(self.p_exponent + q);

        let x = |phi: [f64; 3], two_to_minus_register: f64| -> [f64; 3] {
            [
                FloatOps::maximum(FloatOps::exp(phi[0]) * two_to_minus_register, f64::EPSILON),
                FloatOps::maximum(FloatOps::exp(phi[1]) * two_to_minus_register, f64::EPSILON),
                FloatOps::maximum(FloatOps::exp(phi[2]) * two_to_minus_register, f64::EPSILON),
            ]
        };
        let yz = |x: [f64; 3]| -> ([f64; 3], [f64; 3]) {
            let exp_m1 = [
                FloatOps::exp_m1(-x[0]),
                FloatOps::exp_m1(-x[1]),
                FloatOps::exp_m1(-x[2]),
            ];
            (
                [
                    FloatOps::maximum(1.0 + exp_m1[0], f64::EPSILON),
                    FloatOps::maximum(1.0 + exp_m1[1], f64::EPSILON),
                    FloatOps::maximum(1.0 + exp_m1[2], f64::EPSILON),
                ],
                [-exp_m1[0], -exp_m1[1], -exp_m1[2]],
            )
        };

        let x_0 = x(phis, two_to_zero);
        let x_q = x(phis, two_to_minus_q);
        let (y_q, z_q) = yz(x_q);
        let denominator = 1.0 / (z_q[2] + y_q[2] * z_q[0] * z_q[1]);
        let y_q_saturated = y_q.ew_mul(self.zeros_q[2]);
        let mut gradients = [0.0_f64; 3];
        gradients[0] = y_q_saturated[2] * denominator * z_q[1] + self.zeros_q[0] / z_q[0];
        gradients[1] = y_q_saturated[2] * denominator * z_q[0] + self.zeros_q[1] / z_q[1];
        gradients[2] = y_q_saturated[0] * denominator + y_q_saturated[1] * z_q[0];
        gradients = (gradients.ew_mul(y_q.ew_mul(z_q))).ew_sub(self.zeros_0.ew_mul(x_0));
        (1..self.q_plus_one).for_each(|register_value| {
            let two_to_minus_register = f64::integer_exp2_minus(self.p_exponent + register_value);
            let x_register = x(phis, two_to_minus_register);
            let (y_register, z_register) = yz(x_register);
            let joint_k = self.joint_multiplicities[usize::from(register_value)];
            let left_smaller_k = self.left_multiplicities_smaller[usize::from(register_value)];
            let left_larger_k = self.left_multiplicities_larger[usize::from(register_value)];
            let right_smaller_k = self.right_multiplicities_smaller[usize::from(register_value)];
            let right_larger_k = self.right_multiplicities_larger[usize::from(register_value)];
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
        gradients
    }

    /// The exact analytic 3x3 Hessian of the log-likelihood at `phis`, the M=N=1 specialization of the
    /// joint-model Hessian (docs/joint_mle_math.md section 11), accumulated over the same 1D
    /// multiplicity arrays the `score` uses. It needs no score re-evaluations (unlike a finite
    /// difference), so it is the cheapest Hessian for the damped-Newton step. The derivatives of the
    /// score's ratio terms collapse cleanly because numerator-plus-denominator combinations reduce to
    /// one (for example `A + y_J y_L = 1` and `D + y_J(1 - z_L z_R) = 1`). The astronomically rare
    /// saturated boundary is omitted: it is zero for every realistic B6 register, and the Levenberg
    /// damping absorbs any residual. Validated against a central finite difference of the score by
    /// `two_set_analytic_hessian_matches_finite_difference`.
    fn hessian(&self, phis: [f64; 3]) -> [[f64; 3]; 3] {
        let exp_phi = [
            FloatOps::exp(phis[0]),
            FloatOps::exp(phis[1]),
            FloatOps::exp(phis[2]),
        ];
        let x_at = |level: u8| -> [f64; 3] {
            let scale = f64::integer_exp2_minus(self.p_exponent + level);
            [
                FloatOps::maximum(exp_phi[0] * scale, f64::EPSILON),
                FloatOps::maximum(exp_phi[1] * scale, f64::EPSILON),
                FloatOps::maximum(exp_phi[2] * scale, f64::EPSILON),
            ]
        };
        let yz = |x: [f64; 3]| -> ([f64; 3], [f64; 3]) {
            let exp_m1 = [
                FloatOps::exp_m1(-x[0]),
                FloatOps::exp_m1(-x[1]),
                FloatOps::exp_m1(-x[2]),
            ];
            (
                [
                    FloatOps::maximum(1.0 + exp_m1[0], f64::EPSILON),
                    FloatOps::maximum(1.0 + exp_m1[1], f64::EPSILON),
                    FloatOps::maximum(1.0 + exp_m1[2], f64::EPSILON),
                ],
                [-exp_m1[0], -exp_m1[1], -exp_m1[2]],
            )
        };
        let mut h = [[0.0_f64; 3]; 3];
        for register_value in 1..self.q_plus_one {
            let k = usize::from(register_value);
            let ls = self.left_multiplicities_smaller[k];
            let rs = self.right_multiplicities_smaller[k];
            let ll = self.left_multiplicities_larger[k];
            let rl = self.right_multiplicities_larger[k];
            let jt = self.joint_multiplicities[k];
            if ls == 0.0 && rs == 0.0 && ll == 0.0 && rl == 0.0 && jt == 0.0 {
                continue;
            }
            let x = x_at(register_value);
            let (y, z) = yz(x);
            // Region indices: 0 = left, 1 = right, 2 = intersection.
            let a = z[2] + y[2] * z[0]; // 1 - y_L y_J
            let b = z[2] + y[2] * z[1]; // 1 - y_R y_J
            let d = z[2] + y[2] * z[0] * z[1];
            let n = y[2] * y[0] + y[2] * z[0] * y[1];
            let inv_a2 = 1.0 / (a * a);
            let inv_b2 = 1.0 / (b * b);
            let inv_d2 = 1.0 / (d * d);
            let inv_zl2 = 1.0 / (z[0] * z[0]);
            let inv_zr2 = 1.0 / (z[1] * z[1]);
            // The score's per-level delta (kept with the -1 terms for the diagonal).
            let delta0 = ls * (y[2] * y[0] / a - 1.0)
                + jt * (y[2] * z[1] * y[0] / d - 1.0)
                + ll * (y[0] / z[0] - 1.0);
            let delta1 = rs * (y[2] * y[1] / b - 1.0)
                + jt * (y[2] * z[0] * y[1] / d - 1.0)
                + rl * (y[1] / z[1] - 1.0);
            let delta2 =
                ls * (y[2] * y[0] / a - 1.0) + rs * (y[2] * y[1] / b - 1.0) + jt * (n / d - 1.0);
            // Partial derivatives of delta with respect to each phi (the -1 terms drop out).
            let dd0_dl = -x[0]
                * (ls * y[2] * y[0] * inv_a2
                    + jt * y[2] * z[1] * y[0] * b * inv_d2
                    + ll * y[0] * inv_zl2);
            let dd0_dr = x[1] * jt * y[2] * y[0] * y[1] * z[2] * inv_d2;
            let dd0_dj = -x[2] * (ls * y[2] * y[0] * inv_a2 + jt * y[2] * z[1] * y[0] * inv_d2);
            let dd1_dr = -x[1]
                * (rs * y[2] * y[1] * inv_b2
                    + jt * y[2] * z[0] * y[1] * a * inv_d2
                    + rl * y[1] * inv_zr2);
            let dd1_dj = -x[2] * (rs * y[2] * y[1] * inv_b2 + jt * y[2] * z[0] * y[1] * inv_d2);
            let dd2_dj =
                -x[2] * (ls * y[2] * y[0] * inv_a2 + rs * y[2] * y[1] * inv_b2 + jt * n * inv_d2);
            // H[i][j] = x[i] * d delta[i] / d phi_j, plus the diagonal x[i] * delta[i].
            h[0][0] += x[0] * (delta0 + dd0_dl);
            h[1][1] += x[1] * (delta1 + dd1_dr);
            h[2][2] += x[2] * (delta2 + dd2_dj);
            h[0][1] += x[0] * dd0_dr;
            h[0][2] += x[0] * dd0_dj;
            h[1][2] += x[1] * dd1_dj;
        }
        // Empty boundary (level 0): diagonal only, each region forced empty contributes -x_rho(0).
        let x0 = x_at(0);
        h[0][0] -= self.zeros_0[0] * x0[0];
        h[1][1] -= self.zeros_0[1] * x0[1];
        h[2][2] -= self.zeros_0[2] * x0[2];
        // Symmetric by construction (the saturated boundary, zero in practice, is omitted).
        h[1][0] = h[0][1];
        h[2][0] = h[0][2];
        h[2][1] = h[1][2];
        h
    }

    /// The 2-set joint log-likelihood at `phis`, summed over the same 1D multiplicity arrays the
    /// `score` uses, so it is `O(q)`. The per-register likelihood separates: a register with left value
    /// `a` and right value `b` contributes a term that splits cleanly into a part depending on `a` and a
    /// part depending on `b` (the M=N=1 specialization of docs/joint_mle_math.md section 6), which lets
    /// each side be summed over its 1D bin. Index 0 is the empty boundary (the `zeros_0` factors) and
    /// the index `q_plus_one` is the saturated boundary (the `zeros_q` factors), exactly as the score
    /// handles them. Its gradient is validated against the trusted `score` by
    /// `two_set_log_likelihood_gradient_matches_score`.
    fn log_likelihood(&self, phis: [f64; 3]) -> f64 {
        let q: u8 = self.q_plus_one - 1;
        let exp_phi = [
            FloatOps::exp(phis[0]),
            FloatOps::exp(phis[1]),
            FloatOps::exp(phis[2]),
        ];
        let x_at = |level: u8| -> [f64; 3] {
            let scale = f64::integer_exp2_minus(self.p_exponent + level);
            [
                FloatOps::maximum(exp_phi[0] * scale, f64::EPSILON),
                FloatOps::maximum(exp_phi[1] * scale, f64::EPSILON),
                FloatOps::maximum(exp_phi[2] * scale, f64::EPSILON),
            ]
        };
        let yz = |x: [f64; 3]| -> ([f64; 3], [f64; 3]) {
            let exp_m1 = [
                FloatOps::exp_m1(-x[0]),
                FloatOps::exp_m1(-x[1]),
                FloatOps::exp_m1(-x[2]),
            ];
            (
                [
                    FloatOps::maximum(1.0 + exp_m1[0], f64::EPSILON),
                    FloatOps::maximum(1.0 + exp_m1[1], f64::EPSILON),
                    FloatOps::maximum(1.0 + exp_m1[2], f64::EPSILON),
                ],
                [-exp_m1[0], -exp_m1[1], -exp_m1[2]],
            )
        };
        let ln = |value: f64| FloatOps::natural_log(FloatOps::maximum(value, f64::EPSILON));

        // Empty boundary (level 0): every region forced empty contributes its empty factor ln y(0) =
        // -x(0).
        let x_0 = x_at(0);
        let mut log_likelihood =
            -(self.zeros_0[0] * x_0[0] + self.zeros_0[1] * x_0[1] + self.zeros_0[2] * x_0[2]);

        // Saturated boundary (level q): a saturated simple region contributes ln z(q), and the
        // both-saturated joint contributes the same z_J + y_J z_L z_R combination as the interior
        // joint factor.
        let x_q = x_at(q);
        let (y_q, z_q) = yz(x_q);
        log_likelihood += self.zeros_q[0] * ln(z_q[0])
            + self.zeros_q[1] * ln(z_q[1])
            + self.zeros_q[2] * ln(z_q[2] + y_q[2] * z_q[0] * z_q[1]);

        // Interior exact ranks k = 1..q.
        for register_value in 1..self.q_plus_one {
            let k = usize::from(register_value);
            let left_smaller = self.left_multiplicities_smaller[k];
            let right_larger = self.right_multiplicities_larger[k];
            let right_smaller = self.right_multiplicities_smaller[k];
            let left_larger = self.left_multiplicities_larger[k];
            let joint = self.joint_multiplicities[k];
            if left_smaller == 0.0
                && right_larger == 0.0
                && right_smaller == 0.0
                && left_larger == 0.0
                && joint == 0.0
            {
                continue;
            }
            let x = x_at(register_value);
            let (y, z) = yz(x);
            // a < b: the left and intersection reach a, the right alone reaches b.
            if left_smaller != 0.0 {
                log_likelihood += left_smaller * (-x[0] - x[2] + ln(1.0 - y[0] * y[2]));
            }
            if right_larger != 0.0 {
                log_likelihood += right_larger * (-x[1] + ln(z[1]));
            }
            // a > b: the right and intersection reach b, the left alone reaches a.
            if right_smaller != 0.0 {
                log_likelihood += right_smaller * (-x[1] - x[2] + ln(1.0 - y[1] * y[2]));
            }
            if left_larger != 0.0 {
                log_likelihood += left_larger * (-x[0] + ln(z[0]));
            }
            // a = b: the joint factor.
            if joint != 0.0 {
                log_likelihood += joint * (-x[0] - x[1] - x[2] + ln(z[2] + y[2] * z[0] * z[1]));
            }
        }
        log_likelihood
    }
}

/// Solves the symmetric 3x3 system `m * delta = rhs` by Cramer's rule. Returns `None` if the matrix is
/// (near) singular, so the caller can raise the Levenberg damping and retry.
fn solve_symmetric_3x3(m: [[f64; 3]; 3], rhs: [f64; 3]) -> Option<[f64; 3]> {
    let [a, b, c] = m[0];
    let [_, d, e] = m[1];
    let [_, _, f] = m[2];
    let co_a = d * f - e * e;
    let co_b = c * e - b * f;
    let co_c = b * e - c * d;
    let determinant = a * co_a + b * co_b + c * co_c;
    if !(FloatOps::abs(determinant) > 1.0e-300) {
        return None;
    }
    let inv = 1.0 / determinant;
    let co_d = a * f - c * c;
    let co_e = b * c - a * e;
    let co_f = a * d - b * b;
    // Inverse of the symmetric matrix times rhs.
    let inverse = [
        [co_a * inv, co_b * inv, co_c * inv],
        [co_b * inv, co_d * inv, co_e * inv],
        [co_c * inv, co_e * inv, co_f * inv],
    ];
    let mut delta = [0.0_f64; 3];
    for i in 0..3 {
        delta[i] = inverse[i][0] * rhs[0] + inverse[i][1] * rhs[1] + inverse[i][2] * rhs[2];
        if !delta[i].is_finite() {
            return None;
        }
    }
    Some(delta)
}

/// Computes the three disjoint regions of the 2-set joint MLE: `[left_difference, right_difference,
/// intersection]`, i.e. `[|A \ B|, |B \ A|, |A intersect B|]`. The union cardinality is their sum. This
/// is the analytic Ertl estimator over the register multiplicity arrays, maximized by a small
/// Levenberg-damped Newton loop over the analytic score and exact analytic Hessian (each iteration is
/// O(2^B), not O(number of registers)), cheap enough that the `M = N = 1` joint sketch dispatches here
/// directly.
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
    // The register pairs are single-pass, so the 2D pattern histogram (count of each observed (left
    // value, right value)) and the union harmonic sum and zero count are accumulated in one sweep. The
    // histogram drives the exact log-likelihood the accept step uses, and the 1D multiplicity stats
    // (for the score and Hessian) are derived from it.
    const CAP: usize = crate::mle::REGISTER_MULTIPLICITIES_CAPACITY;
    let mut pattern_counts = [[f64::ZERO; CAP]; CAP];
    let mut union_harmonic_sum = f64::ZERO;
    let mut union_zeros = 0u32;
    for [left_register, right_register] in registers {
        let larger = left_register.max(right_register);
        pattern_counts[usize::from(left_register)][usize::from(right_register)] += 1.0;
        union_harmonic_sum += f64::integer_exp2_minus(larger);
        union_zeros += u32::from(larger.is_zero());
    }
    let stats = TwoSetStats::from_pattern_counts::<P, B>(&pattern_counts);

    let union_cardinality = estimate(union_harmonic_sum, union_zeros);
    if union_zeros == 1_u32 << P::EXPONENT {
        return [f64::ZERO; 3];
    }

    let intersection: f64 = FloatOps::maximum(
        left_cardinality + right_cardinality - union_cardinality,
        f64::EPSILON,
    );
    let left_difference: f64 =
        FloatOps::maximum(union_cardinality - right_cardinality, f64::EPSILON);
    let right_difference: f64 =
        FloatOps::maximum(union_cardinality - left_cardinality, f64::EPSILON);

    let relative_error_limit =
        FloatOps::powi(10.0_f64, -error_exponent) / FloatOps::sqrt(f64::integer_exp2(P::EXPONENT));

    let mut phis = [
        left_difference.natural_log(),
        right_difference.natural_log(),
        intersection.natural_log(),
    ];

    // Levenberg-damped Newton on the log-likelihood. The information matrix is `-Hessian` (positive
    // definite near the maximum), damped by `lambda * I`. We solve `(A + lambda I) delta = gradient`
    // and accept the step when it INCREASES the log-likelihood (the actual objective), lowering lambda
    // toward a pure Newton step. A step that does not increase the objective (or a singular solve)
    // raises lambda and retries. Accepting on the objective value (not the gradient norm) is essential
    // at a weakly identified cell, where the gradient norm has spurious minima. The stopping rule is a
    // per-coordinate gradient tolerance scaled by the precision.
    let mut lambda = 1.0e-3_f64;
    let mut gradient = stats.score(phis);
    let mut current_value = stats.log_likelihood(phis);
    for _ in 0_u16..200_u16 {
        if gradient
            .iter()
            .all(|g| FloatOps::abs(*g) <= relative_error_limit)
        {
            break;
        }
        let hessian = stats.hessian(phis);
        // Information matrix A = -Hessian.
        let information = [
            [-hessian[0][0], -hessian[0][1], -hessian[0][2]],
            [-hessian[1][0], -hessian[1][1], -hessian[1][2]],
            [-hessian[2][0], -hessian[2][1], -hessian[2][2]],
        ];

        let mut accepted = false;
        let mut iterations = 0u8;
        while iterations < 60 {
            iterations += 1;
            let damped = [
                [
                    information[0][0] + lambda,
                    information[0][1],
                    information[0][2],
                ],
                [
                    information[1][0],
                    information[1][1] + lambda,
                    information[1][2],
                ],
                [
                    information[2][0],
                    information[2][1],
                    information[2][2] + lambda,
                ],
            ];
            let Some(delta) = solve_symmetric_3x3(damped, gradient) else {
                lambda *= 4.0;
                continue;
            };
            let candidate = [phis[0] + delta[0], phis[1] + delta[1], phis[2] + delta[2]];
            if candidate.iter().all(|v| v.is_finite()) {
                let candidate_value = stats.log_likelihood(candidate);
                // Accept on an increase of the log-likelihood (the actual objective). The damped
                // information matrix is positive definite, so `delta` is an ascent direction and a
                // small enough step always increases the objective: a rejected step means lambda is
                // too small, so raise it and retry.
                if candidate_value.is_finite() && candidate_value >= current_value - 1.0e-9 {
                    phis = candidate;
                    gradient = stats.score(candidate);
                    current_value = candidate_value;
                    accepted = true;
                    lambda = FloatOps::maximum(lambda / 3.0, 1.0e-12);
                    break;
                }
            }
            lambda *= 4.0;
            if lambda > 1.0e12 {
                break;
            }
        }
        if !accepted {
            break;
        }
    }

    [
        FloatOps::exp(phis[0]),
        FloatOps::exp(phis[1]),
        FloatOps::exp(phis[2]),
    ]
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use sketching_core::CardinalityEstimator;

    type Hll = HyperLogLog<Precision12, Bits6>;

    fn splitmix64(mut x: u64) -> u64 {
        x = x.wrapping_add(0x9E3779B97F4A7C15);
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
        x ^ (x >> 31)
    }

    /// The O(q) `TwoSetStats::log_likelihood` is the objective the damped-Newton 2-set path accepts
    /// steps on, so its gradient must equal the trusted `score` (the analytic ascent gradient the solver
    /// follows). Validated by central finite difference over realistic register values across several
    /// cardinality regimes. Only realistic register levels (0 to 40) are used: at astronomically high
    /// levels x underflows the shared EPSILON clamp, where the analytic score and a finite difference
    /// necessarily disagree, but such levels never occur for real B6 registers (level 40 already needs
    /// cardinality far beyond the dense range).
    #[test]
    fn two_set_log_likelihood_gradient_matches_score() {
        use super::TwoSetStats;
        const CAP: usize = crate::mle::REGISTER_MULTIPLICITIES_CAPACITY;
        let mut state = 0xA5A5_1234_u64;
        let mut worst_grad = 0.0_f64;
        for _ in 0..50_u64 {
            let mut counts = [[0.0_f64; CAP]; CAP];
            for _ in 0..400 {
                state = splitmix64(state);
                let a = (state % 41) as usize;
                state = splitmix64(state);
                let b = (state % 41) as usize;
                counts[a][b] += 1.0;
            }
            // The empty boundary (value 0) and ties, the cases that actually occur.
            for &(a, b) in &[(0, 0), (0, 7), (7, 0), (15, 15), (3, 20), (20, 3)] {
                counts[a][b] += 5.0;
            }
            let stats = TwoSetStats::from_pattern_counts::<Precision12, Bits6>(&counts);
            for &phis in &[
                [9.0_f64, 9.0, 8.0],
                [12.0, 7.0, 10.0],
                [8.0, 11.0, 9.0],
                [10.0, 10.0, 10.0],
            ] {
                // The gradient of the O(q) value must equal the trusted score (the analytic ascent
                // gradient the solver follows). A 4th-order (Richardson) central difference makes the
                // truncation negligible, so this is a tight gate.
                let score = stats.score(phis);
                for j in 0..3 {
                    let h = 1.0e-3 * (1.0 + phis[j].abs());
                    let eval = |delta: f64| {
                        let mut p = phis;
                        p[j] += delta;
                        stats.log_likelihood(p)
                    };
                    let fd = (8.0 * (eval(h) - eval(-h)) - (eval(2.0 * h) - eval(-2.0 * h)))
                        / (12.0 * h);
                    let rel = (fd - score[j]).abs() / score[j].abs().max(1.0);
                    worst_grad = worst_grad.max(rel);
                    // 5e-4 sits safely above the data-dependent Richardson finite-difference floor
                    // (rounding plus O(h^4) truncation on a value of magnitude ~1e4, steepest for the
                    // intersection coordinate) and far below any formula error (the bugs found during
                    // derivation were percent-level or larger). The end-to-end accuracy of the solver
                    // this value drives is covered by `union_region_covariance_predicts_spread`.
                    assert!(
                        rel < 5.0e-4,
                        "gradient mismatch phis {phis:?} coord {j}: fd {fd} vs score {} (relative {rel:.3e})",
                        score[j]
                    );
                }
            }
        }
        std::eprintln!("two_set log-likelihood: worst gradient-vs-score {worst_grad:.2e}");
    }

    /// The exact analytic 3x3 Hessian must equal a central finite difference of the trusted score,
    /// over realistic register values and several cardinality regimes. The score is the gradient of
    /// the log-likelihood, so the Jacobian of the score is the Hessian, and the analytic form must
    /// reproduce it. Realistic register levels only (saturation never occurs, and its boundary is
    /// omitted from the analytic Hessian on purpose).
    #[test]
    fn two_set_analytic_hessian_matches_finite_difference() {
        use super::TwoSetStats;
        const CAP: usize = crate::mle::REGISTER_MULTIPLICITIES_CAPACITY;
        let mut state = 0xBEEF_4321_u64;
        let mut worst = 0.0_f64;
        for _ in 0..50_u64 {
            let mut counts = [[0.0_f64; CAP]; CAP];
            for _ in 0..400 {
                state = splitmix64(state);
                let a = (state % 41) as usize;
                state = splitmix64(state);
                let b = (state % 41) as usize;
                counts[a][b] += 1.0;
            }
            for &(a, b) in &[(0, 0), (0, 7), (7, 0), (15, 15), (3, 20), (20, 3)] {
                counts[a][b] += 5.0;
            }
            let stats = TwoSetStats::from_pattern_counts::<Precision12, Bits6>(&counts);
            for &phis in &[
                [9.0_f64, 9.0, 8.0],
                [12.0, 7.0, 10.0],
                [8.0, 11.0, 9.0],
                [10.0, 10.0, 10.0],
            ] {
                let analytic = stats.hessian(phis);
                // Central finite difference of the score (its Jacobian), symmetrized.
                let mut fd = [[0.0_f64; 3]; 3];
                for j in 0..3 {
                    let h = 1.0e-4 * (1.0 + phis[j].abs());
                    let mut plus = phis;
                    let mut minus = phis;
                    plus[j] += h;
                    minus[j] -= h;
                    let sp = stats.score(plus);
                    let sm = stats.score(minus);
                    for (i, row) in fd.iter_mut().enumerate() {
                        row[j] = (sp[i] - sm[i]) / (2.0 * h);
                    }
                }
                for i in 0..3 {
                    for j in 0..3 {
                        let symmetric_fd = 0.5 * (fd[i][j] + fd[j][i]);
                        let rel =
                            (analytic[i][j] - symmetric_fd).abs() / symmetric_fd.abs().max(1.0);
                        worst = worst.max(rel);
                        assert!(
                            rel < 1.0e-3,
                            "Hessian[{i}][{j}] phis {phis:?}: analytic {} vs fd {symmetric_fd} (relative {rel:.3e})",
                            analytic[i][j]
                        );
                    }
                }
            }
        }
        std::eprintln!("two_set analytic Hessian vs finite difference: worst relative {worst:.2e}");
    }

    /// The Fisher-information covariance must predict the actual per-region spread. For two dense
    /// counters with a controlled overlap, over many seeds, the predicted relative standard error of
    /// each region (sqrt of the covariance diagonal, averaged over seeds) must track the measured
    /// across-seed relative standard deviation of that region's estimate.
    #[test]
    fn union_region_covariance_predicts_spread() {
        type Hll12 = HyperLogLog<Precision12, Bits6>;
        // (left_only, right_only, intersection) truth for three overlap regimes.
        for &(da, db, inter) in &[(5000u64, 5000u64, 5000u64), (10000, 10000, 500)] {
            let seeds = 300u64;
            let mut sum_est = [0.0_f64; 3];
            let mut sum_sq = [0.0_f64; 3];
            let mut sum_pred = [0.0_f64; 3];
            for seed in 1..=seeds {
                let base = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
                // Disjoint value ranges: shared, left-only, right-only.
                let shared = |i: u64| splitmix64(base.wrapping_add(i));
                let left_only = |i: u64| splitmix64(base.wrapping_add(1_000_000_000 + i));
                let right_only = |i: u64| splitmix64(base.wrapping_add(2_000_000_000 + i));
                let mut a = Hll12::default();
                let mut b = Hll12::default();
                for i in 0..inter {
                    a.insert(&shared(i));
                    b.insert(&shared(i));
                }
                for i in 0..da {
                    a.insert(&left_only(i));
                }
                for i in 0..db {
                    b.insert(&right_only(i));
                }
                let a = a.into_hll();
                let b = b.into_hll();
                let (regions, cov) = a.mle_union_region_covariance_from_registers(&b);
                for k in 0..3 {
                    sum_est[k] += regions[k];
                    sum_sq[k] += regions[k] * regions[k];
                    sum_pred[k] += cov[k][k].max(0.0).sqrt();
                }
            }
            let n = seeds as f64;
            for k in 0..3 {
                let mean = sum_est[k] / n;
                let var = (sum_sq[k] / n - mean * mean).max(0.0);
                let empirical_rse = var.sqrt() / mean;
                let predicted_rse = sum_pred[k] / n;
                // The observed Fisher information predicts the spread to within a modest factor (it is
                // mildly conservative, ratio ~1.1, and near-exact for the amplified small-overlap cell).
                assert!(
                    predicted_rse > 0.7 * empirical_rse && predicted_rse < 1.7 * empirical_rse,
                    "da={da} db={db} inter={inter} region {k}: predicted {predicted_rse} vs empirical {empirical_rse}"
                );
            }
        }
    }

    /// The 2x2 joint-sketch error grid must (conservatively) predict the per-cell spread. Builds two
    /// nested chains over many seeds, then compares the predicted overlap-cell standard error to the
    /// measured across-seed standard deviation of that cell. Under the nested-shell correlation
    /// structure the independence approximation in the differential propagation behaves as an
    /// over-estimate, so the prediction stays at or above the measured spread (and tight on the large
    /// cell).
    #[test]
    fn joint_sketch_error_predicts_2x2_spread() {
        type Hll12 = HyperLogLog<Precision12, Bits6>;
        let seeds = 200u64;
        let mut sum_cell = [[0.0_f64; 2]; 2];
        let mut sum_cell_sq = [[0.0_f64; 2]; 2];
        let mut sum_pred = [[0.0_f64; 2]; 2];
        for seed in 1..=seeds {
            let base = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
            let v = |tag: u64, i: u64| splitmix64(base.wrapping_add(tag.wrapping_mul(1 << 40) + i));
            // Left chain a0 subset of a1; right chain b0 subset of b1, sharing a controlled pool.
            let mut a0 = Hll12::default();
            let mut a1 = Hll12::default();
            let mut b0 = Hll12::default();
            let mut b1 = Hll12::default();
            for i in 0..3000 {
                let shared = v(0, i);
                a0.insert(&shared);
                a1.insert(&shared);
                b0.insert(&shared);
                b1.insert(&shared);
            }
            for i in 0..3000 {
                let x = v(1, i);
                a1.insert(&x);
                a0.insert(&x);
            } // grow a0 too so a0 subset a1 holds with a1 strictly larger via tag 2
            for i in 0..3000 {
                a1.insert(&v(2, i));
            }
            for i in 0..3000 {
                let y = v(3, i);
                b1.insert(&y);
                b0.insert(&y);
            }
            for i in 0..3000 {
                b1.insert(&v(4, i));
            }
            let (a0, a1) = (a0.into_hll(), a1.into_hll());
            let (b0, b1) = (b0.into_hll(), b1.into_hll());
            let sketch = JointSketch::estimate(&[a0.mle(), a1.mle()], &[b0.mle(), b1.mle()]);
            let error = HyperLogLog::joint_sketch_error(&[a0, a1], &[b0, b1]);
            for i in 0..2 {
                for j in 0..2 {
                    sum_cell[i][j] += sketch.overlap[i][j];
                    sum_cell_sq[i][j] += sketch.overlap[i][j] * sketch.overlap[i][j];
                    sum_pred[i][j] += error.overlap_se[i][j];
                }
            }
        }
        let n = seeds as f64;
        for i in 0..2 {
            for j in 0..2 {
                let mean = sum_cell[i][j] / n;
                let empirical_sd = ((sum_cell_sq[i][j] / n - mean * mean).max(0.0)).sqrt();
                let predicted_sd = sum_pred[i][j] / n;
                // The independence approximation makes the prediction a conservative upper bound on
                // every cell's spread.
                assert!(
                    predicted_sd > 0.8 * empirical_sd,
                    "cell[{i}][{j}]: predicted {predicted_sd} below empirical {empirical_sd}"
                );
                // It is tight on the dominant (large) cell, where the differencing has few terms.
                if mean > 1000.0 {
                    assert!(
                        predicted_sd < 2.0 * empirical_sd,
                        "cell[{i}][{j}]: predicted {predicted_sd} too loose vs empirical {empirical_sd}"
                    );
                }
            }
        }
    }

    /// The no-dense joint-sketch error (delta method over pre-dense operands) must predict the
    /// per-cell spread for hash-list operands, including the small-overlap amplification.
    #[test]
    fn joint_sketch_error_no_dense_predicts_spread() {
        type Hll14 = HyperLogLog<Precision14, Bits6>;
        for &(da, db, inter) in &[(400u64, 400, 400), (1500, 1500, 80)] {
            let seeds = 400u64;
            let (mut sum, mut sum_sq, mut sum_pred) = (0.0_f64, 0.0_f64, 0.0_f64);
            for seed in 1..=seeds {
                let base = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
                let shared = |i: u64| splitmix64(base.wrapping_add(i));
                let left_only = |i: u64| splitmix64(base.wrapping_add(1_000_000_000 + i));
                let right_only = |i: u64| splitmix64(base.wrapping_add(2_000_000_000 + i));
                let mut a = Hll14::default();
                let mut b = Hll14::default();
                for i in 0..inter {
                    a.insert(&shared(i));
                    b.insert(&shared(i));
                }
                for i in 0..da {
                    a.insert(&left_only(i));
                }
                for i in 0..db {
                    b.insert(&right_only(i));
                }
                assert!(
                    a.is_sorted_hash_list() && b.is_sorted_hash_list(),
                    "operands must stay hash-list (no-dense path)"
                );
                let la = [a];
                let lb = [b];
                let sketch = JointSketch::estimate(&la, &lb);
                let error = HyperLogLog::joint_sketch_error(&la, &lb);
                let cell = sketch.overlap[0][0];
                sum += cell;
                sum_sq += cell * cell;
                sum_pred += error.overlap_se[0][0];
            }
            let n = seeds as f64;
            let mean = sum / n;
            let empirical_sd = ((sum_sq / n - mean * mean).max(0.0)).sqrt();
            let predicted_sd = sum_pred / n;
            // Conservative upper bound on the (near-exact) spread, and tight enough to be useful.
            assert!(
                predicted_sd > 0.8 * empirical_sd && predicted_sd < 5.0 * empirical_sd,
                "no-dense da={da} db={db} inter={inter}: predicted {predicted_sd} vs empirical {empirical_sd}"
            );
            // Hash-list cells are near-exact, far below the dense noise floor a materialized estimate
            // would report (1.04/sqrt(2^14) ~ 0.8 percent).
            assert!(
                predicted_sd / mean < 0.02,
                "no-dense da={da} db={db} inter={inter}: predicted relative error {} too large",
                predicted_sd / mean
            );
        }
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
mod solver_proptest {
    //! Property test of the production 2-set union solver: over arbitrary register-pair
    //! configurations, `mle_union_regions` (Levenberg-damped Newton) must always return three finite,
    //! non-negative regions and never panic, including the degenerate all-equal and saturated inputs.
    use super::*;

    /// Builds `2^P` deterministic register pairs in `0..2^B`, with deterministic operand cardinalities
    /// and a deterministic union estimator, then checks the production solver returns finite,
    /// non-negative regions.
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
        let left_cardinality = 1.0 + (next() % 100_000) as f64;
        let right_cardinality = 1.0 + (next() % 100_000) as f64;
        let estimate = |harmonic_sum: f64, _zeros: u32| {
            P::ALPHA * f64::integer_exp2(P::EXPONENT + P::EXPONENT) / harmonic_sum
        };

        let regions = mle_union_regions::<P, B, _>(
            pairs.iter().copied(),
            left_cardinality,
            right_cardinality,
            &estimate,
            2,
        );
        regions.iter().all(|r| r.is_finite() && *r >= 0.0)
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig { cases: 192, ..Default::default() })]

        #[test]
        fn union_mle_regions_finite_nonnegative_p8_b6(seed in proptest::prelude::any::<u64>()) {
            proptest::prop_assert!(check::<Precision8, Bits6>(seed));
        }

        #[test]
        fn union_mle_regions_finite_nonnegative_p8_b4(seed in proptest::prelude::any::<u64>()) {
            proptest::prop_assert!(check::<Precision8, Bits4>(seed));
        }

        #[test]
        fn union_mle_regions_finite_nonnegative_p6_b5(seed in proptest::prelude::any::<u64>()) {
            proptest::prop_assert!(check::<Precision6, Bits5>(seed));
        }
    }
}
