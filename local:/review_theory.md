# Theoretical Review: "Occupancy-Corrected Sparse Representation for Cardinality Estimation"

## Summary

This paper presents a three-stage adaptive cardinality estimator (value list, hash list with occupancy-inverse bias correction, HLL registers) that achieves sub-register error in the low-cardinality regime. The core theoretical contributions are: (1) an occupancy function (Theorem 1) that models the expected distinct composite count under non-uniform cell probabilities, (2) an optimal Rice parameter lemma for gap compression, and (3) a safeguarded Newton iteration for inverting the occupancy function to recover cardinality estimates.

The theoretical analysis is generally sound but contains several gaps that weaken its rigor. The occupancy derivation is correct in its groupings and boundary cases, but the Rice parameter lemma relies on an unproven empirical steady-state assumption. The Newton iteration is well-safeguarded but lacks formal convergence analysis. The treatment of estimator bias (Jensen) and variance (delta-method) is the weakest part: both are acknowledged as approximate with only empirical justification, no formal bounds are provided.

---

## Strengths

1. **Theorem 1 (Occupancy function) is structurally correct.** The cell-probability derivation properly accounts for the non-uniform composite encoding. The grouping by equal probability is exhaustive: the flag-bit partition at wide widths ($t > B$) cleanly separates $r \leq B+1$ from $r > B+1$, and within each partition the sub-groupings by leading-zero count $\ell$ (flag 0) or register value $r$ (flag 1) are mutually exclusive and collectively exhaustive. The boundary case $t = B$ (no flag bit, pure register tail) is correctly handled as a separate regime.

2. **Derivative formula (Equation 3) is exact.** $g'(n) = \sum_j g_j(-\ln(1-p_j))(1-p_j)^n$ is the correct derivative of $g(n) = \sum_j g_j(1-(1-p_j)^n)$. The monotonicity of $g$ (guaranteeing a unique root) follows immediately from $g'(n) > 0$.

3. **Safeguarded Newton iteration is well-designed.** The bracket-expansion strategy (doubling `high` until $g(\text{high}) \geq D$) combined with the fallback-to-bisection mechanism is a standard and robust approach for root-finding in monotone functions. The algorithm (Algorithm 1) is correctly specified.

4. **Rice parameter lemma derivation is correct** for the stated model (uniform composites, geometric gaps). The connection to Golomb coding optimality for geometric sources is well-placed (Gallager and van Voorhis, 1975).

---

## Weaknesses

### W1. Rice parameter lemma depends on unproven empirical steady-state

The lemma derives $k \approx w - \log_2 D - 0.53$ from first principles, which is correct. However, the substitution $D \approx 2^{P - 1.53}$ to obtain the practical formula $k \approx w - P + 1$ is justified solely by the phrase "observed empirically" (line 339–340). This is not a lemma but a heuristic. For a theoretical paper, the steady-state occupancy $D \approx 2^{P - 1.53}$ at the transition point should be either:

- Derived from the occupancy function $g(n)$ by solving $g(n) = \text{capacity}$, where capacity is the bit-budget threshold, or
- At minimum, stated as a conjecture with a clear error bound, e.g., $D = 2^{P - 1.53} \pm \epsilon$.

As written, the Rice parameter formula is an engineering approximation masquerading as a lemma. The claim "matching the empirically optimal parameter within one step for all tested configurations" (line 344–345) is an empirical claim, not a theoretical guarantee.

### W2. No formal convergence analysis for Newton iteration

The paper states "the Newton loop converges in a bounded number of iterations (typically 5 to 10)" (line 549–550). This is an empirical observation, not a proof. For a rigorous analysis, one should establish:

- A Lipschitz bound on $g''(n)$ over the relevant domain to bound the Newton error reduction factor.
- That the bracket expansion terminates in $O(\log(n/D))$ steps.
- That the combined Newton-bisection scheme converges in $O(\log \log (n_{\max}/\epsilon))$ iterations (standard for safeguarded Newton).

The $O(2^B)$ per-estimate complexity claim is correct for the sum evaluation, but the total complexity should be stated as $O(2^B \cdot \text{iterations})$ with a bound on iterations.

### W3. Jensen's inequality bias: no bound provided

The paper acknowledges (lines 495–500) that $\mathbb{E}[g^{-1}(D)] \neq n$ by Jensen's inequality but offers no quantitative bound. This is a significant gap. A proper analysis should provide:

- A second-order Taylor expansion: $\mathbb{E}[g^{-1}(D)] \approx n + \frac{1}{2} g^{-1}''(n) \cdot \text{Var}(D)$, with an upper bound on $|g^{-1}''(n)|$.
- Since $g^{-1}''(n) = -g''(n) / g'(n)^3$, this reduces to bounding $g''(n) = \sum_j g_j (\ln(1-p_j))^2 (1-p_j)^n$.
- A worst-case bound over the operating regime would show whether the bias is truly negligible or could accumulate across multiple estimation steps.

The empirical MRE values "near zero" (line 498) are suggestive but not a substitute for a bound. The bias could be small on average but large in the tails.

### W4. Delta-method variance: independence assumption is unquantified

The paper states (lines 502–509) that the delta-method approximation "treats the multinomial cell indicators as if they were independent" and is "accurate to within a few percent for the configurations tested." This is the most significant theoretical gap.

In a multinomial occupancy model with $N = \sum_j g_j$ total cells, the cell indicators $X_j$ satisfy $\text{Cov}(X_i, X_j) = -n^2 p_i p_j (1-p_i)^{n-1}(1-p_j)^{n-1}$ for $i \neq j$ (negative correlation: occupying one cell slightly reduces the probability of occupying another). The delta-method ignores all cross terms.

The worst-case error of this approximation should be quantified. In the extreme case of highly non-uniform $p_j$ (which is the case here, since $p_j$ ranges from $2^{-(P+B)}$ to $2^{-(P+2^B-1)}$), the cross terms can be significant. A formal bound would require analyzing:

$$\text{Var}(D) = \sum_j \text{Var}(X_j) + \sum_{i \neq j} \text{Cov}(X_i, X_j)$$

versus the delta-method approximation that drops the second sum entirely.

### W5. No asymptotic analysis

The paper provides no asymptotic results. In particular:

- What is the asymptotic behavior of the MARE as $P \to \infty$ with fixed $B$?
- Does the corrected estimator achieve the information-theoretic lower bound for sparse cardinality estimation?
- How does the error scale with the composite width $w$?

These are standard expectations for a theoretical paper in this area (cf. Flajolet et al. 2007, Ertl 2017).

---

## Major Questions

**MQ1.** What is the formal justification for the transition threshold between the hash list and register stages? The paper states the conversion occurs when "the composite width reaches $w_{\min} = P + B$" (line 366) and "the hash list holds elements until" this point, but the bit-budget calculation that determines this threshold is not derived. The value $w_{\min} = P + B$ is the smallest width that can represent all distinct composites without information loss, but the transition from hash list to registers involves more than just representability — it involves the point at which the occupancy correction becomes unreliable.

**MQ2.** The occupancy function $g(n)$ is derived under the assumption of a uniform hash function. What is the sensitivity of the estimator to hash function quality? For non-uniform hash functions, the cell probabilities $p_j$ deviate from the assumed model, and the inversion $g^{-1}(D)$ will produce biased estimates. A robustness analysis (e.g., bounding the bias under a bounded-degree hash function) would strengthen the theoretical contribution.

**MQ3.** The paper mentions the Rice-coded gap compression is optimal for geometric gap distributions (Lemma), but the actual gap distribution depends on the collision structure of the composite encoding, which is non-uniform. Is the geometric-gap assumption valid for the actual composite distribution? The lemma assumes uniform composites (which is not the case), while the occupancy theorem correctly handles non-uniform cells. There is an asymmetry here.

**MQ4.** For the Newton iteration, what happens at the boundaries? When $n$ is very small (near the value-list/hash-list transition), the occupancy function is nearly linear and Newton's method should converge rapidly. But near the saturation point (where $D$ approaches the total number of cells), the function becomes very flat and Newton steps may overshoot. Does the safeguard handle this correctly? The paper does not discuss boundary behavior of the Newton iteration.

---

## Minor Comments

1. **Notation inconsistency.** The paper uses $R = 2^B - 1$ for the maximum register value (line 400) but also refers to "the saturating register $R = 2^B - 1$" (line 414). This is correct, but the notation $R$ is ambiguous since it could be confused with the register width $B$. Consider using $R_{\max}$ or $2^B - 1$ explicitly.

2. **Theorem 1, flag 0 case.** The paper states "Flag 0 (register $r \leq B + 1$, leading bits stored) has $2^{t-2-\ell}$ cells of probability $2^{-(w-1)}$ for each $\ell \in \{0, \dots, \min(B, t-2)\}$." The upper bound $\min(B, t-2)$ is correct (you cannot have more leading zeros than the number of stored hash bits, and at most $B$ leading zeros before the flag bit would be consumed), but this should be explicitly justified.

3. **Equation (2) saturating register probability.** The saturating register probability $p_R = 2^{-(P + 2^B - 2)}$ is correct: all hash tails with $B$ or more leading zeros (i.e., the tail value $\geq 2^B - 1$) map to the saturating register, and the probability of a $B$-bit prefix being all ones is $2^{-B}$, but since the tail value must be $\geq 2^B - 1$ and the register rank is $\rho(\text{tail}) + 1$ capped at $2^B - 1$, the saturating probability is $2^{-(2^B - 2)}$. This is correct but the derivation is terse.

4. **Table 1 MARE value.** The table reports MARE of 0.85% for the corrected estimator (line 581), while the abstract and text consistently report 0.86% (lines 161, 592). This is a one-percentage-point discrepancy that should be reconciled.

5. **Comparison fairness.** The SetSketch comparison (lines 593–595) uses the same bucket count $m = 1024$ but does not account for the fact that SetSketch stores $b$-bit hashes per bucket while the hash list stores composites of width $w$. A fair comparison should normalize by space usage, not just bucket count.

6. **The term "occupancy-inverse"** is used throughout but not formally defined. It would be helpful to have a definition: "the cardinality estimator $\hat{n}$ is the inverse of the occupancy function $g(n) = \mathbb{E}[D \mid n, w]$."

---

## Score: 6/10 (Weak Accept)

The paper presents a genuinely useful contribution with a correct occupancy model and a well-designed estimator. Theorem 1 is mathematically sound, and the empirical results are impressive. However, the theoretical analysis has significant gaps: the Rice parameter lemma relies on unproven empirical assumptions, the Newton iteration lacks formal convergence analysis, and the treatment of estimator bias and variance is purely empirical with no bounds. For a venue like IEEE Transactions on Information Theory or SICOMP, these gaps are material. The paper would benefit from formal bounds on bias and variance, a convergence proof for the Newton iteration, and a more careful treatment of the Rice parameter derivation.

The empirical results are strong enough to carry the paper, and the three-stage design is practically valuable. But the theoretical contribution, as stated, is incomplete. A revision addressing W1, W3, and W4 would significantly strengthen the paper.
