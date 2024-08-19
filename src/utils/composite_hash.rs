//! Submodule providing the composite hash trait, which is a type of hash
//! that can operate as a variable word and is composable from a hash and
//! a register value and is symmetrically splittable back into a hash and
//! a register value.

use crate::prelude::*;

/// Trait for a composite hash.
pub trait CompositeHash<P: Precision, B: Bits>: VariableWord {
    /// The number of bits in the composite hash.
    const OFFSET: u8 = Self::NUMBER_OF_BITS - B::NUMBER_OF_BITS;

    #[allow(unsafe_code)]
    /// Encode the hash from the provided register value, index and the original unsplitted hash.
    fn encode(register: u8, hash: u64) -> Self::Word {
        debug_assert!(register > 0);

        // We convert the hash into the word
        let mut hash = unsafe { Self::Word::unchecked_from_u64(hash & Self::MASK) };

        // We remove the highers B bits from the hash which will be
        // replaced by the register value.
        hash &= !(unsafe { Self::Word::unchecked_from_u64(B::MASK) } << Self::OFFSET);
        hash |= Self::Word::from(register) << Self::OFFSET;

        hash
    }

    #[allow(unsafe_code)]
    /// Decode the hash into the register value and index.
    fn decode(hash: Self::Word) -> (u8, P::NumberOfRegisters) {
        // We extract the index from the rightmost bits of the hash.
        let index = unsafe {
            P::NumberOfRegisters::unchecked_from_u64(<Self::Word as Into<u64>>::into(
                hash & ((Self::Word::ONE << P::EXPONENT) - Self::Word::ONE),
            ))
        };
        // Next, we extract the register from the leftmost bits of the hash.
        let register = unsafe {
            u8::unchecked_from_u64(<Self::Word as Into<u64>>::into(hash) >> Self::OFFSET)
        };

        (register, index)
    }
}

/// Macro to implement the appropriate CompositeHash trait for a given type.
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

macro_rules! impl_composite_hash_everything {
    ($w:ty) => {
        impl_composite_hash_all_bits!($w, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);
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

impl_composite_hash_everything!(u32);
impl_composite_hash_everything!(u40);
impl_composite_hash_everything!(u48);
impl_composite_hash_everything!(u56);
impl_composite_hash_everything!(u64);
