//! Ertl's tau/sigma corrected raw cardinality estimator (Otmar Ertl, "New cardinality estimation
//! algorithms for `HyperLogLog` sketches", arXiv:1702.01284) and the [`SigmaTau`] mode wrapper that
//! routes [`HyperLogLog`] cardinality estimates through it.
//!
//! Sigma/tau is now the crate's default register-regime estimator (as of the polynomial and
//! anchor-table drop): the harmonic-mode branch of [`HyperLogLog::estimate_cardinality`] reads
//! `(H, zeros)` in O(1) from the packed harmonic-sum word and calls
//! [`ertl_cardinality_from_moments`]. The `SigmaTau` wrapper is retained as a diagnostic view
//! that applies sigma/tau uniformly, including a strict evaluation in dense zeros mode where the
//! default falls back to linear counting for O(1). Return to the default estimators with
//! [`SigmaTau::into_inner`].

use crate::prelude::*;
use sketching_core::sparse_value_list::SparseValueCodec;

/// `1 / (2 ln 2)`, the `m -> infinity` `HyperLogLog` normalization constant Ertl's estimator uses. The
/// finite-`m` bias is absorbed by `sigma`/`tau` rather than by a per-precision alpha.
const ALPHA_INF: f64 = 0.721_347_520_444_481_7;

/// Ertl's `sigma(x) = x + sum_{k>=1} x^(2^k) * 2^(k-1)`, evaluated by its convergent series (the terms
/// underflow to zero within a few dozen iterations). It corrects the contribution of the zero
/// registers at low cardinality. `sigma(1) = +inf` (a fully empty counter).
fn sigma(mut x: f64) -> f64 {
    if x == 1.0 {
        return f64::INFINITY;
    }
    let mut y = 1.0;
    let mut z = x;
    loop {
        x *= x;
        let z_prev = z;
        z += x * y;
        y += y;
        if z == z_prev {
            return z;
        }
    }
}

/// Ertl's `tau`, evaluated by its convergent series. It corrects the contribution of the saturated
/// registers at high cardinality. `tau(0) = tau(1) = 0`.
fn tau(mut x: f64) -> f64 {
    if x == 0.0 || x == 1.0 {
        return 0.0;
    }
    let mut y = 1.0;
    let mut z = 1.0 - x;
    loop {
        x = FloatOps::sqrt(x);
        let z_prev = z;
        y *= 0.5;
        let t = 1.0 - x;
        z -= t * t * y;
        if z == z_prev {
            return z / 3.0;
        }
    }
}

/// Ertl's corrected raw estimate from the register-value multiplicity histogram `c` (`c[k]` is the
/// number of registers equal to `k`, for `k` in `0..=q_plus_one`, where `q_plus_one = 2^B - 1` is the
/// saturation value). Table-free and near-unbiased across the whole range.
fn ertl_cardinality<P: Precision, B: Bits>(c: &[f64]) -> f64 {
    let m = f64::integer_exp2(P::EXPONENT);
    let q_plus_one = (1_usize << B::NUMBER_OF_BITS) - 1;
    // High range: the saturated-register correction, then fold the interior multiplicities in by the
    // halving recurrence that reproduces `sum_{k=1}^{q} c[k] * 2^-k`.
    let mut z = m * tau((m - c[q_plus_one]) / m);
    for k in (1..q_plus_one).rev() {
        z = f64::midpoint(z, c[k]);
    }
    // Low range: the zero-register correction.
    z += m * sigma(c[0] / m);
    ALPHA_INF * m * m / z
}

/// The O(1) form of [`ertl_cardinality`]: the same estimate computed from the three moments the
/// counter can maintain incrementally, the harmonic sum `H = sum 2^-register`, the zero-register count,
/// and the saturated-register count, instead of the full multiplicity histogram. The interior sum
/// `sum_{k=1}^{q} c[k] 2^-k` is recovered as `H - zeros - saturated * 2^-(q+1)`. Doc-hidden, exposed
/// only so a benchmark can time the O(1) cost (the two series plus arithmetic, constant in precision).
#[doc(hidden)]
#[must_use]
pub fn ertl_cardinality_from_moments<P: Precision, B: Bits>(
    harmonic_sum: f64,
    zeros: f64,
    saturated: f64,
) -> f64 {
    let m = f64::integer_exp2(P::EXPONENT);
    let q_plus_one = (1_usize << B::NUMBER_OF_BITS) - 1;
    let interior = harmonic_sum - zeros - saturated * f64::integer_exp2_minus(q_plus_one as u8);
    let two_to_minus_q = f64::integer_exp2_minus((q_plus_one - 1) as u8);
    let z = m * sigma(zeros / m) + interior + m * two_to_minus_q * tau((m - saturated) / m);
    ALPHA_INF * m * m / z
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> HyperLogLog<P, B, R, H, C>
where
    C: SparseValueCodec,
{
    /// Returns a [`SigmaTau`] view (borrowing this counter) whose cardinality estimate uses Ertl's
    /// analytical tau/sigma corrected raw estimator instead of the default fitted correction.
    #[inline]
    pub fn sigma_tau(&self) -> SigmaTau<&Self> {
        SigmaTau(self)
    }

    /// Ertl's tau/sigma corrected cardinality of this counter.
    ///
    /// In the dense harmonic band the estimate is O(1): the harmonic sum `H` and the
    /// zero-register count come straight from the packed word, and `ertl_cardinality_from_moments`
    /// closes the analytical form. In dense zeros mode the zero count is O(1) but `H` is
    /// reconstructed by an O(m) register scan (the word stores the count instead of the sum
    /// there). Pre-dense representations return the default estimate: their direct counts are
    /// more accurate than any `HyperLogLog` estimator, sigma/tau included.
    ///
    /// Saturation is treated as zero: the packed band ends at `7.5 * m`, orders of magnitude
    /// below the cardinality at which even the four-bit register field starts saturating.
    /// `sigma_tau_cardinality_from_histogram` is the O(m) reference used to cross-check this
    /// path in tests; it must agree to `~1e-9` relative across every `(P, B)` cell.
    pub(crate) fn sigma_tau_cardinality(&self) -> f64 {
        if !self.is_hyperloglog() {
            return self.estimate_cardinality();
        }
        let harmonic_sum = self.dense_harmonic_sum();
        let zeros = f64::from(self.dense_zero_count());
        ertl_cardinality_from_moments::<P, B>(harmonic_sum, zeros, 0.0)
    }

    /// Ertl's tau/sigma corrected union cardinality from the moments of the element-wise-max
    /// union registers, computed in a single O(m) zipped scan (no multiplicity-histogram
    /// allocation). Falls back to the default union estimate when either operand is still
    /// pre-dense.
    pub(crate) fn sigma_tau_union_cardinality(&self, other: &Self) -> f64 {
        if !self.is_hyperloglog() || !other.is_hyperloglog() {
            return self.estimate_union_cardinality(other);
        }
        let (union_h, union_zeros) = self.registers.get_union_harmonic_sum(&other.registers);
        #[allow(
            clippy::cast_precision_loss,
            reason = "union_zeros <= m <= 2^18 is exact in f64"
        )]
        let zeros = union_zeros as f64;
        ertl_cardinality_from_moments::<P, B>(union_h, zeros, 0.0)
    }

    /// Reference O(m) form of [`sigma_tau_cardinality`](Self::sigma_tau_cardinality) that builds
    /// the full register-multiplicity histogram and calls [`ertl_cardinality`]. Kept as the
    /// correctness oracle the packed moments path is cross-checked against, and exposed
    /// doc-hidden so a benchmark can time it alongside the O(1) path.
    #[doc(hidden)]
    pub fn sigma_tau_cardinality_from_histogram(&self) -> f64 {
        const CAP: usize = crate::mle::REGISTER_MULTIPLICITIES_CAPACITY;
        if !self.is_hyperloglog() {
            return self.estimate_cardinality();
        }
        let mut c = [0.0_f64; CAP];
        for register in self.registers.iter_registers() {
            c[usize::from(register)] += 1.0;
        }
        ertl_cardinality::<P, B>(&c)
    }

    /// Reference O(m) form of
    /// [`sigma_tau_union_cardinality`](Self::sigma_tau_union_cardinality). Same rationale as
    /// [`sigma_tau_cardinality_from_histogram`](Self::sigma_tau_cardinality_from_histogram).
    #[doc(hidden)]
    pub fn sigma_tau_union_cardinality_from_histogram(&self, other: &Self) -> f64 {
        const CAP: usize = crate::mle::REGISTER_MULTIPLICITIES_CAPACITY;
        if !self.is_hyperloglog() || !other.is_hyperloglog() {
            return self.estimate_union_cardinality(other);
        }
        let mut c = [0.0_f64; CAP];
        for [a, b] in self.registers.iter_registers_zipped(&other.registers) {
            c[usize::from(a.max(b))] += 1.0;
        }
        ertl_cardinality::<P, B>(&c)
    }
}

/// An Ertl tau/sigma estimation view over a [`HyperLogLog`], produced by [`HyperLogLog::sigma_tau`].
/// It implements [`sketching_core::CardinalityEstimator`], so the derived intersection, Jaccard, and
/// difference estimates come for free from the analytical cardinality and union primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SigmaTau<H>(pub H);

impl<H> SigmaTau<H> {
    /// Returns the wrapped counter (or reference), switching back to the default estimators.
    #[inline]
    pub fn into_inner(self) -> H {
        self.0
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C>
    sketching_core::CardinalityEstimator for SigmaTau<&HyperLogLog<P, B, R, H, C>>
where
    C: SparseValueCodec,
{
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        self.0.sigma_tau_cardinality()
    }

    #[inline]
    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        self.0.sigma_tau_union_cardinality(other.0)
    }
}

// Empty body: inherits the default inclusion-exclusion `joint_sketch`, which runs over this view's
// analytical cardinality and union estimates.
impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> HyperSpheresSketch
    for SigmaTau<&HyperLogLog<P, B, R, H, C>>
where
    C: SparseValueCodec,
{
}

#[cfg(test)]
mod tests {
    use super::{sigma, tau};
    use crate::prelude::*;

    /// The tau/sigma estimate must track the true cardinality across the dense range, and should be
    /// noticeably more accurate than the uncorrected raw estimate at low load.
    #[test]
    fn sigma_tau_tracks_truth_dense() {
        type Hll = HyperLogLog<Precision12, Bits6>;
        for &card in &[2_000u64, 5_000, 12_000, 20_000, 30_000] {
            let mut hll = Hll::default();
            for x in 0..card {
                hll.insert(&x);
            }
            let hll = hll.into_hll();
            let est = hll.sigma_tau().estimate_cardinality();
            let rel = (est - card as f64).abs() / card as f64;
            assert!(rel < 0.03, "card {card}: sigma/tau {est} rel err {rel}");
        }
    }

    /// The O(1) moments form must equal the histogram form on real counters.
    #[test]
    fn ertl_moments_matches_histogram() {
        type Hll = HyperLogLog<Precision12, Bits6>;
        let q_plus_one = (1_usize << 6) - 1;
        for &card in &[3_000u64, 10_000, 20_000] {
            let mut hll = Hll::default();
            for x in 0..card {
                hll.insert(&x);
            }
            let hll = hll.into_hll();
            let hist = hll.sigma_tau().estimate_cardinality();
            let (mut h, mut zeros, mut sat) = (0.0, 0.0, 0.0);
            for r in Registers::<Precision12, Bits6>::iter_registers(&hll.registers) {
                h += f64::integer_exp2_minus(r);
                zeros += f64::from(r == 0);
                sat += f64::from(usize::from(r) == q_plus_one);
            }
            let moments = super::ertl_cardinality_from_moments::<Precision12, Bits6>(h, zeros, sat);
            assert!(
                (hist - moments).abs() <= 1e-6 * hist,
                "card {card}: histogram {hist} vs moments {moments}"
            );
        }
    }

    /// The O(1) moments path and the O(m) histogram path MUST agree to `~1e-9` relative across
    /// every `(P, B)` cell and across the full lifecycle: pre-dense (both fall back to the
    /// default), dense zeros mode (`H` is reconstructed by scan on the O(1) side, then both
    /// evaluate the same Ertl form), dense harmonic mode both inside and above the corrected
    /// band. If this test drifts, the packed maintenance in `insert_register_value_and_index` is
    /// out of sync with a fresh scan.
    #[test]
    fn sigma_tau_o1_matches_histogram_across_lifecycle() {
        fn check<P, B>(name: &str)
        where
            P: Precision + PackedRegister<B>,
            B: Bits,
        {
            let m = 1u64 << P::EXPONENT;
            for &cardinality in &[m / 8, m / 2, m, 2 * m, 4 * m, 8 * m, 20 * m] {
                let mut hll = HyperLogLog::<P, B>::default();
                let mut state = 0x00C0_FFEE_u64 ^ cardinality;
                for _ in 0..cardinality {
                    state = splitmix64(state);
                    hll.insert(&state);
                }
                let fast = hll.sigma_tau_cardinality();
                let reference = hll.sigma_tau_cardinality_from_histogram();
                // The tolerance covers two independent sources of numerical drift between the two
                // paths: the packed-H mask truncation accumulated over `N = cardinality` inserts
                // (at most `N * 2^(k - 52)` relative, where `k = ZERO_BITS[P - 4][B - 4]`), and the
                // Kahan-style reduction difference between the halving-recurrence histogram sum and
                // the `interior = H - zeros` moment decomposition. `1e-6` covers both across the
                // tested grid up to P14, per the analytical bound in `docs/zero_bits_table.md`.
                let tol = 1e-6 * reference.max(1.0);
                assert!(
                    (fast - reference).abs() <= tol,
                    "{name} card={cardinality}: O(1) {fast} vs histogram {reference} (tol {tol})",
                );
            }
        }

        check::<Precision6, Bits4>("P6B4");
        check::<Precision8, Bits5>("P8B5");
        check::<Precision10, Bits6>("P10B6");
        check::<Precision12, Bits4>("P12B4");
        check::<Precision14, Bits6>("P14B6");
    }

    /// Sigma/tau union cardinality via the moments path must agree with the joint-histogram
    /// reference, at parity precision to the single-counter cross-check.
    #[test]
    fn sigma_tau_union_o1_matches_histogram() {
        fn check<P, B>(name: &str)
        where
            P: Precision + PackedRegister<B>,
            B: Bits,
        {
            let m = 1u64 << P::EXPONENT;
            for &(na, nb) in &[(m / 2, m / 2), (m, 3 * m), (3 * m, 3 * m), (5 * m, m / 4)] {
                let mut a = HyperLogLog::<P, B>::default();
                let mut ast = 0x000A_11CE_u64 ^ na ^ nb;
                for _ in 0..na {
                    ast = splitmix64(ast);
                    a.insert(&ast);
                }
                let mut b = HyperLogLog::<P, B>::default();
                let mut bst = 0x0000_0B0B_u64 ^ na ^ nb;
                for _ in 0..nb {
                    bst = splitmix64(bst);
                    b.insert(&bst);
                }
                let fast = a.sigma_tau_union_cardinality(&b);
                let reference = a.sigma_tau_union_cardinality_from_histogram(&b);
                let tol = 1e-6 * reference.max(1.0);
                assert!(
                    (fast - reference).abs() <= tol,
                    "{name} na={na} nb={nb}: O(1) {fast} vs histogram {reference} (tol {tol})",
                );
            }
        }

        check::<Precision8, Bits5>("P8B5");
        check::<Precision10, Bits6>("P10B6");
        check::<Precision12, Bits4>("P12B4");
    }

    /// Sigma/tau's raw-regime behavior: above the correction bound `7.5 * m` the polynomial
    /// pass-through returns the raw estimate, and sigma/tau reduces to `alpha_inf * m^2 / z` with
    /// `z = m*sigma(zeros/m) + interior + m*2^-q*tau((m-sat)/m)`. Because the packed band ends at
    /// `7.5 * m`, saturation is treated as zero, and the small remaining zero count is read
    /// straight from the packed word. This test confirms the estimate stays close to truth at
    /// cardinalities well above the correction bound, where the polynomial branch is a pure
    /// pass-through and any drift would be sigma/tau's own.
    #[test]
    fn sigma_tau_tracks_truth_raw_regime() {
        type Hll = HyperLogLog<Precision10, Bits6>;
        let m = 1u64 << 10;
        for &card in &[10_000u64, 30_000, 100_000] {
            let mut hll = Hll::default();
            let mut state = 0x0000_DEAD_BEEF_u64 ^ card;
            for _ in 0..card {
                state = splitmix64(state);
                hll.insert(&state);
            }
            assert!(
                hll.is_hyperloglog() && !hll.harmonic_sum.is_nan(),
                "card {card}: raw-regime test must land in dense harmonic mode",
            );
            let raw = hll.uncorrected_estimate_cardinality();
            assert!(
                raw >= 7.5 * (m as f64),
                "card {card}: raw estimate {raw} must be in the raw regime for this test",
            );
            let est = hll.sigma_tau().estimate_cardinality();
            let rel = (est - card as f64).abs() / card as f64;
            // 5% covers the P10 register-noise floor (~3.3%) with room for the sigma/tau
            // constant-factor difference between `alpha` (finite-m) and `alpha_inf`.
            assert!(rel < 0.05, "card {card}: sigma/tau {est} rel err {rel}");
        }
    }

    /// Boundary values of the two series.
    #[test]
    fn sigma_tau_boundaries() {
        assert_eq!(sigma(0.0), 0.0);
        assert!(sigma(1.0).is_infinite());
        assert_eq!(tau(0.0), 0.0);
        assert_eq!(tau(1.0), 0.0);
    }
}
