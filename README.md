# HyperLogLog-rs
This is a Rust library that provides an implementation of the HyperLogLog (HLL) algorithm. HLL is a probabilistic algorithm that is used for estimating the cardinality of a set with very high accuracy while using a very small amount of memory. The algorithm was invented by Philippe Flajolet and Éric Fusy in 2007, and since then, it has been widely used in many fields, including database systems, search engines, and social networks.

The HLL algorithm is based on the observation that the number of unique elements in a set can be estimated by counting the number of leading zeros in the binary representation of the hash values of the elements in the set. This idea is simple but powerful, and it allows us to estimate the cardinality of a set with very high accuracy (within a few percentage points) using only a small amount of memory (a few kilobytes).

Our Rust implementation of the HLL algorithm is highly efficient, and it provides very accurate estimates of the cardinality of large sets. It is designed to be easy to use and integrate into your existing Rust projects. The crate provides a simple API that allows you to create HLL counters, add elements to them, and estimate their cardinality.

The focus of this particular implementation of the HyperLogLog algorithm is to be as memory efficient as possible. We achieve this by avoiding struct attributes that would store parameters such as the precision or the number of bits, as these would take up unnecessary memory space. Instead, we define these parameters as constants associated to the class, which allows for a more efficient and compact implementation.

However, this approach requires the use of several nightly features of the Rust language, including the const generics feature and the associated type constructors. By using these features, we are able to define the size of the internal data structures used by the algorithm at compile time, based on the precision of the HyperLogLog counter. This results in a more memory-efficient implementation that can be used in a wide range of applications.

We hope that this library will be useful for you in your projects, and we welcome your feedback and contributions. Please feel free to open an issue or submit a pull request if you have any questions or suggestions. Thank you for your interest in our HLL crate, and happy counting!


## Required features
The **generic_const_exprs** feature allows for using constant expressions as generic type parameters, which is necessary in this implementation to pass compile-time constant values as template arguments.

The **const_for** feature enables us to use for loops in const contexts, which is useful for generating arrays at compile time.

The **const_float_bits_conv** feature provides the ability to convert floating-point values to their corresponding bit patterns at compile time, which is useful in computing hash values.

The **const_trait_impl** feature enables implementing traits on const generics, which is essential to ensure that traits are implemented on types with known constant values.

The **const_mut_refs** feature allows taking references to mutable constants, which is useful in generating arrays and computing the hash values.

Finally, the **const_fn_floating_point_arithmetic** feature provides the ability to perform floating-point arithmetic in const functions, which is necessary for computing hash values using the f32::to_bits() method.

## Contributing to this project
Contributions from the community are highly appreciated and can help improve this project. If you have any suggestions, feature requests, or bugs to report, [please open an issue on GitHub](https://github.com/LucaCappelletti94/hyperloglog-rs/issues). Additionally, if you want to contribute to the project, [you can open a pull request with your proposed changes](https://github.com/LucaCappelletti94/hyperloglog-rs/pulls). Before making any substantial changes, please discuss them with the project maintainer in the issue tracker or on the [🍇GRAPE🍇 Discord server](https://discord.gg/Nda2cqYvTN).

If you appreciate this project and would like to support its development, you can star the repository on GitHub or [consider making a financial contribution](https://github.com/sponsors/LucaCappelletti94). The project maintainer has set up a GitHub Sponsors page where you can make a recurring financial contribution to support the project's development. Any financial contribution, no matter how small, is greatly appreciated and helps ensure the continued development and maintenance of this project.