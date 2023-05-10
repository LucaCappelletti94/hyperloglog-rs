#![feature(generic_const_exprs)]
#![feature(const_for)]
#![feature(const_float_bits_conv)]
#![feature(const_trait_impl)]
#![feature(const_mut_refs)]
#![feature(const_fn_floating_point_arithmetic)]

mod alpha;
mod hyperloglog;
mod utils;
mod log;

pub mod prelude {
    pub use crate::alpha::*;
    pub use crate::hyperloglog::*;
    pub use crate::utils::*;
    pub use crate::log::*;
    pub use core::ops::{BitOr, BitOrAssign};
}
