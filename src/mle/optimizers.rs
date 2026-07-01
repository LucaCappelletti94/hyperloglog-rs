//! The Levenberg-damped Newton optimizer for the joint MLE refinement, with the dense linear solvers
//! it uses to take each step.
//!
//! Everything here is allocation-free. The disjoint-region model has at most `K = M*N + M + N <= 80`
//! parameters for the supported `M, N <= 8`, so the working buffers are fixed-size stack arrays sized
//! to [`MAX_K`] and used over their leading `n` entries, the Hessian is a `MAX_K * MAX_K` stack array
//! used over the leading `n * n` block (row-major with stride `n`). The optimizer mutates the
//! parameter slice in place rather than returning a `Vec`.

use crate::utils::{FloatOps, Zero};

/// Maximum number of disjoint regions `K = M*N + M + N` over the supported `M, N <= 8` (`8*8+8+8`).
/// The optimizer working buffers are stack arrays of this length, used over the leading `n` entries.
pub(crate) const MAX_K: usize = 80;

/// Solves the dense linear system `a * x = b` for `x` by Gaussian elimination with partial pivoting,
/// on a stack copy of the augmented matrix. Returns `false` if the matrix is singular (a pivot is
/// effectively zero) or the solution is non-finite, writing the result into `x`. `a` is row-major
/// `n x n` (stride `n`), `b` and `x` are length `n`. Allocation-free over the supported `n <= MAX_K`.
fn solve_linear_system(a: &[f64], b: &[f64], n: usize, x: &mut [f64]) -> bool {
    debug_assert!(n <= MAX_K, "n={n} exceeds MAX_K={MAX_K}");
    // Augmented matrix [a | b], row-major with n+1 columns, in a stack buffer.
    let mut m = [f64::ZERO; MAX_K * (MAX_K + 1)];
    let stride = n + 1;
    for r in 0..n {
        for c in 0..n {
            m[r * stride + c] = a[r * n + c];
        }
        m[r * stride + n] = b[r];
    }

    for col in 0..n {
        // Partial pivot: pick the row at or below `col` with the largest absolute pivot.
        let mut pivot_row = col;
        let mut pivot_abs = FloatOps::abs(m[col * stride + col]);
        for r in (col + 1)..n {
            let candidate = FloatOps::abs(m[r * stride + col]);
            if candidate > pivot_abs {
                pivot_abs = candidate;
                pivot_row = r;
            }
        }
        if pivot_abs < 1e-300 {
            return false;
        }
        if pivot_row != col {
            for c in 0..=n {
                m.swap(col * stride + c, pivot_row * stride + c);
            }
        }

        // Eliminate the pivot column from every other row.
        let pivot = m[col * stride + col];
        for r in 0..n {
            if r == col {
                continue;
            }
            let factor = m[r * stride + col] / pivot;
            if factor == 0.0 {
                continue;
            }
            for c in col..=n {
                m[r * stride + c] -= factor * m[col * stride + c];
            }
        }
    }

    for r in 0..n {
        x[r] = m[r * stride + n] / m[r * stride + r];
        if !x[r].is_finite() {
            return false;
        }
    }
    true
}

/// Solves the symmetric positive-definite system `a * x = b` by Cholesky factorization `a = L L^T`,
/// reusing the caller-provided `factor` scratch (row-major `n x n`, lower triangular) so no allocation
/// happens. `a` is row-major `n x n` (only the lower triangle is read), `b` and `x` are length `n`.
/// Returns `false` if `a` is not positive definite (a non-positive pivot) or the solution is
/// non-finite, in which case the caller falls back to the pivoted Gaussian solver. The damped Newton
/// system `A + lambda*I` is SPD for any `lambda > 0` when `A` is positive semidefinite, so this is the
/// fast path, about half the flops of the pivoted elimination and more stable.
fn solve_spd_cholesky(a: &[f64], b: &[f64], n: usize, factor: &mut [f64], x: &mut [f64]) -> bool {
    // Cholesky: factor[i][j] for j <= i, with a = L L^T.
    for i in 0..n {
        for j in 0..=i {
            let mut sum = a[i * n + j];
            for p in 0..j {
                sum -= factor[i * n + p] * factor[j * n + p];
            }
            if i == j {
                if sum <= 0.0 || !sum.is_finite() {
                    return false;
                }
                factor[i * n + j] = FloatOps::sqrt(sum);
            } else {
                factor[i * n + j] = sum / factor[j * n + j];
            }
        }
    }
    // Forward substitution L y = b, storing y in x.
    for i in 0..n {
        let mut sum = b[i];
        for p in 0..i {
            sum -= factor[i * n + p] * x[p];
        }
        x[i] = sum / factor[i * n + i];
    }
    // Back substitution L^T x = y, in place on x.
    for i in (0..n).rev() {
        let mut sum = x[i];
        for p in (i + 1)..n {
            sum -= factor[p * n + i] * x[p];
        }
        x[i] = sum / factor[i * n + i];
    }
    x[..n].iter().all(|value| value.is_finite())
}

/// Fills the `n x n` Hessian of `objective` at `x` (row-major, stride `n`, into the leading `n * n`
/// entries of `out`) by central-differencing the analytic ascent gradient (perturb each coordinate by
/// a relative `h`, recompute the gradient, take the symmetric difference), then symmetrizes it. The
/// objective fills its ascent gradient into the pre-zeroed buffer it is given, so the buffer is cleared
/// before each evaluation. Allocation-free over the supported `n <= MAX_K`. Test-only: production always
/// has an analytic Hessian, so this is the Hessian source for objectives that do not (the exponential
/// oracle in the cross-validation tests), and the finite-difference reference the analytic Hessian
/// tests check against.
#[cfg(test)]
pub(crate) fn finite_difference_hessian<F: FnMut(&[f64], &mut [f64]) -> f64>(
    objective: &mut F,
    x: &[f64],
    h: f64,
    out: &mut [f64],
) {
    let n = x.len();
    debug_assert!(n <= MAX_K, "n={n} exceeds MAX_K={MAX_K}");
    let mut x_perturbed = [f64::ZERO; MAX_K];
    x_perturbed[..n].copy_from_slice(x);
    let mut grad_plus = [f64::ZERO; MAX_K];
    let mut grad_minus = [f64::ZERO; MAX_K];
    for j in 0..n {
        let step = h * (1.0 + FloatOps::abs(x[j]));
        x_perturbed[j] = x[j] + step;
        for g in &mut grad_plus[..n] {
            *g = f64::ZERO;
        }
        objective(&x_perturbed[..n], &mut grad_plus[..n]);
        x_perturbed[j] = x[j] - step;
        for g in &mut grad_minus[..n] {
            *g = f64::ZERO;
        }
        objective(&x_perturbed[..n], &mut grad_minus[..n]);
        x_perturbed[j] = x[j];
        for i in 0..n {
            out[i * n + j] = (grad_plus[i] - grad_minus[i]) / (2.0 * step);
        }
    }
    // Symmetrize: average the (i, j) and (j, i) entries to cancel the finite-difference asymmetry.
    for i in 0..n {
        for j in (i + 1)..n {
            let avg = f64::midpoint(out[i * n + j], out[j * n + i]);
            out[i * n + j] = avg;
            out[j * n + i] = avg;
        }
    }
}

/// Levenberg-damped Newton: a robust second-order maximizer of a smooth objective, used to refine the
/// joint-MLE warm start. Each iteration obtains the analytic Hessian `H`, forms `A = -H`, and solves
/// `(A + lambda*I) delta = g` with an adaptive damping `lambda`. If the trial step increases the
/// objective it is accepted and `lambda` is decreased (more Newton-like). If it decreases the objective
/// or produces a non-finite step it is rejected and `lambda` is increased (more gradient-descent-like),
/// then retried. A backtracking line search along the ascent gradient is the final fallback. This
/// converges robustly even on the strongly anisotropic deep-cell instances where an undamped Newton
/// step would stall, and accepting on the objective value (not the gradient norm) keeps it correct on
/// the flat ridges of weakly identified cells. The parameter slice `x` is mutated in place to the
/// optimum (its length `n` must not exceed [`MAX_K`]).
pub(crate) struct DampedNewton;

impl DampedNewton {
    /// Hard iteration cap.
    const MAX_ITERATIONS: usize = 200;
    /// Gradient-norm convergence floor.
    const GRADIENT_TOLERANCE: f64 = 1e-9;
    /// Initial (small) damping.
    const INITIAL_LAMBDA: f64 = 1e-3;
    /// Multiplicative damping increase on a rejected step.
    const LAMBDA_UP: f64 = 4.0;
    /// Multiplicative damping decrease on an accepted step.
    const LAMBDA_DOWN: f64 = 3.0;
    /// Upper bound on damping before giving up the Newton attempt and using the line-search fallback.
    const MAX_LAMBDA: f64 = 1e12;

    /// Maximizes `objective` in place over `x` using the analytic Hessian `hessian(x, out)`, which
    /// fills the row-major `n x n` Hessian of the objective at `x` (stride `n`) into the pre-zeroed
    /// `out` buffer. `objective(x, grad)` returns the value to maximize and writes its ascent gradient
    /// into the pre-zeroed `grad` buffer. `step_tolerance` is the convergence scale (the expected
    /// statistical error, `~1/sqrt(m)`).
    pub(crate) fn maximize_map<F, G>(
        x: &mut [f64],
        objective: F,
        hessian_provider: G,
        step_tolerance: f64,
    ) where
        F: FnMut(&[f64], &mut [f64]) -> f64,
        G: Fn(&[f64], &mut [f64]),
    {
        damped_newton_loop(x, objective, hessian_provider, step_tolerance);
    }
}

/// The shared Levenberg-damped Newton loop. The Hessian comes from the analytic `hessian_provider`.
/// Mutates `x` in place.
#[allow(clippy::needless_range_loop)]
fn damped_newton_loop<F, G>(
    x: &mut [f64],
    mut objective: F,
    hessian_provider: G,
    step_tolerance: f64,
) where
    F: FnMut(&[f64], &mut [f64]) -> f64,
    G: Fn(&[f64], &mut [f64]),
{
    let n = x.len();
    debug_assert!(n <= MAX_K, "n={n} exceeds MAX_K={MAX_K}");
    let mut lambda = DampedNewton::INITIAL_LAMBDA;
    let mut gradient = [f64::ZERO; MAX_K];
    let mut probe_gradient = [f64::ZERO; MAX_K];
    let mut hessian = [f64::ZERO; MAX_K * MAX_K];
    // Reused scratch buffers, so the per-iteration and per-lambda-retry work allocates nothing.
    let mut information = [f64::ZERO; MAX_K * MAX_K];
    let mut damped = [f64::ZERO; MAX_K * MAX_K];
    let mut factor = [f64::ZERO; MAX_K * MAX_K];
    let mut delta = [f64::ZERO; MAX_K];
    let mut candidate = [f64::ZERO; MAX_K];

    for _ in 0..DampedNewton::MAX_ITERATIONS {
        for g in &mut gradient[..n] {
            *g = f64::ZERO;
        }
        let current_value = objective(x, &mut gradient[..n]);
        let gradient_norm = FloatOps::sqrt(gradient[..n].iter().map(|g| g * g).sum::<f64>());
        if gradient_norm < DampedNewton::GRADIENT_TOLERANCE {
            break;
        }

        // Build the information matrix A = -H once per outer iteration (the Hessian does not change
        // within the lambda retries, only the damping does).
        for h in &mut hessian[..n * n] {
            *h = f64::ZERO;
        }
        hessian_provider(x, &mut hessian[..n * n]);
        for idx in 0..n * n {
            information[idx] = -hessian[idx];
        }

        // Adaptive Levenberg loop: increase lambda until a step improves the objective. Each retry
        // only re-adds lambda to the diagonal and re-solves (the Hessian assembly above is not redone).
        let mut accepted = false;
        let mut accepted_step_norm = f64::ZERO;
        while lambda <= DampedNewton::MAX_LAMBDA {
            // Damped information matrix A + lambda*I (reused buffer).
            damped[..n * n].copy_from_slice(&information[..n * n]);
            for i in 0..n {
                damped[i * n + i] += lambda;
            }
            // SPD fast path (Cholesky); fall back to the pivoted Gaussian solve if not positive
            // definite. Both write the step into `delta`.
            for f in &mut factor[..n * n] {
                *f = f64::ZERO;
            }
            let solved =
                solve_spd_cholesky(
                    &damped[..n * n],
                    &gradient[..n],
                    n,
                    &mut factor[..n * n],
                    &mut delta[..n],
                ) || solve_linear_system(&damped[..n * n], &gradient[..n], n, &mut delta[..n]);
            if !solved {
                lambda *= DampedNewton::LAMBDA_UP;
                continue;
            }
            for i in 0..n {
                candidate[i] = x[i] + delta[i];
            }
            let finite = candidate[..n].iter().all(|value| value.is_finite());
            let candidate_value = if finite {
                for g in &mut probe_gradient[..n] {
                    *g = f64::ZERO;
                }
                objective(&candidate[..n], &mut probe_gradient[..n])
            } else {
                f64::NEG_INFINITY
            };
            if finite && candidate_value.is_finite() && candidate_value >= current_value {
                accepted_step_norm = FloatOps::sqrt(delta[..n].iter().map(|d| d * d).sum::<f64>());
                x.copy_from_slice(&candidate[..n]);
                accepted = true;
                lambda = (lambda / DampedNewton::LAMBDA_DOWN).max(1e-12);
                break;
            }
            lambda *= DampedNewton::LAMBDA_UP;
        }

        if !accepted {
            // Levenberg could not find an improving step even at huge damping. Fall back to a
            // backtracking line search along the raw ascent gradient (steepest ascent).
            let mut step = 1.0;
            let mut line_accepted = false;
            for _ in 0..40 {
                for i in 0..n {
                    candidate[i] = x[i] + step * gradient[i];
                }
                if candidate[..n].iter().all(|value| value.is_finite()) {
                    for g in &mut probe_gradient[..n] {
                        *g = f64::ZERO;
                    }
                    let candidate_value = objective(&candidate[..n], &mut probe_gradient[..n]);
                    if candidate_value.is_finite() && candidate_value >= current_value {
                        accepted_step_norm = FloatOps::sqrt(
                            gradient[..n]
                                .iter()
                                .map(|g| step * step * g * g)
                                .sum::<f64>(),
                        );
                        x.copy_from_slice(&candidate[..n]);
                        line_accepted = true;
                        break;
                    }
                }
                step *= 0.5;
            }
            if !line_accepted {
                break;
            }
        }

        if accepted_step_norm < step_tolerance {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DampedNewton;

    /// Damped Newton must drive a simple concave quadratic to its maximum, where the damping keeps the
    /// Newton step stable. The optimum of `-((x0 - 5)^2 + (x1 - 1)^2)` is at `(5, 1)`, and its Hessian
    /// is the constant `-2 I`.
    #[test]
    fn damped_newton_maximizes_quadratic() {
        let mut x = [0.0, 0.0];
        DampedNewton::maximize_map(
            &mut x,
            |x: &[f64], g: &mut [f64]| {
                g[0] = -2.0 * (x[0] - 5.0);
                g[1] = -2.0 * (x[1] - 1.0);
                -((x[0] - 5.0) * (x[0] - 5.0) + (x[1] - 1.0) * (x[1] - 1.0))
            },
            |_x: &[f64], hessian: &mut [f64]| {
                // Row-major 2x2 Hessian of the quadratic: -2 on the diagonal, 0 off-diagonal.
                hessian[0] = -2.0;
                hessian[1] = 0.0;
                hessian[2] = 0.0;
                hessian[3] = -2.0;
            },
            1e-9,
        );
        assert!(
            (x[0] - 5.0).abs() < 1e-4 && (x[1] - 1.0).abs() < 1e-4,
            "{x:?}"
        );
    }
}
