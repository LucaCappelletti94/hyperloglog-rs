# Handoff: encode the zero-register count into the harmonic-sum word to make Ertl sigma/tau O(1)

## Goal

Make `HyperLogLog::sigma_tau()` (Ertl's analytical tau/sigma cardinality estimator) run in O(1) per estimate instead of the current O(m) register scan, without adding any per-counter memory. Do it by packing the zero-register count into the unused low mantissa bits of the already-stored `harmonic_sum` word, active only in the dense bias-corrected (HLL++) band and dormant before and after.

## Why

The shipped fitted-polynomial bias correction (`HYPERLOGLOG_CORRECTION_COEFFS` in `src/correction_coefficients.rs`) is unreliable: it fails at P8 (about 2.3 percent bias where the table is 0.3), it oscillates across the corrected range, and it damages the estimate at high load where the raw estimator is already unbiased. Ertl's tau/sigma estimator (same paper as the MLE, arXiv:1702.01284) is the robust table-free replacement: on real counters it holds 0.08 to 0.19 percent bias across P8, P10, P12, with none of the polynomial's failure modes and no fitted coefficients at all. The one drawback measured so far is speed: the current implementation rebuilds the register multiplicity histogram on every call, so it is O(m) (about 1.1 microseconds at P10, 4.4 at P12, 17 at P14, versus 6.4 nanoseconds for the O(1) polynomial). This work removes that drawback.

The intended consumer is HyperBall on graphs with billions of nodes, so there are billions of counters and every byte per counter matters. A separate zero-count field (even a `u16`) would cost gigabytes at that scale, which is why the count is packed into the existing 8-byte harmonic-sum word for zero extra memory.

## Background: the three dense regimes and the current word encoding

The counter's estimation state lives entirely in one `f64` field, `harmonic_sum` (`src/hyperloglog.rs`), whose interpretation is mode-dependent and self-describing (no separate discriminator byte):

- Pre-dense (value list or hash list): the word is a metadata bitfield. Discriminated by the sign bit via `is_hyperloglog()` (`src/hyperloglog.rs`, the `harmonic_sum.to_bits().leading_zeros() != 0` test).
- Dense, zeros mode (low load, below the linear-counting crossover): the word is a NaN-boxed zero-register count. Discriminated by `harmonic_sum.is_nan()`. The estimate here is linear counting, O(1) from the decoded count. Helpers: `encode_dense_zeros`, `decode_dense_zeros`, `DENSE_ZEROS_NAN_TAG`, `DENSE_ZEROS_MASK`.
- Dense, harmonic mode (bias-corrected below `7.5*m`, raw above it): the word is the real harmonic sum `H = sum 2^-register`. `dense_harmonic_sum()` returns it in O(1) here (and O(m), a scan, in zeros mode). `number_of_zero_registers()` is the opposite: O(1) in zeros mode (decode), O(m) in harmonic mode (scan).

So today, in the harmonic regime we have `H` in O(1) but the zero count only via an O(m) scan. That is the gap this handoff closes.

`correction_upper_bound::<P>()` is `7.5 * m` (the top of the corrected band). `bias_corrected_raw_cardinality(harmonic_sum)` computes the raw estimate and, below the bound, applies `correct_cardinality` with the polynomial. The zeros-mode transition is `maybe_switch_to_zeros_mode` (around `src/hyperloglog.rs:459`), which fires when the linear-counting estimate is at or below `HYPERLOGLOG_LINEAR_COUNT_THRESHOLD[P-4][B-4]`.

## The Ertl estimator and the moments identity

Already implemented in `src/sigma_tau.rs`:

- `sigma(x) = x + sum_{k>=1} x^(2^k) 2^(k-1)`, with `sigma(0)=0`, `sigma(1)=inf`.
- `tau(x)`: the convergent series, `tau(0)=tau(1)=0`.
- `ertl_cardinality(c)`: from the full multiplicity histogram `c` (`c[k]` = number of registers equal to `k`, `k` in `0..=q_plus_one`, `q_plus_one = 2^B - 1`). O(m) to build `c`.
- `ertl_cardinality_from_moments(H, zeros, saturated)`: the O(1) form. Its identity is the crux of this work:

```
q_plus_one = 2^B - 1,  q = q_plus_one - 1,  m = 2^P,  alpha_inf = 1/(2 ln 2)
interior = H - zeros - saturated * 2^-(q_plus_one)        // sum_{k=1}^{q} c[k] 2^-k
z = m * sigma(zeros/m) + interior + m * 2^-q * tau((m - saturated)/m)
n = alpha_inf * m^2 / z
```

There is a unit test `ertl_moments_matches_histogram` proving `ertl_cardinality_from_moments` equals `ertl_cardinality` on real counters. So once the counter can hand back `(H, zeros, saturated)` in O(1), sigma/tau is O(1).

`saturated` (count at `q_plus_one`) is essentially always 0 for B6 in any realistic range (register value 63 needs astronomical cardinality), so it can be treated as 0 in the corrected band, or tracked cheaply if a fully general estimator is wanted. Only `zeros` needs the new encoding.

## Why both H and zeros are needed at once, and only in the band

`z = H + m*(sigma(zeros/m) - zeros/m) + saturation terms`. The linear part of `zeros` cancels against `H`, but `sigma(x) - x = x^2 + 2 x^4 + ...` is nonlinear, so `zeros` itself is required, not just its contribution to `H`. It is only needed where it is nonzero: below the crossover the counter is in zeros mode already (count is the word, estimate is linear counting), and above the corrected band the counter has essentially no zeros. So the encoding is live exactly in the bias-corrected band `[crossover, 7.5*m]`, which is also exactly where the raw estimate has bias to correct.

## Empirical facts (measured, `examples/zero_count_probe.rs`)

Maximum zero-register count while dense and in the bias-corrected regime, per cell, scales as roughly `0.10 * m` for B6 and `0.25 * m` for B4 (B4 is the worst case). Measured maxima:

| P | B4 | B5 | B6 |
|---|---|---|---|
| 4 | 0 | 0 | 0 |
| 8 | 57 | 36 | 1 |
| 10 | 263 | 168 | 106 |
| 12 | 1024 | 692 | 431 |
| 15 | 9882 | 6590 | 3482 |

So the bits needed, `ceil(log2(max+1))`, are 0 at P4, about 6 at P8, 11 at P12, 14 at P15, and by the `0.25*m` bound about 16 to 17 at P18. This is per cell, not uniform, hence a per-`(P, B)` const rather than a fixed `u16`. Note the probe undershoots at P16 and above (it printed an identical 7011 for all three widths there because the sweep did not land on the crossover), so step 1 below is to tighten it before freezing the const.

Precision impact of stealing `k` low mantissa bits from `H`: the harmonic sum in the band has magnitude about `0.1*m` to `0.4*m`, so it lives in the high mantissa bits and its low bits are already summation noise (the fine `2^-r` terms are below its ULP). Reading it with the low `k` bits cleared gives relative error `2^(k-52)`, at worst about `1.5e-11` (k=16, P18). Accumulated over `m` incremental updates it is at most `m * 2^(k-52)`, about `4e-6` at P18. The noise floor at P18 is about `0.2` percent, so both are 6 to 8 orders of magnitude below anything observable. Accuracy cost is nil.

## Design

Add a fourth interpretation of the harmonic-mode word, confined to the bias-corrected band: the high `52 - ZERO_BITS[P][B]` mantissa bits hold the harmonic sum, the low `ZERO_BITS[P][B]` bits hold the zero-register count. In the raw regime (`raw >= 7.5*m`) the word is a plain full-precision harmonic sum (the count is zero there anyway). In zeros mode and pre-dense the word is unchanged from today.

Discrimination stays free: `is_hyperloglog()` then `is_nan()` then the raw magnitude against `7.5*m` already tell you which of the four states you are in, all from the word.

Centralize the encoding so every reader agrees:

- `harmonic_sum_in_band()` returns `H` with the low `ZERO_BITS` bits cleared (or the raw word when not in the packed band).
- `packed_zeros_in_band()` returns the zero count from the low bits.
- All estimators (the default polynomial `bias_corrected_raw_cardinality`, sigma/tau, and the MLE if it reads the word) must read `H` through the masked accessor in the band, or explicitly accept the `~1e-11` perturbation. Prefer masking so the default estimate and any bit-exact test are unaffected.

Maintenance:

- Union (HyperBall's hot path) already does an O(m) element-wise-max pass and recomputes `H`. Count the zeros in that same pass and pack them at the end. Free.
- Zeros-mode to harmonic-mode transition (`maybe_switch_to_zeros_mode` and its inverse): when materializing `H` from a scan, pack the current zero count in the same pass.
- Single insert: after the incremental `H` update, if the register left zero, decrement the packed count; keep `H` in the high bits by masking, adding the delta, re-clearing the low bits, then OR-ing the count back. This is the only place packing adds work, and it is off HyperBall's critical path.

## Implementation plan (numbered, each step has a deliverable)

1. Tighten `examples/zero_count_probe.rs` so it lands on the crossover at P16, P17, P18 (sweep finer fractions around each cell's linear-count threshold, take the max over more seeds), and freeze `const ZERO_BITS: [[u8; 3]; 15]` indexed `[P-4][B-4]` = measured `ceil(log2(max+1))` plus a 1 to 2 bit safety margin. Deliverable: the const, plus a note on the B4/P18 corner (if it needs 17 bits, decide clamp-and-cap or accept the tiny residual scan there).

2. Add the packing primitives next to the existing NaN-box helpers in `src/hyperloglog.rs`: `pack_harmonic_with_zeros(h: f64, zeros: u32) -> f64`, `unpack_harmonic(word: f64) -> f64` (mask low bits), `unpack_zeros(word: f64) -> u32`, all parameterized by `ZERO_BITS[P][B]`. Unit-test round-trip and the masked-`H` relative error bound. Deliverable: helpers plus tests.

3. Route reads through the masked accessor. Introduce `harmonic_sum_in_band()` / `packed_zeros_in_band()` and switch `bias_corrected_raw_cardinality`, `estimation_regime`, and `dense_harmonic_sum` to use the masked `H` in the band. Confirm the existing suite is unchanged (the default estimate shifts by at most `~1e-11`). Deliverable: green suite with reads centralized.

4. Maintain the packing at the three sites: union/merge, the zeros-to-harmonic transition, and single insert (`insert_register_value_and_index`). Add a `debug_assert` that the decoded zero count equals a fresh `number_of_zero_registers()` scan whenever in the band. Deliverable: maintenance wired, debug-assert passing across a proptest.

5. Make sigma/tau O(1). Rewrite `sigma_tau_cardinality` (and `sigma_tau_union_cardinality`) to, in the band, read `(H, zeros)` in O(1) and call `ertl_cardinality_from_moments` (saturated = 0 in the band, or read it if you tracked it). Keep the O(m) histogram path as the fallback for the non-`alloc` build or for correctness cross-check. Deliverable: O(1) sigma/tau.

6. Boundaries and invariants. Handle: the raw regime (word is full-precision `H`, no packed bits, sigma/tau reduces to `alpha_inf*m^2/H`); saturation (decide track-or-assume-zero); `PartialEq` (the packing is deterministic from the registers, so equal register states pack equal, but confirm the bit-exact comparison introduced by the zeros-mode work still holds); serde round-trip of the packed word. Deliverable: invariants documented and tested.

7. Validate and measure. Assert O(1) sigma/tau equals the O(m) histogram sigma/tau to about `1e-9` across `(P, B)` and cardinalities. Re-run `examples/sigma_tau_compare.rs` for the error table, and measure O(1) `ns/call` (expect low hundreds of nanoseconds, dominated by the two series, constant in P, versus 1 to 17 microseconds for the O(m) path). Add a union-throughput measurement to confirm the merge-time packing does not regress HyperBall's hot path. Deliverable: before/after speed and error numbers.

## Gotchas and risks

- Do not let the packed low bits leak into any comparison or hash that assumes the word is the pure harmonic sum. Mask first. This includes `PartialEq` and any serialization checksum.
- The incremental single-insert path must re-clear the low bits after adding the harmonic delta, or the delta's low bits will corrupt the packed count. The union path avoids this by recomputing `H` from scratch.
- The `ZERO_BITS` const must be sized for the worst seed, not the mean. The probe currently undershoots at high P (step 1 fixes this). If B4/P18 genuinely needs 17 bits, either widen there or accept a one-cell O(m) fallback (B4 at P18 is an unusual configuration).
- Saturation (`c[q_plus_one] > 0`) never happens for B6 in range, but if a fully general estimator is wanted for small B, track the saturated count too (or keep the O(m) fallback when saturation is present).
- The masked `H` perturbs the default polynomial estimate by about `1e-11`. That is negligible, but if any test asserts bit-exact default estimates, it must read through the masked accessor too.

## Current state (uncommitted on branch `hll-union-merger`)

- `src/sigma_tau.rs`: `sigma`, `tau`, `ertl_cardinality` (O(m) histogram), `ertl_cardinality_from_moments` (O(1) given moments), `SigmaTau<H>` wrapper, `HyperLogLog::sigma_tau()`, `sigma_tau_cardinality` / `sigma_tau_union_cardinality` (currently O(m)), and tests including `ertl_moments_matches_histogram`.
- `src/lib.rs`: `pub mod sigma_tau;` and `SigmaTau` in the prelude.
- `examples/sigma_tau_compare.rs`: four-way accuracy (uncorrected, polynomial, table, sigma/tau) and timing, including a precomputed-moments O(1) timing probe.
- `examples/zero_count_probe.rs`: the max-zeros-in-band measurement (needs the step-1 tightening at P16 to P18).
- `src/hyperloglog.rs`: a `shipped_correction_audit` test module (a diagnostic that evaluates the shipped polynomial vs uncorrected vs the table on the Monte Carlo report ground truth, split by in-domain vs below-domain). Keep or fold into the correction discussion.

None of this is committed. The sigma/tau estimator, the moments identity, and the empirical zero-count measurement are all in place. What remains is the packing (steps 2 to 6) and the measurement (step 7).

## Gates

`cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features` clean, `cargo test --release` green, and the lib must build with and without default features. The insert and union paths are hot for HyperBall, so keep an eye on their benchmarks (`benches/hyperloglog_insert.rs`, `benches/hyperloglog_union.rs`) across this change.
