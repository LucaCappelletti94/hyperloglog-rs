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

## Available fuzz targets
At this time, I have prepared and executed four different fuzzing targets. Of course, though to the inherent probabilistic nature of the HLL counters there is some additional difficulty in comparing it against the power bounds of the fuzzing harness. However, I have found that the fuzzing harnesses are able to find some interesting bugs in the library, and I have been able to fix them as a result. Here is the list of fuzzing targets that I have prepared:

### registers_symmetry
Here we test whether two registered manipulated side by side using what are in theory identical methods actually produce the same results and that all relative methods maintain coherence.

```bash
cargo fuzz run registers_symmetry
```

### cardinality
Here we test whether the cardinality estimation is correct for random inputs.

```bash
cargo fuzz run cardinality
```

### intersection_cardinality
Here we test whether the cardinality estimation of HLL intersections is correct for random inputs.

```bash
cargo fuzz run intersection_cardinality
```

### sketching
Here we test whether the sketching of HLL counters is correct for random inputs, and that the various methods available are coherent between one another.

```bash
cargo fuzz run sketching
```