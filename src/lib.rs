#![feature(generic_const_exprs)]
#![feature(const_fn_floating_point_arithmetic)]

mod hyperloglog;
mod small_corrections_lookup_table;
mod alpha;
mod utils;

pub mod prelude {
    pub use crate::hyperloglog::*;
    pub use crate::small_corrections_lookup_table::*;
    pub use crate::alpha::*;
    pub use crate::utils::*;
}