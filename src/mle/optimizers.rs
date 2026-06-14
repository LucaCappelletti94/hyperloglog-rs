//! Compile-time, type-level optimizers for the joint MLE refinement (selected via turbofish).

use crate::utils::Zero;
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use num_traits::Float;

/// A maximizer of a smooth objective, used to refine the joint-MLE warm start. The optimizer is
/// chosen at compile time by type (`O::maximize(..)`), not as a runtime value: each implementor is a
/// zero-sized marker with its hyperparameters as fixed `const`s. The `objective` closure returns the
/// value to MAXIMIZE and fills its ascent gradient into a pre-zeroed buffer; `step_tolerance` is the
/// convergence scale (the expected statistical error, `~1/sqrt(m)`).
pub trait JointOptimizer {
    /// Maximizes `objective` starting from `init`, returning the best point found. `objective(x,
    /// grad)` returns the value to maximize and writes its ascent gradient into the pre-zeroed
    /// `grad` buffer.
    ///
    /// # Examples
    /// Maximize `f(x) = -(x - 3)^2`, whose maximum is at `x = 3`.
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let result = Lbfgs::maximize(
    ///     vec![0.0],
    ///     |x: &[f64], grad: &mut [f64]| {
    ///         grad[0] = -2.0 * (x[0] - 3.0);
    ///         -(x[0] - 3.0).powi(2)
    ///     },
    ///     1e-9,
    /// );
    /// assert!((result[0] - 3.0).abs() < 1e-4);
    /// ```
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
///
/// # Examples
/// ```
/// use hyperloglog_rs::prelude::*;
/// // Maximize -((x0 - 1)^2 + (x1 + 2)^2); optimum at (1, -2).
/// let max = Lbfgs::maximize(
///     vec![0.0, 0.0],
///     |x: &[f64], g: &mut [f64]| {
///         g[0] = -2.0 * (x[0] - 1.0);
///         g[1] = -2.0 * (x[1] + 2.0);
///         -((x[0] - 1.0).powi(2) + (x[1] + 2.0).powi(2))
///     },
///     1e-9,
/// );
/// assert!((max[0] - 1.0).abs() < 1e-4 && (max[1] + 2.0).abs() < 1e-4);
/// ```
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
///
/// # Examples
/// ```
/// use hyperloglog_rs::prelude::*;
/// // Adam runs a fixed budget and returns the best point seen, so it converges loosely.
/// let max = Adam::maximize(
///     vec![0.0],
///     |x: &[f64], g: &mut [f64]| {
///         g[0] = -2.0 * (x[0] - 5.0);
///         -(x[0] - 5.0).powi(2)
///     },
///     1e-9,
/// );
/// assert!((max[0] - 5.0).abs() < 1e-2);
/// ```
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
///
/// # Examples
/// ```
/// use hyperloglog_rs::prelude::*;
/// let max = RmsProp::maximize(
///     vec![0.0],
///     |x: &[f64], g: &mut [f64]| {
///         g[0] = -2.0 * (x[0] + 4.0);
///         -(x[0] + 4.0).powi(2)
///     },
///     1e-9,
/// );
/// assert!((max[0] + 4.0).abs() < 1e-2);
/// ```
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
///
/// # Examples
/// ```
/// use hyperloglog_rs::prelude::*;
/// // The Adam warmup escapes poor basins, then L-BFGS polishes to full precision.
/// let max = <Chain<Adam, Lbfgs>>::maximize(
///     vec![0.0],
///     |x: &[f64], g: &mut [f64]| {
///         g[0] = -2.0 * (x[0] - 7.0);
///         -(x[0] - 7.0).powi(2)
///     },
///     1e-9,
/// );
/// assert!((max[0] - 7.0).abs() < 1e-4);
/// ```
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
