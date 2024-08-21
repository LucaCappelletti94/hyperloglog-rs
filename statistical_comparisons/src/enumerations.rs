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
pub enum HLLVariants<const EXPONENT: usize, P: Precision, H: HasherBuilderAssociated>
where
    P: AllArrays + Named,
    <P as ArrayRegister<Bits4>>::Array: VariableWords<u32>,
    <P as ArrayRegister<Bits5>>::Array: VariableWords<u32>,
    <P as ArrayRegister<Bits6>>::Array: VariableWords<u32>,
    <P as ArrayRegister<Bits4>>::Packed: VariableWords<u32>,
    <P as ArrayRegister<Bits5>>::Packed: VariableWords<u32>,
    <P as ArrayRegister<Bits6>>::Packed: VariableWords<u32>,
{
    TabacHyperLogLogPlus(TabacHLLPlusPlus<P, H>),
    TabacHyperLogLogPF(TabacHLL<P, H>),
    SAHyperLogLog(AlecHLL<P>),
    RustHyperLogLog(RustHLL<P>),
    CE4(CloudFlareHLL<EXPONENT, 4, H>),
    CE5(CloudFlareHLL<EXPONENT, 5, H>),
    CE6(CloudFlareHLL<EXPONENT, 6, H>),
    SimpleHLL(SimpleHLL<EXPONENT>),
    PP4ArrayXxhasher(PlusPlus<P, Bits4, <P as ArrayRegister<Bits4>>::Array, H>),
    PP5ArrayXxhasher(PlusPlus<P, Bits5, <P as ArrayRegister<Bits5>>::Array, H>),
    PP6ArrayXxhasher(PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::Array, H>),
    PP4PackedXxhasher(PlusPlus<P, Bits4, <P as ArrayRegister<Bits4>>::Packed, H>),
    PP5PackedXxhasher(PlusPlus<P, Bits5, <P as ArrayRegister<Bits5>>::Packed, H>),
    PP6PackedXxhasher(PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::Packed, H>),
    LLB4ArrayXxhasher(LogLogBeta<P, Bits4, <P as ArrayRegister<Bits4>>::Array, H>),
    LLB5ArrayXxhasher(LogLogBeta<P, Bits5, <P as ArrayRegister<Bits5>>::Array, H>),
    LLB6ArrayXxhasher(LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::Array, H>),
    LLB5PackedXxhasher(LogLogBeta<P, Bits5, <P as ArrayRegister<Bits5>>::Packed, H>),
    LLB6PackedXxhasher(LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::Packed, H>),
    MLEPP4Xxhasher(MLE<PlusPlus<P, Bits4, <P as ArrayRegister<Bits4>>::Array, H>>),
    MLEPP5Xxhasher(MLE<PlusPlus<P, Bits5, <P as ArrayRegister<Bits5>>::Array, H>>),
    MLEPP6Xxhasher(MLE<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::Array, H>>),
    MLELLB4Xxhasher(MLE<LogLogBeta<P, Bits4, <P as ArrayRegister<Bits4>>::Array, H>>),
    MLELLB5Xxhasher(MLE<LogLogBeta<P, Bits5, <P as ArrayRegister<Bits5>>::Array, H>>),
    MLELLB6Xxhasher(MLE<LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::Array, H>>),
    HybridPP4ArrayXxhasher(Hybrid<PlusPlus<P, Bits4, <P as ArrayRegister<Bits4>>::Array, H>>),
    HybridPP5ArrayXxhasher(Hybrid<PlusPlus<P, Bits5, <P as ArrayRegister<Bits5>>::Array, H>>),
    HybridPP6ArrayXxhasher(Hybrid<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::Array, H>>),
    HybridPP4PackedXxhasher(Hybrid<PlusPlus<P, Bits4, <P as ArrayRegister<Bits4>>::Packed, H>>),
    HybridPP5PackedXxhasher(Hybrid<PlusPlus<P, Bits5, <P as ArrayRegister<Bits5>>::Packed, H>>),
    HybridPP6PackedXxhasher(Hybrid<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::Packed, H>>),
    HybridLLB4ArrayXxhasher(Hybrid<LogLogBeta<P, Bits4, <P as ArrayRegister<Bits4>>::Array, H>>),
    HybridLLB5ArrayXxhasher(Hybrid<LogLogBeta<P, Bits5, <P as ArrayRegister<Bits5>>::Array, H>>),
    HybridLLB6ArrayXxhasher(Hybrid<LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::Array, H>>),
    HybridLLB5PackedXxhasher(Hybrid<LogLogBeta<P, Bits5, <P as ArrayRegister<Bits5>>::Packed, H>>),
    HybridLLB6PackedXxhasher(Hybrid<LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::Packed, H>>),
    HybridMLEPP4Xxhasher(Hybrid<MLE<PlusPlus<P, Bits4, <P as ArrayRegister<Bits4>>::Array, H>>>),
    HybridMLEPP5Xxhasher(Hybrid<MLE<PlusPlus<P, Bits5, <P as ArrayRegister<Bits5>>::Array, H>>>),
    HybridMLEPP6Xxhasher(Hybrid<MLE<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::Array, H>>>),
    HybridMLELLB4Xxhasher(Hybrid<MLE<LogLogBeta<P, Bits4, <P as ArrayRegister<Bits4>>::Array, H>>>),
    HybridMLELLB5Xxhasher(Hybrid<MLE<LogLogBeta<P, Bits5, <P as ArrayRegister<Bits5>>::Array, H>>>),
    HybridMLELLB6Xxhasher(Hybrid<MLE<LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::Array, H>>>),
}
