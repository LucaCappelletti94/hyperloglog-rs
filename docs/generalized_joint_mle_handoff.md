# Handoff: Generalized Joint MLE for HyperLogLog Hypersphere Sketches

This document hands off a focused piece of work to a clean session. The goal is to implement a generalized maximum-likelihood estimator for the hypersphere sketch.

## Goal

Given a vector of `M` nested HyperLogLog counters (the lefts, `A_0 subset A_1 subset ... subset A_{M-1}`) and a vector of `N` nested counters (the rights, `B_0 subset ... subset B_{N-1}`), jointly estimate, in a single optimization, all the disjoint-cell cardinalities that the hypersphere sketch needs:

- `n_ij = |L_i intersect R_j|` for the `M x N` exclusive-overlap grid, where `L_i = A_i \ A_{i-1}` and `R_j = B_j \ B_{j-1}` are the left/right shells (`A_{-1} = empty`).
- `n^A_i = |L_i \ (all B)|` and `n^B_j = |R_j \ (all A)|`, the `M + N` margin differences.

Total parameters: `M*N + M + N` non-negative disjoint region cardinalities. Every element of the universe lives in exactly one region.

This replaces the current sketch, which computes each cell pairwise via `estimate_union_cardinality` + inclusion-exclusion + `saturating_zero_sub` (see `src/sketches.rs`, `HyperSpheresSketch::overlap_and_differences_cardinality_matrices`). That pairwise approach compounds per-pair error and patches non-negativity/consistency ad hoc.

## Confirmed design decision

Optimize the disjoint-cell region model directly: the `M*N + M + N` region cardinalities are the parameters (optimized in log-space, `phi_rho = ln(n_rho)`). Optimizing in disjoint-region space makes non-negativity and global consistency structural, so the `saturating_zero_sub` patchwork disappears and the overlap matrix is consistent by construction. (The alternative, optimizing cumulative unions/intersections and differencing, was considered and rejected.)

## Why this is expected to help (the premise, measured)

The 2-set joint MLE (current `src/mle.rs`, `mle_union_cardinality`) already jointly fits 3 regions (`left_diff`, `right_diff`, `intersection`); it is the `M = N = 1` base case of this generalization.

Measured 2-set MLE union vs the default union estimator (P=*, Bits6, mean relative error over cardinality x overlap cases):

| precision | default | MLE | MLE vs default |
| --- | --- | --- | --- |
| 4 | 0.15287 | 0.11442 | ~25% better |
| 5 | 0.08822 | 0.10375 | ~18% worse (inversion, investigate) |
| 6 | 0.06662 | 0.06737 | tie |
| 8 | 0.05083 | 0.04051 | ~20% better |
| 10 | 0.02863 | 0.02263 | ~21% better |
| 12 | 0.00779 | 0.00755 | ~3% better |

Takeaway: the joint estimator's advantage grows where a single counter is noisy (small precision), up to ~20-25%. The hypothesis is that a multi-set joint estimator, which constrains every region with the register evidence of all `M + N` sketches at once (plus the nesting as a hard prior), should compound this and be globally consistent. Caveat: small randomized sample, and the P5 inversion is unexplained; a larger randomized sweep should precede heavy investment.

## The model (per-register likelihood)

This follows Otmar Ertl's joint estimation framework, generalized from 2 sets to the `M + N` nested-set partition.

- For a single register index, each sketch's register value is the maximum geometric rank over the elements (of the regions it contains) that hash there. Because of nesting, the observed register vector forms two monotone sequences: `r^{A_0} <= r^{A_1} <= ... <= r^{A_{M-1}}` and `r^{B_0} <= ... <= r^{B_{N-1}}`.
- The joint likelihood factorizes over the `m = 2^P` registers. Per register and per "rank level" `k`, each region `rho` contributes, with `x_rho = n_rho * 2^-(P + k)` (matching the existing code's `x = e^phi * 2^-(P + register)`):
  - an "empty" factor `e^{-x_rho}` (no element of region `rho` reached this register at rank `>= k`), contributing `log` term `-x_rho`;
  - an "occupied" factor `1 - e^{-x_rho}`, contributing `log(1 - e^{-x_rho})`.
  The observed monotone register state selects, per region per level, whether it is constrained empty or occupied.
- Building-block variables, identical to the current code: `x = e^phi * 2^-(P + register)`, `y = e^{-x}` (computed via `exp_m1` for stability), `z = 1 - y`.
- This must reduce exactly to `mle_union_cardinality` at `M = N = 1` (3 regions: `left_diff`, `right_diff`, `intersection`). That equality is the primary correctness anchor.

References:
- Otmar Ertl, "New cardinality estimation algorithms for HyperLogLog sketches" (2017), the joint-estimation section: https://oertl.github.io/hyperloglog-sketch-estimation-paper/paper/paper.pdf
- Ertl's C++ reference implementation: https://github.com/oertl/hyperloglog-sketch-estimation-paper
- The existing `K = 2` Rust template: `src/mle.rs`, `mle_union_cardinality` and its Adam optimizer.

## Gradient tooling (research)

The crux and main risk is the gradient of the joint log-likelihood w.r.t. each region for general `M, N`. Ertl derived it by hand for `K = 2` (the `y/z/delta` block in `mle_union_cardinality`); generalizing by hand is error-prone. Options, in recommended order:

### A. Rust forward-mode automatic differentiation (recommended to get correct first)
- Crate: `num-dual` (dual numbers, generic over the scalar; carries a gradient vector). Write the negative log-likelihood once as `fn nll<D: DualNum<f64>>(phis: &[D], stats: &Stats) -> D` using the trait's `exp`/`ln`/`exp_m1`, and read off the exact gradient. Cost is O(P) per evaluation for `P = M*N + M + N` parameters; fine for small `M, N` (P up to ~80).
- Pros: zero hand derivation, exact, immune to algebra mistakes. Cons: slower than a hand gradient; the likelihood must be written generically over the dual scalar (no bare `f64::exp`).
- Alternative for speed/scale: nightly `std::autodiff` (Enzyme, reverse-mode, exact) if nightly is acceptable.

### B. Symbolic derivation + codegen (for a fast production gradient)
- SymPy (Python): build the per-register-state NLL term symbolically in the `phi_rho`, `diff` per region, run `cse` to factor shared subexpressions, and emit Rust with `sympy.printing.rust.rust_code`. Produces a hand-quality analytic gradient programmatically.
- Mathematica / Wolfram Alpha (available to the maintainer): derive and `FullSimplify` the closed-form per-region term. Concrete starting check for the single building block, paste into Wolfram Alpha:
  - `d/dt log(1 - exp(-exp(t) * w))`  ->  `w e^t e^{-w e^t} / (1 - e^{-w e^t})`, i.e. `x * y / z` with `x = w e^t`, `y = e^{-x}`, `z = 1 - y`. This matches the current code's `x, y, z`, confirming the building block.
  - `d/dt (-exp(t) * w)`  ->  `-x` (the empty-factor term).

### C. Numerical gradient (always, as the oracle)
- Central finite differences on the NLL. Use it as the correctness oracle for A or B and as a fallback optimizer gradient. Slow but unimpeachable.

Recommended workflow: implement the NLL once, get its gradient via `num-dual` (A), cross-check against finite differences (C), confirm the optimizer converges and that `M = N = 1` reproduces `mle_union_cardinality`. Only if too slow, derive the analytic gradient via SymPy (B) and re-validate against A and C.

## Implementation plan (TDD + measure)

1. Math: finalize the per-register log-likelihood and gradient for general `M, N`; confirm the `M = N = 1` reduction to `mle_union_cardinality` symbolically and numerically.
2. Skeleton: `joint_sketch_mle(lefts: &[HyperLogLog; M], rights: &[HyperLogLog; N]) -> ([[f64; N]; M], [f64; M], [f64; N])` (overlap grid, left margins, right margins). Behind the `mle` feature. Materialize any hash-list operand into registers first (as the existing MLE methods do).
3. Multiplicity tabulation: iterate registers once, accumulate the statistics needed over the monotone joint register state (generalize the multiplicity vectors in `mle_union_cardinality`).
4. Gradient: forward-mode AD (`num-dual`) first; optional SymPy-generated analytic gradient later for speed.
5. Verify:
   - numerical finite-difference gradient cross-check;
   - base-case equality with `mle_union_cardinality` at `M = N = 1`;
   - TDD against exact nested-set operations for small `M, N` (each cell must match the true `|L_i intersect R_j|` within the error rate). Build nested sets deterministically (e.g. ranges) so the exact cells are known.
6. Measure: full-matrix accuracy and speed vs the current pairwise sketch, at small precision (where there is 20-25% headroom). Expect gains; watch the P5 inversion.

## Repo conventions and gotchas

- `mle` feature `= ["std"]`: MLE needs transcendental `f64` ops (`exp`, `ln`, `exp_m1`, `sqrt`, `powi`) and `Vec`; the crate is `no_std` by default, so all MLE code is behind `#[cfg(feature = "mle")]`.
- Commit messages: a single subject line, never a body. Never add `Co-Authored-By`.
- Format before declaring done: `cargo fmt --all`. Lint: `cargo clippy -p hyperloglog-rs -- -D clippy::all` (pedantic stays as warnings). Run `cargo build` (default, no_std) to confirm the feature gating keeps the default build clean.
- CI gates the library crates only (`hyperloglog-rs`, `hyperloglog-derive`) on stable + MSRV 1.82, with a fast low-precision lib smoke. The exhaustive precision x bits test matrix is O(n^2) at high precision and runs locally, not in CI.
- Tests live in `tests/test_hll.rs`. The `#[test_estimator]` macro auto-gates a test under `cfg(mle)` if the function name contains `mle`; otherwise use an explicit `#[cfg(feature = "mle")]`. Existing MLE tests: `test_mle_union_matches_exact`, `test_mle_cardinality_reasonable`.
- Relevant code:
  - `src/mle.rs`: `estimate_union_cardinality_mle` (2-set joint, the K=2 template), `estimate_cardinality_mle` (Ertl secant cardinality MLE), `mle_union_cardinality`, `mle_cardinality`, the `Adam` optimizer, the element-wise array helper traits.
  - `src/sketches.rs`: `HyperSpheresSketch<f64>` and `NormalizedHyperSpheresSketch`, now implemented for `HyperLogLog`. This is where the generalized estimator should plug in (a new method or an alternative sketch path).
  - `src/hyperloglog.rs`: `estimate_cardinality`, `estimate_union_cardinality_with_cardinalities`, `correct_cardinality`, `convert_hash_list_to_hyperloglog`, the register field, `harmonic_sum`, `iter_registers`/`iter_registers_zipped` (via the `Registers` trait).
- Branch: `hll-union-merger` (start from it or branch off it).

## Open questions and risks

- The P5 union-MLE inversion (MLE worse than the default) is unexplained. Run a larger randomized sweep across precisions/overlaps to characterize the small-precision advantage cleanly before heavy investment.
- Optimizer (Adam) tuning for `M*N + M + N` parameters: convergence and runtime. The 2-set MLE is already ~11x slower than the default union and the cardinality MLE is ~15000x slower than HLL++; the multi-set estimator will be more expensive. Measure, and consider whether it is worth it only at small precision.
- Whether forward-mode AD is fast enough, or a hand/SymPy gradient is required.
- Numerical stability across the joint state (the existing code already uses `exp_m1` and `max(EPSILON)` guards; carry those over).
