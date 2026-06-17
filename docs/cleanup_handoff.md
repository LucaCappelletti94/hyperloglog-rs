# Session handoff: sketch/MLE investigation + feature/terminology cleanup

Branch: `hll-union-merger`. **Nothing is committed.** All work below is uncommitted in the working tree (started from commit `e7ff1f8`). Two independent streams of work are interleaved here; consider committing them as separate logical chunks.

---

## TESTING STATUS (read this first)

Test profiles and what each verifies:

- **Full debug-assertion suite (most rigorous, ~32 min):** `cargo test -p hyperloglog-rs --features mle`. `[profile.test]` runs `opt-level=3` but with `debug-assertions=true` and `overflow-checks=true`, and the suite macro-expands across the full precision (P4..P18) x bits (B4..B6) grid with statistical sampling, hence the duration (lib 291 tests ~667s, integration `tests/test_hll.rs` 203 tests ~1255s, doctests 15 ~1s).
- **Fast suite:** `cargo test -p hyperloglog-rs --release --features mle` (drops debug-asserts/overflow-checks). Use for mechanical-change iteration.
- **Build matrix (all verified clean):** `cargo build --no-default-features` (no_std, no alloc), `--features alloc`, `--features mle`, `cargo build --workspace`.
- **Ignored diagnostics (NOT run in CI):** `diagnose_joint_vs_2set` and `experiment_optimizers` in `src/mle/tests.rs`. Run with e.g. `cargo test -p hyperloglog-rs --release --features mle --lib diagnose_joint_vs_2set -- --ignored --nocapture`.
- **Benchmark example (not a unit test, verified by running):** `cargo run --release --features mle --example joint_matrix_bench` (NOTE: the old `--features "std mle"` no longer works, `std` feature is gone; the example is a std binary so it uses `HashSet`/`Instant` fine). Produces `docs/sketch_benchmark.json` and the SVGs.

What level each piece is at:

| Work | Highest test level reached | Status |
|---|---|---|
| Sketch Phases 1-4 (normalize, error harness, error bars) | full debug-assertion suite green; example runs | DONE |
| MLE warm-start fix + default optimizer = Lbfgs (production) | full debug-assertion suite green | DONE |
| Cleanup A+B (drop std, drop mem_dbg, detach tooling) | full debug-assertion suite green (post-A+B) | DONE |
| Cleanup C (representation rename) + F (spelling) | full debug-assertion suite green (post-C): lib 291 / integ 203 / doc 15, 0 failures | DONE |
| Cleanup D+E (conversion redesign, estimation_regime) | new `test_estimation_regime_and_direct_to_hll` green across the full grid (`--release`); full debug-assertion suite RUNNING | DONE (pending final full-suite confirmation) |

D+E changes logic (not just names), so it needs the full debug-assertion run at the end, not just `--release`.

---

## Stream 1: sketch + MLE investigation (DONE, in working tree)

Production code changes:
- `src/sketches.rs`: `normalized_joint_sketch` moved from a separate trait into `HyperSpheresSketch` as a default method. Added `JointSketch::shell_maxima()` and `JointSketch::normalize()` (absolute decomposition -> `[0,1]` shell fractions). Tested by `normalize_tests` (2 tests).
- `src/mle/sketch.rs`: `joint_sketch_mle_core` now warm-starts from the **repeated 2-set MLE** decomposition (`IeMle<Mle<..>>`) instead of the HLL++ pairwise sketch (the old seed trapped the optimizer). Default optimizer in `joint_sketch_mle_from_registers` changed `Chain<Adam, Lbfgs>` -> `Lbfgs` (the Adam warmup only compensated for the bad seed).
- `src/mle/wrapper.rs`: added `pub(crate) struct IeMle<E>` (inclusion-exclusion over `Mle` views, i.e. repeated 2-set MLE; inherits the default `joint_sketch`, unlike the bare `Mle` view which overrides it).

Key findings (recorded in memory `hll-joint-mle-dominated-by-repeated-2set.md`):
- The joint MLE is **significantly more accurate** than the repeated 2-set MLE on the overlap grid (PAIRED test, t=2.9/2.3/4.1/5.5 at M=2..5, advantage grows with M), but ONLY from the good (2-set) seed. From the HLL++ seed it is worse (paired t ~ -1.3). The earlier "statistically indistinguishable" reading was wrong: it used marginal std bands, but the methods are paired on identical data, so the common variance must be differenced out.
- Absolute gain is small (~2-3 overlap-grid points) at ~40x the time cost.
- `raw HLL == HLL++` in this benchmark regime: cardinalities (0.5M-2.5M at P12) are above `correction_upper_bound`, so the bias correction is a no-op.
- L-BFGS does ~49 outer iters / ~63 objective evals at M=N=3.

Artifacts: `examples/joint_matrix_bench.rs` (random-sphere benchmark: two nested chains, 500k fresh uniform u32 per layer from `0..RANGE` with `RANGE=10M`, `SEEDS=16`); `docs/sketch_benchmark.svg` (3 panels: overlap error with +/-1 std bands, paired joint-advantage with +/-1 sem, log time); `docs/sketch_benchmark_normalized.svg`; generators `docs/make_sketch_svg.py`, `docs/make_sketch_normalized_svg.py`. **`docs/architecture.svg` must stay untouched.**

---

## Stream 2: feature/terminology cleanup

Plan file: `/home/luca/.claude/plans/plan-adequately-search-the-frolicking-hoare.md` (approved). Checkpoints A,B,C,F done; D,E pending.

### Decisions made during the Q&A (all settled unless noted):
- Final Cargo features: `default = []`, `alloc = []` (kept optional for the no_std-no-alloc embedded build), `mle = ["alloc", "dep:num-traits"]`. Dropped: `exact` (earlier), `std`, `mem_dbg`.
- Predicate name for the third representation: `is_hyperloglog` (not `is_hll`).
- Conversions: target-based `to_*` / `into_*`, NOT a source-to-target matrix.
- `estimation_regime()` enum: 4 variants distinguishing the two corrections. Leaning toward correction-mechanism names `Exact`, `HashListCollisionCorrected`, `HyperLogLogBiasCorrected`, `HyperLogLogRaw` (user wanted "specific and longer"; **final variant names not locked**).
- The crate has NO linear counting (verified: only `test_utils/src/set.rs` references `LinearCounting`, as a competitor baseline; the empirical `correction_coefficients.rs` tables subsume the small-range correction).

### A+B (DONE): drop `std` and `mem_dbg`
- `src/lib.rs`: `#![cfg_attr(not(test), no_std)]` (shipped crate is no_std; tests link std and keep the full prelude, no import churn). `std` feature removed from `Cargo.toml`.
- The five `src/mle/*.rs` `use num_traits::Float;` imports are now `#[allow(unused_imports)]` (load-bearing in no_std, shadowed by `f64` inherent methods when std is ambient in test builds). Core is transcendental-free (integer-bit `integer_exp2*` helpers), no core change needed.
- `mem_dbg` removed: feature + dep + all derive/cfg arms in core (`src/hyperloglog.rs`, `src/sketches.rs`, `src/registers/packed_array.rs` where the mem_dbg/non-mem_dbg associated-type pair collapsed to the single form).
- `test_utils` stripped of `mem_dbg`: `cardinality_samples` lost its `MemSize` bound and now passes `0` for the memory field (its only `mem_dbg` use; the generator path `uncorrected_cardinality_samples_by_model` never used it).
- Workspace `exclude`s the mem_dbg-built comparison cluster: `measure_variant`, `statistical_comparisons`, `statistical_comparisons/macro_test_utils`. **Their source is retained but NOT built** (user will rebuild memory tooling later, they dislike the `mem_dbg` crate). Kept members: `optimal-gap-codes`, `hash_list_correction`, `test_utils`, all build.
- `tests/test_hll.rs`: de-gated the leftover `#[cfg(feature = "exact")]` / `#[cfg(all(feature = "exact", feature = "mle"))]` blocks (exact mode is always-on now).

### C (DONE): representation rename + F spelling
Applied across `src/`, `tests/`, `examples/`, and the kept members:
- `is_exact -> is_sorted_value_list`, `is_hash_list -> is_sorted_hash_list`, `is_dense -> is_hyperloglog`, `is_exact_metadata -> is_sorted_value_list_metadata`, `set_exact_mode -> set_sorted_value_list_mode`, `EXACT_MODE_SENTINEL -> SORTED_VALUE_LIST_SENTINEL`, `decode_is_exact -> decode_is_sorted_value_list`.
- All 5 error strings + doc/code-comment prose standardized to "sorted value list" / "sorted hash list" / "HyperLogLog registers". Module + struct docs now state `HyperLogLog` is the hybrid counter that transitions across the three representations.
- Spelling: `simmetry->symmetry`, `comulative->cumulative`, `propertis->properties`, `repourpose->repurpose`.
- **Left intentionally:** `joint_sketch_exact_from_values`/`_from_hash_lists` (the "exact" means exact-vs-MLE computation), and the `HASHLIST_CORRECTION_*` / `HYPERLOGLOG_CORRECTION_*` table constants (internal, clearly named; renaming churns the giant `correction_coefficients.rs`). Rename these if full uniformity is wanted.

### D+E (DONE): conversion API + estimation_regime
- Replaced the conversion matrix `convert_exact_to_hash_list` / `convert_hash_list_to_hyperloglog` (in `src/hyperloglog.rs`) and `materialize_to_registers` (in `src/mle.rs`) with target-based methods that dispatch on the current representation:
  - in-place primitives: `to_sorted_hash_list(&mut self)`, `to_hll(&mut self)` (no-op if already at/past the target).
  - consuming wrappers: `into_sorted_hash_list(self) -> Self`, `into_hll(self) -> Self`.
  - `materialize_to_registers` in `src/mle.rs` is now `counter.clone().into_hll()`; all `.unwrap()`-laden convert sites removed.
- **`to_hll` from a sorted value list goes DIRECT to registers**: decode each stored value via `ValueIter` (`src/composite_hash/gaps/value_list.rs`), re-hash at full width via `index_and_register_and_hash`, `insert_register_value_and_index`. Strictly less lossy than routing through the truncated sorted hash list. Verified equal to the indirect route at low load (no rank truncation there); the divergence only appears at higher load where the hash list truncates ranks.
- **Latent buffer-size bug found and fixed:** both `to_hll` branches build the destination register buffer via the new private helper `full_size_cleared_registers()`, which clones, clears, and grows the (lazily allocated `Vec`-backed) buffer to the full register-array size before scattering registers. Previously `convert_hash_list_to_hyperloglog` cloned the current buffer directly; that was only safe because the hash-list->HLL transition fires at saturation (buffer already full). Now that `to_hll` is public and callable on a small/non-saturated counter (e.g. a tiny sorted value list, or a hand-built small hash list), a direct clone is too short and a high register index writes out of bounds (manifested as SIGSEGV / `free(): invalid next size` in `--release`, an `unsafe get_unchecked` debug panic in dev). The array-backed registers are always full size, so the growth loop is a no-op for them and never calls the `unimplemented!()` array `increase_capacity`.
- `insert_index_register_hash` auto-promotion arm now calls `self.to_hll()`.
- Added `pub enum EstimationRegime { Exact, HashListCollisionCorrected, HyperLogLogBiasCorrected, HyperLogLogRaw }` (re-exported via the prelude glob) + `pub fn estimation_regime(&self) -> EstimationRegime` on `HyperLogLog`. Registers branch computes `raw = ALPHA * m^2 / harmonic_sum` and returns `HyperLogLogRaw` at/above `correction_upper_bound::<P>()`, else `HyperLogLogBiasCorrected`. Variant names were locked as proposed (the user said "Proceed").
- New test `test_estimation_regime_and_direct_to_hll` (lib, `#[test_estimator]` over the full grid): Exact regime + exact count for a value list, `HashListCollisionCorrected` for a fresh counter, `into_hll` yields a register regime with a finite positive estimate, direct==indirect at low load, `into_hll` idempotent. It deliberately does NOT assert the register estimate is near the true count: HLL is inaccurate at low load (575 vs 16 at P10/B4), which is exactly why real counters stay in the hash list. That low-load register inaccuracy is expected, not a conversion bug.

### Deferred follow-ups (NOT in this plan):
- **Sorted-hash-list -> HLL transition accuracy measurement** (the birthday-correction weak spot, worst at high precision P16-18 per memory `hll-followups`). The conversion fires at `SaturationError::Saturation` (`src/hyperloglog.rs` ~line 278), i.e. the hash list is squeezed to the last byte before switching. Plan: sweep cardinality across the saturation->conversion boundary at several precisions, measure single-counter cardinality error vs exact `HashSet`, and decide whether to convert to HLL earlier (small memory cost for accuracy). If the hash list cannot be squeezed as hard as it is, transition earlier.
- README + featuring the sketch as the novelty (task existed, not done).
- Optionally rename the `HASHLIST_CORRECTION_*` consts and `joint_sketch_exact_from_*` for full uniformity.

---

## Gotchas
- `cargo test -p hyperloglog-rs` resolves the WHOLE workspace, so a member referencing a removed feature breaks even `-p` builds. All kept members were fixed; excluded members are in `[workspace] exclude`.
- Background `cargo test` piped through `tail -N` truncates the per-binary `test result` lines; grep for `"test result"` instead.
- The `proc-macro-error2 v2.0.1` future-incompat warning during workspace builds is a transitive dep, not ours.
