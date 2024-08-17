use std::collections::HashSet;
use std::fmt::Display;
use std::hash::RandomState;

use hyperloglog_rs::prelude::*;
use indicatif::ProgressIterator;
use mem_dbg::{MemSize, SizeFlags};
use stattest::test::StatisticalTest;
use stattest::test::WilcoxonWTest;

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
use std::usize;
use streaming_algorithms::HyperLogLog as SAHyperLogLog;
use twox_hash::RandomXxHashBuilder64;

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
            estimator: SourMashHyperLogLog::new(P::EXPONENT as usize, usize::MAX).unwrap(),
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

#[cfg(feature = "std")]
impl<const P: usize> hyperloglog_rs::prelude::Named for SimpleHLL<P> {
    fn name(&self) -> String {
        format!("SHLL<P{}, B8, Vec>", P)
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
impl<P: Precision> hyperloglog_rs::prelude::Named for SAHLL<P> {
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
        true
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


pub(crate) enum SetLikeObjects<const EXPONENT: usize, P: Precision>
where
    P: ArrayRegister<Bits8> + ArrayRegister<Bits6>,
{
    HashSet(HashSet<u64>),
    TabacHyperLogLogPlus(TabacHLLPlusPlus<P>),
    TabacHyperLogLogPF(TabacHLL<P>),
    SAHyperLogLog(SAHyperLogLog<u64>),
    RustHyperLogLog(RustHyperLogLog),
    CardinalityEstimator(CardinalityEstimator<u64, wyhash::WyHash, EXPONENT, 6>),
    HLL6Xxhasher(
        PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>,
    ),
    HLL6WyHash(PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, wyhash::WyHash>),
    HLL8Xxhasher(
        PlusPlus<P, Bits8, <P as ArrayRegister<Bits8>>::ArrayRegister, twox_hash::XxHash64>,
    ),
    HLL8WyHash(PlusPlus<P, Bits8, <P as ArrayRegister<Bits8>>::ArrayRegister, wyhash::WyHash>),
    Beta6Xxhasher(
        LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>,
    ),
    Beta6WyHash(LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, wyhash::WyHash>),
    Beta8Xxhasher(
        LogLogBeta<P, Bits8, <P as ArrayRegister<Bits8>>::ArrayRegister, twox_hash::XxHash64>,
    ),
    Beta8WyHash(LogLogBeta<P, Bits8, <P as ArrayRegister<Bits8>>::ArrayRegister, wyhash::WyHash>),
    #[cfg(feature = "mle")]
    MLEPPWyHash(
        MLE<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, wyhash::WyHash>, 2>,
    ),
    #[cfg(feature = "mle")]
    MLEPPXxhasher(
        MLE<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>, 2>,
    ),
    #[cfg(feature = "mle")]
    MLEBetaWyHash(
        MLE<LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, wyhash::WyHash>, 2>,
    ),
    #[cfg(feature = "mle")]
    MLEBetaXxhasher(
        MLE<
            LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>,
            2,
        >,
    ),
    #[cfg(feature = "mle")]
    HybridMLEPPWyHash(
        Hybrid<
            MLE<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, wyhash::WyHash>, 2>,
        >,
    ),
    #[cfg(feature = "mle")]
    HybridMLEPPXxhasher(
        Hybrid<
            MLE<
                PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>,
                2,
            >,
        >,
    ),
    #[cfg(feature = "mle")]
    HybridMLEBetaWyHash(
        Hybrid<
            MLE<
                LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, wyhash::WyHash>,
                2,
            >,
        >,
    ),
    #[cfg(feature = "mle")]
    HybridMLEBetaXxhasher(
        Hybrid<
            MLE<
                LogLogBeta<
                    P,
                    Bits6,
                    <P as ArrayRegister<Bits6>>::ArrayRegister,
                    twox_hash::XxHash64,
                >,
                2,
            >,
        >,
    ),
    HybridPPWyHash(
        Hybrid<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, wyhash::WyHash>>,
    ),
    HybridPPXxhasher(
        Hybrid<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>>,
    ),
    HybridBetaWyHash(
        Hybrid<LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, wyhash::WyHash>>,
    ),
    HybridBetaXxhasher(
        Hybrid<
            LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>,
        >,
    ),
}

impl<const EXPONENT: usize, P: Precision> Named for SetLikeObjects<EXPONENT, P> {
    fn name(&self) -> String {}
}

impl<const EXPONENT: usize, P: Precision> SetLikeObjects<EXPONENT, P>
where
    P: ArrayRegister<Bits8> + ArrayRegister<Bits6>,
    P: MemSize,
    <P as ArrayRegister<Bits8>>::ArrayRegister: MemSize,
    <P as ArrayRegister<Bits6>>::ArrayRegister: MemSize,
{
    pub(crate) fn all_cardinalities() -> Vec<Self> {
        vec![
            SetLikeObjects::HashSet(HashSet::create(P::EXPONENT)),
            SetLikeObjects::TabacHyperLogLogPlus(TabacHyperLogLogPlus::create(P::EXPONENT)),
            SetLikeObjects::TabacHyperLogLogPF(TabacHyperLogLogPF::create(P::EXPONENT)),
            SetLikeObjects::SAHyperLogLog(SAHyperLogLog::create(P::EXPONENT)),
            SetLikeObjects::RustHyperLogLog(<RustHyperLogLog as TestSetLike<u64>>::create(
                P::EXPONENT,
            )),
            SetLikeObjects::CardinalityEstimator(CardinalityEstimator::create(P::EXPONENT)),
            SetLikeObjects::HLL6Xxhasher(Default::default()),
            SetLikeObjects::HLL6WyHash(Default::default()),
            SetLikeObjects::HLL8Xxhasher(Default::default()),
            SetLikeObjects::HLL8WyHash(Default::default()),
            SetLikeObjects::Beta6Xxhasher(Default::default()),
            SetLikeObjects::Beta6WyHash(Default::default()),
            SetLikeObjects::Beta8Xxhasher(Default::default()),
            SetLikeObjects::Beta8WyHash(Default::default()),
            SetLikeObjects::HybridPPWyHash(Default::default()),
            SetLikeObjects::HybridPPXxhasher(Default::default()),
            SetLikeObjects::HybridBetaWyHash(Default::default()),
            SetLikeObjects::HybridBetaXxhasher(Default::default()),
        ]
    }

    pub(crate) fn all_union() -> Vec<Self> {
        vec![
            SetLikeObjects::HashSet(HashSet::create(P::EXPONENT)),
            SetLikeObjects::TabacHyperLogLogPlus(TabacHyperLogLogPlus::create(P::EXPONENT)),
            SetLikeObjects::TabacHyperLogLogPF(TabacHyperLogLogPF::create(P::EXPONENT)),
            SetLikeObjects::SAHyperLogLog(SAHyperLogLog::create(P::EXPONENT)),
            SetLikeObjects::RustHyperLogLog(<RustHyperLogLog as TestSetLike<u64>>::create(
                P::EXPONENT,
            )),
            SetLikeObjects::CardinalityEstimator(CardinalityEstimator::create(P::EXPONENT)),
            SetLikeObjects::HLL6Xxhasher(Default::default()),
            SetLikeObjects::HLL6WyHash(Default::default()),
            SetLikeObjects::HLL8Xxhasher(Default::default()),
            SetLikeObjects::HLL8WyHash(Default::default()),
            SetLikeObjects::Beta6Xxhasher(Default::default()),
            SetLikeObjects::Beta6WyHash(Default::default()),
            SetLikeObjects::Beta8Xxhasher(Default::default()),
            SetLikeObjects::Beta8WyHash(Default::default()),
            #[cfg(feature = "mle")]
            SetLikeObjects::MLEPPXxhasher(Default::default()),
            #[cfg(feature = "mle")]
            SetLikeObjects::MLEBetaXxhasher(Default::default()),
            #[cfg(feature = "mle")]
            SetLikeObjects::HybridMLEPPXxhasher(Default::default()),
            #[cfg(feature = "mle")]
            SetLikeObjects::HybridMLEBetaXxhasher(Default::default()),
            SetLikeObjects::HybridPPWyHash(Default::default()),
            SetLikeObjects::HybridPPXxhasher(Default::default()),
            SetLikeObjects::HybridBetaWyHash(Default::default()),
            SetLikeObjects::HybridBetaXxhasher(Default::default()),
        ]
    }
}

impl<const EXPONENT: usize, P: Precision> MemSize for SetLikeObjects<EXPONENT, P>
where
    P: ArrayRegister<Bits8> + ArrayRegister<Bits6>,
    P: MemSize,
    <P as ArrayRegister<Bits8>>::ArrayRegister: MemSize,
    <P as ArrayRegister<Bits6>>::ArrayRegister: MemSize,
    P::NumberOfRegisters: MemSize,
{
    fn mem_size(&self, flags: SizeFlags) -> usize {
        match self {
            SetLikeObjects::HashSet(set) => set.mem_size(flags),
            SetLikeObjects::TabacHyperLogLogPlus(set) => set.mem_size(flags),
            SetLikeObjects::TabacHyperLogLogPF(set) => set.mem_size(flags),
            SetLikeObjects::SAHyperLogLog(set) => set.mem_size(flags),
            SetLikeObjects::RustHyperLogLog(set) => set.mem_size(flags),
            SetLikeObjects::CardinalityEstimator(set) => set.mem_size(flags),
            SetLikeObjects::HLL6Xxhasher(set) => set.mem_size(flags),
            SetLikeObjects::HLL6WyHash(set) => set.mem_size(flags),
            SetLikeObjects::HLL8Xxhasher(set) => set.mem_size(flags),
            SetLikeObjects::HLL8WyHash(set) => set.mem_size(flags),
            SetLikeObjects::Beta6Xxhasher(set) => set.mem_size(flags),
            SetLikeObjects::Beta6WyHash(set) => set.mem_size(flags),
            SetLikeObjects::Beta8Xxhasher(set) => set.mem_size(flags),
            SetLikeObjects::Beta8WyHash(set) => set.mem_size(flags),
            #[cfg(feature = "mle")]
            SetLikeObjects::MLEPPWyHash(set) => set.mem_size(flags),
            #[cfg(feature = "mle")]
            SetLikeObjects::MLEPPXxhasher(set) => set.mem_size(flags),
            #[cfg(feature = "mle")]
            SetLikeObjects::MLEBetaWyHash(set) => set.mem_size(flags),
            #[cfg(feature = "mle")]
            SetLikeObjects::MLEBetaXxhasher(set) => set.mem_size(flags),
            #[cfg(feature = "mle")]
            SetLikeObjects::HybridMLEPPWyHash(set) => set.mem_size(flags),
            #[cfg(feature = "mle")]
            SetLikeObjects::HybridMLEPPXxhasher(set) => set.mem_size(flags),
            #[cfg(feature = "mle")]
            SetLikeObjects::HybridMLEBetaWyHash(set) => set.mem_size(flags),
            #[cfg(feature = "mle")]
            SetLikeObjects::HybridMLEBetaXxhasher(set) => set.mem_size(flags),
            SetLikeObjects::HybridPPWyHash(set) => set.mem_size(flags),
            SetLikeObjects::HybridPPXxhasher(set) => set.mem_size(flags),
            SetLikeObjects::HybridBetaWyHash(set) => set.mem_size(flags),
            SetLikeObjects::HybridBetaXxhasher(set) => set.mem_size(flags),
        }
    }
}

fn standard_deviation(values: &[f64], mean: f64) -> f64 {
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn mean_usize(values: &[usize]) -> f64 {
    values.iter().sum::<usize>() as f64 / values.len() as f64
}

/// Transposes a provided vector of vectors.
pub fn transpose<T: Copy + Default>(vec: &Vec<Vec<T>>) -> Vec<Vec<T>> {
    let mut transposed_vector = vec![vec![T::default(); vec.len()]; vec[0].len()];

    let progress_bar = indicatif::ProgressBar::new(vec.len() as u64);
    progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("Transposing: [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
            .unwrap()
            .progress_chars("##-"),
    );

    for i in (0..vec.len()).progress_with(progress_bar) {
        for j in 0..vec[i].len() {
            transposed_vector[j][i] = vec[i][j];
        }
    }

    transposed_vector
}

fn write_csv<P: Precision, T: Display + Default + Copy>(
    transposed_data: &Vec<Vec<T>>,
    feature_name: &str,
    data_name: &str,
    approach_names: &[&str],
) {
    let mut writer = csv::Writer::from_path(&format!(
        "./statistical_tests_reports/{feature_name}_{data_name}_{exponent}.csv",
        feature_name = feature_name,
        data_name = data_name,
        exponent = P::EXPONENT
    ))
    .unwrap();

    writer.write_record(approach_names.iter().copied()).unwrap();

    let progress_bar = indicatif::ProgressBar::new(transposed_data.len() as u64);

    progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("Writing CSV: [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
            .unwrap()
            .progress_chars("##-"),
    );

    for row in transposed_data.iter().progress_with(progress_bar) {
        assert_eq!(row.len(), approach_names.len());
        let row: Vec<String> = row.iter().map(|v| v.to_string()).collect();
        writer.write_record(row).unwrap();
    }

    writer.flush().unwrap();
}

pub fn statistical_report<P: Precision>(
    approach_names: &[&str],
    features: Vec<f64>,
    transposed_absolute_errors: Vec<Vec<f64>>,
    transposed_memory_requirements: Vec<Vec<usize>>,
    feature_name: &str,
) {
    assert_eq!(transposed_absolute_errors[0].len(), approach_names.len());
    assert_eq!(
        transposed_memory_requirements[0].len(),
        approach_names.len()
    );

    // We write out to three CSVs the features, absolute errors and memory requirements
    // with as header the approach names.
    let transposed_features = transpose(&vec![features]);
    write_csv::<P, f64>(&transposed_features, feature_name, "features", &["HashSet"]);
    write_csv::<P, f64>(
        &transposed_absolute_errors,
        feature_name,
        "absolute_errors",
        approach_names,
    );
    // write_csv::<P, usize>(&memory_requirements, feature_name, "memory_requirements", approach_names);

    let absolute_errors = transpose(&transposed_absolute_errors);
    let memory_requirements = transpose(&transposed_memory_requirements);

    // We compute the actual means
    let means: Vec<f64> = absolute_errors.iter().map(|errors| mean(errors)).collect();

    // And the standard deviations
    let stds: Vec<f64> = absolute_errors
        .iter()
        .zip(means.iter())
        .map(|(errors, mean)| standard_deviation(errors, *mean))
        .collect();

    // We open a CSV document where to store the results of the test.
    let mut writer = csv::Writer::from_path(&format!(
        "./statistical_tests_reports/{feature_name}_{}.csv",
        P::EXPONENT
    ))
    .unwrap();

    // We write the header of the CSV document.
    writer
        .write_record(&[
            "feature",
            "first_approach",
            "second_approach",
            "p-value",
            "winner",
            "first_memsize",
            "first_mean",
            "first_std",
            "second_memsize",
            "second_mean",
            "second_std",
            "precision",
            "theoretical_error",
        ])
        .unwrap();

    let progress_bar = indicatif::ProgressBar::new(approach_names.len() as u64);

    progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("Running tests: [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
            .unwrap()
            .progress_chars("##-"),
    );

    for (
        i,
        ((first_approach_name, first_memsize), (first_absolute_errors, (first_mean, first_std))),
    ) in approach_names
        .iter()
        .zip(memory_requirements.iter())
        .zip(absolute_errors.iter().zip(means.iter().zip(stds.iter())))
        .enumerate()
        .progress_with(progress_bar)
    {
        for (
            j,
            (
                (second_approach_name, second_memsize),
                (second_absolute_errors, (second_mean, second_std)),
            ),
        ) in approach_names
            .iter()
            .zip(memory_requirements.iter())
            .zip(absolute_errors.iter().zip(means.iter().zip(stds.iter())))
            .enumerate()
        {
            if i >= j {
                continue;
            }

            let w_test = WilcoxonWTest::paired(first_absolute_errors, second_absolute_errors);

            writer
                .write_record(&[
                    feature_name,
                    first_approach_name,
                    second_approach_name,
                    w_test
                        .as_ref()
                        .map(|w_test| format!("{:.5}", w_test.p_value()))
                        .unwrap_or("Unknown".to_owned())
                        .as_str(),
                    if let Ok(w_test) = w_test.as_ref() {
                        if w_test.p_value() < 0.05 {
                            if first_mean < second_mean {
                                "First"
                            } else {
                                "Second"
                            }
                        } else {
                            "None"
                        }
                    } else {
                        "Unknown"
                    },
                    format!("{}", mean_usize(first_memsize)).as_str(),
                    format!("{}", first_mean).as_str(),
                    format!("{}", first_std).as_str(),
                    format!("{}", mean_usize(second_memsize)).as_str(),
                    format!("{}", second_mean).as_str(),
                    format!("{}", second_std).as_str(),
                    format!("{}", P::EXPONENT).as_str(),
                    format!("{}", P::error_rate()).as_str(),
                ])
                .unwrap();
        }
    }
    // We close the CSV document.
    writer.flush().unwrap();
}
