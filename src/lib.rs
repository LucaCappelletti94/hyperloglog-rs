#![feature(generic_const_exprs)]
#![feature(const_for)]
#![feature(const_float_bits_conv)]
#![feature(const_trait_impl)]
#![feature(const_mut_refs)]
#![feature(const_fn_floating_point_arithmetic)]

pub mod alpha;
pub mod hyperloglog;
pub mod utils;
pub mod log;

pub mod prelude {
    pub use crate::alpha::*;
    pub use crate::hyperloglog::*;
    pub use crate::utils::*;
    pub use core::ops::{BitOr, BitOrAssign};
}
