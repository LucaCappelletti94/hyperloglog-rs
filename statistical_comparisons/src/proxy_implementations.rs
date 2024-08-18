use std::hash::RandomState;

use mem_dbg::{MemDbg, MemSize};

use cardinality_estimator::CardinalityEstimator;
use hyperloglog_rs::prelude::{Estimator, ExtendableApproximatedSet, HasherType, Precision};
use hyperloglogplus::HyperLogLog as TabacHyperLogLog;
use hyperloglogplus::HyperLogLogPF as TabacHyperLogLogPF;
use hyperloglogplus::HyperLogLogPlus as TabacHyperLogLogPlus;
use hypertwobits::h2b::HyperTwoBits as H2B;

use hypertwobits::h3b::HyperThreeBits as H3B;
use rust_hyperloglog::HyperLogLog as RustHyperLogLog;
use simple_hll::HyperLogLog as SimpleHyperLogLog;
use sourmash::signature::SigsTrait;
use sourmash::sketch::hyperloglog::HyperLogLog as SourMashHyperLogLog;
use std::marker::PhantomData;
use streaming_algorithms::HyperLogLog as SAHyperLogLog;

#[derive(Debug, Clone, Default, MemDbg, MemSize)]
pub struct SimpleHLL<const P: usize> {
    estimator: SimpleHyperLogLog<P>,
}

#[derive(Debug, Clone, Default, MemDbg, MemSize)]
pub struct CloudFlareHLL<const P: usize, const B: usize, H: HasherType> {
    estimator: CardinalityEstimator<u64, H, P, B>,
}

#[derive(Debug, Clone, Default, MemDbg, MemSize)]
pub struct HyperTwoBits<S: hypertwobits::h2b::Sketch> {
    estimator: H2B<S>,
}

#[derive(Debug, Clone, Default, MemDbg, MemSize)]
pub struct HyperThreeBits<S: hypertwobits::h3b::Sketch> {
    estimator: H3B<S>,
}

#[derive(Debug, Clone, MemDbg, MemSize)]
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
            estimator: SourMashHyperLogLog::new(P::EXPONENT as usize, usize::MAX).unwrap(),
            _precision: PhantomData,
        }
    }
}

#[derive(Debug, Clone, MemDbg, MemSize)]
pub struct RustHLL<P: Precision> {
    estimator: RustHyperLogLog,
    _precision: PhantomData<P>,
}

impl<P: Precision> Default for RustHLL<P> {
    fn default() -> Self {
        Self {
            estimator: RustHyperLogLog::new_deterministic(
                P::error_rate(),
                6_755_343_421_867_645_123_u128,
            ),
            _precision: PhantomData,
        }
    }
}

#[derive(Clone, MemDbg, MemSize)]
pub struct TabacHLLPlusPlus<P: Precision> {
    estimator: TabacHyperLogLogPlus<u64, RandomState>,
    _precision: PhantomData<P>,
}

impl<P: Precision> Default for TabacHLLPlusPlus<P> {
    fn default() -> Self {
        Self {
            estimator: TabacHyperLogLogPlus::new(P::EXPONENT, RandomState::default())
                .unwrap(),
            _precision: PhantomData,
        }
    }
}

#[derive(Clone, MemDbg, MemSize)]
pub struct TabacHLL<P: Precision> {
    estimator: TabacHyperLogLogPF<u64, RandomState>,
    _precision: PhantomData<P>,
}

impl<P: Precision> Default for TabacHLL<P> {
    fn default() -> Self {
        Self {
            estimator: TabacHyperLogLogPF::new(P::EXPONENT, RandomState::default()).unwrap(),
            _precision: PhantomData,
        }
    }
}

#[derive(Debug, Clone, MemDbg, MemSize)]
pub struct AlecHLL<P: Precision> {
    estimator: SAHyperLogLog<u64>,
    _precision: PhantomData<P>,
}

impl<P: Precision> Default for AlecHLL<P> {
    fn default() -> Self {
        Self {
            estimator: SAHyperLogLog::new(P::error_rate()),
            _precision: PhantomData,
        }
    }
}

#[cfg(feature = "std")]
impl<const P: usize> hyperloglog_rs::prelude::Named for SimpleHLL<P> {
    fn name(&self) -> String {
        format!("SHLL<P{P}, B8, Vec>")
    }
}

#[cfg(feature = "std")]
impl<S: hypertwobits::h2b::Sketch> hyperloglog_rs::prelude::Named for HyperTwoBits<S> {
    fn name(&self) -> String {
        format!(
            "H2B<{}>",
            std::any::type_name::<S>().split("::").last().unwrap()
        )
    }
}

#[cfg(feature = "std")]
impl<S: hypertwobits::h3b::Sketch> hyperloglog_rs::prelude::Named for HyperThreeBits<S> {
    fn name(&self) -> String {
        format!(
            "H3B<{}>",
            std::any::type_name::<S>().split("::").last().unwrap()
        )
    }
}

#[cfg(feature = "std")]
impl<P: Precision> hyperloglog_rs::prelude::Named for SourMash<P> {
    fn name(&self) -> String {
        format!("SM<P{}, B8, Vec>", P::EXPONENT)
    }
}

#[cfg(feature = "std")]
impl<H: HasherType, const P: usize, const B: usize> hyperloglog_rs::prelude::Named
    for CloudFlareHLL<P, B, H>
{
    fn name(&self) -> String {
        format!(
            "CF<P{}, B{}, Mix> + {}",
            P,
            B,
            std::any::type_name::<H>().split("::").last().unwrap()
        )
    }
}

#[cfg(feature = "std")]
impl<P: Precision> hyperloglog_rs::prelude::Named for RustHLL<P> {
    fn name(&self) -> String {
        format!("FrankPP<P{}, B8, Vec> + SipHasher13", P::EXPONENT)
    }
}

#[cfg(feature = "std")]
impl<P: Precision> hyperloglog_rs::prelude::Named for TabacHLLPlusPlus<P> {
    fn name(&self) -> String {
        format!("TabacPP<P{}, B6, Vec> + XxHash64", P::EXPONENT)
    }
}

#[cfg(feature = "std")]
impl<P: Precision> hyperloglog_rs::prelude::Named for TabacHLL<P> {
    fn name(&self) -> String {
        format!("Tabac<P{}, B6, Vec> + XxHash64", P::EXPONENT)
    }
}

#[cfg(feature = "std")]
impl<P: Precision> hyperloglog_rs::prelude::Named for AlecHLL<P> {
    fn name(&self) -> String {
        assert_eq!({P::EXPONENT}, self.estimator.precision());

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

impl<P: Precision> ExtendableApproximatedSet<u64> for AlecHLL<P> {
    fn insert(&mut self, item: &u64) -> bool {
        self.estimator.push(item);
        true
    }
}

impl<const P: usize> Estimator<f64> for SimpleHLL<P> {
    #[expect(clippy::cast_precision_loss, reason = "We do not expect to exceeed 2**54 in set cardinality in our tests.")]
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.count() as f64
    }

    #[expect(clippy::cast_precision_loss, reason = "We do not expect to exceeed 2**54 in set cardinality in our tests.")]
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
    #[expect(clippy::cast_precision_loss, reason = "We do not expect to exceeed 2**54 in set cardinality in our tests.")]
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.estimate() as f64
    }

    #[expect(clippy::cast_precision_loss, reason = "We do not expect to exceeed 2**54 in set cardinality in our tests.")]
    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.merge(&other.estimator);
        copy.estimator.estimate() as f64
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        true
    }
}

impl<S: Clone + hypertwobits::h2b::Sketch + Send + Sync> Estimator<f64> for HyperTwoBits<S> {
    #[expect(clippy::cast_precision_loss, reason = "We do not expect to exceeed 2**54 in set cardinality in our tests.")]
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.count() as f64
    }

    #[expect(clippy::cast_precision_loss, reason = "We do not expect to exceeed 2**54 in set cardinality in our tests.")]
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
    #[expect(clippy::cast_precision_loss, reason = "We do not expect to exceeed 2**54 in set cardinality in our tests.")] 
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.count() as f64
    }

    #[expect(clippy::cast_precision_loss, reason = "We do not expect to exceeed 2**54 in set cardinality in our tests.")]
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
    #[expect(clippy::cast_precision_loss, reason = "We do not expect to exceeed 2**54 in set cardinality in our tests.")]
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.cardinality() as f64
    }

    #[expect(clippy::cast_precision_loss, reason = "We do not expect to exceeed 2**54 in set cardinality in our tests.")]
    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        self.estimator.union(&other.estimator) as f64
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
    }
}

impl<P: Precision> Estimator<f64> for RustHLL<P> {
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.len()
    }

    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.merge(&other.estimator);
        copy.estimator.len()
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
    }
}

impl<P: Precision> Estimator<f64> for TabacHLLPlusPlus<P> {
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.clone().count()
    }

    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.merge(&other.estimator).unwrap();
        copy.estimator.count()
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
    }
}

impl<P: Precision> Estimator<f64> for TabacHLL<P> {
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.clone().count()
    }

    fn estimate_union_cardinality(&self, _other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.merge(&self.estimator).unwrap();
        copy.estimator.count()
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
    }
}

impl<P: Precision> Estimator<f64> for AlecHLL<P> {
    fn estimate_cardinality(&self) -> f64 {
        self.estimator.len()
    }

    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.union(&other.estimator);
        copy.estimator.len()
    }

    fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
        false
    }
}
