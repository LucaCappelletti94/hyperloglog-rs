//! Gap-based composite hash implementation.
use core::marker::PhantomData;
use core::u64;
mod bitreader;
mod bitwriter;
mod optimal_codes;
use super::gap_birthday_paradox::{
    GAP_HASH_BIRTHDAY_PARADOX_CARDINALITIES, GAP_HASH_BIRTHDAY_PARADOX_ERRORS,
};
use super::{
    switch::HashFragment, CompositeHash, CompositeHashError, Debug, LastBufferedBit, Precision,
    SwitchHash,
};
use crate::{bits::Bits, utils::ceil};
use bitreader::{len_rice, BitReader};
use bitwriter::BitWriter;
use optimal_codes::{OPTIMAL_RICE_COEFFICIENTS, OPTIMAL_VBYTE_RICE_COEFFICIENTS};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Gap-based composite hash.
pub struct GapHash<P: Precision, B: Bits, const VBYTE: bool> {
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
        debug_assert!(previous_hash > hash_to_encode);

        let previous_fragment = SwitchHash::<P, B>::scompose_hash(previous_hash, hash_bits);
        let fragment_to_encode = SwitchHash::<P, B>::scompose_hash(hash_to_encode, hash_bits);

        debug_assert!(
            previous_fragment.index > fragment_to_encode.index
                || previous_fragment.register > fragment_to_encode.register
                || previous_fragment.hash_remainder > fragment_to_encode.hash_remainder,
            "The previous register ({}) must be greater or equal to the second register ({})",
            previous_fragment.register,
            fragment_to_encode.register
        );

        // When P::EXPONENT + B::NUMBER_OF_BITS == hash_bits, there is absolutely
        // no hash remainder to include in the uniform portion of the hash, as that
        // part of the hash is solely composed of the index.
        if P::EXPONENT + B::NUMBER_OF_BITS == hash_bits {
            let uniform =
                u64::try_from(previous_fragment.index - fragment_to_encode.index).unwrap();

            let geometric = if previous_fragment.index == fragment_to_encode.index {
                previous_fragment.register - fragment_to_encode.register - 1
            } else {
                fragment_to_encode.register - 1
            };

            GapFragment { uniform, geometric }
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
                    debug_assert!(
                        previous_fragment.register > fragment_to_encode.register,
                        "When the indices are equal and the previous hash remainder ({}) is less or equal to the hash remainder to encode ({}), the previous register ({}) must be strictly greater than the register to encode ({}), otherwise the order would not be maintained.",
                        previous_fragment.hash_remainder,
                        fragment_to_encode.hash_remainder,
                        previous_fragment.register,
                        fragment_to_encode.register
                    );
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
    fn b(hash_bits: u8) -> (u8, u8, u8) {
        let data = if VBYTE {
            OPTIMAL_VBYTE_RICE_COEFFICIENTS[P::EXPONENT as usize - 4]
                [B::NUMBER_OF_BITS as usize - 4]
        } else {
            OPTIMAL_RICE_COEFFICIENTS[P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4]
        };

        for (target_hash_bits, uniform, geometric) in data {
            if *target_hash_bits == hash_bits {
                return (*target_hash_bits, *uniform, *geometric);
            }
        }

        unreachable!("The hash bits ({hash_bits}) must be one of the optimal hash bits.",);
    }

    #[inline]
    fn uniform_coefficient(hash_bits: u8) -> u8 {
        Self::b(hash_bits).1
    }

    #[inline]
    fn geometric_coefficient(hash_bits: u8) -> u8 {
        Self::b(hash_bits).2
    }

    #[inline]
    fn has_rice_coefficients() -> bool {
        !if VBYTE {
            OPTIMAL_VBYTE_RICE_COEFFICIENTS[P::EXPONENT as usize - 4]
                [B::NUMBER_OF_BITS as usize - 4]
                .is_empty()
        } else {
            OPTIMAL_RICE_COEFFICIENTS[P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4]
                .is_empty()
        }
    }

    #[inline]
    fn next_hash_bits(candidate_hash_bits: u8) -> u8 {
        let data = if VBYTE {
            OPTIMAL_VBYTE_RICE_COEFFICIENTS[P::EXPONENT as usize - 4]
                [B::NUMBER_OF_BITS as usize - 4]
        } else {
            OPTIMAL_RICE_COEFFICIENTS[P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4]
        };

        for (target_hash_bits, _, _) in data.iter().rev() {
            if candidate_hash_bits >= *target_hash_bits {
                return *target_hash_bits;
            }
        }

        unreachable!("The hash bits ({candidate_hash_bits}) must be one of the optimal hash bits.",);
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
        debug_assert!(!VBYTE || bit_index % 8 == 0);

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

                    if !Self::has_rice_coefficients() {
                        return Err(CompositeHashError::Saturation);
                    }

                    // We check whether we can dowgrade to the current hash bits, or if we need to
                    // downgrade to a smaller hash bits.
                    let next_hash_bits = Self::next_hash_bits(hash_bits);
                    if hash_bits > next_hash_bits {
                        return Err(CompositeHashError::DowngradableSaturation);
                    };

                    let (duplicates, new_writer_tell) =
                        Self::downgrade_inplace(hashes, number_of_hashes, bit_index, hash_bits, 0);

                    debug_assert_eq!(
                        duplicates, 0,
                        "There should be no duplicates when first prefix-coding a new value."
                    );

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
        let mut iter: PrefixCodeIter<'_, P, B, VBYTE> =
            PrefixCodeIter::new(hashes_ref, number_of_hashes, hash_bits);

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
            prev_to_current_gap.map_or(
                {
                    let size = usize::from(hash_bits);
                    if VBYTE {
                        ceil(size, 8) * 8
                    } else {
                        size
                    }
                },
                |prev_to_current_gap| {
                    let size = len_rice(
                        prev_to_current_gap.uniform,
                        Self::uniform_coefficient(hash_bits),
                    ) + len_rice(
                        u64::from(prev_to_current_gap.geometric),
                        Self::geometric_coefficient(hash_bits),
                    );
                    if VBYTE {
                        ceil(size, 8) * 8
                    } else {
                        size
                    }
                },
            ) + current_to_next_gap.map_or(0, |current_to_next_gap| {
                let size = len_rice(
                    current_to_next_gap.uniform,
                    Self::uniform_coefficient(hash_bits),
                ) + len_rice(
                    u64::from(current_to_next_gap.geometric),
                    Self::geometric_coefficient(hash_bits),
                );
                if VBYTE {
                    ceil(size, 8) * 8
                } else {
                    size
                }
            }) - prev_to_next_gap.map_or(0, |prev_to_next_gap| {
                let size = len_rice(
                    prev_to_next_gap.uniform,
                    Self::uniform_coefficient(hash_bits),
                ) + len_rice(
                    u64::from(prev_to_next_gap.geometric),
                    Self::geometric_coefficient(hash_bits),
                );
                if VBYTE {
                    ceil(size, 8) * 8
                } else {
                    size
                }
            }) - if prev_to_current_gap.is_none() && current_to_next_gap.is_some() {
                let size = usize::from(hash_bits);
                if VBYTE {
                    ceil(size, 8) * 8
                } else {
                    size
                }
            } else {
                0
            };

        debug_assert!(
            !VBYTE || number_of_inserted_bits % 8 == 0,
            "The number of inserted bits ({number_of_inserted_bits}) must be a multiple of 8 when using vbyte."
        );

        let new_bit_index = bit_index + number_of_inserted_bits;

        if new_bit_index > hashes_ref.len() * 8 {
            if hash_bits == Self::SMALLEST_VIABLE_HASH_BITS {
                return Err(CompositeHashError::Saturation);
            }
            return Err(CompositeHashError::DowngradableSaturation);
        }

        if VBYTE {
            // If we are using VBYTE-ed hashes, instead of completing linearly the rewrite of the codes,
            // we can simply shift the codes to the right and insert the new value at the beginning.
            hashes.copy_within(
                last_read_bit_position / 8..bit_index / 8,
                (last_read_bit_position + number_of_inserted_bits) / 8,
            );
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
        let mut bypass: BypassIter<'_> = iter.into_bypass(bit_index);
        // In order to bring the reader a bit more ahead and make more unlikely to get
        // read-write conflicts, we read the next value.
        let mut next = bypass.next();

        // If there is no previos value, we would need to write the encoded value itself but
        // writing such a high value in prefix-free encoding would be inefficient. Therefore,
        // we write the first hash explicitly.
        if let Some(prev_to_current_gap) = prev_to_current_gap {
            if position == 1 {
                if VBYTE {
                    debug_assert_eq!(
                        writer.tell(),
                        ceil(usize::from(hash_bits), 8) * 8,
                        "The writer tell must be 0 if there is a single previous value"
                    );
                } else {
                    debug_assert_eq!(
                        writer.tell(),
                        usize::from(hash_bits),
                        "The writer tell must be {hash_bits} (the hash bits) if there is a single previous value"
                    );
                }
            }

            let wrote_uniform = writer.write_rice(
                prev_to_current_gap.uniform,
                Self::uniform_coefficient(hash_bits),
            );
            let wrote_geometric = writer.write_rice(
                u64::from(prev_to_current_gap.geometric),
                Self::geometric_coefficient(hash_bits),
            );

            let mut total_wrote = wrote_uniform + wrote_geometric;

            // If we are using vbyte, we need to pad to the byte size.
            if VBYTE {
                let padding = ceil(total_wrote, 8) * 8 - total_wrote;
                total_wrote += writer.write_bits(0, padding);
            }

            debug_assert!(
                bypass.len() == 0 || bypass.last_buffered_bit() >= writer.tell(),
                "{position}/{number_of_hashes}) Reader tell ({}) must be greater than writer tell ({}, just wrote {total_wrote}) in insert at hash size {hash_bits}.",
                bypass.last_buffered_bit(),
                writer.tell(),
            );
        } else {
            debug_assert_eq!(
                writer.tell(),
                0,
                "The writer tell must be 0 if there is no previous value"
            );

            let wrote_bits = writer.write_bits(encoded, usize::from(hash_bits));

            if VBYTE {
                let padding = ceil(wrote_bits, 8) * 8 - wrote_bits;
                writer.write_bits(0, padding);
            }
        }

        if let Some(current_to_next_gap) = current_to_next_gap {
            position += 1;

            let wrote_uniform = writer.write_rice(
                current_to_next_gap.uniform,
                Self::uniform_coefficient(hash_bits),
            );
            let wrote_geometric = writer.write_rice(
                u64::from(current_to_next_gap.geometric),
                Self::geometric_coefficient(hash_bits),
            );

            let mut total_wrote = wrote_uniform + wrote_geometric;

            // If we are using vbyte, we need to pad to the byte size.
            if VBYTE {
                let padding = ceil(total_wrote, 8) * 8 - total_wrote;
                total_wrote += writer.write_bits(0, padding);
            }

            debug_assert!(
                bypass.len() == 0 || bypass.last_buffered_bit() > writer.tell(),
                "{position}/{number_of_hashes}) Reader tell ({}) must be greater than writer tell ({}, just wrote {total_wrote}) in insert at hash size {hash_bits}.",
                bypass.last_buffered_bit(),
                writer.tell(),
            );

            if !VBYTE {
                // If our hashes are not vbyted, we have to write all of the remaining hashes one-by-one
                // as we could not just shift the hashes to the right.
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
            let wrote_bits = writer.write_bits(value, usize::from(hash_bits - shift));

            if VBYTE {
                let padding = ceil(wrote_bits, 8) * 8 - wrote_bits;
                writer.write_bits(0, padding);
            }

            value
        } else {
            debug_assert_eq!(bit_index, 0);
            debug_assert_eq!(number_of_hashes, 0);
            return (0, 0);
        };

        let mut duplicates = 0;

        let mut maybe_next = iter.next();
        let mut maybe_double_next = iter.next();

        #[cfg(test)]
        #[cfg(feature = "std")]
        println!("Downgrading at hash size {hash_bits} with shift {shift}.");

        while let Some(next) = maybe_next {
            maybe_next = maybe_double_next;
            maybe_double_next = iter.next();
            position += 1;

            if next == previous_hash {
                duplicates += 1;
                continue;
            }

            let fragment = Self::into_gap_fragment(previous_hash, next, hash_bits - shift);

            let wrote_uniform = writer.write_rice(
                fragment.uniform,
                Self::uniform_coefficient(hash_bits - shift),
            );
            let wrote_geometric = writer.write_rice(
                u64::from(fragment.geometric),
                Self::geometric_coefficient(hash_bits - shift),
            );

            let mut total_wrote = wrote_uniform + wrote_geometric;

            // If we are using vbyte, we need to pad to the byte size.
            if VBYTE {
                let padding = ceil(total_wrote, 8) * 8 - total_wrote;
                total_wrote += writer.write_bits(0, padding);
            }

            #[cfg(test)]
            #[cfg(feature = "std")]
            println!(
                "Just wrote {wrote_uniform} + {wrote_geometric} = {total_wrote}, {previous_hash} - {next}"
            );

            debug_assert!(
                iter.last_buffered_bit() >= writer.tell(),
                "{position}/{number_of_hashes}) Reader tell ({}) must be greater than writer tell ({}, just wrote {wrote_uniform} + {wrote_geometric} = {total_wrote}, {previous_hash} - {next}) in downgrade at hash size {hash_bits} with shift {shift}. Precision: {}, Bits: {}.",
                iter.last_buffered_bit(),
                writer.tell(),
                P::EXPONENT,
                B::NUMBER_OF_BITS
            );
            previous_hash = next;
        }

        let writer_tell = writer.tell();
        drop(writer);

        debug_assert!(
            writer_tell <= bit_index,
            "PFC-ing at bit size {hash_bits} with shift {shift} should decrease the bit index ({bit_index}), but got writer tell {writer_tell}. Precision: {}, Bits: {}.",
            P::EXPONENT,
            B::NUMBER_OF_BITS
        );

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

    #[inline]
    fn target_downgraded_hash_bits(
        _number_of_hashes: usize,
        _bit_index: usize,
        hash_bits: u8,
    ) -> u8 {
        Self::next_hash_bits(hash_bits - 1)
    }

    const SMALLEST_VIABLE_HASH_BITS: u8 = Self::Precision::EXPONENT + Self::Bits::NUMBER_OF_BITS;
    const LARGEST_VIABLE_HASH_BITS: u8 = SwitchHash::<P, B>::LARGEST_VIABLE_HASH_BITS;
    const BIRTHDAY_CARDINALITIES: &[u32] = GAP_HASH_BIRTHDAY_PARADOX_CARDINALITIES
        [P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4];
    const BIRTHDAY_RELATIVE_ERRORS: &[f64] =
        GAP_HASH_BIRTHDAY_PARADOX_ERRORS[P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4];
}

#[derive(Debug)]
/// Iterator over downgraded hashes.
pub enum DispatchedDowngradedIter<'a, P: Precision, B: Bits, const VBYTE: bool> {
    /// Variants for when the prefix-free codes are used.
    PrefixCodeDowngradedIter(PrefixCodeDowngradedIter<'a, P, B, VBYTE>),
    /// Variants for when the prefix-free codes are not used.
    InnerDowngradedIter(<SwitchHash<P, B> as CompositeHash>::Downgraded<'a>),
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> LastBufferedBit
    for DispatchedDowngradedIter<'a, P, B, VBYTE>
{
    fn last_buffered_bit(&self) -> usize {
        match self {
            Self::PrefixCodeDowngradedIter(iter) => iter.last_buffered_bit(),
            Self::InnerDowngradedIter(iter) => iter.last_buffered_bit(),
        }
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool>
    TryFrom<DispatchedDowngradedIter<'a, P, B, VBYTE>>
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

impl<'a, P: Precision, B: Bits, const VBYTE: bool> Iterator
    for DispatchedDowngradedIter<'a, P, B, VBYTE>
{
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

impl<'a, P: Precision, B: Bits, const VBYTE: bool> ExactSizeIterator
    for DispatchedDowngradedIter<'a, P, B, VBYTE>
{
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

#[derive(Debug)]
/// Iterator over downgraded hashes.
pub struct PrefixCodeDowngradedIter<'a, P: Precision, B: Bits, const VBYTE: bool> {
    iter: PrefixCodeIter<'a, P, B, VBYTE>,
    shift: u8,
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> LastBufferedBit
    for PrefixCodeDowngradedIter<'a, P, B, VBYTE>
{
    fn last_buffered_bit(&self) -> usize {
        self.iter.last_buffered_bit()
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> PrefixCodeDowngradedIter<'a, P, B, VBYTE> {
    #[allow(unsafe_code)]
    fn new(hashes: &'a [u8], number_of_hashes: usize, hash_bits: u8, shift: u8) -> Self {
        Self {
            iter: PrefixCodeIter::new(hashes, number_of_hashes, hash_bits),
            shift,
        }
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> Iterator
    for PrefixCodeDowngradedIter<'a, P, B, VBYTE>
{
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        Some(GapHash::<P, B, VBYTE>::downgrade(
            self.iter.next()?,
            self.iter.hash_bits,
            self.shift,
        ))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> ExactSizeIterator
    for PrefixCodeDowngradedIter<'a, P, B, VBYTE>
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

#[derive(Debug)]
/// Iterator over downgraded hashes.
pub struct PrefixCodeIter<'a, P: Precision, B: Bits, const VBYTE: bool> {
    bitstream: BitReader<'a>,
    previous: u64,
    number_of_hashes: usize,
    current_iteration: usize,
    hash_bits: u8,
    previous_index: usize,
    previous_register: u8,
    previous_hash_remainder: u16,
    previous_uniform: u64,
    _phantom: PhantomData<GapHash<P, B, VBYTE>>,
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> LastBufferedBit
    for PrefixCodeIter<'a, P, B, VBYTE>
{
    fn last_buffered_bit(&self) -> usize {
        self.bitstream.last_buffered_bit_position()
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> PrefixCodeIter<'a, P, B, VBYTE> {
    fn last_read_bit_position(&self) -> usize {
        self.bitstream.last_read_bit_position()
    }

    fn into_bypass(self, bit_index: usize) -> BypassIter<'a> {
        BypassIter {
            bitstream: self.bitstream,
            bit_index: bit_index,
        }
    }

    #[allow(unsafe_code)]
    fn new(hashes: &'a [u8], number_of_hashes: usize, hash_bits: u8) -> Self {
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
            previous_index: usize::MAX,
            previous_register: u8::MAX,
            previous_hash_remainder: u16::MAX,
            previous_uniform: u64::MAX,
            _phantom: PhantomData,
        }
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> Iterator for PrefixCodeIter<'a, P, B, VBYTE> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.number_of_hashes == self.current_iteration {
            return None;
        }
        self.current_iteration += 1;

        if self.current_iteration == 1 {
            self.previous = self.bitstream.read_bits(usize::from(self.hash_bits));

            if VBYTE {
                let padding =
                    ceil(usize::from(self.hash_bits), 8) * 8 - usize::from(self.hash_bits);
                let read_bits = self.bitstream.read_bits(padding);
                debug_assert_eq!(read_bits, 0);
            }

            let fragment = SwitchHash::<P, B>::scompose_hash(self.previous, self.hash_bits);
            self.previous_index = fragment.index;
            self.previous_register = fragment.register;
            self.previous_hash_remainder = fragment.hash_remainder;
            self.previous_uniform = fragment.uniform(self.hash_bits);

            return Some(self.previous);
        }

        let uniform = self
            .bitstream
            .read_rice(GapHash::<P, B, VBYTE>::uniform_coefficient(self.hash_bits));
        let geometric = u8::try_from(self.bitstream.read_rice(
            GapHash::<P, B, VBYTE>::geometric_coefficient(self.hash_bits),
        ))
        .unwrap();
        let after_codes_read_tell = self.bitstream.last_read_bit_position();

        if VBYTE {
            let padding = ceil(after_codes_read_tell, 8) * 8 - after_codes_read_tell;
            let read_bits = self.bitstream.read_bits(padding);
            debug_assert_eq!(read_bits, 0);
        }

        // When P::EXPONENT + B::NUMBER_OF_BITS == hash_bits, there is absolutely
        // no hash remainder to include in the uniform portion of the hash, as that
        // part of the hash is solely composed of the index.
        if P::EXPONENT + B::NUMBER_OF_BITS == self.hash_bits {
            let to_decode_index = self.previous_index - usize::try_from(uniform).unwrap();
            self.previous_register = if self.previous_index == to_decode_index {
                self.previous_register - geometric - 1
            } else {
                geometric + 1
            };

            self.previous_index = to_decode_index;
            self.previous =
                SwitchHash::<P, B>::compose_hash(to_decode_index, self.previous_register, 0, self.hash_bits);

            return Some(self.previous);
        }

        self.previous_uniform = if uniform & 1 == 0 {
            self.previous_uniform + (uniform >> 1)
        } else {
            self.previous_uniform - ((uniform >> 1) + 1)
        };
        let (to_decode_index, to_decode_hash_remainder) =
            HashFragment::<P, B>::scompose_uniform(self.previous_uniform, self.hash_bits);

        self.previous_register = if self.previous_index == to_decode_index {
            if self.previous_hash_remainder <= to_decode_hash_remainder {
                self.previous_register - geometric - 1
            } else {
                self.previous_register - geometric
            }
        } else {
            geometric + 1
        };

        self.previous_index = to_decode_index;
        self.previous_hash_remainder = to_decode_hash_remainder;

        self.previous = SwitchHash::<P, B>::compose_hash(
            to_decode_index,
            self.previous_register,
            to_decode_hash_remainder,
            self.hash_bits,
        );

        Some(self.previous)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.number_of_hashes - self.current_iteration,
            Some(self.number_of_hashes - self.current_iteration),
        )
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> ExactSizeIterator
    for PrefixCodeIter<'a, P, B, VBYTE>
{
    fn len(&self) -> usize {
        self.number_of_hashes - self.current_iteration
    }
}

#[derive(Debug)]
/// Iterator over decoded hashes.
pub enum DispatchedDecodedIter<'a, P: Precision, B: Bits, const VBYTE: bool> {
    /// Variants for when the prefix-free codes are used.
    PrefixCodeDecodedIter(PrefixCodeDecodedIter<'a, P, B, VBYTE>),
    /// Variants for when the prefix-free codes are not used.
    InnerDecodedIter(<SwitchHash<P, B> as CompositeHash>::Decoded<'a>),
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> LastBufferedBit
    for DispatchedDecodedIter<'a, P, B, VBYTE>
{
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
    type Error = <SwitchHash<P, B> as CompositeHash>::Decoded<'a>;

    fn try_from(value: DispatchedDecodedIter<'a, P, B, VBYTE>) -> Result<Self, Self::Error> {
        match value {
            DispatchedDecodedIter::PrefixCodeDecodedIter(iter) => Ok(iter),
            DispatchedDecodedIter::InnerDecodedIter(inner) => Err(inner),
        }
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> Iterator
    for DispatchedDecodedIter<'a, P, B, VBYTE>
{
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

impl<'a, P: Precision, B: Bits, const VBYTE: bool> ExactSizeIterator
    for DispatchedDecodedIter<'a, P, B, VBYTE>
{
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
    iter: PrefixCodeIter<'a, P, B, VBYTE>,
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> LastBufferedBit
    for PrefixCodeDecodedIter<'a, P, B, VBYTE>
{
    fn last_buffered_bit(&self) -> usize {
        self.iter.last_buffered_bit()
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> PrefixCodeDecodedIter<'a, P, B, VBYTE> {
    #[allow(unsafe_code)]
    fn new(hashes: &'a [u8], number_of_hashes: usize, hash_bits: u8) -> Self {
        Self {
            iter: PrefixCodeIter::new(hashes, number_of_hashes, hash_bits),
        }
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> Iterator
    for PrefixCodeDecodedIter<'a, P, B, VBYTE>
{
    type Item = (u8, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|hash| GapHash::<P, B, VBYTE>::decode(hash, self.iter.hash_bits))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, P: Precision, B: Bits, const VBYTE: bool> ExactSizeIterator
    for PrefixCodeDecodedIter<'a, P, B, VBYTE>
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}
