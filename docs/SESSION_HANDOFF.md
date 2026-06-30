# Session Handoff — 2025-06-30

## Goal

Fix subagent spawning (all 3 agents failed with "Unable to connect") and complete the Lean proof formalization.

## Repositories

### Paper: ~/github/hyperloglog-rs
- Paper: `docs/hash_list_cardinality_paper.tex`
- Style guide: `docs/PAPER_STYLE.md`
- PDF compiles: `cd docs && rm -f hll_plot_data.csv && pdflatex -interaction=nonstopmode hash_list_cardinality_paper.tex`
- Last commit: `1322538` — "Fix saturating register probability from 2^{-(P+B)} to 2^{-(P+2^B-2)}"

### Proofs: ~/github/hyperloglog-proofs
- Lean 4 + mathlib4 `stable` branch
- Build: `cd ~/github/hyperloglog-proofs && lake build`
- Proof plan: `PROOF_PLAN.md`
- Last commit: `145b356` — "Add proof plan for Lean developer"
- 10 `sorry` remain across Basic.lean (4), Composite.lean (2), Occupancy.lean (3), plus wideCellGroups not yet scaffolded

## Subagent Problem

All 3 task agents (`BasicProver`, `OccupancyProver`, `CompositeProver`) failed identically:
```
Unable to connect. Is the computer able to access the url?
```

This happened when spawning via `task` with `agent: "task"`. The agents never started — no file changes, no build attempts. Possible causes:
- Network/connectivity issue in the harness
- `task` agent type unavailable or misconfigured
- Context too large when spawning (agents inherit full context)
- GPU resource contention (4 agents on one RTX 4090)

### Things to Try
1. Spawn a trivial agent first (e.g. `agent: "quick_task"` with a simple file read) to diagnose
2. Reduce context passed to agents (use `local://` URIs for large artifacts)
3. Try `agent: "oracle"` instead of `agent: "task"` for heavier reasoning
4. Check GPU availability with `nvidia-smi` before spawning
5. Try spawning one agent at a time instead of a batch
6. Check IRC connectivity: `irc op: list` to see if agents appear

## Paper Status

### Completed
- Fixed "HLL++ MLE estimator" to "register MLE" (abstract + conclusion)
- Removed redundant MARE restatement paragraph
- Fixed 5 clarity issues in HLL++ comparison section
- Fixed saturating register probability: `2^{-(P+B)}` to `2^{-(P+2^B-2)}` (theorem, equation, proof)
- Verified HLL++ paper: confirmed "sorted compressed" is correct, hash-size decay is NOT used by HLL++
- PDF: 11 pages, ~324KB, only pre-existing 3pt overfull

### Naming Conventions
- "value list" = exact sparse representation
- "hash list" = approximate sparse representation
- "dense representation" = registers
- "sparse representation" = umbrella term
- Hash list operates in LINEAR-COUNTING band of HLL++, NOT the table-requiring regime

### Prose Rules
- No semicolons, no em-dashes, no ASCII dashes as punctuation, no non-ASCII chars
- Abstract tone: contributive, not competitive
- No hard-wrapping prose at ~70 columns

## Lean Status

### What Works
- Project builds cleanly with `lake build`
- All definitions type-check
- All theorem statements are well-formed
- Probability model matches the corrected paper and the Rust code

### Mathlib API Gotchas (this version)
- `Finset.sum_singleton_right` does NOT exist — use `Finset.sum_union` + set equality
- `Finset.Icc_one_left` does NOT exist
- `Nat.pow_le_pow_of_le_right` does NOT exist in expected location
- `pow_le_one` does NOT exist
- Sigma notation `(Sigma x in s, f x)` does NOT parse — use `Finset.sum s (fun x => f x)`
- `List.mapMv` does NOT exist — use `(list.map f).sum`
- `#todo` is not valid syntax — use `sorry` or `#guard_msgs`

### Probability Model (Corrected)
```
rankProb B r:
  if r = 0: 0
  if r < R: (1/2)^r          where R = 2^B - 1
  if r = R: (1/2)^(R-1)      (geometric tail)
  else: 0

cellProb P B r = (1/2)^P * rankProb B r
```

Sum verification: `Sigma_{r=1}^{R-1} 2^(-r) + 2^(-(R-1)) = (1 - 2^(-(R-1))) + 2^(-(R-1)) = 1`

## Key Decisions
- Register values are `1..2^B-1`, with `2^B-1` saturating
- The paper's original `p_R = 2^{-(P+B)}` was wrong (probability sum > 1)
- The Rust code was always correct: uses `2^-(r_max-1)` where `r_max = 2^B-1`
- Ertl's MLE applies to standard HLL registers, not HLL++ (HLL++ uses empirical bias tables)

## Next Steps
1. Diagnose and fix subagent spawning
2. Spawn parallel provers for the 10 `sorry` items
3. Scaffold `wideCellGroups` (flag-bit partition) — the hardest remaining piece
4. Run `lake build` after each proof to verify

## Machine
- CPU: AMD Ryzen Threadripper PRO 5975WX 32-Cores (64 threads)
- RAM: 1 TB
- Disk: 3.7 TB NVMe
- GPU: NVIDIA GeForce RTX 4090 (24 GB VRAM) — shared resource, check with `nvidia-smi`
