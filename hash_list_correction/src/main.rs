//! Rust script to identify the optimal correction
#![deny(unsafe_code)]
#![deny(unused_macro_rules)]
#![deny(missing_docs)]
extern crate prettyplease;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

mod gap_hash;
use gap_hash::compute_gap_hash_correction;
mod utils;
use clap::Parser;

#[derive(Parser)]
struct Opts {
    only_hashlist: Option<bool>,
}

fn main() {
    let opts = Opts::parse();

    compute_gap_hash_correction(opts.only_hashlist.unwrap_or(false));
}
