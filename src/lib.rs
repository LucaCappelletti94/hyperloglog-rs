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
//! use hyperloglog_rs::HyperLogLog;
//! ```
//!
//! ## Examples
//!
//! ```rust
//! use hyperloglog_rs::HyperLogLog;
//!
//! let mut hll = HyperLogLog::<14, 5>::new();
//! hll.insert(&1);
//! hll.insert(&2);
//!
//! let mut hll2 = HyperLogLog::<14, 5>::new();
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
//! ## References
//!
//! * Flajolet, Philippe, et al. "HyperLogLog: the analysis of a near-optimal cardinality estimation algorithm." DMTCS Proceedings 1 (2007): 127-146.
//! * Heule, Stefan, Marc Nunkesser, and Alexander Hall. "HyperLogLog in practice: algorithmic engineering of a state of the art cardinality estimation algorithm." Proceedings of the 16th international conference on Extending database technology. 2013.

#![feature(generic_const_exprs)]
#![feature(const_float_bits_conv)]
#![feature(const_trait_impl)]
#![feature(const_mut_refs)]
#![feature(const_fn_floating_point_arithmetic)]

pub mod hyperloglog;
pub mod log;
pub mod utils;

pub use crate::hyperloglog::HyperLogLog;

pub mod prelude {
    pub use crate::hyperloglog::*;
    pub use core::ops::{BitOr, BitOrAssign};
}
