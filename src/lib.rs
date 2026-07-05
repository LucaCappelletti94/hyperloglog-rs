#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unused_macro_rules)]
#![deny(unconditional_recursion)]
#![deny(unreachable_patterns)]
#![deny(unused_import_braces)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
// Performance-critical numeric crate: intentional patterns suppressed at crate level.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::similar_names)]
#![allow(clippy::inline_always)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::float_cmp)]
#![allow(clippy::float_cmp_const)]
#![allow(clippy::large_stack_arrays)]
#![allow(clippy::useless_conversion)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::legacy_numeric_constants)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::multiple_bound_locations)]
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::as_ptr_cast_mut)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::infallible_destructuring_match)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::transmute_ptr_to_ptr)]
#![allow(clippy::if_not_else)]
#![allow(clippy::single_range_in_vec_init)]
#![allow(clippy::needless_match)]
#![allow(clippy::unnecessary_unwrap)]
#![allow(clippy::partialeq_ne_impl)]
#![allow(clippy::comparison_chain)]
#![allow(clippy::neg_cmp_op_on_partial_ord)]
#![allow(clippy::unreadable_literal)]
#![allow(unused_mut)]
#![allow(clippy::upper_case_acronyms)]
// The shipped crate is `no_std`. Tests link `std` (for the test harness and convenient std types in
// test code), so `no_std` is applied only outside `test`. The library code itself never uses `std`.
#![cfg_attr(not(test), no_std)]

// `alloc` is optional: it is needed only for the heap-`Vec`-backed register storage (`VecHll`); the
// default array-backed counter and all of the MLE are allocation-free. No production code uses the
// `vec!` macro, so `macro_use` is not needed; the only `vec!` uses are in tests, where it comes from
// the `std` prelude.
#[cfg(feature = "alloc")]
extern crate alloc;

// Re-export shared sketching primitives from the core crate.
pub use sketching_core::prelude::{
    Bits, Bits10, Bits11, Bits12, Bits13, Bits14, Bits15, Bits16, Bits4, Bits5, Bits6, Bits7,
    Bits8, Bits9, Precision, Precision10, Precision11, Precision12, Precision13, Precision14,
    Precision15, Precision16, Precision17, Precision18, Precision4, Precision5, Precision6,
    Precision7, Precision8, Precision9,
};
pub use sketching_core::{
    split_packed_index, CardinalityEstimator as CoreCardinalityEstimator, FloatOps,
    HyperSpheresSketch as CoreHyperSpheresSketch, JointSketch as CoreJointSketch,
    JointSketchError as CoreJointSketchError, Number, One, Packed, PositiveInteger, VariableWord,
    Zero,
};

pub mod adaptive;
#[cfg(feature = "alloc")]
pub(crate) mod broadword;
pub mod composite_hash;
mod correction_coefficients;
pub mod error_model;
pub mod estimator;
mod hash_list;
pub mod hyperloglog;
pub mod mle;
mod registers;
pub mod sigma_tau;
pub mod sketches;
pub mod slice_view;
pub mod utils;

/// Re-exports of the most important traits and structs.
pub mod prelude {
    pub use crate::adaptive::Adaptive;
    pub use crate::error_model::*;
    pub use crate::estimator::HllCardinalityEstimator;
    pub use crate::hyperloglog::*;
    pub use crate::mle::{JointMle, Mle};
    pub use crate::registers::*;
    pub use crate::sketches::*;
    pub use crate::utils::*;
    pub use core::ops::{BitOr, BitOrAssign};
    pub use sketching_core::prelude::*;
}
