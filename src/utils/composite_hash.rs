//! Submodule providing the composite hash trait, which is a type of hash
//! that can operate as a variable word and is composable from a hash and
//! a register value and is symmetrically splittable back into a hash and
//! a register value.

use super::{PositiveInteger, VariableWord};
use crate::prelude::{Bits, Precision};

/// Trait for a composite hash.
pub trait CompositeHash: VariableWord {
    /// Encode the hash from the provided register value, index and the original unsplitted hash.
    fn encode<P: Precision, B: Bits>(
        register: u8,
        index: P::NumberOfRegisters,
        hash: u64,
    ) -> Self::Word;

    /// Decode the hash into the register value and index.
    fn decode<P: Precision, B: Bits>(hash: Self::Word) -> (u8, P::NumberOfRegisters);
}

impl CompositeHash for u32 {
    fn encode<P: Precision, B: Bits>(
        register: u8,
        index: P::NumberOfRegisters,
        hash: u64,
    ) -> Self::Word {
        assert!(P::EXPONENT + B::NUMBER_OF_BITS <= 32);
        debug_assert!(index.to_usize() < (1 << P::EXPONENT));   
        debug_assert!(register > 0);

        // Since the index is extracted from the low part of the hash, we censor the low part of the hash.
        let mut hash = u32::try_from(hash >> 32).unwrap();
        // Next, we censor the B leftmost bits of the hash, and we replace them with the register.
        hash <<= B::NUMBER_OF_BITS;
        hash >>= B::NUMBER_OF_BITS;
        hash |= (register as u32) << (32 - B::NUMBER_OF_BITS);
        // Similarly, we censor the rightmost bits of the hash, and we replace them with the index.
        hash >>= P::EXPONENT;
        hash <<= P::EXPONENT;
        hash |= u32::try_from(index.to_usize()).unwrap();

        hash
    }

    fn decode<P: Precision, B: Bits>(hash: Self::Word) -> (u8, P::NumberOfRegisters) {
        assert!(P::EXPONENT + B::NUMBER_OF_BITS <= 32);
        // We extract the index from the rightmost bits of the hash.
        let index =
            P::NumberOfRegisters::try_from_u64(u64::from(hash & ((1 << P::EXPONENT) - 1))).unwrap();
        // Next, we extract the register from the leftmost bits of the hash.
        let register = u8::try_from(hash >> (32 - B::NUMBER_OF_BITS)).unwrap();

        (register, index)
    }
}

impl CompositeHash for u64 {
    fn encode<P: Precision, B: Bits>(
        register: u8,
        index: P::NumberOfRegisters,
        mut hash: u64,
    ) -> Self::Word {
        assert!(P::EXPONENT + B::NUMBER_OF_BITS <= 64);
        debug_assert!(index.to_usize() < (1 << P::EXPONENT));   
        debug_assert!(register > 0);

        // Next, we censor the B leftmost bits of the hash, and we replace them with the register.
        hash <<= B::NUMBER_OF_BITS;
        hash >>= B::NUMBER_OF_BITS;
        hash |= (register as u64) << (64 - B::NUMBER_OF_BITS);
        // Similarly, we censor the rightmost bits of the hash, and we replace them with the index.
        hash >>= P::EXPONENT;
        hash <<= P::EXPONENT;
        hash |= u64::try_from(index.to_usize()).unwrap();

        hash
    }

    fn decode<P: Precision, B: Bits>(hash: Self::Word) -> (u8, P::NumberOfRegisters) {
        assert!(P::EXPONENT + B::NUMBER_OF_BITS <= 64);
        // We extract the index from the rightmost bits of the hash.
        let index =
            P::NumberOfRegisters::try_from_u64(u64::from(hash & ((1 << P::EXPONENT) - 1))).unwrap();
        // Next, we extract the register from the leftmost bits of the hash.
        let register = u8::try_from(hash >> (64 - B::NUMBER_OF_BITS)).unwrap();

        (register, index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::hash::Hash;
    use core::hash::Hasher;
    use crate::prelude::*;
    use crate::{
        prelude::{Bits, Precision},
        utils::{iter_random_values, splitmix64},
    };

    fn test_composite_hash<H: CompositeHash, P: Precision, B: Bits>() {
        let maximal_register = (1 << B::NUMBER_OF_BITS) - 1;
        let maximal_index = (1 << P::EXPONENT) - 1;

        let mut random_state1 = 56454468575749_u64;
        let random_state2 = 23457874468575749_u64;

        for (register, index) in iter_random_values(1000, Some(maximal_register), random_state1)
            .zip(iter_random_values(1000, Some(maximal_index), random_state2))
        {
            random_state1 = splitmix64(random_state1);

            let mut hasher = twox_hash::XxHash64::default();
            random_state1.hash(&mut hasher);
            let hash = hasher.finish();

            let register: u8 = u8::try_from(register).unwrap().max(1);
            let index = P::NumberOfRegisters::try_from_u64(index).unwrap();

            let encoded = H::encode::<P, B>(register, index, hash);
            let (decoded_register, decoded_index) = H::decode::<P, B>(encoded);

            assert_eq!(register, decoded_register);
            assert_eq!(index, decoded_index);
        }
    }

    /// Macro to generate test_composite_hash tests for a given composite hash, precision and bits.
    macro_rules! test_composite_hash {
        ($hash:ty, $exponent:expr, $($bits:ty),*) => {
            $(
                paste::paste! {
                    #[test]
                    #[cfg(feature = "precision_" $exponent)]
                    fn [<test_composite_hash_ $hash:lower _precision_ $exponent _ $bits:lower>]() {
                        test_composite_hash::<$hash, [<Precision $exponent>], $bits>();
                    }
                }
            )*
        };
    }

    /// Macro to generate test_composite_hash tests for a given composite hash and precisions.
    macro_rules! test_composite_hash_precisions {
        ($hash:ty, $($precision:expr),*) => {
            $(
                test_composite_hash!($hash, $precision, Bits1, Bits2, Bits3, Bits4, Bits5, Bits6, Bits7, Bits8);
            )*
        };
    }

    test_composite_hash_precisions!(u32, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);
    test_composite_hash_precisions!(u64, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);
}
