//! Dependency-free weighted least-squares polynomial fit that compresses an empirical bias curve into
//! per-cell coefficients. The objective is to fit `bias/m` against the domain-mapped load, weighted by
//! the RELATIVE downstream error (so the low-cardinality region is not drowned out
//! by the large biases at high cardinality), solved by Householder QR (numerically stable, no
//! normal-equations squaring, no external linear-algebra dependency).

/// Degree of the fitted polynomial. Nine coefficients (degree 8) keep the corrected estimate well
/// under the estimator's own noise floor at every precision.
pub const DEGREE: usize = 8;
/// Number of stored coefficients (`DEGREE + 1`).
pub const NCOEFF: usize = DEGREE + 1;

/// Floor on `exact/m` when forming the relative-error weight, so near-empty buckets cannot dominate
/// the loss with a tiny denominator (mirror of the Python `REL_WEIGHT_FLOOR`).
const REL_WEIGHT_FLOOR: f64 = 1e-3;

/// A fitted correction: monomial coefficients in the domain-mapped variable `u` in `[-1, 1]`, plus the
/// load domain `[t_lo, t_hi]` (with `t = raw / m`) the fit was mapped over.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct PolyFit {
    /// Monomial coefficients in `u`, low order first (`coeffs[0] + coeffs[1] u + ...`).
    pub coeffs: [f64; NCOEFF],
    /// Actual polynomial degree used (may be less than `DEGREE` if sample count is low).
    pub degree: usize,
    /// Lower end of the fit's load domain (`raw / m`).
    pub t_lo: f64,
    /// Upper end of the fit's load domain (`raw / m`).
    pub t_hi: f64,
}

impl PolyFit {
    /// Corrected cardinality from a raw estimate. This MUST stay identical to the library evaluation
    /// in `src/hyperloglog.rs`, so the generator's reported accuracy reflects the shipped behavior:
    /// clamp the load to the fit domain, map to `[-1, 1]`, Horner over the coefficients to get
    /// `bias / m`, and add it back.
    pub fn corrected(&self, raw: f64, m: f64) -> f64 {
        let t = (raw / m).clamp(self.t_lo, self.t_hi);
        let u = if self.t_hi > self.t_lo {
            2.0 * (t - self.t_lo) / (self.t_hi - self.t_lo) - 1.0
        } else {
            0.0
        };
        let deg = self.degree.min(NCOEFF - 1);
        let mut bias_over_m = self.coeffs[deg];
        for k in (0..deg).rev() {
            bias_over_m = bias_over_m * u + self.coeffs[k];
        }
        raw + m * bias_over_m
    }
}

/// One `(raw estimate, exact cardinality, sample count)` observation for the fit.
pub struct FitSample {
    /// The uncorrected (raw) estimate the correction keys on.
    pub raw: f64,
    /// The exact cardinality at this bucket.
    pub exact: f64,
    /// The number of measurements averaged into this bucket (the fit weight).
    pub count: f64,
}

/// Weighted least-squares fit of `bias/m` against the domain-mapped load, over the samples' load
/// range. Empty input yields the identity correction (all-zero coefficients).
pub fn fit(samples: &[FitSample], m: f64) -> PolyFit {
    fit_with_degree(samples, m, DEGREE)
}

/// Weighted least-squares polynomial fit at an arbitrary degree. The returned [`PolyFit`] stores
/// the actual degree used (clamped to the sample count and `NCOEFF - 1`).
pub fn fit_with_degree(samples: &[FitSample], m: f64, degree: usize) -> PolyFit {
    let mut t_lo = f64::INFINITY;
    let mut t_hi = f64::NEG_INFINITY;
    for s in samples {
        let t = s.raw / m;
        t_lo = t_lo.min(t);
        t_hi = t_hi.max(t);
    }
    let empty = PolyFit {
        coeffs: [0.0; NCOEFF],
        degree: 0,
        t_lo: 0.0,
        t_hi: 0.0,
    };
    if !(t_hi > t_lo) {
        return empty;
    }

    let n = samples.len();
    // Cap degree so that ncols = degree + 1 does not exceed sample count or storage.
    let effective_degree = degree.min(NCOEFF - 1).min(n - 1);
    let ncols = effective_degree + 1;
    if ncols == 0 {
        return empty;
    }
    // Row-major weighted design matrix A (n x ncols) and target b (n).
    let mut a = vec![0.0_f64; n * ncols];
    let mut b = vec![0.0_f64; n];
    for (i, s) in samples.iter().enumerate() {
        let t = s.raw / m;
        let yn = (s.exact - s.raw) / m;
        let u = 2.0 * (t - t_lo) / (t_hi - t_lo) - 1.0;
        let exact_over_m = (t + yn).max(REL_WEIGHT_FLOOR);
        let w = s.count.sqrt() / exact_over_m;
        let mut uk = 1.0;
        for j in 0..ncols {
            a[i * ncols + j] = w * uk;
            uk *= u;
        }
        b[i] = w * yn;
    }

    let solution = householder_qr_solve(&mut a, &mut b, n, ncols);
    let mut coeffs = [0.0_f64; NCOEFF];
    coeffs[..ncols].copy_from_slice(&solution[..ncols]);
    PolyFit {
        coeffs,
        degree: effective_degree,
        t_lo,
        t_hi,
    }
}

/// Solves the least-squares system `min ||A x - b||` for an `n x p` row-major matrix `A` (with
/// `p <= n`) by Householder QR. `A` and `b` are overwritten; the first `p` entries of the returned
/// vector are the solution. Stable for the modest degrees used here without forming `A^T A`.
fn householder_qr_solve(a: &mut [f64], b: &mut [f64], n: usize, p: usize) -> Vec<f64> {
    let mut v = vec![0.0_f64; n];
    for k in 0..p {
        // Householder reflection zeroing A[k+1.., k].
        let mut norm = 0.0;
        for i in k..n {
            let x = a[i * p + k];
            norm += x * x;
        }
        norm = norm.sqrt();
        if norm == 0.0 {
            continue;
        }
        let akk = a[k * p + k];
        let alpha = if akk >= 0.0 { -norm } else { norm };
        for i in k..n {
            v[i] = a[i * p + k];
        }
        v[k] -= alpha;
        let mut vnorm2 = 0.0;
        for i in k..n {
            vnorm2 += v[i] * v[i];
        }
        if vnorm2 == 0.0 {
            continue;
        }
        // Apply H = I - 2 v v^T / vnorm2 to the trailing columns of A and to b.
        for j in k..p {
            let mut dot = 0.0;
            for i in k..n {
                dot += v[i] * a[i * p + j];
            }
            let s = 2.0 * dot / vnorm2;
            for i in k..n {
                a[i * p + j] -= s * v[i];
            }
        }
        let mut dotb = 0.0;
        for i in k..n {
            dotb += v[i] * b[i];
        }
        let sb = 2.0 * dotb / vnorm2;
        for i in k..n {
            b[i] -= sb * v[i];
        }
    }

    // Back-substitution on the upper-triangular R (top p x p of A) against b[0..p].
    let mut x = vec![0.0_f64; p];
    for k in (0..p).rev() {
        let mut acc = b[k];
        for j in (k + 1)..p {
            acc -= a[k * p + j] * x[j];
        }
        let rkk = a[k * p + k];
        x[k] = if rkk != 0.0 { acc / rkk } else { 0.0 };
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds samples whose bias/m follows a known function `g(t)` of the load `t = raw/m`.
    fn samples_from(m: f64, raws: &[f64], g: impl Fn(f64) -> f64) -> Vec<FitSample> {
        raws.iter()
            .map(|&raw| {
                let yn = g(raw / m);
                FitSample {
                    raw,
                    exact: raw + m * yn,
                    count: 100.0,
                }
            })
            .collect()
    }

    /// A polynomial of degree <= 8 is recovered essentially exactly inside the fit domain.
    #[test]
    fn recovers_polynomial_within_domain() {
        let m = 4096.0;
        let g = |t: f64| 0.012 - 0.0021 * t + 0.00033 * t * t - 0.00001 * t * t * t;
        let raws: Vec<f64> = (0..200)
            .map(|i| 2.0 * m + 5.5 * m * (i as f64) / 199.0)
            .collect();
        let fitted = fit(&samples_from(m, &raws, g), m);
        for &raw in &raws {
            let exact = raw + m * g(raw / m);
            let corrected = fitted.corrected(raw, m);
            assert!(
                (corrected - exact).abs() <= 1e-4 * exact,
                "raw {raw}: corrected {corrected} vs exact {exact}",
            );
        }
    }

    /// A cell with fewer points than coefficients drops to a lower degree without panicking.
    #[test]
    fn handles_sparse_cell_without_panic() {
        let m = 64.0;
        let g = |t: f64| 0.05 - 0.01 * t; // linear, recoverable from 5 points
        let raws = [0.2 * m, 0.5 * m, 0.9 * m, 1.3 * m, 1.7 * m];
        let fitted = fit(&samples_from(m, &raws, g), m);
        for &raw in &raws {
            let exact = raw + m * g(raw / m);
            assert!((fitted.corrected(raw, m) - exact).abs() <= 1e-6 * exact);
        }
    }

    /// Outside the fit domain the load clamps to the nearest endpoint, so the bias is the endpoint
    /// value rather than an extrapolation.
    #[test]
    fn clamps_outside_domain() {
        let m = 4096.0;
        let g = |t: f64| 0.01 - 0.001 * t;
        let raws: Vec<f64> = (0..50)
            .map(|i| 2.0 * m + 4.0 * m * (i as f64) / 49.0)
            .collect();
        let fitted = fit(&samples_from(m, &raws, g), m);
        let at_lo = fitted.t_lo * m;
        let bias_at_lo = fitted.corrected(at_lo, m) - at_lo;
        let below = 0.5 * m;
        let corrected_below = fitted.corrected(below, m);
        assert!(
            (corrected_below - (below + bias_at_lo)).abs() <= 1e-6 * (below + bias_at_lo).abs()
        );
    }
}
