#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

mod estimation_tests;
mod cartesian_wilcoxon_test;
pub use cartesian_wilcoxon_test::cartesian_wilcoxon_test;
mod proxy_implementations;
pub mod enumerations;
mod traits;
pub mod reports_generator;
mod csv_utils;