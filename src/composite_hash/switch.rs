//! The SwitchHash, which more accurately follows the HyperLogLog++ paper.

use super::shared::{find, insert_sorted_desc, into_variant, DecodedIter, DowngradedIter};
use super::CompositeHash;
use crate::{bits::Bits, prelude::Precision};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
/// The current hash approach is particularly straightforward.
///
/// The hash is composed as follows:
///
/// [index | register | unused bits of the original hash]
pub struct SwitchHash<P: Precision, B: Bits> {
    _precision: core::marker::PhantomData<P>,
    _bits: core::marker::PhantomData<B>,
}

fn flag<P: Precision>(hash: u64, hash_bits: u8) -> bool {
    ((hash >> (hash_bits - P::EXPONENT - 1)) & 1) == 1
}

const fn smallest_viable_switch_hash<P: Precision, B: Bits>() -> u8 {
    if P::EXPONENT == 4 && B::NUMBER_OF_BITS == 4 {
        return 8;
    }

    if P::EXPONENT < 10 || P::EXPONENT == 10 && B::NUMBER_OF_BITS < 6 {
        return 16;
    }

    if P::EXPONENT < 18 || P::EXPONENT == 18 && B::NUMBER_OF_BITS < 6 {
        return 24;
    }

    32
}

impl<P: Precision, B: Bits> CompositeHash for SwitchHash<P, B> {
    type Precision = P;
    type Bits = B;

    type Decoded<'a> = DecodedIter<'a, Self>;
    type Downgraded<'a> = DowngradedIter<'a, Self>;

    const SMALLEST_VIABLE_HASH_BITS: u8 = smallest_viable_switch_hash::<P, B>();

    fn downgrade_inplace<'a>(
        hashes: &'a mut [u8],
        number_of_hashes: usize,
        hash_bits: u8,
        shift: u8,
    ) {
        let hash_bytes = usize::from(hash_bits) / 8;
        let downgraded_hash_bytes = usize::from(hash_bits - shift) / 8;

        for i in 0..number_of_hashes {
            let current_hash = match hash_bytes {
                2 => u64::from(u16::from_le_bytes([hashes[i * 2], hashes[i * 2 + 1]])),
                3 => u64::from(u32::from_le_bytes([
                    hashes[i * 3],
                    hashes[i * 3 + 1],
                    hashes[i * 3 + 2],
                    0,
                ])),
                4 => u64::from(u32::from_le_bytes([
                    hashes[i * 4],
                    hashes[i * 4 + 1],
                    hashes[i * 4 + 2],
                    hashes[i * 4 + 3],
                ])),
                _ => unreachable!("The hash must be 2, 3 or 4 bytes."),
            };
            let downgraded_hash = Self::downgrade(current_hash, hash_bits, shift);
            let downgraded_bytes = downgraded_hash.to_le_bytes();
            hashes[i * downgraded_hash_bytes..(i + 1) * downgraded_hash_bytes]
                .copy_from_slice(&downgraded_bytes[..downgraded_hash_bytes]);
        }
    }

    #[inline]
    #[must_use]
    fn downgraded<'a>(
        hashes: &'a [u8],
        number_of_hashes: usize,
        hash_bits: u8,
        shift: u8,
    ) -> Self::Downgraded<'a> {
        assert!(
            hash_bits > Self::SMALLEST_VIABLE_HASH_BITS
                || shift == 0 && hash_bits == Self::SMALLEST_VIABLE_HASH_BITS
        );
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

    fn downgrade(hash: u64, hash_bits: u8, shift: u8) -> u64 {
        debug_assert!(hash_bits >= Self::Precision::EXPONENT + Self::Bits::NUMBER_OF_BITS + shift);

        if Self::Precision::EXPONENT + Self::Bits::NUMBER_OF_BITS == hash_bits {
            if shift > 0 {
                unreachable!("The hash is already at the lowest precision, you cannot shift it down by {shift} bits.");
            }
            return hash;
        }

        if flag::<Self::Precision>(hash, hash_bits) {
            // If the register is stored explicitly, we can shift the hash to the right
            // and remove the extra padding to reduce it to the smaller variant.
            if hash_bits - shift == Self::Precision::EXPONENT + Self::Bits::NUMBER_OF_BITS {
                // We also need to remove the flag bit, otherwise we would delete
                // a bit of the register value.
                let (register, index) = Self::decode(hash, hash_bits);

                ((index as u64) << Self::Bits::NUMBER_OF_BITS) | u64::from(register)
            } else {
                hash >> shift
            }
        } else {
            // Otherwise, we need to handle the case with the register stored as the leading
            // zeros of the original hash. We extract the leading zeros and shift it back to
            // the leftmost bits to restore them to their original position.
            let (register, index) = Self::decode(hash, hash_bits);
            let restored_hash = !(hash << (Self::Precision::EXPONENT + 1 + (64 - hash_bits)));
            debug_assert_eq!(restored_hash.leading_zeros(), u32::from(register) - 1);
            Self::encode(index, register, restored_hash, hash_bits - shift)
        }
    }

    #[inline]
    #[must_use]
    /// Encode the hash from the provided register value, index and the original unsplitted hash.
    fn encode(index: usize, register: u8, original_hash: u64, hash_bits: u8) -> u64 {
        debug_assert!(register > 0);
        debug_assert!(index < 1 << Self::Precision::EXPONENT);
        // We start by encoding the index in the rightmost bits of the hash.
        let mut composite_hash = (index as u64) << (hash_bits - Self::Precision::EXPONENT);

        // If the hash barely fits as-is, we do not need to do anything special.
        if Self::Precision::EXPONENT + Self::Bits::NUMBER_OF_BITS == hash_bits {
            composite_hash |= u64::from(register);
            return composite_hash;
        }

        // Depending on whether the registers, i.e. the number of leading zeros in
        // the provided hash, are less than the number of available bits in the hash
        // we are currently encoding minus the bits used for the index, instead of
        // storing the register value after the index, we store the higher bits of
        // the original hash after the index.
        if register > Self::Bits::NUMBER_OF_BITS {
            // In this case, the composite hash has the following structure:
            //
            // [index (exponent bits) | flag = 1 | registers (bitsize) | hash remainder]

            // We set to 1 the flag bit, which indicates that the composite hash
            // has the structure we described above.
            composite_hash |= 1 << (hash_bits - Self::Precision::EXPONENT - 1);

            // We place the register value in the rightmost bits of the hash, after the index
            // and the flag.
            let register_offset =
                hash_bits - Self::Precision::EXPONENT - 1 - Self::Bits::NUMBER_OF_BITS;
            composite_hash |= u64::from(register) << register_offset;

            // We take the bits remaining in the original hash after having remove the last
            // p bits which were used for the index, and we place them in the rightmost bits
            // of the composite hash.
            let mut censored_hash = original_hash
                >> (64 - hash_bits + Self::Precision::EXPONENT + Self::Bits::NUMBER_OF_BITS);

            // We determine the number of bits we intend to keep from the original hash.
            let hash_offset =
                hash_bits - Self::Precision::EXPONENT - 1 - Self::Bits::NUMBER_OF_BITS;

            // We mask out the bits we do not want to keep.
            let mask = (1 << hash_offset) - 1;
            censored_hash &= mask;

            // We place the censored hash in the rightmost bits of the composite hash.
            composite_hash |= censored_hash;

            debug_assert_eq!(
                (composite_hash >> (hash_bits - Self::Precision::EXPONENT - 1)) & 1,
                1
            );
        } else {
            // In this case, the composite hash has the following structure:
            //
            // [index (exponent bits) | flag = 0 | original hash leading values]

            // We do not need to set the flag bit, as we initialized the composite
            // hash with zeros and that bit is certainly zero.

            // We shift the original hash to the right.
            let censored_hash =
                (!original_hash) >> (64 - hash_bits + 1 + Self::Precision::EXPONENT);

            // We place the censored hash in the rightmost bits of the composite hash.
            composite_hash |= censored_hash;

            debug_assert_eq!(
                (composite_hash >> (hash_bits - Self::Precision::EXPONENT - 1)) & 1,
                0
            );
        }

        debug_assert!(
            composite_hash.leading_zeros() >= u32::from(64 - hash_bits),
            "The composite hash {composite_hash} must have at least {} leading zeros, but has only {}",
            64 - hash_bits,
            composite_hash.leading_zeros(),
        );

        composite_hash
    }

    #[must_use]
    #[inline]
    /// Decode the hash into the register value and index.
    fn decode(hash: u64, hash_bits: u8) -> (u8, usize) {
        // We extract the index from the leftmost bits of the hash.
        let index = usize::try_from(hash >> (hash_bits - Self::Precision::EXPONENT)).unwrap();

        // If the hash barely fits as is, we do not need to do anything special.
        if Self::Precision::EXPONENT + Self::Bits::NUMBER_OF_BITS == hash_bits {
            let register = u8::try_from(hash & Self::Bits::MASK).unwrap();
            return (register, index);
        }

        // If the flag bit is set to 1, we have a composite hash with the following structure:
        // [index (exponent bits) | flag = 1 | registers (bitsize) | hash remainder]
        let register = if flag::<Self::Precision>(hash, hash_bits) {
            u8::try_from(
                (hash >> (hash_bits - Self::Bits::NUMBER_OF_BITS - Self::Precision::EXPONENT - 1))
                    & Self::Bits::MASK,
            )
            .unwrap()
        } else {
            // Otherwise, we have a composite hash with the following structure:
            // [index (exponent bits) | flag = 0 | original hash leading values]
            // Therefore, we shift left by exponent bits and the flag bit to get
            // the leading values of the original hash to the rightmost bits,
            // and then we can count the leading zeros to get the register value.
            let mut restored_hash = !(hash << (Self::Precision::EXPONENT + 1 + (64 - hash_bits)));
            restored_hash |= 1_u64 << (64_u64 - Self::Bits::MASK);

            let leading_zeros = restored_hash.leading_zeros();
            u8::try_from(leading_zeros + 1).unwrap()
        };

        (register, index)
    }
}

#[cfg(test)]
mod switch_hash_test {
    use super::*;
    use crate::prelude::{Bits4, Precision4};

    #[test]
    fn test_handpicked_switch_hash_precision4_bits4() {
        let index = 2;
        let register = 1;
        let original_hash: u64 =
            0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_0010;
        let hash_bits = 16;

        let encoded =
            SwitchHash::<Precision4, Bits4>::encode(index, register, original_hash, hash_bits);

        let expected_hash = 0b0010_0000_0000_0000;

        assert_eq!(
            encoded, expected_hash,
            "The encoded hash is not as expected: {:016b}",
            encoded
        );

        let (decoded_register, decoded_index) =
            SwitchHash::<Precision4, Bits4>::decode(expected_hash, 16);

        assert_eq!(decoded_register, register);
        assert_eq!(decoded_index, index);

        let index = 7;
        let register = 12; // 11 leading zeros + 1
        let original_hash =
            0b0000_0000_0001_0110_1010_1101_0000_0101_0010_0010_0100_0101_1100_0100_0010_0111;
        let encoded_hash = 0b0111_0111_1111_1111_0100_1010;
        let (decoded_register, decoded_index) =
            SwitchHash::<Precision4, Bits4>::decode(encoded_hash, 24);
        assert_eq!(decoded_register, register);
        assert_eq!(decoded_index, index);

        let downgraded_hash = SwitchHash::<Precision4, Bits4>::downgrade(encoded_hash, 24, 8);
        let directly_encoded_downgraded_hash =
            SwitchHash::<Precision4, Bits4>::encode(index, register, original_hash, 16);
        assert_eq!(downgraded_hash, directly_encoded_downgraded_hash, "The downgraded hash ({downgraded_hash:016b}) is not as expected ({directly_encoded_downgraded_hash:016b})");
    }

    #[test]
    fn test_downgrading_switch_hash_precision4_bits4() {
        let original_hash: u64 = 0b0000000000001001100000001000110110110000010001100010011101000111;
        // let expected_encoded_24: u64 = 0b0111_0111_1111_1111_1011_0011;
        let index = 7;
        let register = 13;

        let encoded_24 =
            SwitchHash::<Precision4, Bits4>::encode(index, register, original_hash, 24);
        // assert_eq!(encoded_24, expected_encoded_24);

        let (decoded_register_24, decoded_index_24) =
            SwitchHash::<Precision4, Bits4>::decode(encoded_24, 24);
        assert_eq!(decoded_index_24, index);
        assert_eq!(decoded_register_24, register);

        let encoded_16 =
            SwitchHash::<Precision4, Bits4>::encode(index, register, original_hash, 16);
        let (decoded_register_16, decoded_index_16) =
            SwitchHash::<Precision4, Bits4>::decode(encoded_16, 16);
        assert_eq!(decoded_index_16, index);
        assert_eq!(decoded_register_16, register);

        let downgraded_24_to_16 = SwitchHash::<Precision4, Bits4>::downgrade(encoded_24, 24, 8);

        assert_eq!(
            downgraded_24_to_16,
            encoded_16,
            "The downgraded hash ({downgraded_24_to_16:016b}) is not as expected ({encoded_16:016b})"
        );
    }

    #[test]
    fn test_downgrading_switch_hash_precision4_bits4_2() {
        let original_hash: u64 =
            0b0000_0000_0000_0111_0110_0011_0111_1101_0011_1000_1011_0110_1001_0010_1110_0111;
        // let expected_encoded_16: u64 = 0b0111_1111_0111_0110;
        let index = 7;
        let register = 14;

        let encoded_24 =
            SwitchHash::<Precision4, Bits4>::encode(index, register, original_hash, 24);
        // assert_eq!(encoded_24, expected_encoded_24);

        let (decoded_register_24, decoded_index_24) =
            SwitchHash::<Precision4, Bits4>::decode(encoded_24, 24);
        assert_eq!(decoded_index_24, index);
        assert_eq!(decoded_register_24, register);

        let encoded_16 =
            SwitchHash::<Precision4, Bits4>::encode(index, register, original_hash, 16);
        // assert_eq!(encoded_16, expected_encoded_16);

        let (decoded_register_16, decoded_index_16) =
            SwitchHash::<Precision4, Bits4>::decode(encoded_16, 16);
        assert_eq!(decoded_index_16, index);
        assert_eq!(decoded_register_16, register);

        let downgraded_24_to_16 = SwitchHash::<Precision4, Bits4>::downgrade(encoded_24, 24, 8);

        assert_eq!(
            downgraded_24_to_16,
            encoded_16,
            "The downgraded hash ({downgraded_24_to_16:016b}) is not as expected ({encoded_16:016b})"
        );
    }
}
