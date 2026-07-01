# Formal Methods Review: "Occupancy-Corrected Sparse Representation for Cardinality Estimation"

**Reviewer:** Formal Methods / Proof Engineering (CAV/ITP/POPL lens)
**Paper:** Occupancy-Corrected Sparse Representation for Cardinality Estimation

---

## Summary

This paper presents an adaptive three-stage cardinality estimator that progresses from an exact value list to a hash list with occupancy-inverse bias correction, and finally to dense HyperLogLog registers. The central technical contribution is Theorem 1 (occupancy function), which derives the expected number of distinct composite hashes as a function of cardinality $n$ and composite width $w$, accounting for the non-uniform cell probabilities induced by the composite encoding. The estimator inverts this function via safeguarded Newton iteration (Algorithm 1). Lemma 1 provides the optimal Rice parameter for gap compression of the sorted hash list.

From a formal methods perspective, the paper makes several claims that are susceptible to machine-checkable proof: the cell probability assignments in Theorem 1 (especially the flag-bit partition at wide widths), the convergence of the Newton inversion in Algorithm 1, and the numerical correctness of the implementation. The paper acknowledges formalization in Lean as future work, which is a significant understatement given the complexity of the claims.

---

## Strengths

**1. Theorem 1 is well-structured for formalization.** The proof decomposes cleanly into independent cell groups, and the grouping logic follows directly from the composite encoding definition. The `m = 2^P` multiplicative factor is uniform across all groups, which simplifies the formal statement. A proof assistant would naturally represent each cell group as a finite sum with explicit bounds.

**2. The occupancy function is monotone.** The paper correctly notes that $g'(n) > 0$ (Equation 484), which is the key invariant for bracketing-based inversion. This monotonicity is provable: each term $g_j(1 - (1 - p_j)^n)$ is strictly increasing in $n$ because $0 < p_j < 1$ and the derivative $g_j(-\ln(1-p_j))(1-p_j)^n$ is manifestly positive. A formal proof would be short and mechanical.

**3. The Newton safeguard is well-designed.** Algorithm 1 maintains a bracket $[lo, hi]$ and falls back to bisection when the Newton step escapes. This is a standard pattern (e.g., Brent's method) and its correctness argument is straightforward: the bracket is invariant, the interval length is non-increasing under bisection, and the root is always contained. The paper's claim of "typically 5 to 10" iterations is empirically grounded and plausible.

**4. Lemma 1 has a clean proof path.** The optimality of Rice coding for geometric sources follows from Gallager and van Voorhis (1975), cited in the paper. The substitution $\mu = 2^w / D$ is algebraic. Formalizing this lemma would be a lightweight exercise compared to Theorem 1.

**5. The implementation mirrors the math closely.** Reading the Rust code in `hyperloglog.rs:759-911`, the `hash_list_expected_distinct` function directly implements the two cases of Theorem 1 (small width vs. wide width), and `hash_list_cardinality` implements Algorithm 1. This correspondence makes verification more tractable than usual, because the specification and implementation share the same control flow.

---

## Weaknesses

**1. Theorem 1's cell-group derivation for wide widths is intricate and error-prone.** The flag-bit partition at $t > B$ creates two subcases (flag 0: register $r \leq B+1$ with implicit rank, flag 1: register $r > B+1$ with explicit rank), and within flag 0, cells are further grouped by the leading-zero run length $\ell \in \{0, \dots, \min(B, t-2)\}$. The paper's proof sketch states the group sizes and probabilities but does not justify why the grouping is exhaustive or why no overlap exists between groups. For example:

- At wide widths, flag 0 stores $t-1$ leading hash bits. The grouping by $\ell$ (number of leading zeros) partitions these $2^{t-1}$ patterns into $\min(B, t-2) + 1$ groups. But $\sum_{\ell=0}^{\min(B, t-2)} 2^{t-2-\ell} = 2^{t-1} - 2^{t-2-\min(B, t-2)}$, which is strictly less than $2^{t-1}$ when $\min(B, t-2) < t-2$ (i.e., when $B < t-2$). The paper does not account for the remaining $2^{t-2-\min(B, t-2)}$ bit patterns. This is a **gap in the proof** that would be caught immediately by a proof assistant.

- The boundary condition at $r = B+1$ (included in flag 0) versus $r = B+2$ (included in flag 1) is stated but not formally justified. Why is the threshold $B+1$ and not $B$ or $B+2$?

**2. Algorithm 1's termination is not formally guaranteed.** The paper states "typically 5 to 10" iterations but provides no worst-case bound. The loop runs up to 80 iterations in the implementation (`hyperloglog.rs:888`), but:

- The bracket expansion loop (`while expected_distinct(hi) < d`) could theoretically diverge if the expected-distinct function never reaches $d$. The implementation guards against this with `if hi > 1e15 { return hi }`, but this is an ad hoc safety valve, not a proof of termination. For the occupancy function to fail to reach $d$, one would need $g(n) < d$ for all $n$, but $g(n)$ is strictly increasing and $g(n) \to m$ as $n \to \infty$, where $m = 2^P \geq d$ (the distinct count cannot exceed the number of buckets). A formal proof would establish this limit and use it to bound the bracket expansion.

- The Newton loop has a hard cap of 80 iterations. This is acceptable as a safety bound, but the paper does not prove that 80 iterations suffice for convergence to $10^{-12}$ relative accuracy. A formal verification would need to establish a convergence rate (e.g., quadratic convergence of Newton's method for this function, which is analytic and strictly convex).

**3. Numerical stability is not analyzed.** The occupancy function computes $(1 - p_j)^n$ via `exp(n * ln(1 - p_j))`. For large $n$ and small $p_j$, the argument $n \cdot \ln(1 - p_j)$ can underflow:

- At $n \approx 10^{15}$ (the implementation's safety cap) and $p_j = 2^{-24}$ (smallest non-saturating probability at $w = 24$), $n \cdot \ln(1 - p_j) \approx -10^{15} \cdot 2^{-24} \approx -60000$, which underflows to $-\infty$, making `exp` return 0. This is handled by the underflow guard in the custom `exp` implementation (`utils/number.rs:201-204`), which checks `y <= -1022.0`. But the paper does not discuss this case.

- Near the root of the inversion, $n$ is moderate (in the range $[d, 2^{P+1}]$), so underflow is unlikely. However, during the bracket expansion phase, `hi` doubles each iteration and could reach values where $(1 - p_j)^n$ underflows even though the true expected distinct count is still far from $d$. The implementation returns `hi` at $10^{15}$ as a safety valve, but this could return a wildly inaccurate estimate if the bracket expansion has gone too far.

- The derivative computation involves $-\ln(1 - p_j) \cdot (1 - p_j)^n$. When $(1 - p_j)^n$ underflows to 0, the derivative also underflows to 0, which would cause division by zero in the Newton step. The implementation guards against this with `if derivative > 0.0 && newton > lo && newton < hi`, falling back to bisection when the derivative is zero or the Newton step is outside the bracket. But the paper does not explain this fallback logic.

**4. The specification-implementation gap is non-trivial.** Comparing Theorem 1 with the Rust code reveals several discrepancies:

- **Theorem 1 (Eq. 414):** The saturating register at $t = B$ has probability $p_R = 2^{-(P + 2^B - 2)}$. The Rust code (`hyperloglog.rs:794`) uses `f64::integer_exp2_minus(r_max - 1)` where `r_max = 2^B - 1`, giving $2^{-(2^B - 2)}$. These are consistent.

- **Theorem 1 (Eq. 429):** At wide widths, flag 1 sums over $r \in \{B+2, \dots, R-1\}$ where $R = 2^B - 1$, so the upper bound is $2^B - 2$. The Rust code (`hyperloglog.rs:808`) uses `(B::NUMBER_OF_BITS + 2)..r_max` where `r_max = 2^B - 1`, giving the same range. Consistent.

- **Theorem 1 (Eq. 431):** The saturating register at wide widths has probability $2^{-(P + R - 1 + t - 1 - B)}$. The Rust code (`hyperloglog.rs:812-815`) uses `f64::integer_exp2_minus(r_max - 1) * residual_q` where `r_max - 1 = 2^B - 2` and `residual_q = 2^{-(t - 1 - B)}`, giving $2^{-(2^B - 2)} \cdot 2^{-(t - 1 - B)} = 2^{-(2^B - 2 + t - 1 - B)}$. But $R - 1 + t - 1 - B = (2^B - 2) + t - 1 - B = 2^B - B + t - 3$, while the Rust gives $2^B - 2 + t - 1 - B = 2^B - B + t - 3$. These are consistent.

- **The factor $m$:** Theorem 1 multiplies the sum by $m = 2^P$ at the end (after the cell-group sums are computed). The Rust code does the same (`hyperloglog.rs:830`: `expected * m`). However, the intermediate accumulation in the Rust code divides by $m$ at each step (`let p = q / m`), which is a different computational order. This is mathematically equivalent but could introduce different rounding errors in floating-point arithmetic. A formal verification would need to account for this.

**5. The paper does not state the assumptions on the hash function formally.** Theorem 1 assumes $h$ is a "uniform hash function," but this is informal. A formal specification would need to define what "uniform" means (e.g., pairwise independence, full independence, or the random oracle model) and state the assumptions precisely. This is not a minor point: the occupancy function's correctness depends on the hash being at least pairwise independent for the expectation to decompose as a sum.

---

## Major Questions

**MQ1: Is the flag-0 grouping at wide widths exhaustive?** As noted above, the sum of group sizes in flag 0 is $\sum_{\ell=0}^{\min(B, t-2)} 2^{t-2-\ell}$, which equals $2^{t-1} - 2^{t-2-\min(B, t-2)}$. When $B < t-2$, this is strictly less than $2^{t-1}$, leaving unaccounted bit patterns. Are these patterns supposed to fall into flag 1? If so, the partition boundary needs to be re-expressed in terms of the stored bit patterns, not the register value. This is a **potential correctness bug** that would cause the occupancy function to underestimate for certain widths.

**MQ2: Does the bracket expansion always terminate?** The paper states the bracket is initialized at $[D, D]$ and expanded until $g(\text{high}) \geq D$. Since $g(n)$ is strictly increasing and $g(n) \to m$ as $n \to \infty$, and since $D \leq m$ (the distinct count cannot exceed the number of buckets), the bracket expansion is guaranteed to terminate. But the paper does not state this argument. A formal proof would need to establish: (a) $g(n)$ is strictly increasing, (b) $\lim_{n \to \infty} g(n) = m$, and (c) $D \leq m$.

**MQ3: What is the worst-case iteration count for the Newton loop?** The paper says "typically 5 to 10" but does not prove a bound. The implementation uses 80 as a safety cap. For a formal verification, one would need to prove that the occupancy function is sufficiently well-conditioned (e.g., has bounded second derivative) to guarantee Newton's method converges within $K$ iterations for some explicit $K$.

**MQ4: Is the delta-method variance approximation formally justified?** The paper states (lines 502-509) that the variance approximation treats multinomial cell indicators as independent, which is an approximation. The exact variance requires cross terms. For a formal treatment, one would need to bound the error of this approximation (e.g., show it is $O(1/m)$ or similar).

---

## Minor Comments

**MC1: The Rice parameter Lemma 1 proof uses an approximation ($k \approx \log_2(\mu \ln 2)$) without stating the error bound.** The floor function in the implementation ($k = \lfloor \log_2(\mu \ln 2) + 0.5 \rfloor$) is not mentioned in the lemma statement.

**MC2: The paper does not discuss the effect of non-ideal hash functions.** All results assume a uniform hash function. Real-world hash functions (e.g., MurmurHash3, xxHash) have known biases. The formal model would need to account for this, either by assuming a random oracle or by providing robustness bounds.

**MC3: The paper's claim that the estimator is "exact in expectation" (line 862) is slightly misleading.** The inversion $g^{-1}(D)$ is nonlinear, and Jensen's inequality implies $\mathbb{E}[g^{-1}(D)] \neq g^{-1}(\mathbb{E}[D]) = n$. The paper acknowledges this on lines 496-500 but the phrasing "exact in expectation" is imprecise.

**MC4: The implementation's safety cap of $10^{15}$ (line 877) is not discussed in the paper.** This is an engineering decision that affects the formal specification. If the paper claims correctness for all $n$, the cap should be justified or removed.

**MC5: The cell-group probability formulas use $2^{-x}$ notation, which is exact in the mathematical model but subject to floating-point rounding in the implementation.** The formal specification should distinguish between the mathematical model (exact arithmetic) and the implementation (IEEE 754 floating point).

---

## Score: 5/10

**Rationale:** The paper presents a genuinely interesting and well-engineered algorithm. The occupancy function (Theorem 1) is the kind of result that benefits from formal verification, and the flag-bit partition at wide widths contains a **potentially serious gap** (MQ1) that a proof assistant would catch. The Newton inversion is well-designed but lacks a formal convergence proof. The numerical stability analysis is absent, and the specification-implementation gap, while small, is non-trivial in floating-point arithmetic.

The paper would benefit significantly from a formalization effort. Theorem 1 is well-structured for Lean/Coq formalization, and the proof would likely be 50-100 lines of formal code. The gap in the flag-0 grouping should be resolved before publication, as it could affect correctness for certain width configurations.

A score of 5 reflects that the core ideas are sound and the empirical results are convincing, but the formal claims are not fully justified and the proof of Theorem 1 has a gap that needs addressing.

---

*This review was produced from a formal methods / proof engineering perspective. The reviewer did not modify any files other than this review output.*
