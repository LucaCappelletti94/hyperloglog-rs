#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unused_macro_rules)]
#![deny(unconditional_recursion)]
#![deny(unreachable_patterns)]
#![deny(unused_import_braces)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
// The shipped crate is `no_std`. Tests link `std` (for the test harness and convenient std types in
// test code), so `no_std` is applied only outside `test`. The library code itself never uses `std`.
#![cfg_attr(not(test), no_std)]

// `alloc` is optional: it is needed only for the heap-`Vec`-backed register storage (`VecHll`); the
// default array-backed counter and all of the MLE are allocation-free. No production code uses the
// `vec!` macro, so `macro_use` is not needed; the only `vec!` uses are in tests, where it comes from
// the `std` prelude.
#[cfg(feature = "alloc")]
extern crate alloc;

pub mod adaptive;
mod bits;
pub mod composite_hash;
mod correction_coefficients;
pub mod error_model;
pub mod estimator;
mod hash_list;
pub mod hyperloglog;
pub mod mle;
pub mod no_linear_counting;
mod precisions;
mod registers;
pub mod sketches;
pub mod utils;

/// Re-exports of the most important traits and structs.
pub mod prelude {
    pub use crate::adaptive::Adaptive;
    pub use crate::bits::*;
    pub use crate::error_model::*;
    pub use crate::estimator::CardinalityEstimator;
    pub use crate::hyperloglog::*;
    pub use crate::mle::{JointMle, Mle};
    pub use crate::no_linear_counting::NoLinearCounting;
    pub use crate::precisions::*;
    pub use crate::registers::*;
    pub use crate::sketches::*;
    pub use crate::utils::*;
    pub use core::ops::{BitOr, BitOrAssign};
}
