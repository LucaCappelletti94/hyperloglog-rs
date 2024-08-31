use ahash::AHasher;
use hyperloglog_rs::prelude::*;
use macro_test_utils::cardinality_benchmark;
use statistical_comparisons::enumerations::*;
use statistical_comparisons::reports_generator::SetTester;
use twox_hash::{xxh3::Hash64 as XxH3, XxHash64};
use wyhash::WyHash;

#[cardinality_benchmark]
fn main() {
    // We init the logger
    env_logger::init();
    cardinality_benchmarks();
    statistical_comparisons::cartesian_wilcoxon_test("cardinality");
    // cartesian_wilcoxon_test("union");
}
