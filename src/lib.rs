#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

mod array_default;
pub mod bitand;
pub mod bitor;
mod bitor_iter;
pub mod estimated_union_cardinalities;
pub mod hyper_spheres_sketch;
pub mod hyperloglog;
pub mod hyperloglog_array;
mod hyperloglog_multiplicities;
mod hyperloglog_trait;
pub mod iter;
mod max_min;
mod ones;
mod precisions;
mod primitive;
mod sip;
mod utils;
mod zeros;

#[cfg(feature = "serde")]
pub mod serde;

#[cfg(feature = "std")]
mod exact_hyper_spheres_sketch;
#[cfg(feature = "std")]
mod joint_estimation;

pub use crate::estimated_union_cardinalities::EstimatedUnionCardinalities;
pub use crate::hyperloglog::HyperLogLog;

pub mod prelude {
    #[cfg(feature = "std")]
    pub use crate::joint_estimation::*;

    pub use crate::array_default::*;
    pub use crate::estimated_union_cardinalities::*;
    pub use crate::hyper_spheres_sketch::*;
    pub use crate::hyperloglog::*;
    pub use crate::hyperloglog_array::*;
    pub use crate::hyperloglog_multiplicities::*;
    pub use crate::hyperloglog_trait::*;
    pub use crate::iter::*;
    pub use crate::max_min::*;
    pub use crate::ones::*;
    pub use crate::precisions::*;
    pub use crate::primitive::*;
    #[cfg(feature = "serde")]
    pub use crate::serde::*;
    pub use crate::zeros::*;
    pub use core::ops::{BitOr, BitOrAssign};
}
