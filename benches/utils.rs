use cardinality_estimator::CardinalityEstimator;
use hyperloglog_rs::prelude::*;
use hyperloglogplus::HyperLogLog as TabacHyperLogLog;
use hyperloglogplus::HyperLogLogPF as TabacHyperLogLogPF;
use hyperloglogplus::HyperLogLogPlus as TabacHyperLogLogPlus;
use hypertwobits::h2b::HyperTwoBits as H2B;
use hypertwobits::h3b::HyperThreeBits as H3B;
use rust_hyperloglog::HyperLogLog as RustHyperLogLog;
use sourmash::signature::SigsTrait;
use sourmash::sketch::hyperloglog::HyperLogLog as SourMashHyperLogLog;
use std::marker::PhantomData;
use std::usize;
use streaming_algorithms::HyperLogLog as SAHyperLogLog;
use twox_hash::RandomXxHashBuilder64;
use simple_hll::HyperLogLog as SimpleHyperLogLog;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
pub struct SimpleHLL<const P: usize> {
    estimator: SimpleHyperLogLog<P>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
pub struct CloudFlareHLL<const P: usize, const B: usize, H: HasherType> {
    estimator: CardinalityEstimator<u64, H, P, B>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
pub struct HyperTwoBits<S: hypertwobits::h2b::Sketch> {
    estimator: H2B<S>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
pub struct HyperThreeBits<S: hypertwobits::h3b::Sketch> {
    estimator: H3B<S>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
pub struct SourMash<P: Precision> {
    estimator: SourMashHyperLogLog,
    _precision: PhantomData<P>,
}

impl<P: Precision> Default for SourMash<P> {
    fn default() -> Self {
        Self {
            // Since to the best of my knowledge SourMash does not actually use the ksize
            // parameter, we set it to a preposterously large value to ensure that errors
            // will be very apparent if it is used.
            estimator: SourMashHyperLogLog::new(P::EXPONENT, usize::MAX).unwrap(),
            _precision: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RustHLL<P: Precision> {
    estimator: RustHyperLogLog,
    _precision: PhantomData<P>,
}

impl<P: Precision> Default for RustHLL<P> {
    fn default() -> Self {
        Self {
            estimator: RustHyperLogLog::new_deterministic(
                P::error_rate(),
                6755343421867645123_u128,
            ),
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

impl<const P: usize> Named for SimpleHLL<P> {
    fn name(&self) -> String {
        format!("SHLL<P{}, B8, Vec>", P)
    }
}

impl<S: hypertwobits::h2b::Sketch> Named for HyperTwoBits<S> {
    fn name(&self) -> String {
        format!(
            "H2B<{}>",
            std::any::type_name::<S>().split("::").last().unwrap()
        )
    }
}

impl<S: hypertwobits::h3b::Sketch> Named for HyperThreeBits<S> {
    fn name(&self) -> String {
        format!(
            "H3B<{}>",
            std::any::type_name::<S>().split("::").last().unwrap()
        )
    }
}

impl<P: Precision> Named for SourMash<P> {
    fn name(&self) -> String {
        format!("SM<P{}, B8, Vec>", P::EXPONENT)
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

impl<const P: usize> ExtendableApproximatedSet<u64> for SimpleHLL<P> {
    fn insert(&mut self, item: &u64) -> bool {
        self.estimator.add_object(item);
        true
    }
}

impl<S: hypertwobits::h2b::Sketch> ExtendableApproximatedSet<u64> for HyperTwoBits<S> {
    fn insert(&mut self, item: &u64) -> bool {
        self.estimator.insert(item);
        true
    }
}

impl<S: hypertwobits::h3b::Sketch> ExtendableApproximatedSet<u64> for HyperThreeBits<S> {
    fn insert(&mut self, item: &u64) -> bool {
        self.estimator.insert(item);
        true
    }
}

impl<P: Precision> ExtendableApproximatedSet<u64> for SourMash<P> {
    fn insert(&mut self, item: &u64) -> bool {
        self.estimator
            .add_sequence(item.to_le_bytes().as_ref(), false)
            .unwrap();
        true
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

impl<const P: usize> Estimator<f64> for SimpleHLL<P> {
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.count() as f64
    }

    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.merge(&other.estimator);
        copy.estimator.count() as f64
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
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

impl<S: Clone + hypertwobits::h2b::Sketch + Send + Sync> Estimator<f64> for HyperTwoBits<S> {
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.count() as f64
    }

    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.merge(other.estimator.clone());
        copy.estimator.count() as f64
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
    }
}

impl<S: Clone + hypertwobits::h3b::Sketch + Send + Sync> Estimator<f64> for HyperThreeBits<S> {
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.count() as f64
    }

    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.merge(other.estimator.clone());
        copy.estimator.count() as f64
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
    }
}

impl<P: Precision> Estimator<f64> for SourMash<P> {
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.cardinality() as f64
    }

    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        self.estimator.union(&other.estimator) as f64
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
