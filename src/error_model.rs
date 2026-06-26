//! Closed-form theoretical error of the dense-register cardinality estimators.
//!
//! For `m = 2^P` registers each capped at `q = 2^B - 1`, the register value `K` of a single register
//! after inserting `n` distinct elements follows, under the standard Poisson model,
//!
//! ```text
//! P(K = 0)   = (1 - 1/m)^n
//! P(K >= k)  = 1 - (1 - 2^-(k-1)/m)^n     for k = 1..=q   (the top bin q absorbs the tail)
//! ```
//!
//! Two quantities are derived from this distribution and exposed here:
//!
//! - the Cramer-Rao relative standard error of the maximum-likelihood estimator, from the Fisher
//!   information `I1(n) = sum_k (dP_k/dn)^2 / P_k` (total `m * I1`), as
//!   `RSE(n) = 1 / (n * sqrt(m * I1(n)))`. It equals the classic `1.04/sqrt(m)` in the normal range
//!   and rises to infinity as the registers saturate and the Fisher information collapses.
//! - the systematic relative bias of the default raw `alpha * m^2 / harmonic_sum` estimator, from the
//!   expected harmonic sum `E[harmonic] = m * sum_k P_k * 2^-k`, as `raw/n - 1`. It is negligible in
//!   the normal range and tends to -1 at saturation (the raw estimate flatlines at its ceiling).
//!
//! The formulas were validated against measured estimator error (see `examples/saturation_study.rs`
//! and `docs/saturation_theory_check.png`). All math uses [`FloatOps`] so the module stays `no_std`.

use crate::mle::REGISTER_MULTIPLICITIES_CAPACITY;
use crate::prelude::{Bits, Precision};
use crate::utils::FloatOps;

/// `(dP/dn)^2 / P`, the Fisher-information contribution of one register-value bin. An empty bin (no
/// probability mass) carries no information and contributes zero.
#[inline]
fn fisher_term(probability: f64, derivative: f64) -> f64 {
    if probability > 1e-300 {
        derivative * derivative / probability
    } else {
        0.0
    }
}

/// Returns `(fisher_info_per_register, expected_harmonic_sum)` at cardinality `n` for the register
/// value distribution of an `(P, B)` counter.
fn register_fisher_and_harmonic<P: Precision, B: Bits>(n: f64) -> (f64, f64) {
    let m = f64::integer_exp2(P::EXPONENT);
    let q = (1_u8 << B::NUMBER_OF_BITS) - 1; // the saturation cap register value
    debug_assert!(usize::from(q) < REGISTER_MULTIPLICITIES_CAPACITY);

    // P(K >= k) and its n-derivative for k = 1..=q. Here `a_k = 2^-(k-1)/m` is the per-element
    // probability of pushing a register to at least value k, so P(K < k) = (1 - a_k)^n.
    let mut greater_equal = [0.0_f64; REGISTER_MULTIPLICITIES_CAPACITY];
    let mut greater_equal_derivative = [0.0_f64; REGISTER_MULTIPLICITIES_CAPACITY];
    for k in 1..=q {
        let a = f64::integer_exp2_minus(k - 1) / m;
        let log_survival = FloatOps::ln_1p(-a); // ln(1 - a_k), stable for tiny a
        let survival = FloatOps::exp(n * log_survival); // (1 - a_k)^n = P(K < k)
        greater_equal[usize::from(k)] = 1.0 - survival;
        greater_equal_derivative[usize::from(k)] = -survival * log_survival; // d/dn P(K >= k) >= 0
    }

    // k = 0: the empty register, P(K = 0) = (1 - 1/m)^n.
    let log_survival_zero = FloatOps::ln_1p(-1.0 / m);
    let survival_zero = FloatOps::exp(n * log_survival_zero);
    let mut fisher = fisher_term(survival_zero, survival_zero * log_survival_zero);
    let mut harmonic = survival_zero; // weight 2^0 = 1

    // k = 1..q-1: P(K = k) = P(K >= k) - P(K >= k+1).
    for k in 1..q {
        let probability = greater_equal[usize::from(k)] - greater_equal[usize::from(k + 1)];
        let derivative =
            greater_equal_derivative[usize::from(k)] - greater_equal_derivative[usize::from(k + 1)];
        fisher += fisher_term(probability, derivative);
        harmonic += probability * f64::integer_exp2_minus(k);
    }
    // k = q: the saturation bin absorbs the geometric tail, P(K = q) = P(K >= q).
    fisher += fisher_term(
        greater_equal[usize::from(q)],
        greater_equal_derivative[usize::from(q)],
    );
    harmonic += greater_equal[usize::from(q)] * f64::integer_exp2_minus(q);

    (fisher, harmonic * m)
}

/// Cramer-Rao relative standard error of the maximum-likelihood register estimator at cardinality
/// `n`. Equals roughly `1.04/sqrt(2^P)` in the normal range and grows without bound as the registers
/// saturate (the Fisher information collapses to zero). Returns `0` for a non-positive `n` and
/// `f64::INFINITY` at full saturation.
#[must_use]
pub fn register_crlb_relative_standard_error<P: Precision, B: Bits>(n: f64) -> f64 {
    if n <= 0.0 {
        return 0.0;
    }
    let m = f64::integer_exp2(P::EXPONENT);
    let (fisher, _) = register_fisher_and_harmonic::<P, B>(n);
    if fisher <= 0.0 {
        return f64::INFINITY;
    }
    1.0 / (n * FloatOps::sqrt(m * fisher))
}

/// Systematic relative bias of the default raw `alpha * m^2 / harmonic_sum` register estimator at the
/// given *true* cardinality `n`. Negligible in the normal range, tending to -1 as the registers
/// saturate and the raw estimate flatlines at its ceiling `alpha * m * 2^(2^B - 1)`. Returns `0` for
/// a non-positive `n`.
#[must_use]
pub fn register_raw_bias<P: Precision, B: Bits>(n: f64) -> f64 {
    if n <= 0.0 {
        return 0.0;
    }
    let m = f64::integer_exp2(P::EXPONENT);
    let (_, harmonic) = register_fisher_and_harmonic::<P, B>(n);
    let raw = P::ALPHA * m * m / harmonic;
    raw / n - 1.0
}

/// Flat-region relative standard error of the default register estimator, the classic
/// `1.04/sqrt(2^P)` (see [`Precision::error_rate`]). The default estimator's variance does not grow
/// at saturation (it flatlines deterministically); its saturation error is the bias of
/// [`register_raw_bias`], not the variance.
#[must_use]
pub fn register_default_relative_standard_error<P: Precision>() -> f64 {
    P::error_rate()
}

/// The cardinality above which the maximum-likelihood register estimator becomes the more accurate
/// choice over the default raw `alpha * m^2 / harmonic_sum` estimate: the smallest `n` at which the
/// raw estimator's saturation bias ([`register_raw_bias`]) exceeds the MLE's standard error
/// ([`register_crlb_relative_standard_error`]).
///
/// Below the threshold the raw bias is within the MLE's own noise, so the far cheaper raw estimate is
/// just as good. Above it the bias dominates the error and the near-unbiased MLE wins, up to full
/// saturation where the MLE in turn diverges (so the MLE is the better choice on a window starting
/// here, not forever).
///
/// Returns `None` when no such crossover exists below a practical cardinality, which is the case for
/// registers wide enough never to saturate in practice (for example `Bits6`): there the raw estimator
/// stays unbiased and the MLE never repays its much higher cost. Note this is an accuracy crossover
/// only; the MLE is far slower, so being marginally past the threshold does not by itself justify it.
#[must_use]
pub fn mle_preferred_threshold<P: Precision, B: Bits>() -> Option<f64> {
    let m = f64::integer_exp2(P::EXPONENT);
    // The raw estimate cannot exceed its ceiling alpha * m * 2^(2^B - 1); the crossover, when it
    // exists, lies below it. Cap the search at a practical cardinality so the never-saturating
    // wide-register case terminates with `None` rather than an astronomically large threshold.
    let saturation_cap = (1_u8 << B::NUMBER_OF_BITS) - 1;
    let ceiling = P::ALPHA * m * f64::integer_exp2(saturation_cap);
    let cap = FloatOps::maximum(ceiling, 0.0).min(1.0e15);

    // `gap(n) = |raw bias| - MLE standard error`. The raw estimator is biased at BOTH ends: at low
    // load (the linear-counting regime, where the raw estimate is not even used) and at saturation.
    // We want the saturation crossing, so we start in the raw regime (above the correction bound
    // `7.5 * 2^P`, where the raw estimate is actually used), advance past any residual low-load bias
    // until the gap goes negative (the near-unbiased middle), and take the next upward crossing.
    let gap = |n: f64| {
        FloatOps::abs(register_raw_bias::<P, B>(n))
            - register_crlb_relative_standard_error::<P, B>(n)
    };

    let mut n = FloatOps::maximum(7.5 * m, 1.0);
    // Advance to the near-unbiased middle (gap < 0).
    while n <= cap && gap(n) >= 0.0 {
        n *= 1.3;
    }
    if n > cap {
        return None; // no clean middle: the regimes overlap, treat as no meaningful crossover
    }
    // Find the saturation crossing (gap returns to >= 0).
    let mut previous = n;
    n *= 1.3;
    while n <= cap {
        if gap(n) >= 0.0 {
            // Bisect the bracket `[previous, n]` for a tight threshold.
            let (mut low, mut high) = (previous, n);
            for _ in 0..60 {
                let mid = 0.5 * (low + high);
                if gap(mid) >= 0.0 {
                    high = mid;
                } else {
                    low = mid;
                }
            }
            return Some(high);
        }
        previous = n;
        n *= 1.3;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{Bits4, Bits6, Precision10, Precision12, Precision6, Precision8};

    #[test]
    fn crlb_matches_classic_error_rate_in_flat_range() {
        // At moderate register load (n/m well above 1) the Cramer-Rao standard error sits at the
        // classic 1.04/sqrt(m) noise floor. (At very low load, n/m ~ 1, it is higher, which is the
        // regime linear counting is for; that is not tested here.)
        for (rse, classic) in [
            (
                register_crlb_relative_standard_error::<Precision10, Bits6>(5000.0),
                Precision10::error_rate(),
            ),
            (
                register_crlb_relative_standard_error::<Precision8, Bits6>(8000.0),
                Precision8::error_rate(),
            ),
        ] {
            assert!(
                rse > 0.9 * classic && rse < 1.1 * classic,
                "crlb {rse} vs classic {classic}"
            );
        }
    }

    #[test]
    fn crlb_blows_up_at_saturation() {
        // Tiny registers (Bits4, cap 15) fully saturate well beyond the raw ceiling alpha*m*2^15; the
        // standard error stays near the floor in range and blows up far past it as the Fisher
        // information collapses.
        let flat = Precision6::error_rate();
        let near = register_crlb_relative_standard_error::<Precision6, Bits4>(1.0e4);
        let mid = register_crlb_relative_standard_error::<Precision6, Bits4>(3.0e6);
        let deep = register_crlb_relative_standard_error::<Precision6, Bits4>(1.0e7);
        assert!(
            near < 1.5 * flat,
            "near-range crlb {near} should be ~ floor {flat}"
        );
        assert!(
            deep > 5.0 * flat,
            "deep crlb {deep} should blow up past floor {flat}"
        );
        assert!(
            near < mid && mid < deep,
            "crlb must be monotone into saturation"
        );
    }

    #[test]
    fn raw_bias_negligible_then_collapses() {
        // The raw estimator is near-unbiased in range and collapses toward -1 at saturation.
        let in_range = register_raw_bias::<Precision6, Bits4>(1.0e4);
        let saturated = register_raw_bias::<Precision6, Bits4>(3.0e6);
        assert!(
            in_range.abs() < 0.05,
            "in-range raw bias {in_range} should be ~0"
        );
        assert!(
            saturated < -0.3,
            "saturated raw bias {saturated} should be strongly negative"
        );
    }

    #[test]
    fn mle_preferred_threshold_matches_measured_crossover() {
        // Tiny registers saturate, so a finite crossover exists, near the measured saturation
        // crossovers (P6B4 ~7e5, P8B4 ~3e6 from the saturation study).
        let p6b4 = mle_preferred_threshold::<Precision6, Bits4>().expect("P6B4 has a crossover");
        assert!(
            (3.0e5..1.2e6).contains(&p6b4),
            "P6B4 threshold {p6b4} out of expected range"
        );
        let p8b4 = mle_preferred_threshold::<Precision8, Bits4>().expect("P8B4 has a crossover");
        assert!(
            (1.5e6..5.0e6).contains(&p8b4),
            "P8B4 threshold {p8b4} out of expected range"
        );
        // Wide registers never saturate in the practical range, so the MLE is never worth it.
        assert!(mle_preferred_threshold::<Precision10, Bits6>().is_none());
        assert!(mle_preferred_threshold::<Precision12, Bits6>().is_none());
    }

    #[test]
    fn bias_overtakes_variance_in_saturation() {
        // The whole point: in range the variance dominates the error, but at saturation the default
        // estimator's bias overtakes the MLE standard error. This is the measured crossover.
        let n_range = 1.0e4;
        let n_sat = 2.0e6;
        let bias_range = register_raw_bias::<Precision6, Bits4>(n_range).abs();
        let rse_range = register_crlb_relative_standard_error::<Precision6, Bits4>(n_range);
        let bias_sat = register_raw_bias::<Precision6, Bits4>(n_sat).abs();
        let rse_sat = register_crlb_relative_standard_error::<Precision6, Bits4>(n_sat);
        assert!(
            bias_range < rse_range,
            "in range: bias {bias_range} < rse {rse_range}"
        );
        assert!(
            bias_sat > rse_sat,
            "saturation: bias {bias_sat} > rse {rse_sat}"
        );
    }
}
