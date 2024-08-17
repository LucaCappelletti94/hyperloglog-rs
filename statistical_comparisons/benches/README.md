# Benchmarks

## Insert

```bash
RUSTFLAGS='-C target-cpu=native' cargo bench --bench insert --features all_precisions > benches/insert.log
```

## Binary Search

```bash
RUSTFLAGS='-C target-cpu=native' cargo bench --bench binary_search 
```

## Cardinality

```bash
RUSTFLAGS='-C target-cpu=native' cargo bench --bench cardinality --features all_precisions,integer_plusplus,precomputed_beta > benches/cardinality.log
```

## Union

```bash
RUSTFLAGS='-C target-cpu=native' cargo bench --bench union --features all_precisions,integer_plusplus,precomputed_beta > benches/union.log
```

## Hash functions
One of the cardinal parts of any HyperLogLog implementation is the hash function. This benchmark compares the speed of different hash functions, including WyHash, XxHash, and the default Rust hasher. Earlier, it included also a custom implementation of SipHash, but it was removed due to the lack of entropy - potentially a bug in the implementation.

You can run the benchmark with the following command:

```bash
RUSTFLAGS='-C target-cpu=native' cargo bench --bench hash_functions
```

```bash
xx_hasher               time:   [7.9508 ms 7.9619 ms 7.9756 ms]
                        change: [-0.0498% +0.0989% +0.2755%] (p = 0.27 > 0.05)
                        No change in performance detected.
Found 17 outliers among 100 measurements (17.00%)
  2 (2.00%) high mild
  15 (15.00%) high severe

xx3_hasher              time:   [32.412 ms 32.444 ms 32.484 ms]
                        change: [+0.7360% +0.8759% +1.0216%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

wyhash                  time:   [810.74 µs 811.29 µs 811.98 µs]
                        change: [+0.0325% +0.1810% +0.3246%] (p = 0.01 < 0.05)
                        Change within noise threshold.
Found 16 outliers among 100 measurements (16.00%)
  5 (5.00%) high mild
  11 (11.00%) high severe

default_hasher          time:   [4.1909 ms 4.1924 ms 4.1942 ms]
Found 19 outliers among 100 measurements (19.00%)
  7 (7.00%) high mild
  12 (12.00%) high 
```