# 🧮 HyperLogLog-rs
[![downloads](https://img.shields.io/crates/d/hyperloglog-rs)](https://crates.io/crates/hyperloglog-rs)
[![dependents](https://img.shields.io/librariesio/dependents/cargo/hyperloglog-rs)](https://crates.io/crates/hyperloglog-rs/reverse_dependencies)
[![CI](https://github.com/LucaCappelletti94/hyperloglog-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/LucaCappelletti94/hyperloglog-rs/actions)
![license](https://img.shields.io/crates/l/hyperloglog-rs)
[![Latest version](https://img.shields.io/crates/v/hyperloglog-rs.svg)](https://crates.io/crates/hyperloglog-rs)
[![Documentation](https://docs.rs/hyperloglog-rs/badge.svg)](https://docs.rs/hyperloglog-rs)

This is a Rust library that provides an implementation of the HyperLogLog (HLL) algorithm, trying to be parsimonious with memory.
It also provides an implementation based on the MLE algorithm, which is a more accurate version of the HyperLogLog algorithm but is slower.

You can use it to estimate the cardinality of large sets, and determine also the union and intersection of two sets.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
hyperloglog = "0.1"
```

## Examples

```rust
use hyperloglog_rs::prelude::*;

let mut hll = HyperLogLog::<Precision14, 5>::default();
hll.insert(&1);
hll.insert(&2);

let mut hll2 = HyperLogLog::<Precision14, 5>::default();
hll2.insert(&2);
hll2.insert(&3);

let union = &hll | &hll2;

let estimated_cardinality = union.estimate_cardinality();
assert!(estimated_cardinality >= 3.0_f32 * 0.9 &&
        estimated_cardinality <= 3.0_f32 * 1.1);

let intersection_cardinality: f32 = hll.estimate_intersection_cardinality(&hll2);

assert!(intersection_cardinality >= 1.0_f32 * 0.9 &&
        intersection_cardinality <= 1.0_f32 * 1.1);
```

### Hyperloglog with Molteplicities
Within an HyperLogLog counter, it is possible to precompute the number of molteplicities of the values of the registers. This can lead to a significant performance boost for estimating the cardinality (no improvements for unions or intersections estimations), but increases the memory usage of the counter by the number of possible values that may be stored in a register, as deteremined by the provided bit-size. For instance, register of 4 bits can store 16 different values, one of 5 bits can store 32 different values, and so on.

```rust
use hyperloglog_rs::prelude::*;

let mut hll = HyperLogLogWithMultiplicities::<Precision14, 5>::default();

hll.insert(&1);
hll.insert(&1);
hll.insert(&2);

let mut hll2 = HyperLogLogWithMultiplicities::<Precision14, 5>::default();

hll2.insert(&2);
hll2.insert(&3);

let union = &hll | &hll2;

let estimated_cardinality = union.estimate_cardinality();

assert!(estimated_cardinality >= 3.0_f32 * 0.9 &&
        estimated_cardinality <= 3.0_f32 * 1.1);
```

### Using MLE estimation
The [MLE estimation for HyperLogLog counters by Otmar Ertl](https://oertl.github.io/hyperloglog-sketch-estimation-paper/paper/paper.pdf) provides a more accurate estimation of the cardinality of a set, but it is slower than the standard HyperLogLog algorithm. Here is an example of how to use it:

```rust
#[cfg(feature = "std")]
{
        use hyperloglog_rs::prelude::*;

        let mut hll1: HyperLogLog<Precision10, 6> = HyperLogLog::default();
        
        hll1.insert(&1);
        hll1.insert(&2);
        hll1.insert(&3);

        let mle1: MLE<2, HyperLogLog<Precision10, 6>> = hll1.into();

        let mut hll2: HyperLogLog<Precision10, 6> = HyperLogLog::default();

        hll2.insert(&2);
        hll2.insert(&3);
        hll2.insert(&4);

        let mle2: MLE<2, HyperLogLog<Precision10, 6>> = hll2.into();
        
        let estimated_cardinality: f32 = mle1.estimate_cardinality();
        assert!(estimated_cardinality >= 3.0_f32 * 0.9 &&
                estimated_cardinality <= 3.0_f32 * 1.1);

        let estimate_intersection_cardinality: f32 = mle1.estimate_intersection_cardinality(&mle2);

        assert!(estimate_intersection_cardinality >= 2.0_f32 * 0.9 &&
                estimate_intersection_cardinality <= 2.0_f32 * 1.1);
}
```

## No STD
This crate is designed to be as lightweight as possible and does not require any dependencies from the Rust standard library (std). As a result, it can be used in a bare metal or embedded context, where std may not be available. The only feature that requires std is the MLE estimation, which is optional.

## Fuzzing
Fuzzing is a technique for finding security vulnerabilities and bugs in software by providing random input to the code. We make sure that our fuzz targets are continuously updated and run against the latest versions of the library to ensure that any vulnerabilities or bugs are quickly identified and addressed.

[Learn more about how we fuzz here](https://github.com/LucaCappelletti94/hyperloglog-rs/tree/main/fuzz)

## Citations
Some relevant citations to learn more:

* Philippe Flajolet, Eric Fusy, Olivier Gandouet, Frédéric Meunier. "[HyperLogLog: the analysis of a near-optimal cardinality estimation algorithm.](https://hal.science/file/index/docid/406166/filename/FlFuGaMe07.pdf)" In Proceedings of the 2007 conference on analysis of algorithms, pp. 127-146. 2007.
