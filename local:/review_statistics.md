# Review: Occupancy-Corrected Sparse Representation for Cardinality Estimation

**Reviewer:** Statistics / Estimation Theory
**Journal lens:** *Journal of the American Statistical Association* or *Annals of Statistics*

---

## Summary

This paper presents a three-stage adaptive cardinality estimator for streaming data. The core novelty is a hash-list representation whose cardinality estimate is obtained by inverting an analytically derived occupancy function (the expected number of distinct composites as a function of cardinality). The inversion is performed via safeguarded Newton iteration. The estimator targets the low-cardinality regime where standard HyperLogLog and its variants suffer from empty-register noise.

The paper's main empirical claim is that the occupancy-inverse estimator achieves a cardinality-weighted mean absolute relative error (MARE) of 0.86% over its operating range, outperforming linear counting (2.20%), register MLE (1.98%), and SetSketch (3.83%) at precision P=10 and register width B=6.

From a statistical standpoint, the occupancy-inverse estimator is an M-estimator: it solves the estimating equation g(n) - D = 0 where g is the theoretical expectation and D is the observed statistic. The asymptotic theory for M-estimators is well-established (Wald 1949, Mann and Wald 1943), but the paper's finite-sample analysis is incomplete. The key statistical concerns are: (i) the Jensen's inequality bias from applying a nonlinear inverse to a random variable, (ii) the delta-method variance approximation that ignores multinomial dependence, (iii) the choice of cardinality-weighted MARE as the primary metric, and (iv) the fairness of the SetSketch comparison.

---

## Strengths

1. **Analytically derived occupancy function.** The occupancy model (Theorem 1) correctly accounts for the non-uniform cell probabilities induced by the composite encoding. Grouping cells by probability and summing E[D|n,w] = sum_j g_j (1 - (1 - p_j)^n) is the standard occupancy formula applied to a heterogeneous multinomial. The derivation is correct and more careful than what appears in most streaming-algorithms papers, which typically assume uniform hashing into a fixed number of cells.

2. **M-estimator framework is natural.** Inverting g(n) = D to obtain n-hat is the canonical M-estimator approach. The safeguarded Newton iteration with bisection fallback is a sound numerical implementation. The derivative g'(n) is computed analytically, which is efficient and exact.

3. **Three-stage architecture is well-motivated.** The progression from exact value list through hash list to dense registers is a principled approach that avoids probabilistic estimation when the data permits exact counting. This is statistically honest.

4. **Empirical evaluation is thorough.** 256 random seeds per cardinality is a reasonable number for Monte Carlo estimation. The cardinality-weighted MARE metric appropriately emphasizes the middle of the regime where errors are largest.

5. **Jensen's inequality bias is acknowledged.** The paper correctly identifies that E[g^{-1}(D)] != g^{-1}(E[D]) = n by Jensen's inequality, and reports that empirical MRE is near zero, suggesting the bias is small.

---

## Weaknesses

### 1. Jensen's inequality bias is asserted but not quantified

The paper states that the Jensen's inequality bias is "negligible in practice" and points to near-zero MRE in the error table. This is insufficient for a statistical review.

The bias can be quantified via a second-order Taylor expansion. Let D = g(n) + epsilon where epsilon = D - g(n) has mean zero and variance sigma_D^2. Then:

E[n-hat] = E[g^{-1}(D)] approx g^{-1}(E[D]) + (1/2) g^{-1}''(g(n)) sigma_D^2

The first derivative g^{-1}'(y) = 1 / g'(g^{-1}(y)), and the second derivative:

g^{-1}''(y) = -g''(g^{-1}(y)) / [g'(g^{-1}(y))]^3

The bias is therefore:

Bias approx -(1/2) g''(n) sigma_D^2 / [g'(n)]^3

For the occupancy function g(n) = sum_j g_j (1 - (1 - p_j)^n), we have:
- g'(n) = sum_j g_j (-ln(1-p_j)) (1-p_j)^n > 0
- g''(n) = sum_j g_j [ln(1-p_j)]^2 (1-p_j)^n > 0

Since g''(n) > 0 and g'(n) > 0, the bias is **negative** (the estimator is biased downward). This is consistent with the empirical MRE of -0.02% reported in Table 1, but the magnitude should be explained.

The paper does not provide a formula for sigma_D^2. For the multinomial occupancy problem, Var(D) = sum_j g_j (1-p_j)^n - sum_j g_j^2 (1-p_j)^{2n} + (sum_j g_j (1-p_j)^n)^2. The delta-method variance in the paper treats cell indicators as independent, which gives Var(D) approx sum_j g_j (1-p_j)^n (1 - (1-p_j)^n). The true variance includes negative covariance terms between cells (since the total number of balls n is fixed, the occupancy indicators are negatively correlated). The paper's delta-method variance is therefore an **overestimate** of the true variance, which means the true Jensen's bias is even smaller than the delta-method would suggest.

**Recommendation:** The paper should provide a quantitative bound or formula for the Jensen's bias, at least in terms of the occupancy function's derivatives and the variance of D. The fact that the empirical MRE is -0.02% is evidence, but the mechanism should be made explicit.

### 2. Delta-method variance ignores multinomial dependence

The paper acknowledges that the delta-method approximation treats cell indicators as independent, but claims it is "accurate to within a few percent." This claim needs substantiation.

For a multinomial distribution with n balls and cells of probabilities p_1, ..., p_K, the indicator I_j that cell j is occupied has:
- Var(I_j) = p_j (1 - p_j) [1 - p_j^{n-1}] (approximately p_j for small p_j)
- Cov(I_j, I_k) = -p_j p_k [1 - (1 - p_j - p_k)^{n-1}] for j != k (negative covariance)

The sum D = sum_j I_j has variance:

Var(D) = sum_j Var(I_j) + sum_{j != k} Cov(I_j, I_k)

The delta-method approximation ignores the second (negative) term. Since the covariance terms are negative, the delta-method variance is always an **upper bound**. The paper should quantify the ratio of true variance to delta-method variance, at least for the configurations tested.

### 3. No confidence intervals

The paper provides point estimates only. In statistics, a point estimate without an uncertainty quantification is incomplete. The delta-method variance formula, if corrected for multinomial dependence, could be used to construct approximate confidence intervals. The paper should at least discuss the width of such intervals.

For the configurations tested (n ranging from ~50 to ~2200), the standard error of the occupancy-inverse estimator is approximately sqrt(MSE/256) per cardinality point. From Table 1, the MSE is 292 over the entire sparse regime, giving a root-mean-square error of about 17.1. At n = 1000, this corresponds to a relative standard error of about 1.7%. A 95% confidence interval would be approximately n-hat +/- 3.4%. This is meaningful and should be reported.

### 4. Cardinality-weighted MARE is a poor primary metric

The paper's primary metric is cardinality-weighted MARE:

MARE = sum_n w(n) * |n-hat(n) - n| / n

where w(n) weights by cardinality. This metric gives more importance to larger cardinalities, which is reasonable for applications where large sets are more common. However:

(a) **It hides the error distribution within each cardinality.** The MARE is an average over 256 seeds. The variance of the estimator across seeds is not reported. For a proper statistical assessment, the paper should report the standard deviation of the relative error across seeds at each cardinality, or at least the interquartile range.

(b) **The cardinality weighting is arbitrary.** Why linear weighting? A log-weighted or uniform weighting would give different results. The paper should report at least one alternative weighting scheme.

(c) **MARE is not a statistically principled loss function.** The mean absolute relative error is not differentiable at n-hat = n, and it is not the loss function that corresponds to any standard estimator optimality criterion. The mean squared relative error would be more natural for comparing estimators in a decision-theoretic framework.

(d) **The error distribution is asymmetric.** From the CSV data, the uncorrected estimator has errors ranging from 0.1% to over 1400% (at n=550, the uncorrected estimate is 8.38 for n=550, which is absurdly wrong -- this suggests a data point anomaly or a bug in the uncorrected column). The corrected estimator has errors mostly between 0.1% and 1.5%. A summary metric like MARE cannot capture this distributional information.

**Recommendation:** Report the full error distribution (median absolute relative error, interquartile range, 95th percentile) at each cardinality, not just the mean. This would allow the reader to assess estimator stability.

### 5. SetSketch comparison is not apples-to-apples

The paper compares the hash list (P=10, B=6, m=1024) against SetSketch with the same bucket count (m=1024). However:

(a) **Bit budgets differ.** The hash list stores composites at variable width (24 bits down to 16 bits), with Rice coding overhead. The total bit budget for the hash list is approximately D * (average composite width + Rice overhead) bits, where D is the number of distinct composites. SetSketch stores m * B bits (1024 * 6 = 6144 bits) plus the exponential hash values. The paper does not equalize the total bit budget, so the comparison favors the hash list's variable-width encoding.

(b) **SetSketch's parameter b is not tuned for the sparse regime.** The paper tests b in {2, 1.2, 1.001} but does not explain why these values were chosen or whether a different b would perform better. The claim that "the different b values produce nearly identical results" is interesting but unexplained.

(c) **SetSketch's MARE of 3.83% is dominated by low-cardinality error.** As the paper itself notes, SetSketch has MARE of 69% at n=1 and 12% at n=50. The three-stage design's value-list stage (exact through n=52) gives it an enormous advantage at low cardinality that is not a property of the estimation method per se, but of the architecture. A fair comparison would isolate the estimation method by comparing the hash-list stage against a SetSketch variant with an exact-counting mode.

---

## Major Questions

1. **What is the finite-sample bias of the occupancy-inverse estimator?** The paper reports MRE = -0.02% but does not decompose this into Jensen's inequality bias versus any bias from the Newton solver's numerical precision or from the approximation of the occupancy function itself. A second-order Taylor expansion (as sketched above) would provide a principled decomposition.

2. **How does the estimator behave under adversarial hash functions?** The analysis assumes a uniform hash function. In practice, hash functions are deterministic and may have structure. The paper should discuss the robustness of the estimator to non-ideal hashing, at least qualitatively.

3. **What is the convergence rate of the Newton iteration?** The paper states "typically 5 to 10" iterations but does not provide a worst-case bound. For a production system, a worst-case iteration count is important for latency guarantees.

4. **What is the true variance of D, and how does it compare to the delta-method approximation?** The paper claims "a few percent" accuracy but does not provide numerical evidence. A table comparing delta-method variance to empirical variance at several cardinality points would be convincing.

5. **How does the estimator's error change as P varies?** All experiments use P=10. The error properties may depend on the ratio n/m, and different P values may have different optimal behavior. The paper should at least discuss the scaling behavior.

---

## Minor Comments

1. **The CSV data contains an anomaly at n=550.** The uncorrected column shows 8.38, which would imply an estimate of 8.38 for n=550. This is an error of over 10000%, which is inconsistent with the MRE of 0.0010 reported for that row. This appears to be a data entry error in the CSV (the value 8.38 is likely the relative error in some other scale, or there is a bug in the data generation). The corrected column shows 1.0815, which is also suspiciously large for a "corrected" estimate. These data points should be verified.

2. **The Rice parameter derivation assumes geometric gaps.** Lemma 1 states that gaps between sorted uniform order statistics are geometrically distributed. This is true asymptotically (as the composite space grows large relative to D), but for finite composite spaces, the gaps follow a negative hypergeometric distribution. The approximation is reasonable but should be noted.

3. **The Newton iteration safeguard is described but not analyzed.** The paper initializes the bracket at [D, D] and expands high until g(high) >= D. For small n (close to D), this bracket is tight and convergence is fast. For large n (where D << n), the initial bracket may need many doublings. A worst-case analysis would be useful.

4. **The paper does not discuss the effect of hash collisions on the composite encoding.** If the 64-bit hash function produces collisions (which it must, by the pigeonhole principle, for n > 2^64), the composite encoding is affected. For the cardinalities tested (n up to ~2200), this is irrelevant, but a general statement about the hash function's role would be helpful.

5. **The term "occupancy-inverse estimator" is not standard.** The paper should relate this to known estimators in the literature. The occupancy-inverse approach is closely related to the Horvitz-Thompson estimator in survey sampling, where the inverse of the inclusion probability is used to correct for unequal selection probabilities.

6. **The paper does not discuss the bias-variance tradeoff explicitly.** The occupancy-inverse correction reduces bias at the cost of increased variance (since g^{-1} is nonlinear, it amplifies noise). The paper should quantify this tradeoff.

---

## Score: 6/10

**Justification:** The paper presents a well-motivated and analytically sound estimator. The occupancy model is correctly derived, and the M-estimator framework is natural. The empirical results are impressive and the three-stage architecture is well-designed.

However, the statistical analysis is incomplete. The Jensen's inequality bias is asserted but not quantified. The delta-method variance is acknowledged as approximate but not validated against the true multinomial variance. No confidence intervals are provided. The primary metric (cardinality-weighted MARE) hides the error distribution and seed-to-seed variability. The SetSketch comparison is not fully fair.

For a journal in statistics, the paper would benefit from a more rigorous treatment of the estimator's finite-sample properties, a quantified bias analysis, and a discussion of uncertainty quantification. As a computer science paper, it is stronger, but the statistical claims about bias and variance should be substantiated with formulas and bounds, not just empirical observations.

The score of 6 reflects that the core methodology is sound and the results are compelling, but the statistical analysis needs strengthening for a statistics journal.
