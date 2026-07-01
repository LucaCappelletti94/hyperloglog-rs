# Review: Information Theory / Data Compression

**Paper:** Occupancy-Corrected Sparse Representation for Cardinality Estimation

**Reviewer:** Compression and Source Coding

---

## Summary

The paper presents a three-stage adaptive cardinality estimator progressing from an exact value list (Elias-gamma coded gaps) to a hash list (Rice-coded gaps with occupancy-inverse bias correction) to dense HyperLogLog registers. The key novelty is the Rice-coded gap packing of the sorted composite hash list combined with an analytically derived occupancy function, inverted via Newton iteration. The corrected hash-list estimate achieves MARE 0.86% at P=10, B=6, significantly outperforming linear counting (2.20%), register MLE (1.98%), and SetSketch (3.83%).

From an information-theoretic perspective, the paper makes solid use of classical results (Golomb/Rice coding optimality for geometric sources, occupancy formulas for the birthday paradox) but stops short of providing a formal rate analysis, entropy bounds, or rigorous treatment of the compression overhead. The Rice parameter derivation is sound in principle but relies on empirical steady-state assumptions rather than a data-driven adaptive mechanism. The paper also omits any discussion of the metadata overhead required for decompression and merging.

---

## Strengths

1. **Correct application of Golomb-Rice theory.** The paper correctly identifies that gaps between sorted uniform order statistics follow a geometric distribution, and that Golomb coding is provably optimal for geometric sources (Gallager and van Voorhis, 1975). Rice coding as a power-of-two specialisation of Golomb is the right engineering choice, enabling shift-and-mask instead of division.

2. **Non-uniform occupancy model.** Theorem 1 correctly accounts for the non-uniform cell probabilities arising from the composite encoding structure (uniform index bits, geometric register distribution, residual hash bits). This is a non-trivial generalisation of the standard uniform-occupancy birthday paradox formula. The proof correctly groups cells by equal probability and sums over groups.

3. **Closed-form Rice parameter.** The paper derives a closed-form Rice parameter `k = w - P + 1` from first principles rather than relying on a lookup table. The internal analysis (gap_code_optimality.md) confirms this reproduces the empirical table to within one step in 100% of cases, with negligible compression cost (under 1% in all metrics).

4. **Clear explanation of the composite encoding.** The three-form composite hash encoding (Definition 1) is well-specified and correctly captures the trade-off between register explicitness and residual hash bits. The flag-bit partition at wider widths is a clever space-saving mechanism.

5. **Honest acknowledgment of limitations.** The paper correctly notes that the Rice-coded packing efficiency is data-dependent (the estimator variance depends on the gap distribution of input hashes), and explicitly identifies `O(d)` insert cost as the primary limitation.

---

## Weaknesses

1. **Lemma 1 assumes uniformity but composites are non-uniform.** Lemma 1 states "For D distinct composites *uniformly distributed* over a 2^w space", but the composite encoding is explicitly non-uniform (the proof of Theorem 1 explains why). The geometric gap distribution holds for uniform order statistics, but the actual gap law is a *mixture* of geometric distributions corresponding to the different cell probability groups. The paper acknowledges this in the proof but the lemma statement itself is misleading. A reviewer without access to the implementation notes would not know the mixture is close enough to geometric that Rice remains near-optimal.

2. **No formal entropy analysis.** The paper does not compute the theoretical entropy of the gap distribution, nor does it quantify how close Rice coding gets to the entropy bound. For a geometric source with mean μ, the entropy is H = -(1-μ⁻¹)log₂(1-μ⁻¹) - μ⁻¹log₂(μ⁻¹), and the optimal Rice code achieves at most H + 2 bits per symbol. The paper states Rice is optimal but provides no quantitative rate analysis showing the actual bits-per-entry achieved versus the entropy lower bound. A rate analysis would strengthen the compression claims substantially.

3. **Rice parameter is not adaptively tuned.** The parameter k = w - P + 1 is derived from an *empirical* steady-state occupancy (D ≈ 2^(P-1.53)), not from the actual observed gap distribution during operation. The paper does not discuss whether an online Rice parameter estimator (e.g., the adaptive approach in deflate or arithmetic coding) would improve compression. While the cost of a one-step miss is small (as shown in gap_code_optimality.md), the paper does not quantify the potential gain from adaptation, nor does it discuss the computational trade-off.

4. **Elias-gamma vs Rice: no justification.** The value list uses Elias-gamma coding for 64-bit values spread over 2^64, while the hash list uses Rice coding over 2^w (w = 16-24). The paper does not explain why different codes are used for the two stages. Elias-gamma is optimal for a Zipf-like distribution over unbounded integers; Rice is optimal for a geometric distribution. Both are used for gap-coded sorted lists, but the gap distributions at the two stages are fundamentally different: the value list gaps are determined by the input data distribution, while the hash list gaps are determined by the occupancy of the composite space. The paper should explicitly justify this design choice rather than leaving it implicit.

5. **Compression overhead is not accounted for.** The paper does not discuss the metadata overhead required for decompression, merging, and serialization. The implementation stores the following metadata in the f64 harmonic_sum word (64 bits total):
   - 1 bit: mode flag (sorted value list vs sorted hash list vs dense)
   - 5 bits: hash_bits (current composite width)
   - 20 bits: number_of_hashes
   - 21 bits: writer_tell (bit position in the bitstream)
   - 17 bits: number of duplicates
   
   This is 64 bits of metadata per counter, which is 8 bytes. For small hash lists (e.g., 50 entries at ~3.5 bytes each = 175 bytes), this overhead is ~4.5%. The paper does not account for this in its space analysis, nor does it discuss the metadata needed for merging two hash lists (the rank index, prefix-free flag, and width information).

6. **No analysis of Rice parameter sensitivity.** The paper states that k = w - P + 1 matches the empirically optimal parameter within one step, but does not provide a sensitivity analysis showing how the compression ratio degrades as a function of Rice parameter error. A plot of expected code length versus Rice parameter (showing the shallow V-shape) would be convincing evidence that the closed form is adequate.

---

## Major Questions

1. **What is the theoretical entropy of the composite gap distribution, and how close does Rice coding get to it?** The paper should quantify the entropy H of the actual (non-uniform mixture) gap distribution and report the compression ratio H / L where L is the expected Rice code length. This would provide a principled bound on how much better a different coding scheme (e.g., arithmetic coding, adaptive Golomb) could perform.

2. **Would an online Rice parameter estimator improve compression?** The deflate algorithm updates its Rice parameter based on the running average of gaps. Given that the composite width changes during the downgrade schedule, the gap distribution changes dynamically. An online estimator would track the running mean gap and adjust k accordingly. What is the expected improvement, and is it worth the added complexity?

3. **Is the mixture of geometric gap distributions close enough to a pure geometric source for Rice to remain near-optimal?** The composite encoding creates cells with different probabilities, leading to a mixture of geometric gap distributions. The paper should quantify the distance between this mixture and the best-fitting pure geometric distribution (e.g., in KL divergence), and report the resulting loss of optimality for Rice coding.

4. **Why use different codes for the value list and hash list?** Elias-gamma is used for the value list gaps, Rice for the hash list gaps. Both are gap-coded sorted lists. The paper should explicitly justify this choice, or consider whether a unified coding scheme would simplify the implementation without significant compression loss.

5. **How does the metadata overhead affect the compression ratio at different cardinalities?** The 64-bit metadata word is a fixed overhead. At low cardinality (e.g., n = 50, ~175 bytes of data), this is 4.5%. At higher cardinality (e.g., n = 2000, ~7000 bytes), it is negligible. The paper should report the total space used (data + metadata) rather than just the data space.

---

## Minor Comments

1. **Lemma 1 should be restated to reflect non-uniformity.** The lemma statement says "uniformly distributed" but the composites are not perfectly uniform. This should be corrected to "approximately uniformly distributed" or the lemma should be restricted to the uniform-index component with a note about the tail.

2. **The Rice parameter formula k = w - P + 1 is slightly inconsistent with Lemma 1.** Lemma 1 gives k ≈ w - log₂ D - 0.53, and the paper substitutes D ≈ 2^(P-1.53) to get k ≈ w - P + 1.53 - 0.53 = w - P + 1. This arithmetic is correct but should be shown explicitly rather than stated as "this yields the practical approximation".

3. **The paper would benefit from a table showing Rice code lengths at different widths.** For P = 10, B = 6, at widths w = 24, 20, 16, the paper should report: mean gap μ, optimal k, actual k used, expected code length, and entropy. This would make the compression analysis concrete.

4. **The Elias-gamma coding description could be more precise.** The paper says "Elias-gamma coding" for the value list but does not specify the exact encoding (the implementation uses a unary length prefix followed by significant bits, which is the standard Elias-gamma code). The claim that "contiguous values cost a single bit each" is correct (gap-1 = 0 encodes as a single zero bit), but the paper does not quantify the expected code length for random 64-bit values (which is approximately 2 * log₂(2^64) + 1 ≈ 129 bits per value, or ~16 bytes).

5. **The paper could mention prefix codes and the prefix-free property.** The implementation checks `is_prefix_free_encoded` to distinguish between Rice-coded and raw fixed-width layouts. This is an important detail for correct parsing but is not mentioned in the paper.

6. **The reference to Rice (1959) is appropriate but incomplete.** The original Rice paper is about two-dimensional transform coding for image compression, not gap coding. The connection to the current application is through the Rice coding scheme (unary quotient + binary remainder), but a reference to the information-theoretic analysis of Rice/Golomb coding (e.g., Gallager 1975, which is cited) would strengthen the theoretical grounding.

---

## Score: 6.5/10

The paper makes solid engineering choices (Rice coding for geometric gaps, non-uniform occupancy model, closed-form Rice parameter) and the compression is demonstrably effective (3-4 bytes per composite vs 14 bytes for Elias-gamma in the value list). However, from an information-theoretic perspective, the paper stops short of providing a formal rate analysis, entropy bounds, or rigorous treatment of the compression overhead. The Rice parameter derivation, while sound, relies on empirical assumptions rather than adaptive mechanisms. The omission of metadata overhead from the space analysis is a notable gap. The paper would benefit significantly from a quantitative rate analysis showing the compression ratio relative to the entropy bound, and a sensitivity analysis of the Rice parameter.

The score reflects that the compression choices are correct and well-justified at an intuitive level, but the paper lacks the formal information-theoretic analysis that would make it a complete contribution to the information theory literature. It is more of an engineering paper with information-theoretic ingredients than a rigorous information-theoretic study.
