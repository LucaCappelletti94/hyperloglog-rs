#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![cfg_attr(not(feature = "std"), no_std)]

mod basicloglog;
mod bits;
mod estimator;
pub mod hybrid;
mod hyperloglog;
mod hyperloglog_macro;
#[cfg(feature = "beta")]
mod loglogbeta;
#[cfg(feature = "mle")]
mod mle;
#[cfg(feature = "plusplus")]
mod plusplus;
mod precisions;
mod registers;
pub mod sketches;
mod utils;

#[cfg(feature = "serde")]
pub mod serde;

#[cfg(feature = "std")]
mod hashset;

/// Re-exports of the most important traits and structs.
pub mod prelude {
    pub use crate::bits::*;
    pub use crate::estimator::*;
    pub use crate::hybrid::*;
    pub use crate::hyperloglog::*;
    #[cfg(feature = "beta")]
    pub use crate::loglogbeta::*;
    #[cfg(feature = "mle")]
    pub use crate::mle::*;
    #[cfg(feature = "plusplus")]
    pub use crate::plusplus::*;
    pub use crate::precisions::*;
    pub use crate::registers::*;
    pub use crate::sketches::*;
    pub use crate::utils::*;
    pub use core::ops::{BitOr, BitOrAssign};
}
