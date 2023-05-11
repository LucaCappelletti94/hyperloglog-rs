# 🧮 HyperLogLog-rs
[![Build status](https://github.com/lucacappelletti94/hyperloglog-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/lucacappelletti94/hyperloglog-rs/actions)
[![Crates.io](https://img.shields.io/crates/v/hyperloglog-rs.svg)](https://crates.io/crates/hyperloglog-rs)
[![Documentation](https://docs.rs/hyperloglog-rs/badge.svg)](https://docs.rs/hyperloglog-rs)

This is a Rust library that provides an implementation of the HyperLogLog (HLL) algorithm, trying to be parsimonious with memory.

## What is HyperLogLog?
HLL is a probabilistic algorithm that is used for estimating the cardinality of a set with very high accuracy while using a very small amount of memory. The algorithm was invented by Philippe Flajolet and Éric Fusy in 2007, and since then, it has been widely used in many fields, including database systems, search engines, and social networks.

The HLL algorithm is based on the observation that the number of unique elements in a set can be estimated by counting the number of leading zeros in the binary representation of the hash values of the elements in the set. This idea is simple but powerful, and it allows us to estimate the cardinality of a set with very high accuracy (within a few percentage points) using only a small amount of memory (a few kilobytes).

Our Rust implementation of the HLL algorithm is highly efficient, and it provides very accurate estimates of the cardinality of large sets. It is designed to be easy to use and integrate into your existing Rust projects. The crate provides a simple API that allows you to create HLL counters, add elements to them, and estimate their cardinality.

The focus of this particular implementation of the HyperLogLog algorithm is to be as memory efficient as possible. We achieve this by avoiding struct attributes that would store parameters such as the precision or the number of bits, as these would take up unnecessary memory space. Instead, we define these parameters as constants associated to the class, which allows for a more efficient and compact implementation.

However, this approach requires the use of several nightly features of the Rust language, including the const generics feature and the associated type constructors. By using these features, we are able to define the size of the internal data structures used by the algorithm at compile time, based on the precision of the HyperLogLog counter. This results in a more memory-efficient implementation that can be used in a wide range of applications.

We hope that this library will be useful for you in your projects, and we welcome your feedback and contributions. Please feel free to open an issue or submit a pull request if you have any questions or suggestions. Thank you for your interest in our HLL crate, and happy counting!

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
hyperloglog = "0.1"
```

and this to your crate root:

```rust
use hyperloglog_rs::HyperLogLog;
```

## Examples

```rust
use hyperloglog_rs::HyperLogLog;

let mut hll = HyperLogLog::<14, 5>::new();
hll.insert(&1);
hll.insert(&2);

let mut hll2 = HyperLogLog::<14, 5>::new();
hll2.insert(&2);
hll2.insert(&3);

let union = hll | hll2;

let estimated_cardinality = union.estimate_cardinality();
assert!(estimated_cardinality >= 3.0_f32 * 0.9 &&
        estimated_cardinality <= 3.0_f32 * 1.1);
```

## No STD
This crate is designed to be as lightweight as possible and does not require any dependencies from the Rust standard library (std). As a result, it can be used in a bare metal or embedded context, where std may not be available.

All functionality of this crate can be used without std, and the prelude module provides easy access to all the relevant types and traits. If you encounter any issues using this crate in a no_std environment, please don't hesitate to open an issue or submit a pull request on GitHub.

## Required features
Note that some of the features used by this crate, such as #![feature(generic_const_exprs)], require a nightly Rust compiler. If you are using a stable or beta version of Rust, you may need to switch to nightly or disable certain features to use this crate.

The **generic_const_exprs** feature allows for using constant expressions as generic type parameters, which is necessary in this implementation to pass compile-time constant values as template arguments.

The **const_float_bits_conv** feature provides the ability to convert floating-point values to their corresponding bit patterns at compile time, which is useful in computing hash values.

The **const_trait_impl** feature enables implementing traits on const generics, which is essential to ensure that traits are implemented on types with known constant values.

The **const_mut_refs** feature allows taking references to mutable constants, which is useful in generating arrays and computing the hash values.

Finally, the **const_fn_floating_point_arithmetic** feature provides the ability to perform floating-point arithmetic in const functions, which is necessary for computing hash values using the f32::to_bits() method.

## Fuzzing
Fuzzing is a technique for finding security vulnerabilities and bugs in software by providing random input to the code. It can be an effective way of uncovering issues that might not be discovered through other testing methods. In our library, we take fuzzing seriously, and we use the [cargo fuzz](https://github.com/rust-fuzz/cargo-fuzz) tool to ensure our code is robust and secure. cargo fuzz automates the process of generating and running randomized test inputs, and it can help identify obscure bugs that would be difficult to detect through traditional testing methods. We make sure that our fuzz targets are continuously updated and run against the latest versions of the library to ensure that any vulnerabilities or bugs are quickly identified and addressed.

[Learn more about how we fuzz here](https://github.com/LucaCappelletti94/hyperloglog-rs/tree/main/fuzz)

## Contributing to this project
Contributions from the community are highly appreciated and can help improve this project. If you have any suggestions, feature requests, or bugs to report, [please open an issue on GitHub](https://github.com/LucaCappelletti94/hyperloglog-rs/issues). Additionally, if you want to contribute to the project, [you can open a pull request with your proposed changes](https://github.com/LucaCappelletti94/hyperloglog-rs/pulls). Before making any substantial changes, please discuss them with the project maintainer in the issue tracker or on the [🍇GRAPE🍇 Discord server](https://discord.gg/Nda2cqYvTN).

If you appreciate this project and would like to support its development, you can star the repository on GitHub or [consider making a financial contribution](https://github.com/sponsors/LucaCappelletti94). The project maintainer has set up a GitHub Sponsors page where you can make a recurring financial contribution to support the project's development. Any financial contribution, no matter how small, is greatly appreciated and helps ensure the continued development and maintenance of this project.

## Thanks
We would like to thank the GitHub user [Tabac](https://github.com/tabac) for their implementation of HyperLogLog, which was very useful for learning and benchmarking this implementation. The goals of the two implementations are different, but Tabac's implementation already supports the HLL++ algorithm, go check it out if you need it. Their implementation can be found at [here](https://github.com/tabac/hyperloglog.rs).

## Citations
Some relevant citations to learn more:

* [Flajolet, Philippe](https://en.wikipedia.org/wiki/Philippe_Flajolet), Éric Fusy, Olivier Gandouet, and Frédéric Meunier. "[Hyperloglog: the analysis of a near-optimal cardinality estimation algorithm.](https://hal.science/file/index/docid/406166/filename/FlFuGaMe07.pdf)" In Proceedings of the 2007 conference on analysis of algorithms, pp. 127-146. 2007.

* Heule, Stefan, Marc Nunkesser, and Alexander Hall. "[HyperLogLog in practice: algorithmic engineering of a state of the art cardinality estimation algorithm.](https://static.googleusercontent.com/media/research.google.com/it//pubs/archive/40671.pdf)" In Proceedings of the 16th International Conference on Extending Database Technology, pp. 683-692. 2013.

* Heule, Stefan, and Marc Nunkesser. "[HyperLogLog++: Google's take on engineering HLL.](https://agkn.wordpress.com/2013/01/24/hyperloglog-googles-take-on-engineering-hll/)" In Proceedings of the 2013 ACM SIGMOD international conference on Management of data, pp. 1247-1250. 2013.

* Durand, Mathias, and [Philippe Flajolet](https://en.wikipedia.org/wiki/Philippe_Flajolet). "[Loglog counting of large cardinalities.](https://link.springer.com/chapter/10.1007/978-3-540-39658-1_55)" In Algorithms-ESA'03, pp. 605-617. Springer Berlin Heidelberg, 2003.