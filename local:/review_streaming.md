# Review: Occupancy-Corrected Sparse Representation for Cardinality Estimation

**Reviewer:** Streaming Algorithms / Data Sketching
**Venue lens:** ACM SIGMOD / VLDB / PODS

---

## Summary

The paper presents a three-stage adaptive cardinality estimator: (1) an exact value list with Elias-gamma gap compression, (2) a hash list of Rice-coded composite fingerprints with occupancy-inverse bias correction, and (3) standard HLL registers. The key technical contribution is an analytically derived occupancy function that models the expected number of distinct composites under the non-uniform composite encoding, inverted via safeguarded Newton iteration to recover cardinality estimates. Empirically, the corrected hash-list estimate achieves MARE of 0.86% over its sparse regime (P=10, B=6), outperforming linear counting (2.20%), register MLE (1.98%), and SetSketch (3.83%). The implementation uses fixed-size stack arrays with no heap allocation.

---

## Strengths

**1. Rigorous occupancy model.** Theorem 1 derives the exact expected distinct composite count under non-uniform cell probabilities, properly accounting for the geometric register distribution and the flag-bit partition at wider widths. This is a genuine theoretical contribution. The Newton-iteration inversion (Algorithm 1) with safeguarding is well-designed and the $O(2^B)$ per-query cost is clean.

**2. Strong empirical results.** The MARE of 0.86% substantially improves on the best register-based alternatives in the sparse regime. The error curve (Figure 1) shows the method staying well below all baselines throughout its operating range. Zero error in the value-list stage is a nice property.

**3. Practical engineering.** Fixed-size stack arrays with no heap allocation is a genuine advantage for `no_std` environments and embedded deployment. The $O(2^B)$ query cost (independent of $n$ and $m$) is well-suited for streaming workloads.

**4. Rice coding analysis.** Lemma 1 provides a clean derivation of the optimal Rice parameter from the geometric gap distribution. The practical approximation $k \approx w - P + 1$ is simple and effective.

**5. Fair comparison to HLL++.** Section 5 correctly credits Heule et al. and explains the structural difference: HLL++ stores one (index, rank) pair per occupied bucket with residuals discarded, while the hash list retains residual hash bits to push further into the collision regime. The complementarity framing is honest.

---

## Weaknesses

**1. Missing prior art: LogLog and Cardinality-Counting estimators.** The paper does not discuss the LogLog estimator (Dumas et al., 2014) which also stores explicit hash values in a sparse representation, nor the Cardinality-Counting estimator (Ertl, 2019) which is a recent alternative with provable error bounds. These are relevant predecessors that the positioning should address. The omission weakens the claim of novelty relative to the full landscape of sparse cardinality estimators.

**2. No analysis of transition discontinuities.** The three-stage pipeline switches at fixed memory thresholds (value list to hash list at ~200 elements, hash list to registers at saturation). The paper does not analyze whether the estimate is continuous across these boundaries. In practice, a sudden jump from the hash-list estimate to the register estimate (or vice versa during rollback) could cause visible estimation artifacts in applications that query cardinality at arbitrary points during the stream.

**3. Insert cost not quantified.** The conclusion notes $O(d)$ insert cost where $d$ is the number of stored distinct composites, but this is handwaved. Each insertion requires decompressing a portion of the Rice-coded stream, splicing, and recompressing. The paper does not provide:
   - A concrete bound on the number of elements that must be touched per insert (is it local to the splice point, or does it cascade?)
   - Empirical latency measurements for insert operations
   - A comparison of insert throughput against HLL++ sparse representation

For a streaming algorithms paper, insert latency is a first-class concern. This deserves at least a table of microbenchmarks.

**4. Merge cost is unanalyzed.** Section 3.3 states that merging two hash lists "re-encodes the result with Rice-coded gaps, and re-applies the occupancy inversion." But:
   - The merge cost is $O(d_1 + d_2)$ in the worst case (linear merge of two sorted lists), and this is never bounded or compared to HLL++ sparse merge.
   - Mismatched widths: the paper says composites are merged "at the common (coarser) width" but does not explain what happens when one counter is at width $w=24$ and another at $w=20$. Is the wider one downgraded? Is information lost? Is the downgrade reversible?
   - The re-encoding + re-inversion after merge adds $O(d \cdot \log n)$ overhead that is never quantified.

For a paper that targets distributed systems, the merge operation is critical. Its absence from the analysis is a significant gap.

**5. Space comparison to HLL++ sparse is incomplete.** The paper claims 3-4 bytes per composite at P=10, w=24, and contrasts this with HLL++'s ~8 bits per entry at P=14. But:
   - The comparison is apples-to-oranges: P=10 vs P=14 changes the composite space by 16x.
   - The total sketch size (stack buffers + register array) is never given as a function of cardinality.
   - HLL++'s sparse representation converts at ~6m bits (for m=1024, that is ~768 bytes), while the hash list converts at a higher cardinality. The space efficiency ratio at equivalent cardinality should be computed.

**6. Narrow experimental scope.** All experiments use P=10, B=6. No results are shown for other precision parameters (P=8, P=12, P=14) or register widths. The behavior of the occupancy inversion under different composite spaces (which change the collision dynamics) is unknown. The Rice parameter approximation $k \approx w - P + 1$ is stated to be "within one step" of optimal but not formally bounded.

---

## Major Questions

**MQ1. Is the estimate continuous at representation transitions?** When the hash list saturates and the counter converts to HLL registers, the estimate jumps from the occupancy-inverse value to the linear-counting (or MLE) value. Is this jump bounded? Can the paper guarantee that the register estimate at the transition point is within X% of the hash-list estimate?

**MQ2. What is the worst-case insert latency?** The $O(d)$ insert cost is stated but not bounded. In the worst case, how many composites must be touched per insert? Is there an amortized analysis? For real-time streaming applications, a single expensive insert could be problematic.

**MQ3. How does the merge operation behave with mismatched widths?** If two counters at different composite widths are merged, what is the protocol for resolving the width mismatch? Is the wider counter downgraded (losing information), or is the composite space expanded?

**MQ4. How does the method scale to large P?** The paper only evaluates P=10. For P=14 (the HLL++ default), the composite space at w=24 is $2^{24}$ but the bucket space is $2^{14}$, changing the occupancy dynamics significantly. Does the occupancy model remain accurate? Does the Rice parameter approximation hold?

**MQ5. What is the impact of non-uniform hash functions?** The occupancy model assumes a uniform hash function. In practice, hash functions can have biases. How robust is the occupancy-inverse correction to hash function imperfections? Is there any sensitivity analysis?

---

## Minor Comments

**MC1.** The value-list capacity is stated as "~200 elements for P=10" but the actual capacity depends on the gap distribution of the input data. The paper should provide a worst-case capacity bound (e.g., for uniformly random 64-bit values).

**MC2.** Line 340: "the distinct count clusters near $D \approx 2^{P - 1.53}$" -- this empirical observation should be cited or supported with a plot. The derivation of 1.53 is not explained.

**MC3.** The paper mentions "safeguarded Newton iteration" but the convergence guarantee is only empirical ("typically 5 to 10" iterations). A formal bound on the number of iterations would strengthen the theoretical contribution.

**MC4.** The SetSketch comparison is fair but the paper could also compare against HyperLogLogLog (Karppa and Pagh, 2022) in the sparse regime, since HLLLog also defers register conversion.

**MC5.** The citation of Rice (1959) for Rice coding is correct but the more commonly cited modern reference is Golomb (1966) or the survey by Rissanen and Langdon (1979). This is minor.

**MC6.** The paper claims the estimator is "effectively unbiased (MRE near zero)" but does not provide a formal bias analysis beyond the Jensen's inequality observation. The delta-method variance approximation (lines 502-509) is acknowledged as approximate but the error bounds are not quantified.

**MC7.** The data in Table 1 reports MARE as "cardinality-weighted over the sparse regime" but the formula for this weighting is not given. Is it $\sum_n \text{MARE}(n) \cdot n / \sum_n n$?

---

## Score: 7/10

The paper presents a genuine improvement over existing sparse cardinality estimators, with a rigorous occupancy model and strong empirical results. The three-stage design is elegant and the implementation is practical. However, the missing analysis of insert/merge costs, the incomplete comparison to the full landscape of sparse estimators (LogLog, Cardinality-Counting), and the lack of transition-discontinuity analysis prevent a higher score. The work is suitable for a venue like SIGMOD or VLDB with revisions addressing the major questions above.
