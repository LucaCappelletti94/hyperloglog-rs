use hyperloglog_rs::prelude::*;
use statistical_comparisons::cartesian_wilcoxon_test;
use statistical_comparisons::enumerations::*;
use statistical_comparisons::reports_generator::SetTester;
use twox_hash::{XxHash64, xxh3::Hash64 as XxH3};
use wyhash::WyHash;
use ahash::AHasher;

/// Macro to generate the calls to the HLLVariant reports for precisions from 4 to 18.
macro_rules! generate_hll_variant_reports {
    ($hasher:ty, $($exponent:expr),*) => {
        $(
            paste::item! {
                HLLVariants::<$exponent, [<Precision $exponent>], $hasher>::prepare_cardinality_reports();
                // HLLVariants::<$exponent, [<Precision $exponent>]>::prepare_union_reports();
            }
        )*
    };
}

macro_rules! generate_hll_variant_reports_per_hasher {
    ($($hasher:ty),*) => {
        $(
            HyperTwoVariants::<$hasher>::prepare_cardinality_reports();
            // HyperTwoVariants::<$hasher>::prepare_union_reports();
            generate_hll_variant_reports!($hasher, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);
        )*
    };
}

fn main() {
    // We init the logger
    env_logger::init();

    generate_hll_variant_reports_per_hasher!(XxHash64, XxH3, WyHash, AHasher);

    cartesian_wilcoxon_test("cardinality");
    // cartesian_wilcoxon_test("union");
}
