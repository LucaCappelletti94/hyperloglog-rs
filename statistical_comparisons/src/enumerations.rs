//! This module contains the enumerations used in the benchmarks.

use crate::proxy_implementations::{
    AlecHLL, CloudFlareHLL, HyperThreeBits, HyperTwoBits, RustHLL, SimpleHLL, TabacHLL,
    TabacHLLPlusPlus,
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
pub enum HyperTwoVariants {
    H2BM64(HyperTwoBits<M64H2B>),
    H2BM128(HyperTwoBits<M128H2B>),
    H2BM256(HyperTwoBits<M256H2B>),
    H2BM512(HyperTwoBits<M512H2B>),
    H2BM1024(HyperTwoBits<M1024H2B>),
    H2BM2048(HyperTwoBits<M2048H2B>),
    H2BM4096(HyperTwoBits<M4096H2B>),
    H3BM64(HyperThreeBits<M64H3B>),
    H3BM128(HyperThreeBits<M128H3B>),
    H3BM256(HyperThreeBits<M256H3B>),
    H3BM512(HyperThreeBits<M512H3B>),
    H3BM1024(HyperThreeBits<M1024H3B>),
    H3BM2048(HyperThreeBits<M2048H3B>),
    H3BM4096(HyperThreeBits<M4096H3B>),
}

#[allow(missing_docs)]
#[expect(
    clippy::type_complexity,
    reason = "The type is complex due to the use of generics, but these are benchmarks and is to be expected."
)]
#[derive(Clone, Named, ExtendableApproximatedSet, Estimator, TransparentMemSize, EnumIter)]
/// Enumerations will all `HyperLogLog` variants we
/// take into consideration for the benchmarks.
pub enum HLLVariants<const EXPONENT: usize, P: Precision>
where
    P: AllArrays + Named,
{
    TabacHyperLogLogPlus(TabacHLLPlusPlus<P>),
    TabacHyperLogLogPF(TabacHLL<P>),
    SAHyperLogLog(AlecHLL<P>),
    RustHyperLogLog(RustHLL<P>),
    CE4(CloudFlareHLL<EXPONENT, 4, wyhash::WyHash>),
    CE5(CloudFlareHLL<EXPONENT, 5, wyhash::WyHash>),
    CE6(CloudFlareHLL<EXPONENT, 6, wyhash::WyHash>),
    SimpleHLL(SimpleHLL<EXPONENT>),
    PP4ArrayXxhasher(
        PlusPlus<P, Bits4, <P as ArrayRegister<Bits4>>::Array, twox_hash::XxHash64>,
    ),
    PP4ArrayWyHash(PlusPlus<P, Bits4, <P as ArrayRegister<Bits4>>::Array, wyhash::WyHash>),
    PP5ArrayXxhasher(
        PlusPlus<P, Bits5, <P as ArrayRegister<Bits5>>::Array, twox_hash::XxHash64>,
    ),
    PP5ArrayWyHash(PlusPlus<P, Bits5, <P as ArrayRegister<Bits5>>::Array, wyhash::WyHash>),
    PP6ArrayXxhasher(
        PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::Array, twox_hash::XxHash64>,
    ),
    PP6ArrayWyHash(PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::Array, wyhash::WyHash>),
    PP4PackedXxhasher(
        PlusPlus<
            P,
            Bits4,
            <P as ArrayRegister<Bits4>>::Packed,
            twox_hash::XxHash64,
        >,
    ),
    PP4PackedWyHash(
        PlusPlus<P, Bits4, <P as ArrayRegister<Bits4>>::Packed, wyhash::WyHash>,
    ),
    PP5PackedXxhasher(
        PlusPlus<
            P,
            Bits5,
            <P as ArrayRegister<Bits5>>::Packed,
            twox_hash::XxHash64,
        >,
    ),
    PP5PackedWyHash(
        PlusPlus<P, Bits5, <P as ArrayRegister<Bits5>>::Packed, wyhash::WyHash>,
    ),
    PP6PackedXxhasher(
        PlusPlus<
            P,
            Bits6,
            <P as ArrayRegister<Bits6>>::Packed,
            twox_hash::XxHash64,
        >,
    ),
    PP6PackedWyHash(
        PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::Packed, wyhash::WyHash>,
    ),
    LLB4ArrayXxhasher(
        LogLogBeta<P, Bits4, <P as ArrayRegister<Bits4>>::Array, twox_hash::XxHash64>,
    ),
    LLB4ArrayWyHash(
        LogLogBeta<P, Bits4, <P as ArrayRegister<Bits4>>::Array, wyhash::WyHash>,
    ),
    LLB5ArrayXxhasher(
        LogLogBeta<P, Bits5, <P as ArrayRegister<Bits5>>::Array, twox_hash::XxHash64>,
    ),
    LLB5ArrayWyHash(
        LogLogBeta<P, Bits5, <P as ArrayRegister<Bits5>>::Array, wyhash::WyHash>,
    ),
    LLB6ArrayXxhasher(
        LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::Array, twox_hash::XxHash64>,
    ),
    LLB6ArrayWyHash(
        LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::Array, wyhash::WyHash>,
    ),
    LLB4PackedWyHash(
        LogLogBeta<
            P,
            Bits4,
            <P as ArrayRegister<Bits4>>::Packed,
            wyhash::WyHash,
        >,
    ),
    LLB5PackedXxhasher(
        LogLogBeta<
            P,
            Bits5,
            <P as ArrayRegister<Bits5>>::Packed,
            twox_hash::XxHash64,
        >,
    ),
    LLB5PackedWyHash(
        LogLogBeta<
            P,
            Bits5,
            <P as ArrayRegister<Bits5>>::Packed,
            wyhash::WyHash,
        >,
    ),
    LLB6PackedXxhasher(
        LogLogBeta<
            P,
            Bits6,
            <P as ArrayRegister<Bits6>>::Packed,
            twox_hash::XxHash64,
        >,
    ),
    LLB6PackedWyHash(
        LogLogBeta<
            P,
            Bits6,
            <P as ArrayRegister<Bits6>>::Packed,
            wyhash::WyHash,
        >,
    ),
    MLEPP4WyHash(
        MLE<PlusPlus<P, Bits4, <P as ArrayRegister<Bits4>>::Array, wyhash::WyHash>>,
    ),
    MLEPP5WyHash(
        MLE<PlusPlus<P, Bits5, <P as ArrayRegister<Bits5>>::Array, wyhash::WyHash>>,
    ),
    MLEPP6WyHash(
        MLE<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::Array, wyhash::WyHash>>,
    ),
    MLEPP4Xxhasher(
        MLE<PlusPlus<P, Bits4, <P as ArrayRegister<Bits4>>::Array, twox_hash::XxHash64>>,
    ),
    MLEPP5Xxhasher(
        MLE<PlusPlus<P, Bits5, <P as ArrayRegister<Bits5>>::Array, twox_hash::XxHash64>>,
    ),
    MLEPP6Xxhasher(
        MLE<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::Array, twox_hash::XxHash64>>,
    ),
    MLELLB4WyHash(
        MLE<LogLogBeta<P, Bits4, <P as ArrayRegister<Bits4>>::Array, wyhash::WyHash>>,
    ),
    MLELLB5WyHash(
        MLE<LogLogBeta<P, Bits5, <P as ArrayRegister<Bits5>>::Array, wyhash::WyHash>>,
    ),
    MLELLB6WyHash(
        MLE<LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::Array, wyhash::WyHash>>,
    ),
    MLELLB4Xxhasher(
        MLE<LogLogBeta<P, Bits4, <P as ArrayRegister<Bits4>>::Array, twox_hash::XxHash64>>,
    ),
    MLELLB5Xxhasher(
        MLE<LogLogBeta<P, Bits5, <P as ArrayRegister<Bits5>>::Array, twox_hash::XxHash64>>,
    ),
    MLELLB6Xxhasher(
        MLE<LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::Array, twox_hash::XxHash64>>,
    ),

    HybridPP4ArrayXxhasher(
        Hybrid<PlusPlus<P, Bits4, <P as ArrayRegister<Bits4>>::Array, twox_hash::XxHash64>>,
    ),
    HybridPP4ArrayWyHash(
        Hybrid<PlusPlus<P, Bits4, <P as ArrayRegister<Bits4>>::Array, wyhash::WyHash>>,
    ),
    HybridPP5ArrayXxhasher(
        Hybrid<PlusPlus<P, Bits5, <P as ArrayRegister<Bits5>>::Array, twox_hash::XxHash64>>,
    ),
    HybridPP5ArrayWyHash(
        Hybrid<PlusPlus<P, Bits5, <P as ArrayRegister<Bits5>>::Array, wyhash::WyHash>>,
    ),
    HybridPP6ArrayXxhasher(
        Hybrid<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::Array, twox_hash::XxHash64>>,
    ),
    HybridPP6ArrayWyHash(
        Hybrid<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::Array, wyhash::WyHash>>,
    ),
    HybridPP4PackedXxhasher(
        Hybrid<
            PlusPlus<
                P,
                Bits4,
                <P as ArrayRegister<Bits4>>::Packed,
                twox_hash::XxHash64,
            >,
        >,
    ),
    HybridPP4PackedWyHash(
        Hybrid<
            PlusPlus<
                P,
                Bits4,
                <P as ArrayRegister<Bits4>>::Packed,
                wyhash::WyHash,
            >,
        >,
    ),
    HybridPP5PackedXxhasher(
        Hybrid<
            PlusPlus<
                P,
                Bits5,
                <P as ArrayRegister<Bits5>>::Packed,
                twox_hash::XxHash64,
            >,
        >,
    ),
    HybridPP5PackedWyHash(
        Hybrid<
            PlusPlus<
                P,
                Bits5,
                <P as ArrayRegister<Bits5>>::Packed,
                wyhash::WyHash,
            >,
        >,
    ),
    HybridPP6PackedXxhasher(
        Hybrid<
            PlusPlus<
                P,
                Bits6,
                <P as ArrayRegister<Bits6>>::Packed,
                twox_hash::XxHash64,
            >,
        >,
    ),
    HybridPP6PackedWyHash(
        Hybrid<
            PlusPlus<
                P,
                Bits6,
                <P as ArrayRegister<Bits6>>::Packed,
                wyhash::WyHash,
            >,
        >,
    ),
    HybridLLB4ArrayXxhasher(
        Hybrid<
            LogLogBeta<P, Bits4, <P as ArrayRegister<Bits4>>::Array, twox_hash::XxHash64>,
        >,
    ),
    HybridLLB4ArrayWyHash(
        Hybrid<LogLogBeta<P, Bits4, <P as ArrayRegister<Bits4>>::Array, wyhash::WyHash>>,
    ),
    HybridLLB5ArrayXxhasher(
        Hybrid<
            LogLogBeta<P, Bits5, <P as ArrayRegister<Bits5>>::Array, twox_hash::XxHash64>,
        >,
    ),
    HybridLLB5ArrayWyHash(
        Hybrid<LogLogBeta<P, Bits5, <P as ArrayRegister<Bits5>>::Array, wyhash::WyHash>>,
    ),
    HybridLLB6ArrayXxhasher(
        Hybrid<
            LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::Array, twox_hash::XxHash64>,
        >,
    ),
    HybridLLB6ArrayWyHash(
        Hybrid<LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::Array, wyhash::WyHash>>,
    ),
    HybridLLB4PackedWyHash(
        Hybrid<
            LogLogBeta<
                P,
                Bits4,
                <P as ArrayRegister<Bits4>>::Packed,
                wyhash::WyHash,
            >,
        >,
    ),
    HybridLLB5PackedXxhasher(
        Hybrid<
            LogLogBeta<
                P,
                Bits5,
                <P as ArrayRegister<Bits5>>::Packed,
                twox_hash::XxHash64,
            >,
        >,
    ),
    HybridLLB5PackedWyHash(
        Hybrid<
            LogLogBeta<
                P,
                Bits5,
                <P as ArrayRegister<Bits5>>::Packed,
                wyhash::WyHash,
            >,
        >,
    ),
    HybridLLB6PackedXxhasher(
        Hybrid<
            LogLogBeta<
                P,
                Bits6,
                <P as ArrayRegister<Bits6>>::Packed,
                twox_hash::XxHash64,
            >,
        >,
    ),
    HybridLLB6PackedWyHash(
        Hybrid<
            LogLogBeta<
                P,
                Bits6,
                <P as ArrayRegister<Bits6>>::Packed,
                wyhash::WyHash,
            >,
        >,
    ),
    HybridMLEPP4WyHash(
        Hybrid<MLE<PlusPlus<P, Bits4, <P as ArrayRegister<Bits4>>::Array, wyhash::WyHash>>>,
    ),
    HybridMLEPP5WyHash(
        Hybrid<MLE<PlusPlus<P, Bits5, <P as ArrayRegister<Bits5>>::Array, wyhash::WyHash>>>,
    ),
    HybridMLEPP6WyHash(
        Hybrid<MLE<PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::Array, wyhash::WyHash>>>,
    ),
    HybridMLEPP4Xxhasher(
        Hybrid<
            MLE<
                PlusPlus<P, Bits4, <P as ArrayRegister<Bits4>>::Array, twox_hash::XxHash64>,
            >,
        >,
    ),
    HybridMLEPP5Xxhasher(
        Hybrid<
            MLE<
                PlusPlus<P, Bits5, <P as ArrayRegister<Bits5>>::Array, twox_hash::XxHash64>,
            >,
        >,
    ),
    HybridMLEPP6Xxhasher(
        Hybrid<
            MLE<
                PlusPlus<P, Bits6, <P as ArrayRegister<Bits6>>::Array, twox_hash::XxHash64>,
            >,
        >,
    ),
    HybridMLELLB4WyHash(
        Hybrid<
            MLE<LogLogBeta<P, Bits4, <P as ArrayRegister<Bits4>>::Array, wyhash::WyHash>>,
        >,
    ),
    HybridMLELLB5WyHash(
        Hybrid<
            MLE<LogLogBeta<P, Bits5, <P as ArrayRegister<Bits5>>::Array, wyhash::WyHash>>,
        >,
    ),
    HybridMLELLB6WyHash(
        Hybrid<
            MLE<LogLogBeta<P, Bits6, <P as ArrayRegister<Bits6>>::Array, wyhash::WyHash>>,
        >,
    ),
    HybridMLELLB4Xxhasher(
        Hybrid<
            MLE<
                LogLogBeta<
                    P,
                    Bits4,
                    <P as ArrayRegister<Bits4>>::Array,
                    twox_hash::XxHash64,
                >,
            >,
        >,
    ),
    HybridMLELLB5Xxhasher(
        Hybrid<
            MLE<
                LogLogBeta<
                    P,
                    Bits5,
                    <P as ArrayRegister<Bits5>>::Array,
                    twox_hash::XxHash64,
                >,
            >,
        >,
    ),
    HybridMLELLB6Xxhasher(
        Hybrid<
            MLE<
                LogLogBeta<
                    P,
                    Bits6,
                    <P as ArrayRegister<Bits6>>::Array,
                    twox_hash::XxHash64,
                >,
            >,
        >,
    ),
}
