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

mod bits;
pub mod composite_hash;
pub mod hyperloglog;
// #[cfg(feature = "mle")]
// mod mle;
mod correction_coefficients;
mod hashlist;
mod precisions;
mod registers;
pub mod sketches;
pub mod utils;
mod naive_integer_hash;

#[cfg(feature = "serde")]
pub mod serde;

/// Re-exports of the most important traits and structs.
pub mod prelude {
    pub use crate::bits::*;
    pub use crate::hyperloglog::*;
    // #[cfg(feature = "mle")]
    // pub use crate::mle::*;
    pub use crate::precisions::*;
    pub use crate::registers::*;
    pub use crate::sketches::*;
    pub use crate::utils::*;
    pub use core::ops::{BitOr, BitOrAssign};
    pub use crate::naive_integer_hash::NaiveIntegerHash;
}
