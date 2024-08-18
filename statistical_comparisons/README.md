# Statistical comparisons
Statistical tests across different implementations of algorithms with the goal of estimating the cardinality of large sets.

These tests include both the estimation of cardinality and of union of sets.

## How to run
To run the statistical comparisons for quality of the estimation, use the following:

```bash
RUSTFLAGS='-C target-cpu=native' cargo run --release
RUSTFLAGS='-C target-cpu=native' cargo run --release --features precomputed_beta
RUSTFLAGS='-C target-cpu=native' cargo run --release --features integer_plusplus
RUSTFLAGS='-C target-cpu=native' cargo run --release --features plusplus_kmeans
RUSTFLAGS='-C target-cpu=native' cargo run --release --features plusplus_kmeans,integer_plusplus
```