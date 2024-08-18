use hyperloglog_rs::prelude::*;
// use statistical_comparisons::cartesian_wilcoxon_test;
use statistical_comparisons::enumerations::*;
use statistical_comparisons::reports_generator::SetTester;

/// Macro to generate the calls to the HLLVariant reports for precisions from 4 to 18.
macro_rules! generate_hll_variant_reports {
    ($($exponent:expr),*) => {
        $(
            paste::item! {
                HLLVariants::<$exponent, [<Precision $exponent>]>::prepare_cardinality_reports();
                // HLLVariants::<$exponent, [<Precision $exponent>]>::prepare_union_reports();
            }
        )*
    };
}

fn main() {
    // We init the logger
    env_logger::init();

    HyperTwoVariants::prepare_cardinality_reports();
    // HyperTwoVariants::prepare_union_reports();
    generate_hll_variant_reports!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);

    // cartesian_wilcoxon_test("cardinality");
    // cartesian_wilcoxon_test("union");
}
