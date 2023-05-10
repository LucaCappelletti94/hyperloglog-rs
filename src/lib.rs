#![feature(generic_const_exprs)]
#![feature(const_fn_floating_point_arithmetic)]

mod alpha;
mod hyperloglog;
mod small_corrections_lookup_table;
mod specialized_counts;
mod utils;

pub mod prelude {
    pub use crate::alpha::*;
    pub use crate::hyperloglog::*;
    pub use crate::small_corrections_lookup_table::*;
    pub use crate::specialized_counts::*;
    pub use crate::utils::*;
    pub use core::ops::{BitOr, BitOrAssign};
}
