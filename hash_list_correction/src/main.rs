//! Rust script to identify the optimal correction
#![deny(unsafe_code)]
#![deny(unused_macro_rules)]
#![deny(missing_docs)]
extern crate prettyplease;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

mod switch_hash;
use switch_hash::compute_switch_hash_correction;
mod gap_hash;
use gap_hash::compute_gap_hash_correction;
mod utils;

fn main() {
    compute_switch_hash_correction();
    compute_gap_hash_correction();
}
