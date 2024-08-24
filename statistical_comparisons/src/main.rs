use hyperloglog_rs::prelude::*;
use statistical_comparisons::enumerations::*;
use statistical_comparisons::reports_generator::SetTester;
use twox_hash::{XxHash64, xxh3::Hash64 as XxH3};
use wyhash::WyHash;
use ahash::AHasher;
use macro_test_utils::cardinality_benchmark;



#[cardinality_benchmark]
fn main() {
    // We init the logger
    env_logger::init();
    cardinality_benchmarks();
    statistical_comparisons::cartesian_wilcoxon_test("cardinality");
    // cartesian_wilcoxon_test("union");
}
