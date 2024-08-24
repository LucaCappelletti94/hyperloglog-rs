//! This module contains the enumerations used in the benchmarks.

use crate::proxy_implementations::{
    AlecHLL, CloudFlareHLL, HasherBuilderAssociated, HyperThreeBits, HyperTwoBits, RustHLL,
    SimpleHLL, TabacHLL, TabacHLLPlusPlus,
};
use crate::traits::TransparentMemSize;
use hyperloglog_rs::prelude::*;
use hypertwobits::h2b::{
    M1024 as M1024H2B, M128 as M128H2B, M2048 as M2048H2B, M256 as M256H2B, M4096 as M4096H2B,
    M512 as M512H2B, M64 as M64H2B,
};
use hypertwobits::h3b::{
    M1024 as M1024H3B, M128 as M128H3B, M2048 as M2048H3B, M256 as M256H3B, M4096 as M4096H3B,
    M512 as M512H3B, M64 as M64H3B,
};
use macro_test_utils::{Estimator, ExtendableApproximatedSet, Named, TransparentMemSize};
use mem_dbg::MemSize;
use strum_macros::EnumIter;

#[allow(missing_docs)]
#[expect(
    clippy::large_enum_variant,
    reason = "The enum is large due to the use of generics, but these are benchmarks and is to be expected."
)]
#[derive(Clone, Named, ExtendableApproximatedSet, Estimator, TransparentMemSize, EnumIter)]
/// Enumerations will all `HyperTwo` variants we
/// take into consideration for the benchmarks.
pub enum HyperTwoVariants<H: HasherBuilderAssociated> {
    H2BM64(HyperTwoBits<M64H2B, H>),
    H2BM128(HyperTwoBits<M128H2B, H>),
    H2BM256(HyperTwoBits<M256H2B, H>),
    H2BM512(HyperTwoBits<M512H2B, H>),
    H2BM1024(HyperTwoBits<M1024H2B, H>),
    H2BM2048(HyperTwoBits<M2048H2B, H>),
    H2BM4096(HyperTwoBits<M4096H2B, H>),
    H3BM64(HyperThreeBits<M64H3B, H>),
    H3BM128(HyperThreeBits<M128H3B, H>),
    H3BM256(HyperThreeBits<M256H3B, H>),
    H3BM512(HyperThreeBits<M512H3B, H>),
    H3BM1024(HyperThreeBits<M1024H3B, H>),
    H3BM2048(HyperThreeBits<M2048H3B, H>),
    H3BM4096(HyperThreeBits<M4096H3B, H>),
}

#[allow(missing_docs)]
#[expect(
    clippy::type_complexity,
    reason = "The type is complex due to the use of generics, but these are benchmarks and is to be expected."
)]
#[derive(Clone, Named, ExtendableApproximatedSet, Estimator, TransparentMemSize, EnumIter)]
/// Enumerations will all `HyperLogLog` variants we
/// take into consideration for the benchmarks.
pub enum HLLVariants<
    const EXPONENT: usize,
    P: Precision,
    H: HasherBuilderAssociated,
    const BITS: usize,
    B,
> where
    P: Named + ArrayRegister<B>,
    B: Named + Bits,
{
    TabacHyperLogLogPlus(TabacHLLPlusPlus<P, H>),
    TabacHyperLogLogPF(TabacHLL<P, H>),
    SAHyperLogLog(AlecHLL<P>),
    RustHyperLogLog(RustHLL<P>),
    CE(CloudFlareHLL<EXPONENT, BITS, H>),
    SimpleHLL(SimpleHLL<H, EXPONENT>),
    PP(PlusPlus<P, B, <P as ArrayRegister<B>>::Packed, H>),
    LLB(LogLogBeta<P, B, <P as ArrayRegister<B>>::Packed, H>),
    MLEPP(MLE<PlusPlus<P, B, <P as ArrayRegister<B>>::Packed, H>>),
    MLELLB(MLE<LogLogBeta<P, B, <P as ArrayRegister<B>>::Packed, H>>),
    HybridPP(Hybrid<PlusPlus<P, B, <P as ArrayRegister<B>>::Packed, H>>),
    HybridLLB(Hybrid<LogLogBeta<P, B, <P as ArrayRegister<B>>::Packed, H>>),
    HybridMLEPP(Hybrid<MLE<PlusPlus<P, B, <P as ArrayRegister<B>>::Packed, H>>>),
    HybridMLELLB(Hybrid<MLE<LogLogBeta<P, B, <P as ArrayRegister<B>>::Packed, H>>>),
}
