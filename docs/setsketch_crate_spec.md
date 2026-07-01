# SetSketch Rust Crate

## Motivation

The `hyperloglog-rs` project implements a hash-list cardinality estimator with
occupancy-inverse bias correction. To evaluate its accuracy in the low-cardinality
regime, we need a SetSketch implementation to measure SetSketch's error at memory
parity. Without this comparison, the paper's claims about relative accuracy lack
empirical grounding.

SetSketch (Ertl, VLDB 2021) generalizes both HyperLogLog and MinHash via a
continuous base parameter `b`. It provides analytically derived cardinality
corrections for small and large cardinalities (no empirical calibration).
There is no Rust implementation of SetSketch: crates.io returns zero results,
and GitHub has no Rust repositories or code matching "SetSketch".

The only available implementation is a C++ reference in the paper's artifact
repository, which requires Boost, Gradle, and is not designed as a library.

## References

- Paper (local): `docs/wang2021setsketch.pdf`
- Paper (VLDB): https://vldb.org/pvldb/vol14/p2244-ertl.pdf
- Reference implementation: https://github.com/dynatrace-research/set-sketch-paper
- arXiv extended version: https://arxiv.org/abs/2101.00314

## Algorithm Specification

SetSketch maintains `m` integer-valued registers `K[0..m-1]`, each storing a
value from `{0, 1, ..., q+1}`. The register state for a set `S` is:

```
K[i] = max_{d in S} floor(1 - log_b(h_i(d)))
```

where `h_i(d)` are hash functions with exponentially distributed output,
and `b > 1` is the base parameter.

### Parameters

- `m` -- number of registers. Determines estimation accuracy (RSD in range
  `[1/sqrt(m), 1.04/sqrt(m)]` for `b <= 2`).
- `b` -- base parameter. `b = 2` is equivalent to HLL. `b -> 1` approaches
  MinHash. Smaller `b` improves joint estimation and locality sensitivity
  but requires larger `q` (more bits per register).
- `a` -- rate parameter. Controls the lower bound of the cardinality range.
  `a = 1/m` makes SetSketch equivalent to GHLL with stochastic averaging.
- `q` -- maximum register value before saturation. Controls the upper bound
  of the cardinality range.

### Register Update (SetSketch1 -- Independent Hash Values)

For each element `d`:

1. Seed a PRNG with `hash(d)`.
2. Generate `m` exponentially distributed values using exponential spacings:
   ```
   x_j ~ x_{j-1} + Exp(a) / (m + 1 - j)  for j = 1..m
   ```
   where `x_0 = 0`.
3. Shuffle the values and assign to `h_1(d), ..., h_m(d)` (sampling without
   replacement, Fisher-Yates style).
4. For each `j = 1..m`:
   - If `x_j > b^{-K_low}`, break (optimization: skip values too large to
     update any register).
   - `k = clamp(floor(1 - log_b(x_j)), 0, q+1)`
   - Sample register index `i` from `{1..m}` without replacement.
   - If `k <= K_low`, break.
   - `K[i] = max(K[i], k)`
   - Track update count `w`; when `w >= m`, recompute `K_low = min(K)`.

The `K_low` tracking allows asymptotic O(1) insert time for large sets.

### Register Update (SetSketch2 -- Correlated Hash Values)

Alternative update method using truncated exponential distributions over
disjoint intervals. The intervals `[gamma_{j-1}, gamma_j)` are defined by:

```
gamma_j = (1/a) * log(1 + j / (m - j))
```

Values `x_j` are sampled from `Exp(a; gamma_{j-1}, gamma_j)` in ascending
order, then shuffled. This introduces correlation between hash values but
reduces estimation error for small sets.

### Cardinality Estimation

The simple (closed-form) estimator:

```
n_hat = m * (1 - 1/b) / (a * log(b) * sum_{i=1}^{m} b^{-K[i]})
```

with corrections for registers at extreme values:

- **Small cardinality correction** (registers equal to 0):
  Replace `b^{-0}` contributions with `sigma_b(C_0 / m)`, where:
  ```
  sigma_b(x) = x + (b-1) * sum_{k=1}^{inf} x^{b^k} * b^{k-1}
  ```
  The sum converges quickly and can be truncated when terms become negligible.

- **Large cardinality correction** (registers equal to `q+1`):
  Replace `b^{-(q+1)}` contributions with `tau_b(C_{q+1} / m)`, where:
  ```
  tau_b(x) = (1-x) + (b-1) * sum_{k=1}^{inf} (x^{b^{-k}} - 1) * b^{-k}
  ```

The corrected estimator is:

```
sum = sum_{K[i] not extreme} b^{-K[i]}
      + sigma_b(C_0 / m)   [if C_0 > 0 and corrections enabled]
      + tau_b(C_{q+1} / m) [if C_{q+1} > 0 and corrections enabled]
n_hat = factor / sum
```

where `factor = m * (1 - 1/b) / (a * log(b))`.

### Joint Estimation

Given two SetSketches for sets `U` and `V`, count registers where:
- `D_0` = number of registers where `K_U[i] == K_V[i]`
- `D_+` = number of registers where `K_U[i] > K_V[i]`
- `D_-` = number of registers where `K_U[i] < K_V[i]`

The Jaccard similarity is estimated by maximizing the log-likelihood:

```
log L(J) = D_+ * log(p_b(u - v*J)) + D_- * log(p_b(v - u*J))
           + D_0 * log(1 - p_b(u - v*J) - p_b(v - u*J))
```

where `u = n_U / (n_U + n_V)`, `v = n_V / (n_U + n_V)`, and:

```
p_b(x) = -log_b(1 - x * (b-1) / b)
```

Solved via Brent's method on `J in [0, min(n_U/n_V, n_V/n_U)]`.

Fallback to inclusion-exclusion principle if registers have extreme values
(0 or `q+1`) and range correction is not enabled.

## Crate Requirements

### Non-negotiable

- **`no_std`**: The crate must compile without `std` or `alloc`. All buffers
  are fixed-size stack arrays. This matches the `hyperloglog-rs` design
  philosophy and enables use in embedded and kernel contexts.
- **Extensive proptests**: Every estimator, update path, and edge case must
  be covered by property-based tests. Properties include:
  - Idempotency: inserting the same element twice does not change state.
  - Commutativity: insert order does not affect final state.
  - Mergeability: merging two sketches produces the same state as inserting
    all elements into a fresh sketch.
  - Monotonicity: adding elements never decreases the cardinality estimate.
  - Estimator bounds: the estimate is never negative and respects `q` limits.
  - Parity with reference: estimates match the C++ reference implementation
    within floating-point tolerance (once the reference is available for
    comparison).
- **Criterion benchmarks**: Benchmarks for insert, estimate, and merge
  operations across multiple `(m, b, q)` configurations. Include both
  SetSketch1 and SetSketch2 variants.
- **High test coverage**: Aim for 100% branch coverage on all public APIs
  and internal estimation logic.

### API Design

All parameters are compile-time constants encoded in the type system,
matching the `hyperloglog-rs` approach. No dynamic struct attributes,
no heap-allocated configuration, no runtime parameter passing.

**Register count (`m`)**: encoded via a generic trait, mirroring
`hyperloglog-rs` precision types. Concrete types like `SetSketch16`,
`SetSketch32`, `SetSketch64` set `m = 2^P` at compile time. The
trait exposes `M` as a `const usize` and determines the packed array
word count.

**Base (`b`)**: encoded via a const generic or a trait with an
associated `const B: f64`. The value is known at monomorphization
time and drives precomputed lookup tables for `b^{-k}`, `sigma`, and
`tau`. No runtime `f64` field in the struct.

**Maximum register value (`q`)**: encoded as a `const u32` associated
constant on the configuration trait. Determines bits per register
(`ceil(log2(q+2))`) at compile time, which in turn determines the
packed array layout.

**Rate parameter (`a`)**: encoded as a `const f64` associated constant.
Typically `a = 1.0 / M` for HLL-equivalent behavior, but configurable.

The struct itself contains only the packed register array and the
`K_low` / update-counter state. Example sketch:

```rust
pub struct SetSketch<P, B, Q, A>
where
    P: Registers,      // M: usize (register count)
    B: Base,            // B: f64 (base parameter)
    Q: MaxValue,        // Q: u32 (max register value)
    A: Rate,            // A: f64 (rate parameter)
{
    registers: PackedArray<{ P::M }, { bits_per_register::<Q>() }>,
    k_low: u16,
    update_count: usize,
}
```

`no_std` compatible: use `core` only, fixed-size arrays, no heap
allocation. Public API: `new()`, `insert()`, `estimate()`, `merge()`,
`clear()`. Feature-gated joint estimation.

### Hash Function

Accept an external hash function (trait-based, like `hyperloglog-rs`),
defaulting to a fast 64-bit hash. The internal PRNG for exponential spacings
is separate from the element hash function.

### Floating-Point Considerations

The `sigma` and `tau` functions involve infinite series. Implement with:
- Precomputed lookup tables for common `(m, b)` configurations (compile-time
  const arrays).
- Runtime convergence check: truncate when `|term| < 1e-15 * running_sum`.
- All intermediate computations in `f64`.

### PRNG for Exponential Spacings

The SetSketch1 update requires generating exponentially distributed spacings.
Implement a lightweight PRNG (e.g., xoshiro128** or WyRand) seeded from the
element hash value. The PRNG must be deterministic and reproducible.

## Benchmark Plan

Compare SetSketch error against the hash-list estimator at memory parity:

1. For each hash-list configuration `(P, B)`, measure total memory usage
   (`6 * 2^P` bits for the register array).
2. Find the SetSketch `(m, b, q)` configuration with equivalent memory:
   `m * ceil(log2(q+2))` bits = `6 * 2^P` bits.
3. Run both estimators over the same cardinality range and seeds.
4. Report MARE for both, normalized by memory used.

This enables the paper to make a concrete claim about whether the hash-list
occupancy correction outperforms SetSketch's analytical corrections in the
low-cardinality regime.

## Implementation Notes from C++ Reference

The C++ reference implementation (`sketch.hpp`, 2019 lines) provides:

- `SetSketchEstimator` class with `estimateCardinalitySimple()` and
  `estimateCardinalityML()` methods.
- `sigma()` and `tau()` functions with convergence loops.
- `Mapping` class for register value computation.
- `Registers` and `RegistersWithLowerBound` state containers.
- `SetSketch1` and `SetSketch2` sketch implementations.
- Joint estimation via `estimateJointNew()`, `estimateJointInclExcl()`, etc.
- Uses Boost.Math for the MLE solver (`toms748_solve`) and bisect.

The Rust implementation should replicate the simple estimator and update
logic exactly, then add joint estimation as a feature-gated extension.
The MLE estimator (requiring numerical root finding) is lower priority
since the simple estimator is what the paper uses in practice.

## Fixture Generation from C++ Reference

The Rust implementation must produce bit-identical register states and
estimates matching the C++ reference. To guarantee this, generate golden
fixtures from the C++ code and consume them as integration tests.

### Fixture Generation Process

1. Clone the C++ reference: `git clone --recursive https://github.com/dynatrace-research/set-sketch-paper.git`
2. Build the C++ tests with Boost and Gradle.
3. Run the cardinality test (`cardinality_test.cpp`) with a fixed set of
   configurations and seeds to produce fixture data.
4. For each fixture, record:
   - Configuration: `(m, b, a, q)`, sketch variant (SetSketch1 or SetSketch2)
   - Input: array of 64-bit hash seeds (the elements inserted)
   - Expected register state: `K[0..m-1]` after all inserts
   - Expected cardinality estimate: output of `estimateCardinalitySimple()`
   - Expected joint estimates (if two sketches): `D_0`, `D_+`, `D_-`, and
     the Jaccard similarity from `estimateJointNew()`

### Fixture Format

Store fixtures as binary files in `fixtures/` with a simple header:

```
[4 bytes: magic "SSKT"]
[1 byte: variant (1=SetSketch1, 2=SetSketch2)]
[4 bytes: m (register count)]
[8 bytes: b as f64]
[8 bytes: a as f64]
[4 bytes: q as u32]
[4 bytes: num_elements as u32]
[num_elements * 8 bytes: element seeds as u64]
[m * 2 bytes: expected register state as u16]
[8 bytes: expected cardinality estimate as f64]
```

Generate at least 50 fixtures covering:

- Small sets (n < m): verify sigma correction and K_low tracking.
- Medium sets (n ~ m): verify standard estimator behavior.
- Large sets (n >> m): verify tau correction and saturation handling.
- Multiple `b` values: `b = 2` (HLL-equivalent), `b = 1.2`, `b = 1.001`.
- Both SetSketch1 and SetSketch2 variants.
- Edge cases: empty sketch, single element, all registers saturated.

### Fixture Tests

Each fixture is consumed by an integration test that:

1. Constructs a SetSketch with the recorded configuration.
2. Inserts all elements from the fixture.
3. Asserts the register state matches byte-for-byte.
4. Asserts the cardinality estimate matches within `1e-12` relative error.
5. For joint fixtures, asserts `D_0`, `D_+`, `D_-` counts and Jaccard
   estimate match.

These tests run on every CI build and serve as the primary regression
guard against divergence from the reference implementation.

## Timing Comparison with C++ Reference

The Rust implementation must be at least as fast as the C++ reference.
Measure and compare wall-clock timing for insert, estimate, and merge
operations.

### Timing Methodology

1. Build the C++ reference in release mode with `-O3 -march=native`.
2. Build the Rust crate in release mode with `opt-level = 3`.
3. For each configuration `(m, b, q)` and cardinality `n`:
   - Time inserting `n` elements (wall clock, multiple trials, report median).
   - Time calling `estimate()` once.
   - Time merging two sketches of size `n`.
4. Report Rust time / C++ time ratio. Target: ratio <= 1.0 (Rust is faster
   or equal).

### Timing Benchmarks in Criterion

Add a `benches/timing.rs` that mirrors the C++ `performance_test.cpp`
structure:

- Benchmark insert throughput (elements per second) for increasing `n`.
- Benchmark estimate latency (nanoseconds per call).
- Benchmark merge latency (nanoseconds per merge).
- Compare SetSketch1 vs SetSketch2 update paths.

Store C++ timing results as baseline artifacts in `benches/cpp_baseline/`
and assert that Rust timings do not regress beyond 10% of the C++ baseline.
If Rust is slower, investigate and optimize before merging.

## Research Question: Hash-List Approach Applied to SetSketch

While implementing SetSketch, investigate whether the hash-list approach
(sorted distinct composites with Rice-coded gaps and occupancy correction)
could replace SetSketch's dense register array in the low-cardinality regime.

### Key Differences from HLL Composites

The hash list in `hyperloglog-rs` stores composites that concatenate bucket
index, register rank, and residual hash bits. The register rank follows a
geometric distribution (probability `2^{-r}` for rank `r`), which is what
makes the occupancy model tractable.

SetSketch registers follow an exponential distribution: `P(K[i] <= k) =
exp(-n*a*b^{-k})`. The register values are `floor(1 - log_b(X))` where
`X ~ Exp(a)`. This is a different distribution from the geometric ranks
in HLL, so the existing occupancy model does not apply directly.

### Investigation Points

1. **Composite definition**: What would a "SetSketch composite" encode?
   In HLL, the composite is `(bucket_index, rank, residual_bits)`. For
   SetSketch, the equivalent would be `(register_index, register_value)`,
   but register values can range from 0 to `q+1` (up to 65534 for
   `b = 1.001`), making the composite space much larger than HLL's.

2. **Gap distribution**: Rice coding is optimal for geometrically
   distributed gaps. SetSketch composites would have a different gap
   distribution due to the exponential register values. Would Rice
   coding still be efficient, or would a Golomb or custom code be needed?

3. **Occupancy model**: The birthday-paradox occupancy function would
   need rederivation for SetSketch's non-uniform composite probabilities.
   The cell groups would follow from the exponential distribution rather
   than the geometric register distribution.

4. **Practical benefit**: Even if theoretically possible, would a sparse
   SetSketch representation save enough space to justify the complexity?
   SetSketch already uses fewer bits per entry than MinHash (2 bytes vs
   4 bytes for `b = 1.001`). The hash-list compression ratio would need
   to be significant to be worthwhile.

Document findings in a short note within the crate (e.g., `docs/hashlist_setsketch.md`)
even if the conclusion is that it is not practical.

## Packed Array Representation

The SetSketch register array should be stored as a packed bit array,
matching the `hyperloglog-rs` approach, rather than an array of native
integer types. This eliminates padding waste and ensures the memory
footprint is exactly `m * ceil(log2(q+2))` bits.

### Register Width Calculation

Given `q` (maximum register value before saturation), the number of bits
per register is `ceil(log2(q+2))` (values range from 0 to `q+1`, which
is `q+2` distinct values).

Examples:

- `b = 2`, `q = 63`: 6 bits per register (same as HLL with `B = 6`).
- `b = 1.2`, `q = 254`: 8 bits per register.
- `b = 1.001`, `q = 65534`: 16 bits per register.

### Implementation

Use the same `PackedArray` approach as `hyperloglog-rs`:

- Store registers in a fixed-size array of `u64` words.
- Provide `get(index)` and `set(index, value)` methods that extract and
  insert bits at the correct offsets.
- The total word count is `ceil(m * bits_per_register / 64)`.
- All operations are compile-time sized: `m` and `bits_per_register`
  are known from the generic type parameters.

This ensures the Rust implementation uses exactly the same memory as the
theoretical minimum, and enables direct comparison with the C++ reference
(which stores registers as `uint16_t` for `b = 1.001`, matching the 16-bit
packed representation).
