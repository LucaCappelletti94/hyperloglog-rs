//! The current hash.

use crate::{bits::Bits, prelude::Precision};

use super::shared::{find, insert_sorted_desc, into_variant, DecodedIter, DowngradedIter};
use super::CompositeHash;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
/// The current hash approach is particularly straightforward.
///
/// The hash is composed as follows:
///
/// [index | register | unused bits of the original hash]
pub struct CurrentHash<P: Precision, B: Bits> {
    _precision: core::marker::PhantomData<P>,
    _bits: core::marker::PhantomData<B>,
}

/// Returns the byte size of the smallest viable hash for the current precision and number of bits.
const fn smallest_viable_hash<P: Precision, B: Bits>() -> u8 {
    if P::EXPONENT == 4 && B::NUMBER_OF_BITS == 4 {
        return 8;
    }

    if P::EXPONENT < 9
        || P::EXPONENT == 9 && B::NUMBER_OF_BITS < 6
        || P::EXPONENT == 10 && B::NUMBER_OF_BITS < 5
    {
        return 16;
    }

    if P::EXPONENT <= 15
        || P::EXPONENT == 16 && B::NUMBER_OF_BITS < 6
    {
        return 24;
    }

    32
}

impl<P: Precision, B: Bits> CompositeHash for CurrentHash<P, B> {
    type Precision = P;
    type Bits = B;

    type Decoded<'a> = DecodedIter<'a, Self>;
    type Downgraded<'a> = DowngradedIter<'a, Self>;

    #[inline]
    #[must_use]
    fn downgraded<'a>(
        hashes: &'a [u8],
        number_of_hashes: usize,
        hash_bits: u8,
        shift: u8,
    ) -> Self::Downgraded<'a> {
        assert!(hash_bits > Self::SMALLEST_VIABLE_HASH_BITS || shift == 0 && hash_bits == Self::SMALLEST_VIABLE_HASH_BITS);
        DowngradedIter::new(into_variant(hashes, number_of_hashes, hash_bits), shift)
    }

    #[inline]
    #[must_use]
    fn decoded<'a>(hashes: &'a [u8], number_of_hashes: usize, hash_bits: u8) -> Self::Decoded<'a> {
        assert!(
            hash_bits >= Self::SMALLEST_VIABLE_HASH_BITS,
            "The hash bits ({hash_bits}) must be greater or equal to the smallest viable hash bits ({})",
            Self::SMALLEST_VIABLE_HASH_BITS,
        );
        DecodedIter::from(into_variant(hashes, number_of_hashes, hash_bits))
    }

    #[inline]
    #[must_use]
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
    fn encode(index: usize, register: u8, original_hash: u64, hash_bits: u8) -> u64 {
        ((index as u64) << (hash_bits - P::EXPONENT))
            | (u64::from(register) << (hash_bits - B::NUMBER_OF_BITS - P::EXPONENT))
            | ((original_hash >> (P::EXPONENT + 32 - hash_bits))
                & ((1_u64 << (hash_bits - B::NUMBER_OF_BITS - P::EXPONENT)) - 1_u64))
    }

    #[must_use]
    #[inline]
    /// Decode the hash into the register value and index.
    fn decode(hash: u64, hash_bits: u8) -> (u8, usize) {
        debug_assert!(
            hash.leading_zeros() >= u32::from(64 - hash_bits),
            "The hash ({hash:064b}) should be composed of {hash_bits} and therefore must have at least {} leading zeros, but has only {}",
            64 - hash_bits,
            hash.leading_zeros(),
        );
        // We extract the index from the rightmost bits of the hash.
        let index = usize::try_from(hash >> (hash_bits - P::EXPONENT)).unwrap();
        // Next, we extract the register from the rightmost bits of the hash, minus the bits used for the index.
        let register =
            u8::try_from((hash >> (hash_bits - B::NUMBER_OF_BITS - P::EXPONENT)) & B::MASK)
                .unwrap();

        (register, index)
    }

    #[inline]
    #[must_use]
    /// Downgrade the hash into a smaller hash.
    fn downgrade(hash: u64, hash_bits: u8, shift: u8) -> u64 {
        debug_assert!(hash_bits > Self::SMALLEST_VIABLE_HASH_BITS || shift == 0 && hash_bits == Self::SMALLEST_VIABLE_HASH_BITS);
        debug_assert!(shift < hash_bits);
        hash >> shift
    }

    #[inline]
    #[must_use]
    #[allow(unsafe_code)]
    fn find<'a>(
        hashes: &'a [u8],
        number_of_hashes: usize,
        index: usize,
        register: u8,
        original_hash: u64,
        hash_bits: u8,
    ) -> Result<usize, (usize, u64)> {
        find::<Self>(
            hashes,
            number_of_hashes,
            index,
            register,
            original_hash,
            hash_bits,
        )
    }

    #[inline]
    #[must_use]
    #[allow(unsafe_code)]
    fn insert_sorted_desc<'a>(
        hashes: &'a mut [u8],
        number_of_hashes: usize,
        index: usize,
        register: u8,
        original_hash: u64,
        hash_bits: u8,
    ) -> bool {
        insert_sorted_desc::<Self>(
            hashes,
            number_of_hashes,
            index,
            register,
            original_hash,
            hash_bits,
        )
    }

    #[inline]
    /// Downgrade the hash into a smaller hash in place.
    fn downgrade_inplace<'a>(
        hashes: &'a mut [u8],
        number_of_hashes: usize,
        hash_bits: u8,
        shift: u8,
    ) {
        if shift == 0 {
            return;
        }

        let hash_bytes = usize::from(hash_bits / 8);
        let shift_bytes = usize::from(shift / 8);

        debug_assert!(
            number_of_hashes * hash_bytes <= hashes.len(),
            "The slice len ({}) must be greater or equal to the product of the slice size ({hash_bytes}) and the number of elements ({number_of_hashes})",
            hashes.len(),
        );
        debug_assert!(shift_bytes < hash_bytes);
        debug_assert!(hash_bytes > 1);
        debug_assert!(shift_bytes > 0);

        for i in 0..number_of_hashes {
            hashes.copy_within(
                (i * hash_bytes + shift_bytes)..(i + 1) * hash_bytes,
                i * (hash_bytes - shift_bytes),
            );
        }
    }

    const SMALLEST_VIABLE_HASH_BITS: u8 = smallest_viable_hash::<P, B>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_handpicked_current_hash() {
        let original_hash: u64 = 5400060802014620068;
        let index = 4;
        let register = 2;

        assert_eq!(original_hash.leading_zeros(), u32::from(register - 1));
        assert_eq!(original_hash & 0b1111, index as u64);

        let encoded_hash_32 =
            CurrentHash::<Precision4, Bits4>::encode(index, register, original_hash, 32);
        let expected_hash_32 = 1121548954;
        assert_eq!(encoded_hash_32, expected_hash_32);
        let (register_32, index_32) = CurrentHash::<Precision4, Bits4>::decode(encoded_hash_32, 32);

        assert_eq!(index_32, index);
        assert_eq!(register_32, register);

        let encoded_hash_16 =
            CurrentHash::<Precision4, Bits4>::encode(index, register, original_hash, 16);
        let (register_16, index_16) = CurrentHash::<Precision4, Bits4>::decode(encoded_hash_16, 16);
        assert_eq!(index_16, index);
        assert_eq!(register_16, register);

        let expected_32: u64 = (4 << (32 - 4))
            + (2 << (32 - 4 - 4))
            + ((5400060802014620068 >> (4 + 32 - 32)) & ((1 << (32 - 4 - 4)) - 1));
        let expected_16: u64 =
            (4 << 12) + (2 << 8) + ((5400060802014620068 >> (4 + 32 - 16)) & 0b11111111);
        assert_eq!(expected_32 >> 16, expected_16);

        assert_eq!(expected_32, expected_hash_32);

        let (register_16, index_16) = CurrentHash::<Precision4, Bits4>::decode(17050, 16);
        assert_eq!(index_16, index);
        assert_eq!(register_16, register);

        let expected_hash_16 = 17113;
        assert_eq!(expected_16, expected_hash_16);
        assert_eq!(encoded_hash_16, expected_hash_16);

        let downgraded_32_to_16 =
            CurrentHash::<Precision4, Bits4>::downgrade(encoded_hash_32, 32, 16);
        assert_eq!(downgraded_32_to_16, expected_hash_16 as u64);
    }
}
