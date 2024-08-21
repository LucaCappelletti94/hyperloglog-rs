//! Submodule providing the composite hash trait, which is a type of hash
//! that can operate as a variable word and is composable from a hash and
//! a register value and is symmetrically splittable back into a hash and
//! a register value.

use crate::prelude::*;

/// Trait for a composite hash.
pub trait CompositeHash<P: Precision, B: Bits>: VariableWord {
    /// The number of bits in the composite hash.
    const OFFSET: u8 = Self::NUMBER_OF_BITS - B::NUMBER_OF_BITS - P::EXPONENT;
    /// The mask for to only keep the bits used for the padding.
    const PADDING_MASK: u64 = (1 << Self::OFFSET) - 1;

    #[allow(unsafe_code)]
    /// Encode the hash from the provided register value, index and the original unsplitted hash.
    ///
    /// # Arguments
    /// * `register` - The register value to be encoded.
    /// * `hash` - The original hash to be encoded.
    ///
    /// # Implementation
    /// The hash we receive is expected to be in the following form:
    ///
    /// ```text
    /// | bits used for the leading zeros count | potentially unused bits | bits used for the index |
    /// ```
    ///
    /// We need to ensure that the higher bits are the bits of the index, as we will
    /// sort the hashes and the index needs to be the primary sorting key. Next, we
    /// want to sort by the number of leading zeros, followed by any eventual unused bits.
    /// The resulting hash therefore, will be in the following form:
    ///
    /// ```text
    /// | bits used for the index | number of leading zeros | potentially unused bits |
    /// ```
    fn encode(register: u8, index: P::NumberOfRegisters, hash: u64) -> Self::Word {
        debug_assert!(register > 0);
        debug_assert!(u64::from(register) <= B::MASK);
        debug_assert!(index < P::NUMBER_OF_REGISTERS);

        // We remove the portion used for the index and apply the padding mask,
        // which ensures that now only the bits used for the padding (if any) are kept.
        let mut hash = (hash >> P::EXPONENT) & Self::PADDING_MASK;

        // Next, we place the index in the rightmost bits of the hash.
        hash |= index.into() << (Self::NUMBER_OF_BITS - P::EXPONENT);

        // Next, we place the register in the rightmost bits of the hash, minus the bits used for the index.
        hash |= u64::from(register) << Self::OFFSET;

        unsafe { Self::unchecked_from_u64(hash) }
    }

    #[allow(unsafe_code)]
    /// Decode the hash into the register value and index.
    fn decode(hash: Self::Word) -> (u8, P::NumberOfRegisters) {
        // We extract the index from the rightmost bits of the hash.
        let index = unsafe {
            P::NumberOfRegisters::unchecked_from_u64(
                (hash >> (Self::NUMBER_OF_BITS - P::EXPONENT)).into(),
            )
        };
        // Next, we extract the register from the rightmost bits of the hash, minus the bits used for the index.
        let register = unsafe { u8::unchecked_from_u64((hash >> Self::OFFSET).into() & B::MASK) };

        (register, index)
    }
}

/// Macro to implement the appropriate [`CompositeHash`] trait for a given type.
macro_rules! impl_composite_hash {
    ($w:ty, $(($exponent:expr, $bits:ty)),*) => {
        $(
            paste::paste! {
                #[cfg(feature = "precision_" $exponent)]
                impl CompositeHash<[<Precision $exponent>], $bits> for $w {}
            }
        )*
    };
}

macro_rules! impl_composite_hash_all_bits {
    ($w:ty, $($exponent:expr),*) => {
        $(
            impl_composite_hash!($w, ($exponent, Bits1), ($exponent, Bits2), ($exponent, Bits3), ($exponent, Bits4), ($exponent, Bits5), ($exponent, Bits6), ($exponent, Bits7), ($exponent, Bits8));
        )*
    };
}

impl_composite_hash!(
    u8,
    (4, Bits4),
    (4, Bits3),
    (4, Bits2),
    (4, Bits1),
    (5, Bits3),
    (5, Bits2),
    (5, Bits1),
    (6, Bits2),
    (6, Bits1),
    (7, Bits1)
);

impl_composite_hash_all_bits!(u16, 4, 5, 6, 7, 8);

impl_composite_hash!(
    u16,
    (9, Bits7),
    (9, Bits6),
    (9, Bits5),
    (9, Bits4),
    (9, Bits3),
    (9, Bits2),
    (9, Bits1),
    (10, Bits6),
    (10, Bits5),
    (10, Bits4),
    (10, Bits3),
    (10, Bits2),
    (10, Bits1),
    (11, Bits5),
    (11, Bits4),
    (11, Bits3),
    (11, Bits2),
    (11, Bits1),
    (12, Bits4),
    (12, Bits3),
    (12, Bits2),
    (12, Bits1),
    (13, Bits3),
    (13, Bits2),
    (13, Bits1),
    (14, Bits2),
    (14, Bits1),
    (15, Bits1)
);

impl_composite_hash_all_bits!(u24, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

impl_composite_hash!(
    u24,
    (17, Bits7),
    (17, Bits6),
    (17, Bits5),
    (17, Bits4),
    (17, Bits3),
    (17, Bits2),
    (17, Bits1),
    (18, Bits6),
    (18, Bits5),
    (18, Bits4),
    (18, Bits3),
    (18, Bits2),
    (18, Bits1)
);

impl<P: Precision, B: Bits> CompositeHash<P, B> for u32 {}
impl<P: Precision, B: Bits> CompositeHash<P, B> for u40 {}
impl<P: Precision, B: Bits> CompositeHash<P, B> for u48 {}
impl<P: Precision, B: Bits> CompositeHash<P, B> for u56 {}
impl<P: Precision, B: Bits> CompositeHash<P, B> for u64 {}

#[cfg(test)]
mod tests {
    use super::*;
    use hyperloglog_derive::test_all_precisions_and_bits;

    fn test_composite_hash_for_word<P: Precision, B: Bits<Word = u8>, W: CompositeHash<P, B>>() {
        let data_type = core::any::type_name::<W>();
        for ((pseudo_hash, register), index) in iter_random_values::<u64>(1_000, None, None)
            .zip(iter_random_values::<B>(1_000, None, None))
            .zip(iter_random_values::<P::NumberOfRegisters>(
                1_000,
                Some(P::NUMBER_OF_REGISTERS),
                None,
            ))
        {
            let register = register.max(1);
            let composed_hash: W::Word = W::encode(register, index, pseudo_hash);
            let (decoded_register, decoded_index) = W::decode(composed_hash);
            assert!(
                decoded_register > 0,
                "{data_type}) Failed to decode register - register must be greater than 0"
            );
            assert!(
                u64::from(decoded_register) < B::MASK,
                "{data_type}) Failed to decode index - index must be less than the mask"
            );
            assert_eq!(
                register, decoded_register,
                "{data_type}) Failed to decode register"
            );
            assert_eq!(index, decoded_index, "{data_type}) Failed to decode index");
        }
    }

    #[test_all_precisions_and_bits]
    fn test_composite_hash<P: Precision, B: Bits<Word = u8>>() {
        test_composite_hash_for_word::<P, B, u32>();
        test_composite_hash_for_word::<P, B, u40>();
        test_composite_hash_for_word::<P, B, u48>();
        test_composite_hash_for_word::<P, B, u56>();
        test_composite_hash_for_word::<P, B, u64>();
    }
}
