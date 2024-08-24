#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

mod cartesian_wilcoxon_test;
mod estimation_tests;
pub use cartesian_wilcoxon_test::cartesian_wilcoxon_test;
pub mod enumerations;
mod proxy_implementations;
pub mod reports_generator;
mod traits;
