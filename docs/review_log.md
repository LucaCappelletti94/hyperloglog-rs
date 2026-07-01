# Editorial Review Log

Paper: `hash_list_cardinality_paper.tex`
Target: arXiv submission, 6--8 pages
Reviewer role: journal editor / senior area chair

---

## Pass 1: Structure and Flow

**Focus:** Section ordering, logical progression, signposting.

**Findings:**
- The paper flows well: Intro -> Method -> Occupancy Model -> Results -> HLL++ Distinction -> Discussion -> Conclusion.
- Missing: no motivation for *why* low-cardinality accuracy matters in practice. The introduction jumps straight into technical description. Need a paragraph on application domains (graph analytics, HyperBall, sparse network analysis) where per-node degree estimation at low cardinality is critical.
- The roadmap paragraph at the end of the introduction is good.

**Actions:**
- Add motivation paragraph in the introduction covering graph analytics applications where nodes follow geometric degree distributions and low-cardinality accuracy is essential.

---

## Pass 2: Abstract Quality

**Focus:** Does the abstract stand alone? Does it state the problem, method, result, and significance?

**Findings:**
- Problem: stated (low-cardinality error floor).
- Method: stated (three-stage adaptive representation, occupancy inversion).
- Result: stated (0.05% MRE vs 1.04% for HLL++, 20x reduction).
- Significance: stated (table-free, precision-independent).
- Missing: no mention of *why this matters* (application domains).
- The "20x reduction" claim needs verification: 1.04 / 0.05 = 20.8, so it is correct.

**Actions:**
- Add one sentence on practical significance (graph analytics, sparse data).

---

## Pass 3: Introduction Motivation

**Focus:** Does the introduction build a compelling case? Are the stakes clear?

**Findings:**
- The introduction describes HLL, its error floor, and the low-cardinality problem. Good.
- Missing: the *application motivation*. Many real-world workloads operate precisely in the low-cardinality regime:
  - Graph analytics (HyperBall, graph neural networks, neighborhood aggregation): node degrees follow a geometric/power-law distribution. The vast majority of nodes have very small degree (often single digits to low hundreds). Per-node cardinality estimation in this range needs to be accurate, not just asymptotically correct.
  - Sparse network analysis: flow counting, session tracking, IoT device monitoring.
  - The adaptive upgrade path (exact -> hash list -> registers) matches the natural workload: start small, grow as needed.
- This is a significant gap. Without it, the paper reads as a pure algorithmic contribution without grounding in why anyone would care.

**Actions:**
- Add a substantial paragraph (or two) on graph analytics applications, degree distributions, and why the adaptive representation is practically valuable. Reference HyperBall specifically.

---

## Pass 4: Technical Accuracy (Method Section)

**Focus:** Are the technical descriptions correct? Any claims that overreach?

**Findings:**
- Composite hash encoding: correct. The three forms match the implementation.
- Rice coding: correct. The parameter derivation is sound.
- The claim "the occupancy D is pinned by the downgrade schedule at approximately 2^(P - 1.53)" -- this is an empirical observation, not a theorem. Should be clearer that this is an empirical property of the implementation's downgrade schedule.
- The lemma statement says "uniformly distributed" but the composites are not perfectly uniform (geometric register component). This is acknowledged in the proof but should be noted in the lemma statement.

**Actions:**
- Clarify that the occupancy approximation is empirical.
- Note non-uniformity in the lemma statement.

---

## Pass 5: Mathematical Rigor

**Focus:** Are theorems, lemmas, and proofs correct? Any gaps?

**Findings:**
- Theorem 1 (occupancy function): correct. The cell grouping is well-motivated.
- The proof is brief but correct: it follows from the birthday-paradox occupancy formula applied to non-uniform cells.
- The derivative formula (eq:deriv) is correct: d/dn of (1 - (1-p)^n) = -ln(1-p) * (1-p)^n.
- The occupancy inversion proof of convergence: the occupancy function is strictly increasing (derivative > 0), so the inverse exists. The safeguarded Newton method converges because the function is monotone. This is standard but worth stating more explicitly.
- Missing: a statement about the variance of the estimator. The paper reports empirical MRE/MSE but does not derive a theoretical variance bound.

**Actions:**
- Add a brief remark on convergence guarantees.
- Acknowledge that theoretical variance analysis is left for future work.

---

## Pass 6: Clarity of Definitions

**Focus:** Are all terms defined before use? Is notation consistent?

**Findings:**
- P: precision (number of registers is 2^P). Defined in intro.
- B: bits per register. Defined implicitly.
- w: composite width. Defined in Method section.
- t: tail width (w - P). Defined.
- D: distinct composite count. Defined.
- R: max register value (2^B - 1). Defined in Theorem 1.
- n: cardinality. Used throughout without explicit definition in the method section.
- g(n): occupancy function. Defined in Theorem 1.
- The notation is generally consistent. The only issue is that n is used without being explicitly defined as "true cardinality" in the method section (it's clear from context but should be stated).

**Actions:**
- Add explicit definition of n at the start of the occupancy section.

---

## Pass 7: Figure and Table Quality

**Focus:** Are figures informative? Are tables well-formatted?

**Findings:**
- Table 1: well-formatted with booktabs. Clear comparison of three estimators.
- Figure 1: the error plot is informative. The dual-axis design (error + composite width) is effective.
- The figure caption is detailed and explains what the reader should see.
- Missing: a figure showing the memory usage across the three regimes would help readers understand the trade-off.

**Actions:**
- Consider adding a note about memory usage in the text (a figure would push the paper over 8 pages).

---

## Pass 8: Results Presentation

**Focus:** Are results presented fairly? Are baselines appropriate?

**Findings:**
- Three baselines: uncorrected (D), HLL++ (registers), corrected (occupancy inverse). Good.
- 256 seeds is a reasonable sample size.
- The results are presented at a single precision (P=10, B=6). This is a limitation that should be acknowledged.
- The comparison is fair: all three estimators are evaluated on the same data.
- The claim "20x reduction" in the abstract refers to MRE (1.04% / 0.05% = 20.8). This is correct but could be clearer.

**Actions:**
- Acknowledge single-precision evaluation as a limitation.
- Clarify the "20x" claim.

---

## Pass 9: Comparison with Prior Work

**Focus:** Is the related work fair and comprehensive?

**Findings:**
- HLL++ comparison (Section 6) is thorough: representation, correction, stochastic averaging.
- SetSketch discussion (Section 7.1) is thoughtful: identifies structural similarity and adaptation path.
- HyperLogLogLog discussion (Section 7.2) correctly identifies complementary regimes.
- Missing: no mention of Ertl's MLE work (which is in the references but not discussed in the body). Ertl's MLE is relevant because it's another correction technique that could be compared.
- Also missing: no mention of the original HyperLogLog's linear counting, which is the standard low-cardinality correction.

**Actions:**
- Add a paragraph comparing with Ertl's MLE and linear counting.

---

## Pass 10: Discussion Depth

**Focus:** Is the discussion section substantive? Does it go beyond surface-level comparison?

**Findings:**
- SetSketch discussion: good analysis of structural similarity and adaptation path. Could go deeper into *why* the occupancy model would work for SetSketch (the per-bucket k-smallest values form a truncated geometric distribution, which is similar to the hash-list composite distribution).
- HyperLogLogLog discussion: correct identification of complementary regimes. Could mention that combining both would give a four-stage system: value list -> hash list -> HLL registers -> HLLLog compressed registers.
- Instantaneous codes section: good technical depth. The optimality argument is clear.

**Actions:**
- Deepen the SetSketch discussion with the truncated geometric distribution argument.
- Mention the four-stage system possibility.

---

## Pass 11: Conclusion Strength

**Focus:** Does the conclusion summarize the contribution? Does it state limitations and future work?

**Findings:**
- The conclusion restates the main results (0.6% MRE, table-free, precision-independent). Good.
- Missing: no statement of limitations (single-precision evaluation, no theoretical variance bound, insert cost in hash list is O(n)).
- Missing: no future work direction.

**Actions:**
- Add limitations and future work to the conclusion.

---

## Pass 12: References Completeness

**Focus:** Are all cited works properly referenced? Are key works missing?

**Findings:**
- Flajolet et al. (2007): original HLL. Present.
- Ohan et al. (2013): HLL++. Present.
- Ertl (2017): MLE. Present in references but not discussed in body.
- Wang et al. (2021): SetSketch. Present.
- Dahlgaard et al. (2022): HyperLogLogLog. Present.
- Gallager and van Voorhis (1975): Golomb coding. Present.
- Rice (1959): Rice coding. Present.
- Missing: no reference for linear counting (Flajolet et al. cover it, but a specific citation would help).
- Missing: reference for HyperBall or graph analytics applications (needed for the new motivation paragraph).

**Actions:**
- Add HyperBall/graph analytics references.
- Add linear counting reference.

---

## Pass 13: Language and Readability

**Focus:** Is the prose clear? Are sentences well-constructed? Is the tone appropriate?

**Findings:**
- The prose is generally clear and technical.
- Some sentences are long and could be split (e.g., the first sentence of the introduction is 20+ words before the first clause boundary).
- The tone is appropriate for a technical paper.
- Some phrases are repetitive ("the hash list" appears frequently; could use "this representation" or "the sorted hash list" for variety).
- The phrase "table-free" is used repeatedly. It's a key selling point but could be varied ("coefficient-free", "calibration-free").

**Actions:**
- Split long sentences.
- Vary terminology where appropriate.

---

## Pass 14: Grammar and Style

**Focus:** Grammar, punctuation, LaTeX formatting.

**Findings:**
- Grammar is generally correct.
- The use of "we" is consistent (first person plural throughout).
- LaTeX formatting is clean.
- The equation numbering is consistent.
- The cross-references work (verified by compilation).
- Minor: some equations could be better formatted (e.g., eq:small and eq:wide are long and could be broken into multiple lines).

**Actions:**
- Minor formatting improvements to long equations.

---

## Pass 15: Consistency Check

**Focus:** Are claims consistent across sections? Are numbers consistent?

**Findings:**
- The abstract claims "0.05% MRE" and the results section reports "-0.05% MRE". Consistent (the sign indicates direction of bias, which is negligible).
- The abstract claims "20x reduction" and the results show 1.04 / 0.05 = 20.8. Consistent.
- The method section describes three stages; the introduction describes three stages. Consistent.
- The occupancy function in Theorem 1 matches the implementation in the code. Verified against `src/hyperloglog.rs`.

**Actions:**
- No changes needed.

---

## Pass 16: Novelty Claim Justification

**Focus:** Is the novelty claim well-supported? Are the distinctions from prior work clear?

**Findings:**
- The novelty claim is threefold: (i) sorted hash list as intermediate representation, (ii) Rice-coded gap compression, (iii) table-free occupancy inversion.
- Each component is individually known (sorted storage, gap coding, occupancy inversion) but the combination is new. This is a fair claim.
- The distinction from HLL++ is well-articulated in Section 6.
- Missing: a more explicit statement about why this combination has not been explored before. Is it because prior work focused on the register regime? Is it because the hash-list representation was not considered?

**Actions:**
- Add a sentence explaining why this combination has not been explored (prior work focused on improving the register estimator; the hash-list representation was not considered as a viable intermediate stage).

---

## Pass 17: Practical Relevance (Graph Analytics / HyperBall)

**Focus:** Is the practical motivation compelling? Are the application domains well-described?

**Findings:**
- This is the major gap identified in Pass 3. The paper needs a substantial addition here.
- Graph analytics applications:
  - HyperBall: a graph analytics library that uses cardinality estimation for neighborhood aggregation. Node degrees follow a geometric/power-law distribution. Most nodes have very small degree (1-10 neighbors), so per-node cardinality estimation operates in the low-cardinality regime.
  - Graph neural networks: neighborhood sampling requires counting distinct neighbors.
  - Sparse network analysis: flow counting, session tracking.
- The key insight: geometric degree distributions mean that the *majority* of nodes operate in the low-cardinality regime, where HLL's asymptotic guarantee does not apply. The adaptive representation (exact -> hash list -> registers) matches this natural workload: most nodes stay in the hash list, only high-degree nodes upgrade to registers.
- This is a strong practical motivation that should be prominent in the introduction.

**Actions:**
- Add a substantial paragraph on graph analytics, HyperBall, and geometric degree distributions. Make this a central motivation, not an afterthought.

---

## Pass 18: Edge Cases and Limitations

**Focus:** Are edge cases handled? Are limitations acknowledged?

**Findings:**
- Edge cases:
  - Empty counter: handled (is_empty check).
  - Full counter: handled (is_full check).
  - Single element: handled (value list is exact).
  - Hash collisions: handled by occupancy inversion.
- Limitations:
  - Single-precision evaluation (P=10, B=6). Should be acknowledged.
  - Insert cost in hash list is O(n) (sorted splice). Should be acknowledged.
  - No theoretical variance bound. Should be acknowledged.
  - The occupancy model assumes uniform hashing. Real hash functions may deviate.

**Actions:**
- Add limitations section or paragraph.

---

## Pass 19: Overall Coherence

**Focus:** Does the paper read as a unified whole? Are the sections connected?

**Findings:**
- The paper flows well from problem statement to method to results to discussion.
- The transition from Method to Occupancy Model is smooth.
- The transition from Results to HLL++ Distinction is natural.
- The Discussion section covers related work but could better connect back to the main contribution.
- The conclusion could better synthesize the practical motivation (graph analytics) with the technical contribution.

**Actions:**
- Strengthen the connection between discussion and main contribution.
- Synthesize practical motivation in the conclusion.

---

## Pass 20: Final Polish

**Focus:** Final read-through. Any remaining issues?

**Findings:**
- After all previous passes, the paper should be polished.
- Final check: compile and verify page count.
- Final check: verify all cross-references resolve.
- Final check: verify all citations are in the bibliography.

**Actions:**
- Compile and verify.
- Final read-through.

---

## Summary of Changes

| Pass | Category | Changes |
|------|----------|---------|
| 1 | Structure | Add graph analytics motivation to introduction |
| 2 | Abstract | Add practical significance sentence |
| 3 | Motivation | Add substantial paragraph on graph analytics, HyperBall, geometric degree distributions |
| 4 | Technical | Clarify empirical nature of occupancy approximation |
| 5 | Math | Add convergence guarantee remark, acknowledge variance analysis gap |
| 6 | Definitions | Add explicit definition of n |
| 7 | Figures | Add memory usage note in text |
| 8 | Results | Acknowledge single-precision limitation, clarify "20x" claim |
| 9 | Prior work | Add Ertl MLE comparison, linear counting reference |
| 10 | Discussion | Deepen SetSketch discussion, mention four-stage system |
| 11 | Conclusion | Add limitations and future work |
| 12 | References | Add HyperBall/graph analytics references, linear counting reference |
| 13 | Language | Split long sentences, vary terminology |
| 14 | Style | Minor equation formatting improvements |
| 15 | Consistency | No changes needed |
| 16 | Novelty | Add explanation of why combination has not been explored |
| 17 | Practical | Major addition: graph analytics, HyperBall, geometric degree distributions |
| 18 | Limitations | Add limitations paragraph |
| 19 | Coherence | Strengthen discussion-to-contribution connection |
| 20 | Polish | Compile and verify |

## Additional Passes (User Feedback)

### Pass 21: Linear Counting vs HLL++ Correction Range

**User feedback:** The hash list applies to the range of linear counting,
not all the way to the HLL++ correction range.

**Findings:** Verified against code: at P=10, B=6, the linear counting
threshold is n=2320, while the hash list converts to registers at
n=2153. The HLL++ bias-correction polynomials do not engage until
n > 2320. The paper incorrectly labeled the register baseline as
"HyperLogLog++" when it was actually using linear counting in this
range.

**Actions:**
- Updated table caption to note the baseline uses linear counting.
- Renamed table row from "HyperLogLog++ (registers)" to
  "Registers (linear counting)".
- Updated figure legend entry to "registers (linear counting)".
- Updated figure caption to clarify the baseline estimator.
- Updated HLL++ distinction section to accurately describe the
  hash list covering the linear-counting regime.
- Updated abstract and conclusion to reflect accurate comparison.

### Pass 22: Linear Counting Table-Free Clarification

**User feedback:** Linear counting doesn't use tables.

**Actions:**
- Reworded the introduction to clarify that linear counting is
  table-free and accurate only in a narrow band, while the HLL++
  bias-correction polynomials (which do use tables) apply beyond
  that band.

### Pass 23: Value List Topological Ordering

**User feedback:** The sorted value list benefits from topological
ordering of node IDs (LLP from WebGraph), where topologically close
nodes have similar IDs, making deltas small and Elias-gamma encoding
very compact. Sorting serves dual purpose: delta compression AND
fast set operations.

**Actions:**
- Expanded the value list description to explain the dual purpose
  of sorting (set operations + delta compression).
- Added discussion of LLP ordering and its effect on delta sizes.
- Added WebGraph/LLP reference (Boldi et al., 2014).
- Fixed stray Chinese character "少数" to "few".

### Pass 24: Graph Analytics Motivation (HyperBall)

**User feedback:** Add content about HyperBall and graph applications
where nodes follow geometric distributions, with most nodes having
very small degree, making low-cardinality accuracy essential.

**Actions:**
- Added substantial motivation paragraph in the introduction covering
  graph analytics, HyperBall, geometric degree distributions, and
  why the adaptive representation is practically valuable.
- Added HyperBall reference.
- Updated conclusion to synthesize the graph analytics motivation.
### Pass 25: Prose Formatting

**Actions:**
- Removed semicolons from prose (lines 342, 462, 517, 704).
  Replaced with commas or restructured sentences.
- Replaced em-dash pair (lines 225-227) with parenthetical
  construction per CLAUDE.md prose rules.

### Pass 26: Technical Accuracy and Consistency

**User feedback:** Generalize third stage from "registers" to any
high-cardinality representation. Discuss sign-bit tagging mechanism
and alternatives. Add no_std mention.

**Actions:**
- Renamed "Registers" stage to "High-cardinality representation" in
  introduction, abstract, representation transitions, and conclusion.
- Added discussion of representation tagging: sign-bit of harmonic_sum
  field (zero = register mode, one = pre-register), alternative schemes
  (enum discriminant, flags byte, sentinel values).
- Added no_std paragraph in Method section: core-only dependency,
  custom FloatOps, stack histograms, optional alloc feature.
- Mentioned no_std in conclusion.
- Installed algorithm.sty and algpseudocode.sty from CTAN to local
  texmf tree. Replaced enumerated pseudocode with proper algorithm
  environment.
- Added Chamberlain et al. (2022) ELPH citation for GNN subgraph
  sketching.

**User feedback:** Explain brute-force Rice parameter approach in
instantaneous codes section.

**Actions:**
- Rewrote Instantaneous Codes section to describe initial brute-force
  search over all (P, B, w) configurations producing 341-entry table,
  analysis revealing closed form k = w - (P - 1), and validation
  (82.4% exact, 100% within one step, under 0.1% overhead).

**Benchmarks:** Ran hash_list_correction_plot to verify numbers.

**Actions:**
- Corrected abstract: MARE 0.86% (not 0.05%), baseline 2.21% (not
  1.04%), improvement 2.6x (not 20x).
- Updated table with fresh benchmark numbers.
- Added MARE comparison text below table.
- Fixed "remains below 0.6% throughout" to "remains below 1% for
  most of the regime, rising to 3.6% near conversion".
- Updated conclusion error numbers to match.

### Pass 27: Regime Clarification and Distribution Fix

**User feedback:** Hash list does NOT operate in the HLL++ correction
table regime. Not all graphs have geometric distributions.

**Actions:**
- Rewrote "Correction" bullet in HLL++ distinction section to make
  explicit: hash list operates in linear-counting regime (below
  n = 2320 at P = 10), well before HLL++ polynomials engage. The
  occupancy inverse replaces linear counting, not the HLL++ table.
  The HLL++ polynomial table remains necessary for the intermediate
  regime above linear counting.
- Changed "geometric or power-law distribution" to "heavy-tailed
  degree distributions (power-law or scale-free graphs)" to avoid
  overclaiming about geometric distributions.

### Pass 28: Strict Numerical and Factual Verification

**Method:** Exhaustive line-by-line verification of every numerical
claim, code behavior claim, equation, and citation against source
code and fresh benchmark output.

**Errors found and fixed:**
- Correction coefficients table is 4.0 KB (405 f64 + 90 f64 + 45 u32
  = 4140 bytes), not 38 KB. Fixed in two locations (introduction and
  HLL++ distinction section).
- Conclusion referenced "geometric degree distributions" while
  introduction correctly said "heavy-tailed degree distributions".
  Fixed conclusion to match.
- Conclusion claimed "single-precision evaluation" but the
  implementation uses f64 (double precision) throughout. Removed
  the claim entirely.

**Verified correct (no changes needed):**
- Abstract MARE numbers (0.86%, 2.21%, 2.6x) match benchmark.
- Table MAE/MRE/MSE match benchmark.
- LARGEST_VIABLE_HASH_BITS = 24 for P=10 (confirmed in switch.rs).
- Linear count threshold = 2320 for P=10, B=6 (confirmed in
  correction_coefficients.rs).
- Hash list regime boundaries: value_list_end=52, hll_start=2153
  (confirmed from benchmark).
- Occupancy function equations match implementation in
  hash_list_expected_distinct().
- Rice parameter closed form k = w - (P-1) matches implementation.
- Brute-force table size: 341 entries (from gap_code_optimality.md).
- 82.4% exact match, 100% within one step (from
  gap_code_optimality.md).
- no_std claim verified against Cargo.toml and src/lib.rs.
- Sign-bit representation tagging verified against is_hyperloglog().
- O(2^B) computational cost per estimate verified.
- O(n) insert cost in hash list (sorted splice in bit buffer).
- SetSketch and HyperLogLogLog descriptions match literature.
- Gallager 1975 Golomb coding optimality claim is correct.

### Pass 29: Theory/Code Separation

**User feedback:** Do not mix code implementation details (struct
field names) with theoretical discussion. Tagging discussion should
be abstract, using examples of unused bits.

**Actions:**
- Removed all references to \texttt{harmonic\_sum} field from the
  paper. Rewrote representation tagging discussion to be abstract:
  a floating-point word has its sign bit fixed to zero for valid
  values, leaving it available as a discriminant. The NaN payload
  tagging is kept as a concrete example of this principle.
- Added SetSketch analysis: per-bucket minima store redundant bucket
  index bits (the bucket is implicit from storage location), which
  could serve as a tag.
- Made HyperLogLogLog tagging claim clearly speculative: would
  require analyzing its variable-length encoding for redundant bits.

### Pass 30: Display Math for Long Equations

**User feedback:** Use \[\] or equation environments for long
equations instead of inline math.

**Actions:**
- Converted occupancy function E[D | n, w] = sum g_j(1-(1-p_j)^n)
  from inline to \[\] display math.
- Converted Rice parameter k = floor(log_2(mu ln 2) + 0.5) from
  inline to \[\] display math.
- Verified all other long equations are already in equation/multline
  environments (occupancy small/wide forms, derivative, inverse).

### Pass 31: Literature Review and Novelty Attribution

**User feedback:** Verify novelty claims against HLL++ paper and
other cited papers. Do not overclaim. Be clear about what builds on
prior work vs. what is novel.

**Actions:**
- Read original HLL paper (Flajolet et al. 2007). Confirmed: no
  value list, no hash list, no gap compression. Only linear counting
  (m*log(m/V)) for small cardinalities.
- Verified HLL++ stochastic averaging uses composite hash encoding
  (bucket index + register rank + residual bits) but stores them
  folded into a fixed register array, not as a sorted distinct list.
- Rewrote abstract: removed misleading "mean error reduction" claim,
  added per-cardinality error detail (below 0.01% at boundary,
  below 1% across most of range), added no_std mention.
- Rewrote novelty claim: composite hash encoding is from HLL++
  stochastic averaging. Novel contributions are: (a) Rice-coded gap
  packing of the sorted hash list, (b) occupancy-inverse bias
  correction made necessary by the packing pushing cardinality into
  the collision regime.
