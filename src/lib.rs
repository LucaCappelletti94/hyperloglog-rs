//! # HyperLogLog
//!
//! This crate provides an implementation of the HyperLogLog algorithm, which is a probabilistic algorithm used to estimate
//! the number of distinct elements in a set. The algorithm uses a fixed amount of memory and is able to estimate the number
//! of distinct elements with a small relative error.
//!
//! The `HyperLogLog` struct provided by this crate is parametrized by two constants: `PRECISION` and `BITS`. `PRECISION`
//! determines the number of bits used to index a register, and `BITS` determines the number of bits used to represent
//! the hashed value of an element. The optimal values of these constants depend on the expected number of distinct elements
//! and the available memory.
//!
//! This implementation already provides almost all the benefits available from [HyperLogLog++](https://static.googleusercontent.com/media/research.google.com/it//pubs/archive/40671.pdf).
//! We **do not** intend to integrate the sparse registers feature, as the use cases for this library focus of cases
//! where registers need to be a relatively small number and a dense set. Except for that, all
//! other observations provided in the HLL++ paper are already implemented.
//!
//! ## No STD
//! This crate is designed to be as lightweight as possible and does not require any dependencies from the Rust standard library (std). As a result, it can be used in a bare metal or embedded context, where std may not be available.
//!
//! All functionality of this crate can be used without std, and the prelude module provides easy access to all the relevant types and traits. If you encounter any issues using this crate in a no_std environment, please don't hesitate to open an issue or submit a pull request on GitHub.
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! hyperloglog = "0.1"
//! ```
//!
//! and this to your crate root:
//!
//! ```rust
//! use hyperloglog_rs::prelude::*;
//! ```
//!
//! ## Examples
//!
//! ```rust
//! use hyperloglog_rs::prelude::*;
//!
//! let mut hll = HyperLogLog::<Precision14, 5>::new();
//! hll.insert(&1);
//! hll.insert(&2);
//!
//! let mut hll2 = HyperLogLog::<Precision14, 5>::new();
//! hll2.insert(&2);
//! hll2.insert(&3);
//!
//! let union = hll | hll2;
//!
//! let estimated_cardinality = union.estimate_cardinality();
//! assert!(estimated_cardinality >= 3.0_f32 * 0.9 &&
//!         estimated_cardinality <= 3.0_f32 * 1.1);
//! ```
//!
//! ## Fuzzing
//! Fuzzing is a technique for finding security vulnerabilities and bugs in software by
//! providing random input to the code. It can be an effective way of uncovering issues
//! that might not be discovered through other testing methods. In our library,
//! we take fuzzing seriously, and we use the [cargo fuzz](https://github.com/rust-fuzz/cargo-fuzz)
//! tool to ensure our code is robust and secure. cargo fuzz automates the process of generating
//! and running randomized test inputs, and it can help identify obscure bugs that would be
//! difficult to detect through traditional testing methods. We make sure that our fuzz targets
//! are continuously updated and run against the latest versions of the library to ensure that
//! any vulnerabilities or bugs are quickly identified and addressed.
//!
//! [Learn more about how we fuzz here](https://github.com/LucaCappelletti94/hyperloglog-rs/tree/main/fuzz)
//!
//! ## References
//!
//! * [Flajolet, Philippe](https://en.wikipedia.org/wiki/Philippe_Flajolet), Éric Fusy, Olivier Gandouet, and Frédéric Meunier. "[Hyperloglog: the analysis of a near-optimal cardinality estimation algorithm.](https://hal.science/file/index/docid/406166/filename/FlFuGaMe07.pdf)" In Proceedings of the 2007 conference on analysis of algorithms, pp. 127-146. 2007.
#![feature(const_float_bits_conv)]
#![feature(const_fn_floating_point_arithmetic)]
#![cfg_attr(not(feature="std"), no_std)]

mod array_default;
mod bias;
pub mod bitor;
pub mod bitand;
pub mod estimated_union_cardinalities;
pub mod hyper_spheres_sketch;
pub mod hyperloglog;
pub mod hyperloglog_array;
pub mod iter;
pub mod log;
mod max_min;
mod precisions;
mod primitive;
mod raw_estimate_data;
pub mod serde;
pub mod utils;
mod zeros;
mod ones;
mod hyperloglog_trait;

#[cfg(feature = "std")]
mod hyperloglog_multeplicities;
#[cfg(feature = "std")]
mod optimizers;
#[cfg(feature = "std")]
mod exact_hyper_spheres_sketch;


pub use crate::estimated_union_cardinalities::EstimatedUnionCardinalities;
pub use crate::hyperloglog::HyperLogLog;

pub mod prelude {
    pub use crate::array_default::*;
    pub use crate::estimated_union_cardinalities::*;
    pub use crate::hyper_spheres_sketch::*;
    pub use crate::hyperloglog::*;
    pub use crate::hyperloglog_trait::*;
    pub use crate::hyperloglog_array::*;
    pub use crate::iter::*;
    pub use crate::max_min::*;
    pub use crate::precisions::*;
    pub use crate::primitive::*;
    pub use crate::serde::*;
    pub use crate::utils::*;
    pub use crate::zeros::*;
    pub use crate::ones::*;
    pub use core::ops::{BitOr, BitOrAssign};

    #[cfg(feature = "std")]
    pub use crate::hyperloglog_multeplicities::*;
    #[cfg(feature = "std")]
    pub use crate::optimizers::*;
}
