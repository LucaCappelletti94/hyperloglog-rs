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
mod correction_coefficients;
mod hash_list;
pub mod hyperloglog;
#[cfg(feature = "mle")]
mod mle;
mod precisions;
mod registers;
pub mod sketches;
pub mod utils;

/// Re-exports of the most important traits and structs.
pub mod prelude {
    pub use crate::bits::*;
    pub use crate::hyperloglog::*;
    pub use crate::precisions::*;
    pub use crate::registers::*;
    pub use crate::sketches::*;
    pub use crate::utils::*;
    pub use core::ops::{BitOr, BitOrAssign};
}
