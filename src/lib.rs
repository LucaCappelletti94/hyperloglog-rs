#![feature(generic_const_exprs)]
#![feature(const_fn_floating_point_arithmetic)]

mod hyperloglog;
mod small_corrections_lookup_table;
mod number_of_bits_lookup_table;

pub mod prelude {
    pub use crate::hyperloglog::*;
    pub use crate::number_of_bits_lookup_table::*;
    pub use crate::small_corrections_lookup_table::*;
}