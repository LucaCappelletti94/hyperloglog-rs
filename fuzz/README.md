# How to fuzz HyperLogLog-rs
Fuzzing is an essential part of any library's testing suite. It helps to identify potential bugs and vulnerabilities that traditional testing may not catch. HyperLogLog-rs provides a simple and easy way to run fuzz tests using cargo-fuzz. 

To fuzz this library, simply install [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) as such:

```bash
cargo install cargo-fuzz
```

and then you can start one how the fuzzer harness as such:

```bash
cargo fuzz run registers_symmetry
```

Happy fuzzing!