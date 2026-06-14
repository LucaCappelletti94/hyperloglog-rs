#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unused_macro_rules)]
#![deny(unconditional_recursion)]
#![deny(unreachable_patterns)]
#![deny(unused_import_braces)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![cfg_attr(not(feature = "std"), no_std)]

// The `alloc` crate is not auto-injected in no_std builds; bring it in so the allocation-backed
// register and the no_std MLE compile with `alloc` but without `std`. The `vec!` macro it provides
// is only needed by the MLE, so `macro_use` is applied only then to avoid an unused-import warning.
#[cfg(feature = "alloc")]
#[cfg_attr(feature = "mle", macro_use)]
extern crate alloc;

mod bits;
pub mod composite_hash;
mod correction_coefficients;
pub mod estimator;
mod hash_list;
pub mod hyperloglog;
#[cfg(feature = "mle")]
pub mod mle;
mod precisions;
mod registers;
pub mod sketches;
pub mod utils;

/// Re-exports of the most important traits and structs.
pub mod prelude {
    pub use crate::bits::*;
    pub use crate::estimator::CardinalityEstimator;
    pub use crate::hyperloglog::*;
    #[cfg(feature = "mle")]
    pub use crate::mle::{Adam, Chain, JointOptimizer, Lbfgs, Mle, RmsProp};
    pub use crate::precisions::*;
    pub use crate::registers::*;
    pub use crate::sketches::*;
    pub use crate::utils::*;
    pub use core::ops::{BitOr, BitOrAssign};
}
