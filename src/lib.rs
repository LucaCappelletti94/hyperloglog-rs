#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

mod array_default;
mod bits;
pub mod estimated_union_cardinalities;
pub mod hyper_spheres_sketch;
pub mod hyperloglog;
mod hyperloglog_array;
pub mod hyperloglog_array_trait;
mod hyperloglog_trait;
pub mod iter;
mod precisions;
mod registers;
mod utils;
#[cfg(feature = "std")]
mod mle;

#[cfg(feature = "serde")]
pub mod serde;

#[cfg(feature = "std")]
mod exact_hyper_spheres_sketch;

pub use crate::estimated_union_cardinalities::EstimatedUnionCardinalities;
pub use crate::hyperloglog::HyperLogLog;

pub mod prelude {
    pub use crate::array_default::*;
    pub use crate::bits::*;
    pub use crate::estimated_union_cardinalities::*;
    pub use crate::hyper_spheres_sketch::*;
    pub use crate::hyperloglog::*;
    pub use crate::hyperloglog_array::*;
    pub use crate::hyperloglog_array_trait::*;
    pub use crate::hyperloglog_trait::*;
    pub use crate::iter::*;
    pub use crate::precisions::*;
    pub use crate::registers::*;
    #[cfg(feature = "serde")]
    pub use crate::serde::*;
    #[cfg(feature = "std")]
    pub use crate::mle::*;
    pub use crate::utils::*;
    pub use core::ops::{BitOr, BitOrAssign};
}
