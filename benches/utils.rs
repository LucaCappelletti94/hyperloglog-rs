use cardinality_estimator::CardinalityEstimator;
use hyperloglog_rs::prelude::*;
use hyperloglogplus::HyperLogLog as TabacHyperLogLog;
use hyperloglogplus::HyperLogLogPF as TabacHyperLogLogPF;
use hyperloglogplus::HyperLogLogPlus as TabacHyperLogLogPlus;
use rust_hyperloglog::HyperLogLog as RustHyperLogLog;
use std::marker::PhantomData;
use streaming_algorithms::HyperLogLog as SAHyperLogLog;
use twox_hash::RandomXxHashBuilder64;

#[derive(Debug, Clone, Default)]
pub struct CloudFlareHLL<const P: usize, const B: usize, H: HasherType> {
    estimator: CardinalityEstimator<u64, H, P, B>,
}

#[derive(Debug, Clone)]
pub struct RustHLL<P: Precision> {
    estimator: RustHyperLogLog,
    _precision: PhantomData<P>,
}

impl<P: Precision> Default for RustHLL<P> {
    fn default() -> Self {
        Self {
            estimator: RustHyperLogLog::new_deterministic(P::error_rate(), 6755343421867645123_u128),
            _precision: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct TabacHLLPlusPlus<P: Precision> {
    estimator: TabacHyperLogLogPlus<u64, RandomXxHashBuilder64>,
    _precision: PhantomData<P>,
}

impl<P: Precision> Default for TabacHLLPlusPlus<P> {
    fn default() -> Self {
        Self {
            estimator: TabacHyperLogLogPlus::new(
                P::EXPONENT as u8,
                RandomXxHashBuilder64::default(),
            )
            .unwrap(),
            _precision: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct TabacHLL<P: Precision> {
    estimator: TabacHyperLogLogPF<u64, RandomXxHashBuilder64>,
    _precision: PhantomData<P>,
}

impl<P: Precision> Default for TabacHLL<P> {
    fn default() -> Self {
        Self {
            estimator: TabacHyperLogLogPF::new(P::EXPONENT as u8, RandomXxHashBuilder64::default())
                .unwrap(),
            _precision: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SAHLL<P: Precision> {
    estimator: SAHyperLogLog<u64>,
    _precision: PhantomData<P>,
}

impl<P: Precision> Default for SAHLL<P> {
    fn default() -> Self {
        Self {
            estimator: SAHyperLogLog::new(P::error_rate()),
            _precision: PhantomData,
        }
    }
}

impl<H: HasherType, const P: usize, const B: usize> Named for CloudFlareHLL<P, B, H> {
    fn name(&self) -> String {
        format!(
            "CF<P{}, B{}, Mix> + {}",
            P,
            B,
            std::any::type_name::<H>().split("::").last().unwrap()
        )
    }
}

impl<P: Precision> Named for RustHLL<P> {
    fn name(&self) -> String {
        format!("FrankPP<P{}, B8, Vec> + SipHasher13", P::EXPONENT)
    }
}

impl<P: Precision> Named for TabacHLLPlusPlus<P> {
    fn name(&self) -> String {
        format!("TabacPP<P{}, B6, Vec> + XxHash64", P::EXPONENT)
    }
}

impl<P: Precision> Named for TabacHLL<P> {
    fn name(&self) -> String {
        format!("Tabac<P{}, B6, Vec> + XxHash64", P::EXPONENT)
    }
}

impl<P: Precision> Named for SAHLL<P> {
    fn name(&self) -> String {
        assert_eq!(P::EXPONENT as u8, self.estimator.precision());

        format!("SA<P{}, B6, Vec> + XxHash64", self.estimator.precision())
    }
}

impl<H: HasherType, const P: usize, const B: usize> SetProperties for CloudFlareHLL<P, B, H> {
    fn is_empty(&self) -> bool {
        todo!()
    }

    fn is_full(&self) -> bool {
        todo!()
    }
}

impl<P: Precision> SetProperties for RustHLL<P> {
    fn is_empty(&self) -> bool {
        todo!()
    }

    fn is_full(&self) -> bool {
        todo!()
    }
}

impl<P: Precision> SetProperties for TabacHLLPlusPlus<P> {
    fn is_empty(&self) -> bool {
        todo!()
    }

    fn is_full(&self) -> bool {
        todo!()
    }
}

impl<P: Precision> SetProperties for TabacHLL<P> {
    fn is_empty(&self) -> bool {
        todo!()
    }

    fn is_full(&self) -> bool {
        todo!()
    }
}

impl<P: Precision> SetProperties for SAHLL<P> {
    fn is_empty(&self) -> bool {
        todo!()
    }

    fn is_full(&self) -> bool {
        todo!()
    }
}

impl<H: HasherType, const P: usize, const B: usize> ApproximatedSet<u64>
    for CloudFlareHLL<P, B, H>
{
    fn may_contain(&self, _item: &u64) -> bool {
        todo!()
    }
}

impl<P: Precision> ApproximatedSet<u64> for RustHLL<P> {
    fn may_contain(&self, _item: &u64) -> bool {
        todo!()
    }
}

impl<P: Precision> ApproximatedSet<u64> for TabacHLLPlusPlus<P> {
    fn may_contain(&self, _item: &u64) -> bool {
        todo!()
    }
}

impl<P: Precision> ApproximatedSet<u64> for TabacHLL<P> {
    fn may_contain(&self, _item: &u64) -> bool {
        todo!()
    }
}

impl<P: Precision> ApproximatedSet<u64> for SAHLL<P> {
    fn may_contain(&self, _item: &u64) -> bool {
        todo!()
    }
}

impl<H: HasherType, const P: usize, const B: usize> MutableSet for CloudFlareHLL<P, B, H> {
    fn clear(&mut self) {
        todo!()
    }
}

impl<P: Precision> MutableSet for RustHLL<P> {
    fn clear(&mut self) {
        todo!()
    }
}

impl<P: Precision> MutableSet for TabacHLLPlusPlus<P> {
    fn clear(&mut self) {
        todo!()
    }
}

impl<P: Precision> MutableSet for TabacHLL<P> {
    fn clear(&mut self) {
        todo!()
    }
}

impl<P: Precision> MutableSet for SAHLL<P> {
    fn clear(&mut self) {
        todo!()
    }
}

impl<H: HasherType, const P: usize, const B: usize> ExtendableApproximatedSet<u64>
    for CloudFlareHLL<P, B, H>
{
    fn insert(&mut self, item: &u64) -> bool {
        self.estimator.insert(item);
        true
    }
}

impl<P: Precision> ExtendableApproximatedSet<u64> for RustHLL<P> {
    fn insert(&mut self, item: &u64) -> bool {
        self.estimator.insert(item);
        true
    }
}

impl<P: Precision> ExtendableApproximatedSet<u64> for TabacHLLPlusPlus<P> {
    fn insert(&mut self, item: &u64) -> bool {
        self.estimator.insert(item);
        true
    }
}

impl<P: Precision> ExtendableApproximatedSet<u64> for TabacHLL<P> {
    fn insert(&mut self, item: &u64) -> bool {
        self.estimator.insert(item);
        true
    }
}

impl<P: Precision> ExtendableApproximatedSet<u64> for SAHLL<P> {
    fn insert(&mut self, item: &u64) -> bool {
        self.estimator.push(item);
        true
    }
}

impl<H: HasherType, const P: usize, const B: usize> Estimator<f64> for CloudFlareHLL<P, B, H> {
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.estimate() as f64
    }

    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.merge(&other.estimator);
        copy.estimator.estimate() as f64
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
    }
}

impl<P: Precision> Estimator<f64> for RustHLL<P> {
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.len() as f64
    }

    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.merge(&other.estimator);
        copy.estimator.len() as f64
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
    }
}

impl<P: Precision> Estimator<f64> for TabacHLLPlusPlus<P> {
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.clone().count() as f64
    }

    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.merge(&other.estimator).unwrap();
        copy.estimator.count() as f64
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
    }
}

impl<P: Precision> Estimator<f64> for TabacHLL<P> {
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.clone().count() as f64
    }

    fn estimate_union_cardinality(&self, _other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.merge(&self.estimator).unwrap();
        copy.estimator.count() as f64
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
    }
}

impl<P: Precision> Estimator<f64> for SAHLL<P> {
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.len() as f64
    }

    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.union(&other.estimator);
        copy.estimator.len() as f64
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
    }
}
