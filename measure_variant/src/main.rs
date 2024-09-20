//! Script to log and compare the progress of two variants of HLL.

use hyperloglog_rs::prelude::*;
use mem_dbg::MemSize;
use test_utils::prelude::{
    cardinality_samples, compare_features, read_csv, write_csv, CardinalitySample, Set,
};
use wyhash::WyHash;

// type HLL1 = HyperLogLog<
//     Precision4,
//     Bits6,
//     <Precision4 as PackedRegister<Bits6>>::Array,
//     twox_hash::XxHash64,
// >;

#[derive(Clone, MemSize)]
struct Uncorrected<P: Precision + PackedRegister<B>, B: Bits> {
    hll: HyperLogLog<P, B, <P as PackedRegister<B>>::Array, WyHash>,
}

impl<P: Precision + PackedRegister<B>, B: Bits> Default for Uncorrected<P, B> {
    fn default() -> Self {
        Self {
            hll: HyperLogLog::default(),
        }
    }
}

impl<P: Precision + PackedRegister<B>, B: Bits> Set for Uncorrected<P, B> {
    #[inline]
    fn cardinality(&self) -> f64 {
        self.hll.uncorrected_estimate_cardinality()
    }

    #[inline]
    fn insert_element(&mut self, value: u64) {
        self.hll.insert(&value);
    }

    #[inline]
    fn model_name(&self) -> String {
        format!("Uncorrected<P{}, B{}>", P::EXPONENT, B::NUMBER_OF_BITS)
    }

    #[inline]
    fn union(&self, _other: &Self) -> f64 {
        unimplemented!()
    }
}

const ITERATIONS: u64 = 1_000 * 64;

fn measure<S: Set + Default + MemSize>(
    reference: Option<&[CardinalitySample]>,
) -> Vec<CardinalitySample> {
    let maximum_cardinality = 1 << 19;
    let model_name = S::default().model_name();
    let path = format!("{}.csv.gz", model_name);

    let stored_reports = if let Ok(stored_reports) = read_csv(path.as_str()) {
        stored_reports
    } else {
        let total_reports = cardinality_samples::<S>(ITERATIONS, maximum_cardinality);
        write_csv(total_reports.iter(), &path);
        total_reports
    };

    if let Some(reference) = reference {
        let old_errors: Vec<f64> = reference
            .iter()
            .map(|report| report.relative_error_mean)
            .collect();
        let new_errors: Vec<f64> = stored_reports
            .iter()
            .map(|report| report.relative_error_mean)
            .collect();
        let error_benchmark = compare_features(
            new_errors.as_slice(),
            old_errors.as_slice(),
            "Relative Error",
        );

        error_benchmark.print();
    }

    stored_reports
}

/// Main function to compare the progress of two variants of HLL.
fn main() {
    let _reference = measure::<Uncorrected<Precision4, Bits4>>(None);
    // measure::<CloudFlareHLL<4, 6, XxHash64>>(Some(&reference));
    // measure::<TabacHLLPlusPlus<Precision4, XxHash64>>(Some(&reference));
}
