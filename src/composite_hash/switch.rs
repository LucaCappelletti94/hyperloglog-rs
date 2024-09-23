//! Composite hash which does not employ Prefix-free codes.

use core::slice::Iter;

use super::gaps::InsertMetadata;
use super::{LastBufferedBit, SaturationError};
use crate::composite_hash::GapHash;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct HashFragment<P: Precision, B: Bits> {
    pub(super) index: u32,
    pub(super) register: u8,
    pub(super) hash_remainder: u32,
    _precision: core::marker::PhantomData<P>,
    _bits: core::marker::PhantomData<B>,
}

impl<P: Precision, B: Bits> HashFragment<P, B> {
    #[inline]
    pub const fn hash_remainder_size(hash_bits: u8) -> u8 {
        if hash_bits == P::EXPONENT + B::NUMBER_OF_BITS {
            return 0;
        }
        hash_bits - 1 - P::EXPONENT
    }

    #[inline]
    fn restored_hash(&self, hash_bits: u8) -> u64 {
        !(u64::from(self.hash_remainder) << (P::EXPONENT + 1 + 64 - hash_bits))
    }

    #[inline]
    fn register_flag(&self) -> bool {
        self.register - 1 > B::NUMBER_OF_BITS
    }

    #[inline]
    pub(super) fn uniform(&self, hash_bits: u8) -> u32 {
        (self.index << Self::hash_remainder_size(hash_bits)) | self.hash_remainder
    }
}

impl<P: Precision, B: Bits> SwitchHash<P, B> {
    #[inline]
    /// Returns the provided [`SwitchHash`] splitted into its components.
    pub(super) fn scompose_hash(hash: u32, hash_bits: u8) -> HashFragment<P, B> {
        // We extract the index from the leftmost bits of the hash.
        let index = hash >> (hash_bits - P::EXPONENT);

        // If the hash barely fits as is, we do not need to do anything special.
        if P::EXPONENT + B::NUMBER_OF_BITS == hash_bits {
            let register = (u64::from(hash) & B::MASK) as u8;
            debug_assert!(
                register > 0,
                "The register value ({register}) must be greater than 0. Obtained from hash {hash:032b} with bits {hash_bits} and index {index}, with precision {} and bits {}.",
                P::EXPONENT,
                B::NUMBER_OF_BITS
            );
            return HashFragment {
                index,
                register,
                hash_remainder: 0,
                _precision: core::marker::PhantomData,
                _bits: core::marker::PhantomData,
            };
        }

        let register_flag = flag::<P>(hash, hash_bits);

        // If the flag bit is set to 1, we have a composite hash with the following structure:
        // [index (exponent bits) | flag = 1 | registers (bitsize) | hash remainder]
        if register_flag {
            let shift = hash_bits - 1 - P::EXPONENT - B::NUMBER_OF_BITS;
            let hash_remainder_mask = (1 << shift) - 1;
            let register = (u64::from(hash >> shift) & B::MASK) as u8;
            debug_assert!(
                register > 0,
                "The register value ({register}) must be greater than 0. Obtained from hash {hash:064b} with bits {hash_bits}"
            );
            let hash_remainder = hash & hash_remainder_mask;

            HashFragment {
                index,
                register,
                hash_remainder,
                _precision: core::marker::PhantomData,
                _bits: core::marker::PhantomData,
            }
        } else {
            // Otherwise, we have a composite hash with the following structure:
            // [index (exponent bits) | flag = 0 | original hash leading values]
            // Therefore, we shift left by exponent bits and the flag bit to get
            // the leading values of the original hash to the rightmost bits,
            // and then we can count the leading zeros to get the register value.
            let shift = P::EXPONENT + 1 + (64 - hash_bits);
            let restored_hash = !(u64::from(hash) << shift);
            let hash_remainder_mask = (1 << (hash_bits - 1 - P::EXPONENT)) - 1;
            let hash_remainder = hash & hash_remainder_mask;

            let leading_zeros = (restored_hash | (1_u64 << (64_u64 - B::MASK))).leading_zeros();
            let register = (leading_zeros + 1) as u8;
            debug_assert!(register > 0);

            HashFragment {
                index,
                register,
                hash_remainder,
                _precision: core::marker::PhantomData,
                _bits: core::marker::PhantomData,
            }
        }
    }

    #[inline]
    pub(super) fn compose_hash(
        index: u32,
        register: u8,
        hash_remainder: u32,
        hash_bits: u8,
    ) -> u32 {
        debug_assert!(register > 0);

        let mut hash = index << (hash_bits - P::EXPONENT);

        if P::EXPONENT + B::NUMBER_OF_BITS == hash_bits {
            return hash | u32::from(register);
        }

        if register - 1 > B::NUMBER_OF_BITS {
            let shift = hash_bits - 1 - P::EXPONENT - B::NUMBER_OF_BITS;
            hash |= (u32::from(register) << shift) | (1 << (hash_bits - P::EXPONENT - 1));
        }

        hash |= hash_remainder;

        hash
    }
}

#[cfg(test)]
mod test_compose_scompose_hash {
    use crate::prelude::*;
    use crate::utils::iter_random_values;
    use hyperloglog_derive::test_precisions_and_bits;

    use super::*;

    #[test_precisions_and_bits]
    fn test_compose_scompose_hash<P: Precision, B: Bits>()
    where
        P: PackedRegister<B>,
    {
        for value in iter_random_values::<u64>(10_000, None, None) {
            let (index, register, original_hash) =
                <HyperLogLog<P, B, <P as PackedRegister<B>>::Array>>::index_and_register_and_hash(
                    &value,
                );
            for hash_bits in SwitchHash::<P, B>::SMALLEST_VIABLE_HASH_BITS
                ..=SwitchHash::<P, B>::LARGEST_VIABLE_HASH_BITS
            {
                let encoded_hash =
                    SwitchHash::<P, B>::encode(index, register, original_hash, hash_bits);

                let fragment = SwitchHash::<P, B>::scompose_hash(encoded_hash, hash_bits);

                assert_eq!(fragment.index, u32::try_from(index).unwrap());
                assert_eq!(fragment.register, register);

                let recomposed_hash = SwitchHash::<P, B>::compose_hash(
                    u32::try_from(index).unwrap(),
                    register,
                    u32::from(fragment.hash_remainder),
                    hash_bits,
                );

                assert_eq!(recomposed_hash, encoded_hash);
            }
        }
    }
}

#[inline]
fn flag<P: Precision>(hash: u32, hash_bits: u8) -> bool {
    ((hash >> (hash_bits - P::EXPONENT - 1)) & 1) == 1
}

#[inline]
pub(super) const fn smallest_viable_switch_hash<P: Precision, B: Bits>() -> u8 {
    (P::EXPONENT + B::NUMBER_OF_BITS).div_ceil(8) * 8
}

#[inline]
const fn maximal_viable_switch_hash<P: Precision, B: Bits>() -> u8 {
    if P::EXPONENT == 4 && B::NUMBER_OF_BITS == 4 {
        return 8;
    }

    if P::EXPONENT < 8 {
        return 16;
    }

    if P::EXPONENT < 15 {
        return 24;
    }

    32
}

impl<P: Precision, B: Bits> SwitchHash<P, B> {
    const SMALLEST_VIABLE_HASH_BITS: u8 = smallest_viable_switch_hash::<P, B>();
    pub(super) const LARGEST_VIABLE_HASH_BITS: u8 = maximal_viable_switch_hash::<P, B>();

    #[inline]
    #[must_use]
    pub(super) fn downgraded(
        hashes: &[u8],
        number_of_hashes: u32,
        hash_bits: u8,
        bit_index: u32,
        shift: u8,
    ) -> DowngradedIter<'_, P, B> {
        assert_eq!(bit_index, number_of_hashes * u32::from(hash_bits));
        DowngradedIter::new(into_variant(hashes, number_of_hashes, hash_bits), shift)
    }

    #[inline]
    #[must_use]
    pub(super) fn decoded(
        hashes: &[u8],
        number_of_hashes: u32,
        hash_bits: u8,
        bit_index: u32,
    ) -> DecodedIter<'_, P, B> {
        assert!(
            hash_bits >= Self::SMALLEST_VIABLE_HASH_BITS,
            "The hash bits ({hash_bits}) must be greater or equal to the smallest viable hash bits ({})",
            Self::SMALLEST_VIABLE_HASH_BITS,
        );
        assert_eq!(bit_index, number_of_hashes * u32::from(hash_bits));
        DecodedIter::from(into_variant(hashes, number_of_hashes, hash_bits))
    }

    #[inline]
    #[allow(unsafe_code)]
    pub(super) fn find(
        hashes: &[u8],
        number_of_hashes: u32,
        encoded_hash: u32,
        hash_bits: u8,
        bit_index: u32,
    ) -> Result<usize, (usize, u32)> {
        assert!(hash_bits >= 8);
        assert!(hash_bits == 8 || hash_bits == 16 || hash_bits == 24 || hash_bits == 32);
        assert!(hash_bits >= Self::SMALLEST_VIABLE_HASH_BITS);
        assert!(bit_index == number_of_hashes * u32::from(hash_bits));

        let hashes: &[u8] =
            &hashes[..usize::try_from(number_of_hashes).unwrap() * usize::from(hash_bits / 8)];

        match hash_bits / 8 {
            1 => {
                let hash = u8::try_from(encoded_hash).unwrap();
                hashes
                    // The hashes are sorted in a descending order, so we need to find the first hash that is less than the encoded hash.
                    .binary_search_by(|a| hash.cmp(a))
                    .map(|index| index * usize::from(hash_bits))
                    .map_err(|index| (index * usize::from(hash_bits), encoded_hash))
            }
            2 => {
                // We transmute the hash to a u16, as we will be comparing it with other u16s.
                let hashes: &[u16] = unsafe {
                    core::slice::from_raw_parts(hashes.as_ptr().cast::<u16>(), hashes.len() / 2)
                };
                let hash = u16::try_from(encoded_hash).unwrap();
                hashes
                    // The hashes are sorted in a descending order, so we need to find the first hash that is less than the encoded hash.
                    .binary_search_by(|a| hash.cmp(a))
                    .map(|index| index * usize::from(hash_bits))
                    .map_err(|index| (index * usize::from(hash_bits), encoded_hash))
            }
            3 => {
                // We transmute the hash to a hash of [u8; 3], as we will be comparing it with other [u8; 3]s.
                // Since these values are stored in little-endian, we need to reverse the bytes into a u32 before
                // comparing them.
                let hashes: &[[u8; 3]] = unsafe {
                    core::slice::from_raw_parts(hashes.as_ptr().cast::<[u8; 3]>(), hashes.len() / 3)
                };
                let hash: u32 = u32::try_from(encoded_hash).unwrap();
                hashes
                    // The hashes are sorted in a descending order, so we need to find the first hash that is less than the encoded hash.
                    .binary_search_by(|a| {
                        let mut four_bytes = [0; 4];
                        four_bytes[..3].copy_from_slice(a);
                        let a = u32::from_le_bytes(four_bytes);
                        hash.cmp(&a)
                    })
                    .map(|index| index * usize::from(hash_bits))
                    .map_err(|index| (index * usize::from(hash_bits), encoded_hash))
            }
            4 => {
                // We transmute the hash to a u32, as we will be comparing it with other u32s.
                let hashes: &[u32] = unsafe {
                    core::slice::from_raw_parts(hashes.as_ptr().cast::<u32>(), hashes.len() / 4)
                };
                let hash = u32::try_from(encoded_hash).unwrap();
                hashes
                    // The hashes are sorted in a descending order, so we need to find the first hash that is less than the encoded hash.
                    .binary_search_by(|a| hash.cmp(a))
                    .map(|index| index * usize::from(hash_bits))
                    .map_err(|index| (index * usize::from(hash_bits), encoded_hash))
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub(super) fn insert_sorted_desc(
        hashes: &mut [u8],
        number_of_hashes: u32,
        bit_index: u32,
        encoded_hash: u32,
        hash_bits: u8,
    ) -> Result<Option<InsertMetadata>, SaturationError> {
        assert!(hash_bits >= 8);
        assert!(
            hash_bits >= Self::SMALLEST_VIABLE_HASH_BITS,
            "The hash bits ({}) must be greater or equal to the smallest viable hash bits ({})",
            hash_bits,
            Self::SMALLEST_VIABLE_HASH_BITS,
        );
        assert!(hash_bits == 8 || hash_bits == 16 || hash_bits == 24 || hash_bits == 32);
        assert_eq!(bit_index, number_of_hashes * u32::from(hash_bits));
        let hash_bytes = usize::from(hash_bits / 8);

        match Self::find(hashes, number_of_hashes, encoded_hash, hash_bits, bit_index) {
            Ok(_) => Ok(None),
            Err((bits, encoded_hash)) => {
                // We check that there is indeed no hash identical to the provided
                // encoded hash:
                debug_assert!(
                    Self::downgraded(hashes, number_of_hashes, hash_bits, bit_index, 0)
                        .all(|hash| hash != encoded_hash),
                    "The hash ({encoded_hash}) must not be present in the hashes",
                );

                // If we are in the case of a growable hash_list and the hashes are not yet
                // saturated, we can raise a ExtendableSaturation error.
                if hashes.len() * 8 < usize::from(hash_bits) + usize::try_from(bit_index).unwrap()
                    && (!GapHash::<P, B>::has_rank_index()
                        && hashes.len()
                            != (B::NUMBER_OF_BITS_USIZE * (1 << P::EXPONENT)).div_ceil(64) * 8
                        || GapHash::<P, B>::has_rank_index()
                            && hashes.len()
                                != (B::NUMBER_OF_BITS_USIZE * (1 << P::EXPONENT)).div_ceil(64) * 8
                                    - usize::try_from(GapHash::<P, B>::rank_index_padded_size(
                                        hash_bits,
                                    ))
                                    .unwrap())
                {
                    return Err(SaturationError::ExtendableSaturation);
                }

                if usize::try_from(bit_index).unwrap() / 8 + hash_bytes > hashes.len() {
                    return Err(SaturationError::Saturation(bit_index));
                }

                let index = bits / usize::from(hash_bits);

                match hash_bytes {
                    1 => {
                        hashes.copy_within(
                            index..usize::try_from(number_of_hashes).unwrap(),
                            index + 1,
                        );
                        hashes[index] = u8::try_from(encoded_hash).unwrap();
                    }
                    2 => {
                        let hashes: &mut [u16] = unsafe {
                            core::slice::from_raw_parts_mut(
                                hashes.as_mut_ptr().cast::<u16>(),
                                hashes.len() / 2,
                            )
                        };
                        hashes.copy_within(
                            index..usize::try_from(number_of_hashes).unwrap(),
                            index + 1,
                        );
                        hashes[index] = u16::try_from(encoded_hash).unwrap();
                    }
                    3 => {
                        let hashes: &mut [[u8; 3]] = unsafe {
                            core::slice::from_raw_parts_mut(
                                hashes.as_mut_ptr().cast::<[u8; 3]>(),
                                hashes.len() / 3,
                            )
                        };
                        hashes.copy_within(
                            index..usize::try_from(number_of_hashes).unwrap(),
                            index + 1,
                        );
                        let mut three_bytes = [0; 3];
                        three_bytes.copy_from_slice(&encoded_hash.to_le_bytes()[..3]);
                        hashes[index] = three_bytes;
                    }
                    4 => {
                        let hashes: &mut [u32] = unsafe {
                            core::slice::from_raw_parts_mut(
                                hashes.as_mut_ptr().cast::<u32>(),
                                hashes.len() / 4,
                            )
                        };
                        hashes.copy_within(
                            index..usize::try_from(number_of_hashes).unwrap(),
                            index + 1,
                        );
                        hashes[index] = u32::try_from(encoded_hash).unwrap();
                    }
                    _ => unreachable!(),
                }
                Ok(Some(InsertMetadata {
                    hash_bits,
                    duplicates: 0,
                    bit_index: (number_of_hashes + 1) * u32::from(hash_bits),
                }))
            }
        }
    }

    #[inline(always)]
    pub(super) fn downgrade(hash: u32, hash_bits: u8, shift: u8) -> u32 {
        debug_assert!(hash_bits >= P::EXPONENT + B::NUMBER_OF_BITS + shift);

        if shift == 0 {
            return hash;
        }

        if P::EXPONENT + B::NUMBER_OF_BITS == hash_bits {
            unreachable!("The hash is already at the lowest precision, you cannot shift it down by {shift} bits.");
        }

        let fragmented = Self::scompose_hash(hash, hash_bits);

        debug_assert!(
            fragmented.index < 1 << P::EXPONENT,
            "The index ({}) must be less than 2^({})",
            fragmented.index,
            P::EXPONENT,
        );

        debug_assert!(
            fragmented.register > 0,
            "The register value ({}) must be greater than 0",
            fragmented.register
        );

        if fragmented.register_flag() {
            // If the register is stored explicitly, we can shift the hash to the right
            // and remove the extra padding to reduce it to the smaller variant.
            if hash_bits - shift == P::EXPONENT + B::NUMBER_OF_BITS {
                // We also need to remove the flag bit, otherwise we would delete
                // a bit of the register value.
                (fragmented.index << B::NUMBER_OF_BITS) | u32::from(fragmented.register)
            } else {
                hash >> shift
            }
        } else {
            // Otherwise, we need to handle the case with the register stored as the leading
            // zeros of the original hash. We extract the leading zeros and shift it back to
            // the leftmost bits to restore them to their original position.
            Self::encode(
                usize::try_from(fragmented.index).unwrap(),
                u8::try_from(fragmented.register).unwrap(),
                fragmented.restored_hash(hash_bits),
                hash_bits - shift,
            )
        }
    }

    #[inline]
    #[must_use]
    /// Encode the hash from the provided register value, index and the original unsplitted hash.
    pub(super) fn encode(index: usize, register: u8, original_hash: u64, hash_bits: u8) -> u32 {
        debug_assert!(register > 0);
        debug_assert!(
            index < 1 << P::EXPONENT,
            "The index ({index}) must be less than 2^({})",
            P::EXPONENT,
        );
        debug_assert!(
            hash_bits >= P::EXPONENT + B::NUMBER_OF_BITS,
            "The hash bits ({hash_bits}) must be greater or equal to the sum of the exponent ({}) and the number of bits ({})",
            P::EXPONENT,
            B::NUMBER_OF_BITS,
        );
        // We start by encoding the index in the rightmost bits of the hash.
        let mut composite_hash = u32::try_from(index).unwrap() << (hash_bits - P::EXPONENT);

        // If the hash barely fits as-is, we do not need to do anything special.
        if P::EXPONENT + B::NUMBER_OF_BITS == hash_bits {
            return composite_hash | u32::from(register);
        }

        // Depending on whether the registers, i.e. the number of leading zeros in
        // the provided hash, are less than the number of available bits in the hash
        // we are currently encoding minus the bits used for the index, instead of
        // storing the register value after the index, we store the higher bits of
        // the original hash after the index.
        if register - 1 > B::NUMBER_OF_BITS {
            // In this case, the composite hash has the following structure:
            //
            // [index (exponent bits) | flag = 1 | registers (bitsize) | hash remainder]

            // We set to 1 the flag bit, which indicates that the composite hash
            // has the structure we described above.
            composite_hash |= 1 << (hash_bits - P::EXPONENT - 1);

            // We place the register value in the rightmost bits of the hash, after the index
            // and the flag.
            let register_offset = hash_bits - P::EXPONENT - 1 - B::NUMBER_OF_BITS;
            composite_hash |= u32::from(register) << register_offset;

            // We take the bits remaining in the original hash after having remove the last
            // p bits which were used for the index, and we place them in the rightmost bits
            // of the composite hash.
            let mut censored_hash =
                u32::try_from(original_hash >> (64 - hash_bits + P::EXPONENT + B::NUMBER_OF_BITS))
                    .unwrap();

            // We determine the number of bits we intend to keep from the original hash.
            let hash_offset = hash_bits - P::EXPONENT - 1 - B::NUMBER_OF_BITS;

            // We mask out the bits we do not want to keep.
            let mask = (1 << hash_offset) - 1;
            censored_hash &= mask;

            // We place the censored hash in the rightmost bits of the composite hash.
            composite_hash |= censored_hash;

            debug_assert_eq!((composite_hash >> (hash_bits - P::EXPONENT - 1)) & 1, 1);
        } else {
            // In this case, the composite hash has the following structure:
            //
            // [index (exponent bits) | flag = 0 | original hash leading values]

            // We do not need to set the flag bit, as we initialized the composite
            // hash with zeros and that bit is certainly zero.

            // We shift the original hash to the right.
            let censored_hash =
                u32::try_from((!original_hash) >> (64 - hash_bits + 1 + P::EXPONENT)).unwrap();

            // We place the censored hash in the rightmost bits of the composite hash.
            composite_hash |= censored_hash;

            debug_assert_eq!((composite_hash >> (hash_bits - P::EXPONENT - 1)) & 1, 0);
        }

        debug_assert!(
            composite_hash.leading_zeros() >= u32::from(32 - hash_bits),
            "The composite hash {composite_hash} must have at least {} leading zeros, but has only {}",
            32 - hash_bits,
            composite_hash.leading_zeros(),
        );

        composite_hash
    }

    #[must_use]
    #[inline]
    /// Decode the hash into the register value and index.
    pub(super) fn decode(hash: u32, hash_bits: u8) -> (u8, usize) {
        debug_assert!(
            hash.leading_zeros() >= u32::from(32 - hash_bits),
            "The hash ({hash:032b}) should be composed of {hash_bits} bits and therefore must have at least {} leading zeros, but has only {}",
            32 - hash_bits,
            hash.leading_zeros(),
        );

        let fragmented = Self::scompose_hash(hash, hash_bits);

        debug_assert!(
            fragmented.index < 1 << P::EXPONENT,
            "While decoding the hash ({hash:032b}), the index ({}) must be less than 2^({})",
            fragmented.index,
            P::EXPONENT
        );

        (fragmented.register as u8, fragmented.index as usize)
    }
}

/// Iterator variants.
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub(super) enum IterVariants<'a> {
    /// Variant for 8-bit hashes.
    u8(Iter<'a, u8>),
    /// Variant for 16-bit hashes.
    u16(Iter<'a, u16>),
    /// Variant for 24-bit hashes.
    u24(Iter<'a, [u8; 3]>),
    /// Variant for 32-bit hashes.
    u32(Iter<'a, u32>),
}

impl<'a> From<Iter<'a, u8>> for IterVariants<'a> {
    fn from(iter: Iter<'a, u8>) -> Self {
        IterVariants::u8(iter)
    }
}

impl<'a> From<Iter<'a, u16>> for IterVariants<'a> {
    fn from(iter: Iter<'a, u16>) -> Self {
        IterVariants::u16(iter)
    }
}

impl<'a> From<Iter<'a, [u8; 3]>> for IterVariants<'a> {
    fn from(iter: Iter<'a, [u8; 3]>) -> Self {
        IterVariants::u24(iter)
    }
}

impl<'a> From<Iter<'a, u32>> for IterVariants<'a> {
    fn from(iter: Iter<'a, u32>) -> Self {
        IterVariants::u32(iter)
    }
}

impl<'a> IterVariants<'a> {
    #[inline]
    pub(super) const fn hash_bits(&self) -> u8 {
        match self {
            IterVariants::u8(_) => 8,
            IterVariants::u16(_) => 16,
            IterVariants::u24(_) => 24,
            IterVariants::u32(_) => 32,
        }
    }
}

impl Iterator for IterVariants<'_> {
    type Item = u32;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterVariants::u8(iter) => iter.next().copied().map(u32::from),
            IterVariants::u16(iter) => iter.next().copied().map(u32::from),
            IterVariants::u24(iter) => iter.next().map(|hash| {
                let mut bytes = [0; 4];
                bytes[..3].copy_from_slice(hash);
                u32::from_le_bytes(bytes)
            }),
            IterVariants::u32(iter) => iter.next().copied(),
        }
    }
}

impl<'a> ExactSizeIterator for IterVariants<'a> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            IterVariants::u8(iter) => iter.len(),
            IterVariants::u16(iter) => iter.len(),
            IterVariants::u24(iter) => iter.len(),
            IterVariants::u32(iter) => iter.len(),
        }
    }
}

#[derive(Debug)]
/// An iterator over the decoded hashes.
pub struct DecodedIter<'a, P, B> {
    /// The iterator variant.
    variant: IterVariants<'a>,
    /// The initial length of the iterator.
    number_of_hashes: usize,
    /// Precision and Bits markers
    _phantom: core::marker::PhantomData<&'a (P, B)>,
}

impl<'a, P, B> LastBufferedBit for DecodedIter<'a, P, B> {
    #[inline]
    fn last_buffered_bit(&self) -> u32 {
        (self.number_of_hashes - self.variant.len()) as u32 * u32::from(self.variant.hash_bits())
    }

    #[inline]
    fn hash_bits(&self) -> u8 {
        self.variant.hash_bits()
    }
}

impl<'a, P: Precision, B: Bits> Iterator for DecodedIter<'a, P, B> {
    type Item = (u8, usize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.variant.next().map(|hash| {
            let (register, index) = SwitchHash::<P, B>::decode(hash, self.variant.hash_bits());
            (register, index)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.variant.size_hint()
    }
}

impl<'a, P, B> From<IterVariants<'a>> for DecodedIter<'a, P, B> {
    fn from(variant: IterVariants<'a>) -> Self {
        DecodedIter {
            number_of_hashes: variant.len(),
            variant,
            _phantom: core::marker::PhantomData,
        }
    }
}

#[derive(Debug)]
/// An iterator over the decoded hashes.
pub struct DowngradedIter<'a, P, B> {
    /// The iterator variant.
    variant: IterVariants<'a>,
    /// The number of bits to shift.
    shift: u8,
    /// The number of hashes when the iterator is newly created.
    number_of_hashes: usize,
    /// The composite hash type.
    _phantom: core::marker::PhantomData<&'a (P, B)>,
}

impl<'a, P, B> LastBufferedBit for DowngradedIter<'a, P, B> {
    #[inline]
    fn last_buffered_bit(&self) -> u32 {
        (self.number_of_hashes - self.variant.len()) as u32 * u32::from(self.variant.hash_bits())
    }

    #[inline]
    fn hash_bits(&self) -> u8 {
        self.variant.hash_bits()
    }
}

impl<'a, P: Precision, B: Bits> Iterator for DowngradedIter<'a, P, B> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.variant
            .next()
            .map(|hash| SwitchHash::<P, B>::downgrade(hash, self.variant.hash_bits(), self.shift))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.variant.size_hint()
    }
}

impl<'a, P, B> DowngradedIter<'a, P, B> {
    #[inline]
    /// Create a new iterator from the provided iterator and the number of bits to shift.
    pub(super) fn new(variant: IterVariants<'a>, shift: u8) -> Self {
        DowngradedIter {
            number_of_hashes: variant.len(),
            variant,
            shift,
            _phantom: core::marker::PhantomData,
        }
    }
}

#[inline]
#[must_use]
#[allow(unsafe_code)]
pub(super) fn into_variant(
    hashes: &[u8],
    number_of_hashes: u32,
    hash_bits: u8,
) -> IterVariants<'_> {
    assert!(hash_bits >= 8);
    assert!(hash_bits == 8 || hash_bits == 16 || hash_bits == 24 || hash_bits == 32);
    let hashes = &hashes[..usize::try_from(number_of_hashes).unwrap() * usize::from(hash_bits / 8)];
    match hash_bits {
        8 => IterVariants::u8(hashes.iter()),
        16 => {
            let hashes: &[u16] = unsafe {
                core::slice::from_raw_parts(hashes.as_ptr().cast::<u16>(), hashes.len() / 2)
            };
            IterVariants::u16(hashes.iter())
        }
        24 => {
            let hashes: &[[u8; 3]] = unsafe {
                core::slice::from_raw_parts(hashes.as_ptr().cast::<[u8; 3]>(), hashes.len() / 3)
            };
            IterVariants::u24(hashes.iter())
        }
        32 => {
            let hashes: &[u32] = unsafe {
                core::slice::from_raw_parts(hashes.as_ptr().cast::<u32>(), hashes.len() / 4)
            };
            IterVariants::u32(hashes.iter())
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod switch_hash_test {
    use super::*;
    use crate::prelude::{Bits4, Precision4};

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
