# Why the hash-list gap codes are optimal, and the cost of deriving them

The hash-list representation stores distinct composite hashes sorted, and codes the gaps between consecutive values with a Rice code. For years the Rice parameter per `(precision, bits, hash_width)` lived in a generated lookup table (`OPTIMAL_RICE_COEFFICIENTS`, 341 entries, produced by the `optimal-gap-codes` crate). This note records why those parameters are what they are, how well a closed form reproduces them, and exactly how much compression a closed form gives up. The conclusion is that the parameter is a one-line function of the precision and the width, with the only non-derivable part being the upper width bound.

## Why Rice is optimal here

Three facts chain together.

1. The gaps are geometric. Take `D` distinct hashes spread uniformly over the `2^w` composite space and sort them. The spacings between consecutive sorted values are the spacings of uniform order statistics, which are exponential in the continuous limit and geometric once discretized, with mean `mu = 2^w / D`. So the thing being coded is a geometric source whose mean is fixed entirely by the width `w` and the occupancy `D`.

2. Golomb is the provably optimal prefix code for a geometric source (Gallager and van Voorhis, 1975), and Rice is Golomb restricted to a power-of-two parameter `m = 2^k`. The power-of-two restriction is what lets encode and decode be a shift plus a mask instead of a division, and it gives up almost nothing. The optimal parameter for a geometric mean `mu` is `m` near `mu * ln 2`, so the optimal Rice parameter is `k = round(log2(mu) - 0.5288) = w - log2(D) - 0.5288`.

3. The occupancy is pinned by the downgrade schedule. The list holds a flat plateau near `2^(P - 1.53)` distinct hashes and the register-array budget forces the occupancy down at wide `w`. That is the entire content of the table: `k` tracks `log2` of the mean gap, and the mean gap is set by the width and the budget-pinned occupancy.

The composite hash is not perfectly uniform (uniform index bits, then a geometric register and a residual), so the real gap law is a mixture rather than pure geometric. That mixture is why the empirical optimum occasionally lands one Rice step away from the pure-geometric prediction.

## The closed form

```
k(P, B, w):
    w_min = P + B
    if w == w_min { 0 }            // narrowest width: list is full, gaps ~ 1, Rice-0
    else { max(0, w - (P - 1)) }   // k tracks log2(mean gap) = w - log2(occupancy)
```

The `max(0, ...)` is conceptual: every non-narrowest width satisfies `w >= P + B + 1`, so `w - (P - 1) >= B + 2 >= 6`, and the clamp never fires. The implementation therefore omits it.

Diffed against all 341 table entries:

| model | exact | within plus or minus one |
|---|---|---|
| one-line `w - (P - 1)` | 82.4% | 100% |
| budget-occupancy model | 93.8% | 100% |

The budget model solves the occupancy from `D * (w - log2 D + 1.414) = frac * 2^P * B` with `frac = 0.84`, which is not a free knob: it matches the generator's own `rate < 0.8` sampling cutoff. It is more accurate but needs a `log2` and a bisection. The one-line form needs neither.

## The cost of the misses

A parameter that is one step off is nearly free, because the Rice expected-length curve is a shallow V around its optimum. So the 18% miss rate of the one-line form is not an 18% cost. Measuring the extra expected bits under the geometric model, placing the true mean at the center of each table value's optimality interval (the position most favorable to the table, hence conservative against the formula):

| scope | one-line typical | one-line worst | budget typical | budget worst |
|---|---|---|---|---|
| on the affected widths only | 2.92% | 5.98% | 3.81% | 7.89% |
| all widths, occupancy-weighted | 0.055% | 0.110% | 0.027% | 0.055% |
| all widths, uniform per-width | 0.513% | 1.053% | 0.235% | 0.486% |
| max single width | 7.13% | | 10.67% | |

The occupancy weighting is realistic but is dominated by the full narrowest state, which the formula codes exactly (`k = 0`), so it reads very low. The uniform weighting refuses to let that dominate and still lands under a quarter percent for the budget model and about half a percent for the one-line form. All of this is on the hash-list representation, the transient pre-dense low-cardinality regime measured in a few KB, so the absolute give-up is a handful of bytes that the counter passes through on its way to dense. Correctness is unaffected, since Rice round-trips for any parameter.

The one-line form costs roughly twice the budget model, both negligible, so the extra machinery of the budget model is not worth it. The shipped code uses the one-line form.

## What is not derivable: the width bound

The table encodes two things, the Rice parameter per width and the set of widths the codec uses. The width set is always the contiguous range `P + B ..= w_max`. The lower bound `w_min = P + B` is `SMALLEST_VIABLE_HASH_BITS`. The upper bound `w_max` is the widest width at which gap coding is used before the list goes dense. For `P >= 11` it equals `LARGEST_VIABLE_HASH_BITS`, but for smaller precisions it is truncated below it in an irregular way (for example `P7` tops out at 14 while `LARGEST_VIABLE` is 16, and `P8` jumps to 21), because that is where the list saturates. This bound is a structural property of the dense-switch threshold, not a clean function of `P` and `B`, so it is kept as a small frozen constant (`MAX_GAP_HASH_BITS`, 45 entries) rather than computed. With `w_max` fixed, the width set is reproduced exactly, so switching to the formula changes only the `k` values, at the cost measured above.
