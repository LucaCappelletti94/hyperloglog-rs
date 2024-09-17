//! This module contains implementations of the `Set` trait for various HyperLogLog
use core::hash::BuildHasher;

use hyperloglog_rs::prelude::PackedRegister;
use hyperloglog_rs::prelude::Bits;
use hyperloglog_rs::prelude::HyperLogLog;
use hyperloglog_rs::prelude::Registers;
use mem_dbg::{MemDbg, MemSize};

use cardinality_estimator::CardinalityEstimator;
use hyperloglog_rs::prelude::{HasherType, Precision};
use hyperloglogplus::HyperLogLog as TabacHyperLogLog;
use hyperloglogplus::HyperLogLogPF as TabacHyperLogLogPF;
use hyperloglogplus::HyperLogLogPlus as TabacHyperLogLogPlus;
use hypertwobits::h2b::HyperTwoBits as H2B;

use crate::traits::Set;
use hypertwobits::h3b::HyperThreeBits as H3B;
use rust_hyperloglog::HyperLogLog as RustHyperLogLog;
use simple_hll::HyperLogLog as SimpleHyperLogLog;
use sourmash::signature::SigsTrait;
use sourmash::sketch::hyperloglog::HyperLogLog as SourMashHyperLogLog;
use std::marker::PhantomData;
use streaming_algorithms::HyperLogLog as SAHyperLogLog;

/// Trait to associate a Hasher with a HasherBuilder
pub trait HasherBuilderAssociated: HasherType + MemSize {
    /// The associated HasherBuilder
    type Builder: BuildHasher + Default + Clone + Send + Sync + MemSize;
}

impl HasherBuilderAssociated for twox_hash::XxHash64 {
    type Builder = twox_hash::RandomXxHashBuilder64;
}

impl HasherBuilderAssociated for twox_hash::xxh3::Hash64 {
    type Builder = twox_hash::xxh3::RandomHashBuilder64;
}

impl HasherBuilderAssociated for wyhash::WyHash {
    type Builder = wyhash::WyHasherBuilder;
}

impl HasherBuilderAssociated for ahash::AHasher {
    type Builder = ahash::RandomState;
}

#[derive(Debug, Clone, Default, MemDbg, MemSize)]
/// Wrapper for the SimpleHyperLogLog implementation
pub struct SimpleHLL<H: HasherType, const P: usize> {
    estimator: SimpleHyperLogLog<H, P>,
}

#[derive(Debug, Clone, Default, MemDbg, MemSize)]
/// Wrapper for the CloudFlare HyperLogLog implementation
pub struct CloudFlareHLL<const P: usize, const B: usize, H: HasherType> {
    estimator: CardinalityEstimator<u64, H, P, B>,
}

#[derive(Debug, Clone, Default, MemDbg, MemSize)]
/// Wrapper for the HyperTwoBits implementation
pub struct HyperTwoBits<S: hypertwobits::h2b::Sketch, H: HasherBuilderAssociated> {
    estimator: H2B<S, H::Builder>,
}

#[derive(Debug, Clone, Default, MemDbg, MemSize)]
/// Wrapper for the HyperThreeBits implementation
pub struct HyperThreeBits<S: hypertwobits::h3b::Sketch, H: HasherBuilderAssociated> {
    estimator: H3B<S, H::Builder>,
}

#[derive(Debug, Clone, MemDbg, MemSize)]
/// Wrapper for the SourMash implementation
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
/// Wrapper for the RustHyperLogLog implementation
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
/// Wrapper for the TabacHyperLogLogPlus implementation
pub struct TabacHLLPlusPlus<P: Precision, H: HasherBuilderAssociated> {
    estimator: TabacHyperLogLogPlus<u64, H::Builder>,
    _precision: PhantomData<P>,
}

impl<P: Precision, H: HasherBuilderAssociated> Default for TabacHLLPlusPlus<P, H> {
    fn default() -> Self {
        Self {
            estimator: TabacHyperLogLogPlus::new(P::EXPONENT, H::Builder::default()).unwrap(),
            _precision: PhantomData,
        }
    }
}

#[derive(Clone, MemDbg, MemSize)]
/// Wrapper for the TabacHyperLogLog implementation
pub struct TabacHLL<P: Precision, H: HasherBuilderAssociated> {
    estimator: TabacHyperLogLogPF<u64, H::Builder>,
    _precision: PhantomData<P>,
}

impl<P: Precision, H: HasherBuilderAssociated> Default for TabacHLL<P, H> {
    fn default() -> Self {
        Self {
            estimator: TabacHyperLogLogPF::new(P::EXPONENT, H::Builder::default()).unwrap(),
            _precision: PhantomData,
        }
    }
}

#[derive(Debug, Clone, MemDbg, MemSize)]
/// Wrapper for the Alec HyperLogLog implementation
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

impl<H: HasherType, P: Precision, B: Bits, R: Registers<P, B>> Set
    for HyperLogLog<P, B, R, H>
{
    fn insert_element(&mut self, value: u64) {
        self.insert(&value);
    }

    fn cardinality(&self) -> f64 {
        self.estimate_cardinality()
    }

    fn union(&self, other: &Self) -> f64 {
        self.estimate_union_cardinality(other)
    }

    fn model_name(&self) -> String {
        format!(
            "HLL<P{}, B{}> + {}",
            P::EXPONENT,
            B::NUMBER_OF_BITS,
            core::any::type_name::<H>().split("::").last().unwrap()
        )
    }
}

impl<H: HasherType, const P: usize> Set for SimpleHLL<H, P> {
    fn insert_element(&mut self, item: u64) {
        self.estimator.add_object(&item);
    }

    fn cardinality(&self) -> f64 {
        self.estimator.count() as f64
    }

    fn union(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.merge(&other.estimator);
        copy.estimator.count() as f64
    }

    fn model_name(&self) -> String {
        format!(
            "SimpleHLL<P{}, B6> + {}",
            P,
            core::any::type_name::<H>().split("::").last().unwrap()
        )
    }
}

impl<S: hypertwobits::h2b::Sketch + Clone, H: HasherBuilderAssociated> Set for HyperTwoBits<S, H> {
    fn insert_element(&mut self, item: u64) {
        self.estimator.insert(&item);
    }

    fn cardinality(&self) -> f64 {
        self.estimator.count() as f64
    }

    fn union(&self, other: &Self) -> f64 {
        let mut copy = self.estimator.clone();
        copy.merge(other.estimator.clone());
        copy.count() as f64
    }

    fn model_name(&self) -> String {
        format!(
            "H2B<{}> + {}",
            core::any::type_name::<S>().split("::").last().unwrap(),
            core::any::type_name::<H>().split("::").last().unwrap()
        )
    }
}

impl<S: hypertwobits::h3b::Sketch + Clone, H: HasherBuilderAssociated> Set
    for HyperThreeBits<S, H>
{
    fn insert_element(&mut self, item: u64) {
        self.estimator.insert(&item);
    }

    fn cardinality(&self) -> f64 {
        self.estimator.count() as f64
    }

    fn union(&self, other: &Self) -> f64 {
        let mut copy = self.estimator.clone();
        copy.merge(other.estimator.clone());
        copy.count() as f64
    }

    fn model_name(&self) -> String {
        format!(
            "H3B<{}> + {}",
            core::any::type_name::<S>().split("::").last().unwrap(),
            core::any::type_name::<H>().split("::").last().unwrap()
        )
    }
}

impl<P: Precision> Set for SourMash<P> {
    fn insert_element(&mut self, item: u64) {
        self.estimator
            .add_sequence(item.to_le_bytes().as_ref(), false)
            .unwrap();
    }

    fn cardinality(&self) -> f64 {
        self.estimator.cardinality() as f64
    }

    fn union(&self, other: &Self) -> f64 {
        self.estimator.union(&other.estimator) as f64
    }

    fn model_name(&self) -> String {
        format!("SM<P{}, B8> + Vec", P::EXPONENT)
    }
}

impl<H: HasherType, const P: usize, const B: usize> Set for CloudFlareHLL<P, B, H> {
    fn insert_element(&mut self, item: u64) {
        self.estimator.insert(&item);
    }

    fn cardinality(&self) -> f64 {
        self.estimator.estimate() as f64
    }

    fn union(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.merge(&other.estimator);
        copy.estimator.estimate() as f64
    }

    fn model_name(&self) -> String {
        format!(
            "CF<P{}, B{}, Mix> + {}",
            P,
            B,
            core::any::type_name::<H>().split("::").last().unwrap()
        )
    }
}

impl<P: Precision> Set for RustHLL<P> {
    fn insert_element(&mut self, item: u64) {
        self.estimator.insert(&item);
    }

    fn cardinality(&self) -> f64 {
        self.estimator.len()
    }

    fn union(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.merge(&other.estimator);
        copy.estimator.len()
    }

    fn model_name(&self) -> String {
        format!("FrankPP<P{}, B8> + SipHasher13", P::EXPONENT)
    }
}

impl<H: HasherBuilderAssociated, P: Precision> Set for TabacHLLPlusPlus<P, H> {
    fn insert_element(&mut self, item: u64) {
        self.estimator.insert(&item);
    }

    fn cardinality(&self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.count()
    }

    fn union(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.merge(&other.estimator).unwrap();
        copy.estimator.count()
    }

    fn model_name(&self) -> String {
        format!(
            "TabacPP<P{}, B6> + {}",
            P::EXPONENT,
            core::any::type_name::<H>().split("::").last().unwrap()
        )
    }
}

impl<H: HasherBuilderAssociated, P: Precision> Set for TabacHLL<P, H> {
    fn insert_element(&mut self, item: u64) {
        self.estimator.insert(&item);
    }

    fn cardinality(&self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.count()
    }

    fn union(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.merge(&other.estimator).unwrap();
        copy.estimator.count()
    }

    fn model_name(&self) -> String {
        format!(
            "Tabac<P{}, B6> + {}",
            P::EXPONENT,
            core::any::type_name::<H>().split("::").last().unwrap()
        )
    }
}

impl<P: Precision> Set for AlecHLL<P> {
    fn insert_element(&mut self, item: u64) {
        self.estimator.push(&item);
    }

    fn cardinality(&self) -> f64 {
        self.estimator.len()
    }

    fn union(&self, other: &Self) -> f64 {
        let mut copy = self.clone();
        copy.estimator.union(&other.estimator);
        copy.estimator.len()
    }

    fn model_name(&self) -> String {
        format!("SA<P{}, B6> + XxHash64", self.estimator.precision())
    }
}
