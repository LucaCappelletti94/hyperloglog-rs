# HyperLogLog benchmarks
This directory contains the benchmarks for the HyperLogLog data structure, including several implementations and the
comparison with other set-like data structures such as HashSet. If you want to include some more benchmarks, please
feel free to open a pull request.

## How to run the benchmarks
To run the benchmarks, use the following cargo command. Do not exclude the `--release` flag, and the `RUST_FLAGS= -C target-cpu=native` flag is recommended to enable the SIMD instructions.

```bash
RUSTFLAGS="-C target-cpu=native" cargo run --release
```