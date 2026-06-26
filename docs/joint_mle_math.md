# Per-register joint log-likelihood and gradient for the generalized HyperLogLog joint MLE

This document pins the math for the generalized joint MLE (Step 2 of the work described in `generalized_joint_mle_handoff.md`). It derives the per-register joint log-likelihood and its gradient for general `M` lefts and `N` rights in the disjoint-cell region model, states the boundary conventions, and shows that the result reduces exactly to the existing 2-set estimator `mle_union_cardinality` (`src/mle.rs`) at `M = N = 1`.

All claims here are validated two ways: symbolically against the existing `M = N = 1` code (every term is matched to a line), and numerically by Monte-Carlo against a brute-force simulation of the region model for `M = N = 1` and `M = 2, N = 1` (the inclusion-exclusion likelihood matches the simulated frequencies to within sampling noise, including the boundary cases of register value 0 and saturation).

## 1. Sets, shells, and disjoint regions

We have `M` nested left counters `A_0 subset A_1 subset ... subset A_{M-1}` and `N` nested right counters `B_0 subset B_1 subset ... subset B_{N-1}`. Define the shells

- left shell `L_i = A_i \ A_{i-1}` for `i = 0..M-1`, with `A_{-1} = empty`,
- right shell `R_j = B_j \ B_{j-1}` for `j = 0..N-1`, with `B_{-1} = empty`.

The disjoint regions that partition the observed universe (every element that is in at least one set lives in exactly one region) are

- the overlap grid `O_ij = L_i intersect R_j`, cardinality `n_ij`, for `i = 0..M-1`, `j = 0..N-1`,
- the left margins `D^A_i = L_i \ B_{N-1}` (the part of left shell `i` in no right set), cardinality `a_i`,
- the right margins `D^B_j = R_j \ A_{M-1}` (the part of right shell `j` in no left set), cardinality `b_j`.

That is `M*N + M + N` non-negative region cardinalities. Collect them as `n_rho`, `rho` ranging over all regions, and optimize in log-space with `phi_rho = ln(n_rho)`.

Which counters contain a region (this drives everything below):

- `O_ij` is contained in the left counters `A_i, A_{i+1}, ..., A_{M-1}` and the right counters `B_j, B_{j+1}, ..., B_{N-1}` (every set whose index is at least the shell index, by nesting).
- `D^A_i` is contained in the left counters `A_i, ..., A_{M-1}` only (no right set).
- `D^B_j` is contained in the right counters `B_j, ..., B_{N-1}` only (no left set).

## 2. Single-register model and building blocks

Fix one register index. `m = 2^P` registers, `P = P::EXPONENT`. The largest storable register value is `q + 1 = 2^B - 1` (`B = B::NUMBER_OF_BITS`). Values `1..q` are exact geometric ranks, `0` means empty, `q + 1` means saturated.

Under the standard Poissonized HyperLogLog model, the elements of a region `rho` that hash to this register with a given rank are independent Poisson. Each region `rho` therefore deposits a maximum rank `M_rho` in `{0, 1, ..., q+1}` at this register, independently across regions, with

```
P(M_rho <= k) = exp(-x_rho(k)),   x_rho(k) = n_rho * 2^-(P + k),   for 0 <= k <= q,
P(M_rho <= q+1) = 1                                                 (the cap is certain).
```

Define the building-block scalars used by the code, for a region `rho` at level `k`:

```
x = n_rho * 2^-(P + k) = e^{phi_rho} * 2^-(P + k)
y = exp(-x)                         (survival: P(M_rho <= k))
z = 1 - y = 1 - exp(-x)             (occupancy: P(M_rho >= k+1))
```

`y` is computed via `exp_m1` for stability, exactly as in `src/mle.rs` (`y = 1 + (-x).exp_m1()`, `z = -(-x).exp_m1()`).

Two identities make every formula below collapse to the code's `y, z`:

- **Halving identity.** `x_rho(k-1) = 2 * x_rho(k)`, hence `y_rho(k-1) = exp(-2 x_rho(k)) = y_rho(k)^2`. Adjacent levels are squares of each other.
- **Exact-value mass.** For `1 <= k <= q`,
  `P(M_rho = k) = P(M_rho <= k) - P(M_rho <= k-1) = y_rho(k) - y_rho(k)^2 = y_rho(k) * z_rho(k)`.
  At the ends: `P(M_rho = 0) = y_rho(0) = exp(-n_rho 2^-P)`, and `P(M_rho = q+1) = 1 - y_rho(q) = z_rho(q)`.

## 3. Ceilings and the observed register state

At this register the observed data is the pair of monotone vectors

```
left values:  a^*_0 <= a^*_1 <= ... <= a^*_{M-1}      (a^*_i = register value of A_i)
right values: b^*_0 <= b^*_1 <= ... <= b^*_{N-1}      (b^*_j = register value of B_j)
```

(The stars distinguish observed register values from the margin cardinalities `a_i, b_j` of Section 1.) Each counter value is the maximum of `M_rho` over the regions it contains:

```
a^*_i = max over { O_i'j : i' <= i, all j } and { D^A_i' : i' <= i } of M_rho
b^*_j = max over { O_ij' : j' <= j, all i } and { D^B_j' : j' <= j } of M_rho
```

Because a region's max cannot exceed any counter that contains it, each region `rho` has a **ceiling** equal to the minimum observed value over the counters that contain it. By the monotonicity of the observed vectors this minimum is taken at the smallest containing index:

```
ceil(O_ij)  = min(a^*_i, b^*_j)
ceil(D^A_i) = a^*_i
ceil(D^B_j) = b^*_j
```

## 4. General per-register likelihood (inclusion-exclusion)

The observed event is "every counter equals its value exactly". Write it as "every counter is `<= its value`" minus the boundary where some counter is strictly below. Inclusion-exclusion over which counters are pushed one level down gives a signed sum of pure "all counters `<= something`" events, and each such event factorizes over the independent regions into a product of survivals. The result, the per-register likelihood, is

```
P_reg(phi) = sum over u in {0,1}^M, v in {0,1}^N  of
             (-1)^{|u| + |v|}  *  exp( - sum over regions rho of x_rho( gamma_rho(u, v) ) )
```

where the knockdown vectors `u` (one bit per left counter) and `v` (one bit per right counter) lower the corresponding counter value by one, and the region's effective level is the ceiling recomputed from the knocked-down values. The ceiling is the minimum knocked-down value over ALL counters that contain the region, not just the lowest-index one: a knockdown can break the monotonicity of the observed values (for example `a^* = [5, 5]` knocked by `u = [0, 1]` becomes `[5, 4]`), so a higher-index counter can become the binding constraint. With `suffix_min_a(i) = min over i' >= i of (a^*_{i'} - u_{i'})` and `suffix_min_b(j) = min over j' >= j of (b^*_{j'} - v_{j'})`:

```
gamma(O_ij)(u, v) = min( suffix_min_a(i), suffix_min_b(j) )
gamma(D^A_i)(u)   = suffix_min_a(i)
gamma(D^B_j)(v)   = suffix_min_b(j)
```

For `M = N = 1` each region is contained in at most one counter per side, so the suffix minima are trivial and these reduce to `min(a^* - u, b^* - v)`, `a^* - u`, `b^* - v`. The distinction only matters for `M >= 2` or `N >= 2`. (Implementation note: getting this wrong, by using only the lowest-index counter, yields a likelihood that is self-consistent and passes a finite-difference gradient check but is the wrong model, manifesting as deep overlap cells that are not recovered and whose error does not shrink with precision.)

This is exact for any `M, N` and any monotone observed state. The negative log-likelihood minimized by the optimizer is the sum over registers,

```
NLL(phi) = - sum over registers of ln P_reg(phi).
```

### Boundary conventions

The survival `x_rho(k)` is extended to the out-of-range levels so the single formula above covers the empty and saturated registers:

- **Strictly-below-empty.** If a counter's value is `0`, knocking it down targets level `-1`. Define `x_rho(-1) = +inf`, so `exp(-x_rho(-1)) = 0`: every inclusion-exclusion term that knocks a zero-valued counter vanishes. Equivalently, force `u_i = 0` whenever `a^*_i = 0` (and `v_j = 0` whenever `b^*_j = 0`). A region capped at ceiling `0` then contributes the empty factor `exp(-x_rho(0)) = y_rho(0)`.
- **Saturation.** If a counter's value is `q + 1`, its un-knocked branch targets level `q + 1`. Define `x_rho(q+1) = 0` so `exp(0) = 1` (reaching the cap is certain). The knocked branch targets level `q`, contributing `y_rho(q)`. A region whose ceiling is `q + 1` therefore contributes `1 - y_rho(q) = z_rho(q)` through the two branches, the correct saturated mass.

## 5. General gradient

Because `x_rho(k) = e^{phi_rho} 2^-(P+k)` depends on `phi_rho` only through the factor `e^{phi_rho}`, we have `d x_rho(k) / d phi_rho = x_rho(k)`. Differentiating `ln P_reg` term by term,

```
d ln P_reg / d phi_sigma
  = - [ sum_{u,v} (-1)^{|u|+|v|} * x_sigma(gamma_sigma(u,v)) * exp( - sum_rho x_rho(gamma_rho(u,v)) ) ]
      / [ sum_{u,v} (-1)^{|u|+|v|} *                          exp( - sum_rho x_rho(gamma_rho(u,v)) ) ]
```

i.e. the gradient component for region `sigma` is minus the `P_reg`-weighted (signed) average of `x_sigma` evaluated at `sigma`'s effective level. The denominator is `P_reg` itself. The full gradient of the NLL is the negated sum of these over registers.

This is exactly the quantity forward-mode autodiff (`num-dual`) returns when `P_reg` is written once with the trait's `exp`/`exp_m1`. We will use that as the primary implementation and the closed form above (plus central finite differences) as the cross-check oracle, per the handoff's recommended workflow. The `2^{M+N}` signed terms are cheap because they are tabulated once per distinct observed register pattern (Section 7), and `M, N` are small.

## 6. Reduction to `mle_union_cardinality` at M = N = 1

With `M = N = 1` there are three regions: `L = D^A_0` (left difference), `R = D^B_0` (right difference), `J = O_00` (intersection), with `phi = (ln n_L, ln n_R, ln n_J)`. This matches the code's `phis = [left_difference.ln(), right_difference.ln(), intersection.ln()]` and the returned union `e^{phi_L} + e^{phi_R} + e^{phi_J}`. At one register the observation is `(a, b) = (a^*_0, b^*_0)`. The inclusion-exclusion sum has four terms (`u, v in {0,1}`), with `gamma_L = a - u`, `gamma_R = b - v`, `gamma_J = min(a - u, b - v)`. Writing `y_L = y_L(a)`, `y_R = y_R(b)`, `y_J = y_J(min(a,b))` and using `y(k-1) = y(k)^2`:

**Case `a < b` (left smaller).** `min(a - u, b - v) = a - u` always, so the sum factorizes:

```
P_reg = [ y_L(a) y_J(a) - y_L(a-1) y_J(a-1) ] * [ y_R(b) - y_R(b-1) ]
      = y_L y_J (1 - y_L y_J) * y_R z_R(b)
```

The score for `L` is

```
d ln P_reg / d phi_L = -x_L(a) + x_L(a) * y_L y_J / (1 - y_L y_J)
                     = x_L(a) * ( y_L y_J / (1 - y_L y_J) - 1 ).
```

Note `1 - y_L y_J = z_J + y_J z_L` (since `z_J + y_J z_L = (1 - y_J) + y_J(1 - y_L) = 1 - y_L y_J`). This is exactly the code's `left_reciprocal = left_smaller_k * (y[2] y[0] / (z[2] + y[2] z[0]) - 1)`, multiplied by `x_L(a)` and weighted by the multiplicity of registers with `a < b` at value `a` (`left_multiplicities_smaller[a]`), matching the `gradients += x_register .* delta` accumulation. The `R` score is `x_R(b)(y_R/z_R - 1)`, matching `right_larger_k * (y[1]/z[1] - 1)`.

**Case `a > b` (left larger).** Symmetric. `J` is capped at `b`, so `L` alone must reach `a`: `P_reg = y_L(a) z_L(a) * [ y_R y_J (1 - y_R y_J) ]`. The `L` score is

```
d ln P_reg / d phi_L = -x_L(a) + x_L(a) y_L / z_L = x_L(a) ( y_L / z_L - 1 ),
```

matching the code's `left_larger_k * (y[0]/z[0] - 1)`.

**Case `a = b = k` (joint).** All four terms contribute. With `Y_L = y_L(k)`, `Y_R = y_R(k)`, `Y_J = y_J(k)`:

```
P_reg = Y_L Y_R Y_J - Y_L^2 Y_R Y_J^2 - Y_L Y_R^2 Y_J^2 + Y_L^2 Y_R^2 Y_J^2
      = Y_L Y_R Y_J * ( z_J + y_J z_L z_R ).
```

The factor `z_J + y_J z_L z_R` is exactly the code's `zj_plus_yjoint_zlr = z[2] + y[2] z[0] z[1]`, whose reciprocal appears throughout the joint gradient block. Differentiating `ln P_reg`:

```
d ln P_reg / d phi_J = x_J(k) * ( (y_J y_L + y_J z_L y_R) / (z_J + y_J z_L z_R) - 1 )
```

which is the code's joint term `joint_k * ((y[2] y[0] + yjoint_right_zleft) * reciprocal_zj_plus_yjoint_zlr - 1)` with `yjoint_right_zleft = y[2] z[0] y[1]`. The `L` and `R` joint terms match `yjoint_left_zright` and `yjoint_right_zleft` over the same denominator identically.

**Boundary terms.** The all-empty contribution (`ceil = 0` for a region) gives `ln y_rho(0) = -x_rho(0)`, whose `phi`-derivative is `-x_rho(0)`. This is the code's `zeros_0 .* x_0` subtraction, with `zeros_0` counting, per region, the registers where that region is forced empty. The saturated contribution gives `ln z_rho(q)` and the denominator `z_q[2] + y_q[2] z_q[0] z_q[1]` for the joint, exactly the `zeros_q` block and `denominator = 1/(z_q[2] + y_q[2] z_q[0] z_q[1])`.

Every term of `mle_union_cardinality` is therefore an instance of the general formula at `M = N = 1`. This is the primary correctness anchor for the implementation.

## 7. Aggregation by multiplicities

`P_reg` depends on the register only through the observed pattern `(a^*_0..a^*_{M-1}, b^*_0..b^*_{N-1})`. The NLL is therefore a sum over distinct patterns weighted by how many registers show each pattern. The 2-set code realizes this with the multiplicity vectors (`left_multiplicities_smaller/larger`, `right_...`, `joint_multiplicities`, plus the `zeros_0`/`zeros_q` boundary counts). The generalization tabulates, in one pass over the `m` registers, the count of each distinct monotone pattern, and the optimizer then iterates over distinct patterns rather than registers. For small `M, N` the number of distinct patterns is bounded by the register count and in practice far smaller.

## 8. Wolfram Alpha cross-checks

Single building block (occupancy log-derivative), confirming `x y / z`:

```
d/dt log(1 - exp(-exp(t) * w))     ->   w e^t e^{-w e^t} / (1 - e^{-w e^t})
```

Empty-factor log-derivative, confirming `-x`:

```
d/dt ( -exp(t) * w )               ->   -exp(t) * w
```

Joint-case likelihood factorization (paste the right side and the left side, confirm equal). With `L = e^{-x_L}`, `R = e^{-x_R}`, `J = e^{-x_J}` standing for `Y_L, Y_R, Y_J`:

```
L*R*J - L^2*R*J^2 - L*R^2*J^2 + L^2*R^2*J^2  =  L*R*J*((1-J) + J*(1-L)*(1-R))
```

Joint-case `phi_J` score (with `y_* = exp(-x_*)`, `z_* = 1 - y_*`), confirm it simplifies to the code's joint term:

```
d/dt log( exp(-A e^t) ... )   ->   use A = 2^-(P+k); the simplified result is
x_J * ( (y_J y_L + y_J z_L y_R) / (z_J + y_J z_L z_R) - 1 )
```

## 9. Polynomial evaluation (the level-factor / block-collapse form)

The inclusion-exclusion sum of section 4 has `2^(M+N)` terms per register pattern, which is the cost wall for larger `M, N` (M=N=5 took ~15 s, M=N=8 would take hours). Exploiting two structural facts, the same `P_reg` and its gradient can be evaluated in `O((M+N)^2 + M*N)` per pattern. This is what the production estimator uses, and the `2^(M+N)` form is retained only as the test oracle.

The two facts:

1. Independence across rank levels. By Poisson thinning, the occupancy `O_rho(k)` ("region `rho` has an element of rank exactly `k` at this register") is independent across regions AND levels, with `P(O_rho(k) = 1) = z_rho(k) = 1 - exp(-x_rho(k))`. Each counter value is the maximum occupied level over the regions it contains, so the likelihood is a product over the `q` rank levels rather than a sum over `2^(M+N)` corners.

2. Nested-chain monotonicity. The left counters form a chain and the right counters form a chain. The counters that attain a given value `w` are a contiguous block (the observed values are sorted), and because nesting makes "contained in `A_l`" monotone in `l`, hitting the smallest counter in a block hits all of them. So each block collapses to its smallest index.

The result factorizes as `P_reg = exp(base) * prod over distinct values w of Q_w`:

- CDF base: `base = -sum over regions of x_rho(ceil_rho)` with `ceil(O_ij) = min(a^*_i, b^*_j)`, `ceil(D^A_i) = a^*_i`, `ceil(D^B_j) = b^*_j`. A region whose ceiling is `q + 1` (saturated) contributes 0.
- For each value `w` that some counter attains, let `p` be the smallest left index with `a^*_p = w` and `r` the smallest right index with `b^*_r = w` (either may be absent). Define the hitter sets `L_w = {D^A_p} union {O_{p,j} : b^*_j >= w}` and `R_w = {D^B_r} union {O_{i,r} : a^*_i >= w}`, and `y_rho(w) = exp(-x_rho(min(w, q)))` (the saturated top bucket uses level `q`). With `PL = prod_{L_w} y`, `PR = prod_{R_w} y`, and `PLR = prod_{L_w union R_w} y` (the union shares only the cell `O_{p,r}`):
  - both blocks present: `Q_w = 1 - PL - PR + PLR`,
  - only the left block: `Q_w = 1 - PL`,
  - only the right block: `Q_w = 1 - PR`.

Gradient: `ln P_reg = base + sum_w ln Q_w`. The base contributes `-x_rho(ceil_rho)` to `d/dphi_rho` (one term per region). For each value `w`, `d ln Q_w / d phi_rho = (1 / Q_w) * dQ_w` where `dQ_w/dphi_rho` is assembled from `d(prod y)/dphi_rho = (prod y) * (-x_rho(w))` for the regions in each product (only the hitter regions of level `w` have nonzero contribution). Both the value and the gradient are cancellation-free (products of positive factors), unlike the signed `2^(M+N)` sum.

This polynomial form reproduces the section-4 formula to machine precision (validated against both the `2^(M+N)` oracle and a Monte-Carlo simulation for `M = N = 1`, `M = 2, N = 1`, `M = 2, N = 2`, `M = 3, N = 2`, including ties, zeros, and saturation). For `M = N = 1` each region is contained in at most one counter per side, so the blocks are trivial and the achievement factors reduce exactly to the three cases of section 6.

Reference for the open problem this resolves: Otmar Ertl notes (arXiv 1702.01284) that joint estimation across more than two sketches "would scale at least exponentially with the number of involved HyperLogLog sketches" for arbitrary sets. The nested-chain structure here (only `M*N + M + N` disjoint regions, not a `2^k`-region Venn diagram) is what makes a polynomial likelihood possible.

## 10. Optimizer (Levenberg-damped Newton)

The warm-started MAP objective (log-likelihood plus the marginal-anchor log-prior) is maximized by a single second-order optimizer, Levenberg-damped Newton (`DampedNewton` in `src/mle/optimizers.rs`). Each iteration forms the information matrix `A = -H` from the analytic MAP Hessian of section 11, solves the damped system `(A + lambda*I) delta = g` (Cholesky fast path, pivoted Gaussian fallback), and accepts the step only when it increases the objective, decreasing `lambda` toward a pure Newton step on acceptance and increasing it (with a final steepest-ascent line search) on rejection. Accepting on the objective value rather than the gradient norm keeps it correct on the flat ridges of weakly identified deep cells, where the gradient norm has spurious minima.

An earlier design exposed several optimizers behind a `JointOptimizer` trait (`Lbfgs`, `Adam`, `RmsProp`, the `Chain<A, B>` composition, and `FisherScoring`) selectable by turbofish. They were retired once the analytic Hessian (section 11) made damped Newton both faster and more accurate than every first-order alternative: it converges in far fewer objective evaluations and, because each step uses the true curvature, it reaches a higher attained MAP objective on the anisotropic deep-cell instances where a greedy first-order method stalls on the ridge.

This mirrors Ertl's 2-set joint MLE, which uses a quasi-Newton method warm-started from the inclusion-exclusion estimate and reports convergence in roughly 13-42 iterations (arXiv 1702.01284, Table 1). The damping and the analytic Hessian are the additions, motivated by the multi-modality and the anisotropy that appear once the disjoint-region model has more than three cells.

## 11. Analytic Hessian of the polynomial objective

The damped-Newton optimizer needs the Hessian of the MAP objective in `phi`. This section derives it in closed form from the cancellation-free polynomial likelihood of section 9, so the optimizer can build the `K x K` Hessian once per iteration (one pass over the patterns) instead of finite-differencing the analytic gradient at a cost of about `2 K` gradient evaluations per iteration. The finite-difference Hessian (`finite_difference_hessian` in `src/mle/optimizers.rs`, a test-only helper) is retained as the trusted correctness oracle, and the analytic form is validated to match it to about `1e-6` relative across random patterns and shapes.

### 11.1 Building blocks and their second derivatives

Every region's `x_rho(k) = e^{phi_rho} 2^-(P+k)` depends on its own `phi_rho` alone, so `d x_rho / d phi_rho = x_rho` and `d^2 x_rho / d phi_rho^2 = x_rho`, with all cross and other partials zero. Write `x_rho` for `x_rho(w)` at the level `w` in context.

For a product of survivals over a hitter set `S`, `PS = prod_{rho in S} y_rho(w)` with `y_rho = exp(-x_rho(w))`, the first derivative is `d PS / d phi_sigma = -x_sigma PS` when `sigma in S` (and 0 otherwise), because `d y_sigma / d phi_sigma = -x_sigma y_sigma`. The second derivative follows by differentiating again, using `d x_sigma / d phi_sigma = x_sigma`:

- both `sigma, tau in S`, `sigma != tau`: `d^2 PS / d phi_sigma d phi_tau = x_sigma x_tau PS`.
- `sigma == tau in S`: `d^2 PS / d phi_sigma^2 = (x_sigma^2 - x_sigma) PS` (the extra `-x_sigma` comes from differentiating the `x_sigma` factor itself).
- any `sigma not in S`: 0.

Compactly, with the indicator `[sigma in S]`,

```
d PS / d phi_sigma             = -x_sigma [sigma in S] PS
d^2 PS / d phi_sigma d phi_tau = ( x_sigma x_tau [sigma in S][tau in S]
                                   - x_sigma [sigma == tau][sigma in S] ) PS
```

### 11.2 Hessian of a single ln Q_w

`Q_w` is a signed sum of these products: `Q_w = 1 - PL - PR + PLR` when both blocks are present (`PL` over `L_w`, `PR` over `R_w`, `PLR` over `L_w union R_w`), or `Q_w = 1 - PL` / `Q_w = 1 - PR` for the one-sided cases. The first and second derivatives of `Q_w` are the same signed combination of the product derivatives above:

```
dQ_w/dphi_sigma         = -(dPL + dPR - dPLR)            (both-sided, drop the absent blocks otherwise)
d^2 Q_w/dphi_sigma dphi_tau = -(d^2 PL + d^2 PR - d^2 PLR)
```

Then `ln Q_w` contributes, by the quotient and product rules,

```
d^2 ln Q_w / d phi_sigma d phi_tau
  = (1 / Q_w) d^2 Q_w / d phi_sigma d phi_tau
    - (1 / Q_w^2) (dQ_w/d phi_sigma)(dQ_w/d phi_tau).
```

The first term is the curvature of `Q_w` reweighted by `1 / Q_w`, and the second is the rank-one outer product of the already-computed `ln Q_w` gradient, with a minus sign. Summed over the distinct values `w`, plus the base, gives the per-register log-likelihood Hessian.

### 11.3 Hessian of the base

The base `-sum_rho x_rho(ceil_rho)` is separable across regions and linear in each `e^{phi_rho}`, so its Hessian is diagonal: `d^2 base / d phi_sigma^2 = -x_sigma(ceil_sigma)` (zero for a saturated ceiling, and zero off the diagonal). This equals the base's gradient contribution, since `d^2 e^{phi} / d phi^2 = e^{phi}`.

### 11.4 Full per-register and per-pattern Hessian

`ln P_reg = base + sum_w ln Q_w`, so its Hessian is the diagonal base term plus the sum of the `ln Q_w` Hessians of 11.2. A register pattern that occurs `count` times contributes `count` times this Hessian. The implementation accumulates value, gradient, and Hessian in one pass over the patterns sharing the `y`, `x`, `PL`, `PR`, `PLR` work (`joint_pattern_ll_grad_hess_poly` in `src/mle/likelihood.rs`).

### 11.5 Hessian of the marginal-anchor prior

Each anchor contributes `-(weight/2) (ln S - ln estimate)^2` to the MAP objective, with `S = sum_{rho in regions} e^{phi_rho}` and `n_rho = e^{phi_rho}`. Let `r = ln S - ln estimate` (the residual). The gradient is `d/dphi_sigma = -weight r (n_sigma / S)` for `sigma in regions`. Differentiating again, using `d S / d phi_tau = n_tau`, `d r / d phi_tau = n_tau / S`, and `d (n_sigma / S) / d phi_tau = (n_sigma / S)[sigma == tau] - (n_sigma n_tau / S^2)`:

```
d^2 / d phi_sigma d phi_tau
  = -weight [ (n_sigma n_tau / S^2)
              + r ( (n_sigma / S)[sigma == tau] - n_sigma n_tau / S^2 ) ]
  = -weight [ (1 - r) (n_sigma n_tau / S^2) + r (n_sigma / S) [sigma == tau] ]
```

for `sigma, tau` both in the anchor's region set (zero otherwise). This couples every pair of regions inside one anchor (a dense block on those indices), unlike the diagonal-only base. The full MAP Hessian is the log-likelihood Hessian of 11.4 plus the sum of these anchor blocks.

### 11.6 M = N = 1 sanity anchor

At `M = N = 1` the three regions are `L = D^A_0`, `R = D^B_0`, `J = O_00`, and (for `a = b = k`, the joint case of section 6) the single achievement factor is `Q_k = z_J + y_J z_L z_R` in the notation there, with `y_* = e^{-x_*}`, `z_* = 1 - y_*`. Specializing 11.2 to the hitter sets `L_k = {L, J}` and `R_k = {R, J}` (so `PL = y_L y_J`, `PR = y_R y_J`, `PLR = y_L y_R y_J`) reproduces the curvature of the closed-form 2-set joint term: the diagonal `d^2 ln Q_k / d phi_J^2` matches differentiating the section-6 score `x_J ((y_J y_L + y_J z_L y_R)/(z_J + y_J z_L z_R) - 1)` once more in `phi_J`, and the off-diagonal `d^2 / d phi_L d phi_J` matches differentiating that same score in `phi_L`. This is the smallest case where the dense (non-diagonal) structure of the `ln Q_w` Hessian appears, and the analytic-vs-finite-difference test exercises it directly.
