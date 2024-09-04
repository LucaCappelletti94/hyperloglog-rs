//! Gap-based composite hash implementation.
use core::marker::PhantomData;
mod bitreader;
mod bitwriter;
mod optimal_codes;
use super::{
    switch::HashFragment, CompositeHash, CompositeHashError, Debug, LastBufferedBit, Precision,
    SwitchHash,
};
use crate::{bits::Bits, utils::ceil};
use bitreader::{len_rice, len_unary, BitReader};
use bitwriter::BitWriter;
use optimal_codes::{OPTIMAL_RICE_COEFFICIENTS, OPTIMAL_VBYTE_RICE_COEFFICIENTS};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Gap-based composite hash.
pub struct GapHash<P: Precision, B: Bits, const VBYTE: bool = true> {
    switch: PhantomData<SwitchHash<P, B>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Struct representing the portions to be encoded in the gap encoding.
pub struct GapFragment {
    /// The bits expected to have uniform distribution.
    pub uniform: u64,
    /// The bits expected to have geometric distribution.
    pub geometric: u8,
}

impl<P: Precision, B: Bits, const VBYTE: bool> GapHash<P, B, VBYTE> {
    #[inline]
    /// Returns the gap encoding for the given SwitchHash.
    pub fn into_gap_fragment(
        previous_hash: u64,
        hash_to_encode: u64,
        hash_bits: u8,
    ) -> GapFragment {
        assert!(previous_hash > hash_to_encode);

        let previous_fragment = SwitchHash::<P, B>::scompose_hash(previous_hash, hash_bits);
        let fragment_to_encode = SwitchHash::<P, B>::scompose_hash(hash_to_encode, hash_bits);

        debug_assert!(
            previous_fragment.index >= fragment_to_encode.index,
            "The previous index ({}) must be greater or equal to the second index ({})",
            previous_fragment.index,
            fragment_to_encode.index
        );

        debug_assert!(
            previous_fragment.index > fragment_to_encode.index
                || previous_fragment.register >= fragment_to_encode.register,
            "The previous register ({}) must be greater or equal to the second register ({})",
            previous_fragment.register,
            fragment_to_encode.register
        );

        // When P::EXPONENT + B::NUMBER_OF_BITS == hash_bits, there is absolutely
        // no hash remainder to include in the uniform portion of the hash, as that
        // part of the hash is solely composed of the index. 
        if P::EXPONENT + B::NUMBER_OF_BITS == hash_bits {
            let uniform = u64::try_from(previous_fragment.index - fragment_to_encode.index).unwrap();

            let geometric = if previous_fragment.index == fragment_to_encode.index {
                previous_fragment.register - fragment_to_encode.register - 1
            } else {
                fragment_to_encode.register - 1
            };

            GapFragment {
                uniform,
                geometric
            }
        } else {
            // The uniform portion of the hash is composed by the index and the hash remainder.
            let previous_uniform = previous_fragment.uniform(hash_bits);
            let to_encode_uniform = fragment_to_encode.uniform(hash_bits);

            // The geometric portion of the hash is composed by the difference between the registers
            // when the indices are equal, otherwise it is the fragment to encode register itself.
            let geometric = if previous_fragment.index == fragment_to_encode.index {
                if previous_fragment.hash_remainder <= fragment_to_encode.hash_remainder {
                    // Since hashes must be strictly in descending order, and the indices are equal,
                    // and the previous hash remainder is equal or less than the hash remainder to encode,
                    // we can safely assume that the previous register is strictly greater than the register to encode.
                    previous_fragment.register - fragment_to_encode.register - 1
                } else {
                    // Otherwise, the previous register may be equal or greater to the register to encode.
                    previous_fragment.register - fragment_to_encode.register
                }
            } else {
                fragment_to_encode.register - 1
            };

            let uniform = if previous_uniform > to_encode_uniform {
                ((previous_uniform - to_encode_uniform) << 1) - 1
            } else {
                (to_encode_uniform - previous_uniform) << 1
            };
            GapFragment { uniform, geometric }
        }

    }

    #[inline]
    fn from_gap_fragment(
        fragment: GapFragment,
        previous_hash: u64,
        hash_bits: u8,
    ) -> u64 {
        let previous_fragment = SwitchHash::<P, B>::scompose_hash(previous_hash, hash_bits);

        // When P::EXPONENT + B::NUMBER_OF_BITS == hash_bits, there is absolutely
        // no hash remainder to include in the uniform portion of the hash, as that
        // part of the hash is solely composed of the index.
        if P::EXPONENT + B::NUMBER_OF_BITS == hash_bits {
            let to_decode_index = previous_fragment.index - usize::try_from(fragment.uniform).unwrap();
            let register = if previous_fragment.index == to_decode_index {
                previous_fragment.register - fragment.geometric - 1
            } else {
                fragment.geometric + 1
            };

            return SwitchHash::<P, B>::compose_hash(to_decode_index, register, 0, hash_bits);
        }

        let previous_uniform = previous_fragment.uniform(hash_bits);

        let to_decode_uniform = if fragment.uniform & 1 == 0 {
            previous_uniform + (fragment.uniform >> 1)
        } else {
            previous_uniform - ((fragment.uniform >> 1) + 1)
        };
        let (to_decode_index, to_decode_hash_remainder) = HashFragment::<P, B>::scompose_uniform(to_decode_uniform, hash_bits);

        let to_decode_register = if previous_fragment.index == to_decode_index {
            if previous_fragment.hash_remainder <= to_decode_hash_remainder {
                previous_fragment.register - fragment.geometric - 1
            } else {
                previous_fragment.register - fragment.geometric
            }
        } else {
            fragment.geometric + 1
        };

        SwitchHash::<P, B>::compose_hash(to_decode_index, to_decode_register, to_decode_hash_remainder, hash_bits)
    }
}

#[cfg(test)]
mod test_compose_scompose_gap {
    use crate::prelude::*;
    use crate::utils::iter_random_values;
    use hyperloglog_derive::test_precisions_and_bits;

    use super::*;

    #[test_precisions_and_bits]
    fn test_compose_scompose_gap<P: Precision, B: Bits>()
    where
        P: ArrayRegister<B>,
    {
        for (first, second) in iter_random_values::<u64>(10_000, None, None).zip(iter_random_values::<u64>(10_000, None, Some(675_398_754_524_577))) {
            let (first_index, first_register, first_original_hash) =
                <PlusPlus<P, B, <P as ArrayRegister<B>>::Packed>>::index_and_register_and_hash(
                    &first,
                );
            let (second_index, second_register, second_original_hash) =
                <PlusPlus<P, B, <P as ArrayRegister<B>>::Packed>>::index_and_register_and_hash(
                    &second,
                );
            for hash_bits in SwitchHash::<P, B>::SMALLEST_VIABLE_HASH_BITS
                ..=SwitchHash::<P, B>::LARGEST_VIABLE_HASH_BITS
            {
                let first_encoded_hash =
                    SwitchHash::<P, B>::encode(first_index, first_register, first_original_hash, hash_bits);
                let second_encoded_hash = SwitchHash::<P, B>::encode(
                    second_index,
                    second_register,
                    second_original_hash,
                    hash_bits,
                );

                // We for happenstance the two hashes are identical, which may happen while testing
                // lots of cases expecially for small hash sizes, we skip the test.
                if first_encoded_hash == second_encoded_hash {
                    continue;
                }

                let (smaller, larger) = if first_encoded_hash < second_encoded_hash {
                    (first_encoded_hash, second_encoded_hash)
                } else {
                    (second_encoded_hash, first_encoded_hash)
                };

                let fragment = GapHash::<P, B>::into_gap_fragment(larger, smaller, hash_bits);
                let decoded_hash = GapHash::<P, B>::from_gap_fragment(fragment, larger, hash_bits);

                assert_eq!(smaller, decoded_hash);
            }
        }
    }
}

impl<P: Precision, B: Bits, const VBYTE: bool> GapHash<P, B, VBYTE> {
    /// Returns whether the hashes are currently to be considered prefix-free-encoded.
    #[inline]
    #[must_use]
    pub fn is_prefix_free_encoded(
        number_of_hashes: usize,
        hash_bits: u8,
        bit_index: usize,
    ) -> bool {
        hash_bits < Self::LARGEST_VIABLE_HASH_BITS
            || number_of_hashes * usize::from(hash_bits) > bit_index
    }

    #[inline]
    const fn b(hash_bits: u8) -> u8 {
        if VBYTE {
            OPTIMAL_VBYTE_RICE_COEFFICIENTS[P::EXPONENT as usize - 4]
                [B::NUMBER_OF_BITS as usize - 4]
                [hash_bits as usize - Self::SMALLEST_VIABLE_HASH_BITS as usize]
        } else {
            OPTIMAL_RICE_COEFFICIENTS[P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4]
                [hash_bits as usize - Self::SMALLEST_VIABLE_HASH_BITS as usize]
        }
    }
}

impl<P: Precision, B: Bits, const VBYTE: bool> CompositeHash for GapHash<P, B, VBYTE> {
    type Precision = P;
    type Bits = B;

    type Decoded<'a> = DispatchedDecodedIter<'a, P, B, VBYTE>;
    type Downgraded<'a> = DispatchedDowngradedIter<'a, P, B, VBYTE>;

    #[inline]
    #[must_use]
    fn downgraded(
        hashes: &[u8],
        number_of_hashes: usize,
        hash_bits: u8,
        bit_index: usize,
        shift: u8,
    ) -> Self::Downgraded<'_> {
        // If we are employing prefix-free codes, we use the DowngradedIter
        if Self::is_prefix_free_encoded(number_of_hashes, hash_bits, bit_index) {
            DispatchedDowngradedIter::PrefixCodeDowngradedIter(PrefixCodeDowngradedIter::new(
                hashes,
                number_of_hashes,
                hash_bits,
                bit_index,
                shift,
            ))
        } else {
            DispatchedDowngradedIter::InnerDowngradedIter(SwitchHash::<P, B>::downgraded(
                hashes,
                number_of_hashes,
                hash_bits,
                bit_index,
                shift,
            ))
        }
    }

    #[inline]
    #[must_use]
    fn decoded(
        hashes: &[u8],
        number_of_hashes: usize,
        hash_bits: u8,
        bit_index: usize,
    ) -> Self::Decoded<'_> {
        assert!(
            hash_bits >= Self::SMALLEST_VIABLE_HASH_BITS,
            "The hash bits ({hash_bits}) must be greater or equal to the smallest viable hash bits ({})",
            Self::SMALLEST_VIABLE_HASH_BITS,
        );
        if Self::is_prefix_free_encoded(number_of_hashes, hash_bits, bit_index) {
            DispatchedDecodedIter::PrefixCodeDecodedIter(PrefixCodeDecodedIter::new(
                hashes,
                number_of_hashes,
                hash_bits,
                bit_index,
            ))
        } else {
            DispatchedDecodedIter::InnerDecodedIter(SwitchHash::<P, B>::decoded(
                hashes,
                number_of_hashes,
                hash_bits,
                bit_index,
            ))
        }
    }

    #[inline]
    #[must_use]
    /// Encode the hash from the provided register value, index and the original unsplitted hash.
    fn encode(index: usize, register: u8, original_hash: u64, hash_bits: u8) -> u64 {
        debug_assert!(register > 0);
        debug_assert!(
            index < 1 << Self::Precision::EXPONENT,
            "The index ({index}) must be less than 2^({})",
            Self::Precision::EXPONENT,
        );
        SwitchHash::<P, B>::encode(index, register, original_hash, hash_bits)
    }

    #[must_use]
    #[inline]
    /// Decode the hash into the register value and index.
    fn decode(hash: u64, hash_bits: u8) -> (u8, usize) {
        SwitchHash::<P, B>::decode(hash, hash_bits)
    }

    #[inline]
    #[must_use]
    /// Downgrade the hash into a smaller hash.
    fn downgrade(hash: u64, hash_bits: u8, shift: u8) -> u64 {
        SwitchHash::<P, B>::downgrade(hash, hash_bits, shift)
    }

    #[inline]
    #[allow(unsafe_code)]
    fn find(
        hashes: &[u8],
        number_of_hashes: usize,
        index: usize,
        register: u8,
        original_hash: u64,
        hash_bits: u8,
        bit_index: usize,
    ) -> Result<usize, (usize, u64)> {
        debug_assert!(register > 0);
        debug_assert!(
            index < 1 << Self::Precision::EXPONENT,
            "The index ({index}) must be less than 2^({})",
            Self::Precision::EXPONENT,
        );

        if Self::is_prefix_free_encoded(number_of_hashes, hash_bits, bit_index) {
            let encoded_hash = Self::encode(index, register, original_hash, hash_bits);
            Self::downgraded(hashes, number_of_hashes, hash_bits, bit_index, 0)
                .position(|hash| hash == encoded_hash)
                .map_or_else(|| Err((index, encoded_hash)), Ok)
        } else {
            SwitchHash::<P, B>::find(
                hashes,
                number_of_hashes,
                index,
                register,
                original_hash,
                hash_bits,
                bit_index,
            )
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn insert_sorted_desc(
        hashes: &mut [u8],
        number_of_hashes: usize,
        bit_index: usize,
        index: usize,
        register: u8,
        original_hash: u64,
        hash_bits: u8,
    ) -> Result<Option<usize>, CompositeHashError> {
        if !Self::is_prefix_free_encoded(number_of_hashes, hash_bits, bit_index) {
            return match SwitchHash::<P, B>::insert_sorted_desc(
                hashes,
                number_of_hashes,
                bit_index,
                index,
                register,
                original_hash,
                hash_bits,
            ) {
                Err(_) => {
                    assert!(
                        bit_index + usize::from(hash_bits) >= hashes.len() * 8,
                        "This method should only be called upon preliminary saturation, but was called with bit index {bit_index} and hash bits {hash_bits} and total available bits {}.",
                        hashes.len() * 8
                    );

                    // safe because the slice is originally allocated as u64s
                    debug_assert!(hashes.len() % core::mem::size_of::<u64>() == 0);
                    let hashes_64 = unsafe {
                        core::slice::from_raw_parts_mut(
                            hashes.as_mut_ptr() as *mut u64,
                            hashes.len() / core::mem::size_of::<u64>(),
                        )
                    };

                    debug_assert!(Self::downgraded(
                        hashes,
                        number_of_hashes,
                        hash_bits,
                        bit_index,
                        0
                    )
                    .is_sorted_by(|a, b| b < a));

                    let mut writer = BitWriter::new(hashes_64);

                    let mut iter = SwitchHash::<P, B>::downgraded(
                        hashes,
                        number_of_hashes,
                        hash_bits,
                        bit_index,
                        0,
                    );
                    let mut position = 0;

                    // We write the first hash explicitly, as otherwise it would be
                    // written in a very inefficient way.
                    let mut previous_hash = iter.next().unwrap();
                    position += 1;
                    writer.write_bits(previous_hash, usize::from(hash_bits));

                    for value in iter {
                        position += 1;

                        let fragment = Self::into_gap_fragment(previous_hash, value, hash_bits);

                        writer.write_rice(fragment.uniform, Self::b(hash_bits));
                        writer.write_unary(u64::from(fragment.geometric));
                        let last_buffered_bit_position = usize::from(hash_bits) * (1 + position);

                        debug_assert!(
                            last_buffered_bit_position >= writer.tell(),
                            "{position}/{number_of_hashes}) Reader tell ({}) must be greater than writer tell ({}, {previous_hash} - {value}) in prefix-coding at hash size {hash_bits}.",
                            last_buffered_bit_position,
                            writer.tell(),
                        );
                        previous_hash = value;
                    }

                    let new_writer_tell = writer.tell();
                    drop(writer);

                    debug_assert!(
                        new_writer_tell < bit_index,
                        "The conversion to prefix-free codes at bit size {hash_bits} should decrease the bit index, but got writer tell {new_writer_tell} and bit index {bit_index}."
                    );

                    debug_assert!(Self::downgraded(
                        hashes,
                        number_of_hashes,
                        hash_bits,
                        new_writer_tell,
                        0
                    )
                    .is_sorted_by(|a, b| b < a));

                    // And we try to insert the hash again.
                    Self::insert_sorted_desc(
                        hashes,
                        number_of_hashes,
                        new_writer_tell,
                        index,
                        register,
                        original_hash,
                        hash_bits,
                    )
                }
                result => result,
            };
        }

        debug_assert!(
            Self::is_prefix_free_encoded(number_of_hashes, hash_bits, bit_index),
            "The hashes must be prefix-free encoded to be able to use prefix-free codes."
        );

        // We check that all hashes are still ordered in descending order
        debug_assert!(
            Self::downgraded(hashes, number_of_hashes, hash_bits, bit_index, 0)
                .is_sorted_by(|a, b| b < a),
            "Illegal hashes state: attempting to insert a value with hash bits {hash_bits}, number of hashes {number_of_hashes} and bit index {bit_index} at index {index} and register {register} with original hash {original_hash}.",
        );

        let hashes_ref: &[u8] =
            unsafe { core::slice::from_raw_parts(hashes.as_ptr() as *const u8, hashes.len()) };

        let encoded = Self::encode(index, register, original_hash, hash_bits);

        // iter until we find where we should insert
        let mut iter: PrefixCodeDowngradedIter<'_, P, B, VBYTE> =
            Self::downgraded(hashes_ref, number_of_hashes, hash_bits, bit_index, 0)
                .try_into()
                .unwrap();

        let mut previous_hash = None;
        let mut next_value = None;

        let mut position = 0;
        let mut last_read_bit_position = 0;

        while let Some(value) = iter.next() {
            // The values are sorted in descending order, so we can stop when we find a value
            // that is less than or equal to the value we want to insert
            if encoded >= value {
                // if the value is equal to the encoded value, we don't need to insert it
                if value == encoded {
                    return Ok(None);
                }
                next_value = Some(value);
                break;
            }

            last_read_bit_position = iter.last_read_bit_position();
            previous_hash = Some(value);
            position += 1;
        }

        let prev_to_current_gap = previous_hash
            .map(|previous_hash| Self::into_gap_fragment(previous_hash, encoded, hash_bits));

        let current_to_next_gap =
            next_value.map(|next_value| Self::into_gap_fragment(encoded, next_value, hash_bits));

        let prev_to_next_gap = previous_hash.and_then(|previous_hash| {
            next_value
                .map(|next_value| Self::into_gap_fragment(previous_hash, next_value, hash_bits))
        });

        // We check that we would be actually able to insert the new value, given the current
        // bit index and the size the new value would require.
        let number_of_inserted_bits =
            prev_to_current_gap.map_or(usize::from(hash_bits), |prev_to_current_gap| {
                len_rice(prev_to_current_gap.uniform, Self::b(hash_bits))
                    + len_unary(prev_to_current_gap.geometric)
            }) + current_to_next_gap.map_or(0, |current_to_next_gap| {
                len_rice(current_to_next_gap.uniform, Self::b(hash_bits))
                    + len_unary(current_to_next_gap.geometric)
            }) - prev_to_next_gap.map_or(0, |prev_to_next_gap| {
                len_rice(prev_to_next_gap.uniform, Self::b(hash_bits))
                    + len_unary(prev_to_next_gap.geometric)
            });

        let new_bit_index = bit_index + number_of_inserted_bits;

        if new_bit_index > hashes_ref.len() * 8 {
            if hash_bits == Self::SMALLEST_VIABLE_HASH_BITS {
                return Err(CompositeHashError::Saturation);
            }
            return Err(CompositeHashError::DowngradableSaturation);
        }

        let hashes64 = unsafe {
            core::slice::from_raw_parts_mut(
                hashes.as_mut_ptr() as *mut u64,
                hashes.len() / core::mem::size_of::<u64>(),
            )
        };

        let mut writer = BitWriter::new(hashes64);

        writer.seek(last_read_bit_position);
        let insert_position = position;

        // Now that we have determined where to insert the new value, the subsequent values
        // will be solely read from the bitstream and written to the writer.
        let mut bypass: BypassIter<'_> = iter.into();
        // In order to bring the reader a bit more ahead and make more unlikely to get
        // read-write conflicts, we read the next value.
        let mut next = bypass.next();

        // If there is no previos value, we would need to write the encoded value itself but
        // writing such a high value in prefix-free encoding would be inefficient. Therefore,
        // we write the first hash explicitly.
        if let Some(prev_to_current_gap) = prev_to_current_gap {
            if position == 1 {
                debug_assert_eq!(
                    writer.tell(),
                    usize::from(hash_bits),
                    "The writer tell must be {hash_bits} (the hash bits) if there is a single previous value"
                );
            }

            writer.write_rice(prev_to_current_gap.uniform, Self::b(hash_bits));
            writer.write_unary(u64::from(prev_to_current_gap.geometric));

            debug_assert!(
                bypass.len() == 0 || bypass.last_buffered_bit() >= writer.tell(),
                "{position}/{number_of_hashes}) Reader tell ({}) must be greater than writer tell ({}) in insert at hash size {hash_bits}.",
                bypass.last_buffered_bit(),
                writer.tell(),
            );
        } else {
            debug_assert_eq!(
                writer.tell(),
                0,
                "The writer tell must be 0 if there is no previous value"
            );

            writer.write_bits(encoded, usize::from(hash_bits));
        }

        if let Some(current_to_next_gap) = current_to_next_gap {
            position += 1;

            writer.write_rice(current_to_next_gap.uniform, Self::b(hash_bits));
            writer.write_unary(u64::from(current_to_next_gap.geometric));
            debug_assert!(
                bypass.len() == 0 || bypass.last_buffered_bit() > writer.tell(),
                "{position}/{number_of_hashes}) Reader tell ({}) must be greater than writer tell ({}) in insert at hash size {hash_bits}.",
                bypass.last_buffered_bit(),
                writer.tell(),
            );

            while let Some((value, n_bits)) = next {
                next = bypass.next();
                writer.write_bits(value, n_bits);
                debug_assert!(
                    bypass.len() == 0 || bypass.last_buffered_bit() > writer.tell(),
                    "{position}/{number_of_hashes}) Reader tell ({}) must be greater than writer tell ({}) in insert at hash size {hash_bits}.",
                    bypass.last_buffered_bit(),
                    writer.tell(),
                );
                position += 1;
            }

            // We check that all hashes are still ordered in descending order
            let writer_tell = writer.tell();

            // We check that practice matches theory:
            assert_eq!(
                writer_tell,
                new_bit_index,
                "Expected writer tell to match bit index {bit_index} + value variation {number_of_inserted_bits} = ({new_bit_index})"
            );
        }

        drop(writer);

        debug_assert!(
            Self::downgraded(hashes, number_of_hashes + 1, hash_bits, new_bit_index, 0)
                .is_sorted_by(|a, b| b < a)
        );
        // We check if the decoded value was insert at position 'insert_position'
        debug_assert_eq!(
            Self::decoded(hashes, number_of_hashes + 1, hash_bits, new_bit_index)
                .nth(insert_position)
                .unwrap()
                .0,
            register
        );

        Ok(Some(new_bit_index))
    }

    #[inline]
    #[allow(unsafe_code)]
    /// Downgrade the hash into a smaller hash in place.
    fn downgrade_inplace(
        hashes: &mut [u8],
        number_of_hashes: usize,
        bit_index: usize,
        hash_bits: u8,
        shift: u8,
    ) -> (u32, usize) {
        if shift == 0 {
            return (0, bit_index);
        }

        if !Self::is_prefix_free_encoded(number_of_hashes, hash_bits, bit_index) {
            return SwitchHash::<P, B>::downgrade_inplace(
                hashes,
                number_of_hashes,
                bit_index,
                hash_bits,
                shift,
            );
        }

        // safe because the slice is originally allocated as u64s
        debug_assert!(hashes.len() % core::mem::size_of::<u64>() == 0);
        let hashes_64 = unsafe {
            core::slice::from_raw_parts_mut(
                hashes.as_mut_ptr() as *mut u64,
                hashes.len() / core::mem::size_of::<u64>(),
            )
        };

        let mut writer = BitWriter::new(hashes_64);

        debug_assert!(
            Self::downgraded(hashes, number_of_hashes, hash_bits, bit_index, 0)
                .is_sorted_by(|a, b| b < a)
        );

        let mut iter = Self::downgraded(hashes, number_of_hashes, hash_bits, bit_index, shift);
        let mut position = 0;

        // We write the first hash explicitly, as otherwise it would be
        // written in a very inefficient way.
        let mut previous_hash = if let Some(value) = iter.next() {
            writer.write_bits(value, usize::from(hash_bits - shift));
            value
        } else {
            debug_assert_eq!(bit_index, 0);
            debug_assert_eq!(number_of_hashes, 0);
            return (0, 0);
        };

        let mut duplicates = 0;

        while let Some(value) = iter.next() {
            position += 1;
            if value == previous_hash {
                duplicates += 1;
                continue;
            }

            let fragment = Self::into_gap_fragment(previous_hash, value, hash_bits);

            writer.write_rice(fragment.uniform, Self::b(hash_bits));
            writer.write_unary(u64::from(fragment.geometric));

            debug_assert!(
                iter.last_buffered_bit() > writer.tell(),
                "{position}/{number_of_hashes}) Reader tell ({}) must be greater than writer tell ({}, {previous_hash} - {value}) in downgrade at hash size {hash_bits} with shift {shift}.",
                iter.last_buffered_bit(),
                writer.tell(),
            );
            previous_hash = value;
        }

        let writer_tell = writer.tell();
        drop(writer);

        debug_assert!(Self::downgraded(
            hashes,
            number_of_hashes - duplicates,
            hash_bits - shift,
            writer_tell,
            0
        )
        .is_sorted_by(|a, b| b < a));

        (u32::try_from(duplicates).unwrap(), writer_tell)
    }

    const SMALLEST_VIABLE_HASH_BITS: u8 = Self::Precision::EXPONENT + Self::Bits::NUMBER_OF_BITS;
    const LARGEST_VIABLE_HASH_BITS: u8 = SwitchHash::<P, B>::LARGEST_VIABLE_HASH_BITS;
    const BIRTHDAY_CARDINALITIES: [u32; 6] = SwitchHash::<P, B>::BIRTHDAY_CARDINALITIES;
    const BIRTHDAY_RELATIVE_ERRORS: [f64; 6] = SwitchHash::<P, B>::BIRTHDAY_RELATIVE_ERRORS;
}

#[derive(Debug)]
/// Iterator over downgraded hashes.
pub enum DispatchedDowngradedIter<'a, P: Precision, B: Bits, const VBYTE: bool> {
    /// Variants for when the prefix-free codes are used.
    PrefixCodeDowngradedIter(PrefixCodeDowngradedIter<'a, P, B, VBYTE>),
    /// Variants for when the prefix-free codes are not used.
    InnerDowngradedIter(<SwitchHash::<P, B> as CompositeHash>::Downgraded<'a>),
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> LastBufferedBit for DispatchedDowngradedIter<'a, P, B, VBYTE> {
    fn last_buffered_bit(&self) -> usize {
        match self {
            Self::PrefixCodeDowngradedIter(iter) => iter.last_buffered_bit(),
            Self::InnerDowngradedIter(iter) => iter.last_buffered_bit(),
        }
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> TryFrom<DispatchedDowngradedIter<'a, P, B, VBYTE>>
    for PrefixCodeDowngradedIter<'a, P, B, VBYTE>
{
    type Error = DispatchedDowngradedIter<'a, P, B, VBYTE>;

    fn try_from(value: DispatchedDowngradedIter<'a, P, B, VBYTE>) -> Result<Self, Self::Error> {
        match value {
            DispatchedDowngradedIter::PrefixCodeDowngradedIter(iter) => Ok(iter),
            DispatchedDowngradedIter::InnerDowngradedIter(_) => Err(value),
        }
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> Iterator for DispatchedDowngradedIter<'a, P, B, VBYTE> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::PrefixCodeDowngradedIter(iter) => iter.next(),
            Self::InnerDowngradedIter(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::PrefixCodeDowngradedIter(iter) => iter.size_hint(),
            Self::InnerDowngradedIter(iter) => iter.size_hint(),
        }
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> ExactSizeIterator for DispatchedDowngradedIter<'a, P, B, VBYTE> {
    fn len(&self) -> usize {
        match self {
            Self::PrefixCodeDowngradedIter(iter) => iter.len(),
            Self::InnerDowngradedIter(iter) => iter.len(),
        }
    }
}

#[derive(Debug)]
/// Bypass iterator which instead of executing any operation on the [`BitReader`] stream,
/// just reads u64 words up until the end of the stream.
struct BypassIter<'a> {
    /// The bitstream to read from.
    bitstream: BitReader<'a>,
    /// The expected end of the current bit-stream.
    bit_index: usize,
}

impl Iterator for BypassIter<'_> {
    type Item = (u64, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.bitstream.last_read_bit_position() >= self.bit_index {
            return None;
        }
        let n_bits = core::cmp::min(64, self.bit_index - self.bitstream.last_read_bit_position());
        Some((self.bitstream.read_bits(n_bits), n_bits))
    }
}

impl ExactSizeIterator for BypassIter<'_> {
    fn len(&self) -> usize {
        ceil(
            self.bit_index
                .saturating_sub(self.bitstream.last_read_bit_position()),
            64,
        )
    }
}

impl<'a> LastBufferedBit for BypassIter<'a> {
    fn last_buffered_bit(&self) -> usize {
        self.bitstream.last_buffered_bit_position()
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> From<PrefixCodeDowngradedIter<'a, P, B, VBYTE>> for BypassIter<'a> {
    fn from(iter: PrefixCodeDowngradedIter<'a, P, B, VBYTE>) -> Self {
        Self {
            bitstream: iter.bitstream,
            bit_index: iter.bit_index,
        }
    }
}

#[derive(Debug)]
/// Iterator over downgraded hashes.
pub struct PrefixCodeDowngradedIter<'a, P: Precision, B: Bits, const VBYTE: bool> {
    bitstream: BitReader<'a>,
    previous: u64,
    number_of_hashes: usize,
    /// The expected number of bits to be read.
    bit_index: usize,
    current_iteration: usize,
    hash_bits: u8,
    shift: u8,
    _phantom: PhantomData<GapHash<P, B, VBYTE>>,
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> From<PrefixCodeDowngradedIter<'a, P, B, VBYTE>> for &'a [u8] {
    fn from(iter: PrefixCodeDowngradedIter<'a, P, B, VBYTE>) -> Self {
        iter.bitstream.into()
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> LastBufferedBit for PrefixCodeDowngradedIter<'a, P, B, VBYTE> {
    fn last_buffered_bit(&self) -> usize {
        self.bitstream.last_buffered_bit_position()
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> PrefixCodeDowngradedIter<'a, P, B, VBYTE> {
    fn last_read_bit_position(&self) -> usize {
        self.bitstream.last_read_bit_position()
    }

    #[allow(unsafe_code)]
    fn new(
        hashes: &'a [u8],
        number_of_hashes: usize,
        hash_bits: u8,
        bit_index: usize,
        shift: u8,
    ) -> Self {
        Self {
            previous: u64::MAX,
            number_of_hashes,
            current_iteration: 0,
            bitstream: BitReader::new(unsafe {
                core::slice::from_raw_parts_mut(
                    hashes.as_ptr() as *mut u32,
                    hashes.len() / core::mem::size_of::<u32>(),
                )
            }),
            hash_bits,
            bit_index,
            shift,
            _phantom: PhantomData,
        }
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> Iterator for PrefixCodeDowngradedIter<'a, P, B, VBYTE> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.number_of_hashes == self.current_iteration {
            return None;
        }
        self.current_iteration += 1;

        if self.current_iteration == 1 {
            self.previous = self.bitstream.read_bits(usize::from(self.hash_bits));
            return Some(GapHash::<P, B>::downgrade(self.previous, self.hash_bits, self.shift));
        }

        let uniform = self.bitstream.read_rice(GapHash::<P, B>::b(self.hash_bits));
        let geometric = u8::try_from(self.bitstream.read_unary()).unwrap();

        let fragment = GapFragment { uniform, geometric };

        self.previous = GapHash::<P, B>::from_gap_fragment(fragment, self.previous, self.hash_bits);

        debug_assert!(
            self.previous.leading_zeros() >= 64 - u32::from(self.hash_bits),
            "The hash ({}), being theoretically {} bits long, has more than {} leading zeros",
            self.previous,
            self.hash_bits,
            64 - u32::from(self.hash_bits),
        );

        Some(GapHash::<P, B>::downgrade(self.previous, self.hash_bits, self.shift))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.number_of_hashes, Some(self.number_of_hashes))
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> ExactSizeIterator for PrefixCodeDowngradedIter<'a, P, B, VBYTE> {
    fn len(&self) -> usize {
        self.number_of_hashes
    }
}

#[derive(Debug)]
/// Iterator over decoded hashes.
pub enum DispatchedDecodedIter<'a, P: Precision, B: Bits, const VBYTE: bool> {
    /// Variants for when the prefix-free codes are used.
    PrefixCodeDecodedIter(PrefixCodeDecodedIter<'a, P, B, VBYTE>),
    /// Variants for when the prefix-free codes are not used.
    InnerDecodedIter(<SwitchHash::<P, B> as CompositeHash>::Decoded<'a>),
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> LastBufferedBit for DispatchedDecodedIter<'a, P, B, VBYTE> {
    fn last_buffered_bit(&self) -> usize {
        match self {
            Self::PrefixCodeDecodedIter(iter) => iter.last_buffered_bit(),
            Self::InnerDecodedIter(iter) => iter.last_buffered_bit(),
        }
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> TryFrom<DispatchedDecodedIter<'a, P, B, VBYTE>>
    for PrefixCodeDecodedIter<'a, P, B, VBYTE>
{
    type Error = <SwitchHash::<P, B> as CompositeHash>::Decoded<'a>;

    fn try_from(value: DispatchedDecodedIter<'a, P, B, VBYTE>) -> Result<Self, Self::Error> {
        match value {
            DispatchedDecodedIter::PrefixCodeDecodedIter(iter) => Ok(iter),
            DispatchedDecodedIter::InnerDecodedIter(inner) => Err(inner),
        }
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> Iterator for DispatchedDecodedIter<'a, P, B, VBYTE> {
    type Item = (u8, usize);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::PrefixCodeDecodedIter(iter) => iter.next(),
            Self::InnerDecodedIter(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::PrefixCodeDecodedIter(iter) => iter.size_hint(),
            Self::InnerDecodedIter(iter) => iter.size_hint(),
        }
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> ExactSizeIterator for DispatchedDecodedIter<'a, P, B, VBYTE> {
    fn len(&self) -> usize {
        match self {
            Self::PrefixCodeDecodedIter(iter) => iter.len(),
            Self::InnerDecodedIter(iter) => iter.len(),
        }
    }
}

#[derive(Debug)]
/// Iterator over decoded hashes.
pub struct PrefixCodeDecodedIter<'a, P: Precision, B: Bits, const VBYTE: bool> {
    iter: PrefixCodeDowngradedIter<'a, P, B, VBYTE>,
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> LastBufferedBit for PrefixCodeDecodedIter<'a, P, B, VBYTE> {
    fn last_buffered_bit(&self) -> usize {
        self.iter.last_buffered_bit()
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> PrefixCodeDecodedIter<'a, P, B, VBYTE> {
    #[allow(unsafe_code)]
    fn new(hashes: &'a [u8], number_of_hashes: usize, hash_bits: u8, bit_index: usize) -> Self {
        Self {
            iter: PrefixCodeDowngradedIter::new(hashes, number_of_hashes, hash_bits, bit_index, 0),
        }
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> Iterator for PrefixCodeDecodedIter<'a, P, B, VBYTE> {
    type Item = (u8, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|hash| GapHash::<P, B>::decode(hash, self.iter.hash_bits - self.iter.shift))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> ExactSizeIterator for PrefixCodeDecodedIter<'a, P, B, VBYTE> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}
