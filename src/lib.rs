#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

mod basicloglog;
mod bits;
mod estimated_union_cardinalities;
mod estimator;
mod hybrid;
pub mod hyper_spheres_sketch;
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
mod utils;

#[cfg(feature = "serde")]
pub mod serde;

#[cfg(feature = "std")]
mod exact_hyper_spheres_sketch;

/// Re-exports of the most important traits and structs.
pub mod prelude {
    pub use crate::bits::*;
    pub use crate::estimator::*;
    pub use crate::hybrid::*;
    pub use crate::hyper_spheres_sketch::*;
    pub use crate::hyperloglog::*;
    #[cfg(feature = "beta")]
    pub use crate::loglogbeta::*;
    #[cfg(feature = "mle")]
    pub use crate::mle::*;
    #[cfg(feature = "plusplus")]
    pub use crate::plusplus::*;
    pub use crate::precisions::*;
    pub use crate::registers::*;
    pub use crate::utils::*;
    pub use core::ops::{BitOr, BitOrAssign};
}
