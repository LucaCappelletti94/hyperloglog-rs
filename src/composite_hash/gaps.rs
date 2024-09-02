//! Gap-based composite hash implementation.
use core::marker::PhantomData;
mod bitreader;
mod bitwriter;
mod optimal_codes;
mod prefix_free_codes;
use crate::{bits::Bits, utils::ceil};
use bitreader::BitReader;
use bitwriter::BitWriter;
use prefix_free_codes::{CodeRead, CodeSize, CodeWrite};

use super::{CompositeHash, CompositeHashError, Debug, LastBufferedBit, Precision};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Gap-based composite hash.
pub struct GapHash<CH> {
    _phantom: PhantomData<CH>,
}

/// Trait defining the combination between a given combo of Precision
/// and Bits and which `PrefixFreeCode` to use for which combination.
pub trait PrefixFreeCode {
    /// Prefix-free code for when we are writing an hash of 8 bits.
    type Code8: CodeSize + CodeRead + CodeWrite + Default + 'static;
    /// Prefix-free code for when we are writing an hash of 16 bits.
    type Code16: CodeSize + CodeRead + CodeWrite + Default + 'static;
    /// Prefix-free code for when we are writing an hash of 24 bits.
    type Code24: CodeSize + CodeRead + CodeWrite + Default + 'static;
    /// Prefix-free code for when we are writing an hash of 32 bits.
    type Code32: CodeSize + CodeRead + CodeWrite + Default + 'static;
}

impl<CH: CompositeHash> GapHash<CH> {
    /// Returns whether the hashes are currently to be considered prefix-free-encoded.
    #[inline]
    #[must_use]
    pub fn is_prefix_free_encoded(
        number_of_hashes: usize,
        hash_bits: u8,
        bit_index: usize,
    ) -> bool {
        hash_bits < CH::LARGEST_VIABLE_HASH_BITS
            || number_of_hashes * usize::from(hash_bits) > bit_index
    }
}

pub(super) const fn smallest_viable_gap_hash<P: Precision, B: Bits>() -> u8 {
    assert!(P::EXPONENT >= 4);
    assert!(B::NUMBER_OF_BITS == 4 || B::NUMBER_OF_BITS == 5 || B::NUMBER_OF_BITS == 6);
    assert!(P::EXPONENT <= 18);
    match (P::EXPONENT, B::NUMBER_OF_BITS) {
        (4, 4) => 8,
        (4, 5) => 9,
        (4, 6) => 10,
        (5, 4) => 9,
        (5, 5) => 10,
        (5, 6) => 11,
        (6, 4) => 10,
        (6, 5) => 11,
        (6, 6) => 12,
        (7, 4) => 11,
        (7, 5) => 12,
        (7, 6) => 13,
        (8, 4) => 12,
        (8, 5) => 13,
        (8, 6) => 14,
        (9, 4) => 13,
        (9, 5) => 14,
        (9, 6) => 16,
        (10, 4) => 15,
        (10, 5) => 16,
        (10, 6) => 17,
        (11, 4) => 16,
        (11, 5) => 17,
        (11, 6) => 18,
        (12, 4) => 17,
        (12, 5) => 18,
        (12, 6) => 19,
        (13, 4) => 19,
        (13, 5) => 20,
        (13, 6) => 21,
        (14, 4) => 20,
        (14, 5) => 21,
        (14, 6) => 22,
        (15, 4) => 22,
        (15, 5) => 23,
        (15, 6) => 24,
        (16, 4) => 23,
        (16, 5) => 24,
        (16, 6) => 25,
        (17, 4) => 24,
        (17, 5) => 25,
        (17, 6) => 26,
        (18, 4) => 26,
        (18, 5) => 27,
        (18, 6) => 28,
        _ => unreachable!(),
    }
}

impl<CH: CompositeHash> CompositeHash for GapHash<CH>
where
    CH: PrefixFreeCode,
    Self: super::BirthDayParadoxCorrection,
{
    type Precision = <CH as CompositeHash>::Precision;
    type Bits = <CH as CompositeHash>::Bits;

    type Decoded<'a> = DispatchedDecodedIter<'a, CH>;
    type Downgraded<'a> = DispatchedDowngradedIter<'a, CH>;

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
            DispatchedDowngradedIter::InnerDowngradedIter(CH::downgraded(
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
            DispatchedDecodedIter::InnerDecodedIter(CH::decoded(
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
        CH::encode(index, register, original_hash, hash_bits)
    }

    #[must_use]
    #[inline]
    /// Decode the hash into the register value and index.
    fn decode(hash: u64, hash_bits: u8) -> (u8, usize) {
        CH::decode(hash, hash_bits)
    }

    #[inline]
    #[must_use]
    /// Downgrade the hash into a smaller hash.
    fn downgrade(hash: u64, hash_bits: u8, shift: u8) -> u64 {
        CH::downgrade(hash, hash_bits, shift)
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
            CH::find(
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
            match CH::insert_sorted_desc(
                hashes,
                number_of_hashes,
                bit_index,
                index,
                register,
                original_hash,
                hash_bits,
            ) {
                Err(_) => {
                    // Otherwise, we need to switch to prefix mode.
                    let new_writer_tell =
                        match hash_bits {
                            8 => to_prefix_code_inplace_with_writer::<
                                <CH as PrefixFreeCode>::Code8,
                                CH,
                            >(
                                hashes, number_of_hashes, bit_index, hash_bits
                            ),
                            16 => to_prefix_code_inplace_with_writer::<
                                <CH as PrefixFreeCode>::Code16,
                                CH,
                            >(
                                hashes, number_of_hashes, bit_index, hash_bits
                            ),
                            24 => to_prefix_code_inplace_with_writer::<
                                <CH as PrefixFreeCode>::Code24,
                                CH,
                            >(
                                hashes, number_of_hashes, bit_index, hash_bits
                            ),
                            32 => to_prefix_code_inplace_with_writer::<
                                <CH as PrefixFreeCode>::Code32,
                                CH,
                            >(
                                hashes, number_of_hashes, bit_index, hash_bits
                            ),
                            _ => unreachable!(),
                        };

                    // And we try to insert the hash again.
                    return Self::insert_sorted_desc(
                        hashes,
                        number_of_hashes,
                        new_writer_tell,
                        index,
                        register,
                        original_hash,
                        hash_bits,
                    );
                }
                result => return result,
            }
        }

        match hash_bits {
            8 => insert_sorted_desc_with_writer::<<CH as PrefixFreeCode>::Code8, CH>(
                hashes,
                number_of_hashes,
                bit_index,
                index,
                register,
                original_hash,
                hash_bits,
            ),
            16 => insert_sorted_desc_with_writer::<<CH as PrefixFreeCode>::Code16, CH>(
                hashes,
                number_of_hashes,
                bit_index,
                index,
                register,
                original_hash,
                hash_bits,
            ),
            24 => insert_sorted_desc_with_writer::<<CH as PrefixFreeCode>::Code24, CH>(
                hashes,
                number_of_hashes,
                bit_index,
                index,
                register,
                original_hash,
                hash_bits,
            ),
            32 => insert_sorted_desc_with_writer::<<CH as PrefixFreeCode>::Code32, CH>(
                hashes,
                number_of_hashes,
                bit_index,
                index,
                register,
                original_hash,
                hash_bits,
            ),
            _ => unreachable!(),
        }
    }

    #[inline]
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
            return CH::downgrade_inplace(hashes, number_of_hashes, bit_index, hash_bits, shift);
        }

        match hash_bits - shift {
            8 => downgrade_inplace_with_writer::<<CH as PrefixFreeCode>::Code8, CH>(
                hashes,
                number_of_hashes,
                bit_index,
                hash_bits,
                shift,
            ),
            16 => downgrade_inplace_with_writer::<<CH as PrefixFreeCode>::Code16, CH>(
                hashes,
                number_of_hashes,
                bit_index,
                hash_bits,
                shift,
            ),
            24 => downgrade_inplace_with_writer::<<CH as PrefixFreeCode>::Code24, CH>(
                hashes,
                number_of_hashes,
                bit_index,
                hash_bits,
                shift,
            ),
            _ => unreachable!(),
        }
    }

    const SMALLEST_VIABLE_HASH_BITS: u8 = smallest_viable_gap_hash::<Self::Precision, Self::Bits>();
    const LARGEST_VIABLE_HASH_BITS: u8 = CH::LARGEST_VIABLE_HASH_BITS;
}

#[allow(unsafe_code)]
fn downgrade_inplace_with_writer<CW: CodeWrite, CH: CompositeHash + PrefixFreeCode>(
    hashes: &mut [u8],
    number_of_hashes: usize,
    bit_index: usize,
    hash_bits: u8,
    shift: u8,
) -> (u32, usize)
where
    GapHash<CH>: super::BirthDayParadoxCorrection,
{
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
        GapHash::<CH>::downgraded(hashes, number_of_hashes, hash_bits, bit_index, 0)
            .is_sorted_by(|a, b| b < a)
    );

    let mut iter = GapHash::<CH>::downgraded(hashes, number_of_hashes, hash_bits, bit_index, shift);
    let mut position = 0;

    // We write the first hash explicitly, as otherwise it would be
    // written in a very inefficient way.
    let mut prev_value = if let Some(value) = iter.next() {
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
        if value == prev_value {
            duplicates += 1;
            continue;
        }

        let just_wrote_bits = CW::write(&mut writer, prev_value - value - 1);

        debug_assert!(
            iter.last_buffered_bit() > writer.tell(),
            "{position}/{number_of_hashes}) Reader tell ({}) must be greater than writer tell ({}, wrote {just_wrote_bits}, {prev_value} - {value}) in downgrade at hash size {hash_bits} with shift {shift}.",
            iter.last_buffered_bit(),
            writer.tell(),
        );
        prev_value = value;
    }

    let writer_tell = writer.tell();
    drop(writer);

    debug_assert!(GapHash::<CH>::downgraded(
        hashes,
        number_of_hashes - duplicates,
        hash_bits - shift,
        writer_tell,
        0
    )
    .is_sorted_by(|a, b| b < a));

    (u32::try_from(duplicates).unwrap(), writer_tell)
}

#[allow(unsafe_code)]
fn to_prefix_code_inplace_with_writer<CW: CodeWrite + 'static, CH: CompositeHash + PrefixFreeCode>(
    hashes: &mut [u8],
    number_of_hashes: usize,
    bit_index: usize,
    hash_bits: u8,
) -> usize
where
    GapHash<CH>: super::BirthDayParadoxCorrection,
{
    #[cfg(test)]
    #[cfg(feature = "std")]
    println!(
        "to_prefix_code_inplace_with_writer({}, {}, {}, {})",
        number_of_hashes,
        bit_index,
        hash_bits,
        hashes.len()
    );

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

    debug_assert!(
        GapHash::<CH>::downgraded(hashes, number_of_hashes, hash_bits, bit_index, 0)
            .is_sorted_by(|a, b| b < a)
    );

    let mut writer = BitWriter::new(hashes_64);

    let mut iter = CH::downgraded(hashes, number_of_hashes, hash_bits, bit_index, 0);
    let mut position = 0;

    // We write the first hash explicitly, as otherwise it would be
    // written in a very inefficient way.
    let mut prev_value = iter.next().unwrap();
    position += 1;
    writer.write_bits(prev_value, usize::from(hash_bits));

    for value in iter {
        position += 1;

        let just_wrote_bits = CW::write(&mut writer, prev_value - value - 1);
        let last_buffered_bit_position = usize::from(hash_bits) * (1 + position);

        debug_assert!(
            last_buffered_bit_position >= writer.tell(),
            "{position}/{number_of_hashes}) Reader tell ({}) must be greater than writer tell ({}, wrote {just_wrote_bits}, {prev_value} - {value}) in prefix-coding at hash size {hash_bits}.",
            last_buffered_bit_position,
            writer.tell(),
        );
        prev_value = value;
    }

    let writer_tell = writer.tell();
    drop(writer);

    debug_assert!(
        writer_tell < bit_index,
        "The conversion to prefix-free codes at bit size {hash_bits} should decrease the bit index, but got writer tell {writer_tell} and bit index {bit_index}."
    );

    debug_assert!(
        GapHash::<CH>::downgraded(hashes, number_of_hashes, hash_bits, writer_tell, 0)
            .is_sorted_by(|a, b| b < a)
    );

    writer_tell
}

#[allow(unsafe_code)]
fn insert_sorted_desc_with_writer<
    'a,
    CW: CodeWrite + CodeSize + 'static,
    CH: CompositeHash + PrefixFreeCode,
>(
    hashes: &'a mut [u8],
    number_of_hashes: usize,
    bit_index: usize,
    index: usize,
    register: u8,
    original_hash: u64,
    hash_bits: u8,
) -> Result<Option<usize>, CompositeHashError>
where
    GapHash<CH>: super::BirthDayParadoxCorrection,
{
    debug_assert!(
        GapHash::<CH>::is_prefix_free_encoded(number_of_hashes, hash_bits, bit_index),
        "The hashes must be prefix-free encoded to be able to use prefix-free codes."
    );
    debug_assert!(register > 0);
    debug_assert!(
        index < 1 << <CH as CompositeHash>::Precision::EXPONENT,
        "The index ({index}) must be less than 2^({})",
        <CH as CompositeHash>::Precision::EXPONENT,
    );
    // safe because the slice is originally allocated as u64s
    debug_assert!(
        hashes.len() % core::mem::size_of::<u64>() == 0,
        "Expected the length of the hashes to be a multiple of the size of u64, but got {}",
        hashes.len()
    );

    // We check that all hashes are still ordered in descending order
    debug_assert!(
        GapHash::<CH>::downgraded(hashes, number_of_hashes, hash_bits, bit_index, 0)
            .is_sorted_by(|a, b| b < a),
        "Illegal hashes state: attempting to insert a value with hash bits {hash_bits}, number of hashes {number_of_hashes} and bit index {bit_index} at index {index} and register {register} with original hash {original_hash}.",
    );

    let hashes_ref: &[u8] =
        unsafe { core::slice::from_raw_parts(hashes.as_ptr() as *const u8, hashes.len()) };

    let encoded = CH::encode(index, register, original_hash, hash_bits);

    // iter until we find where we should insert
    let mut iter: PrefixCodeDowngradedIter<'a, CH> =
        GapHash::<CH>::downgraded(hashes_ref, number_of_hashes, hash_bits, bit_index, 0)
            .try_into()
            .unwrap();

    let mut prev_value = u64::MAX;
    let mut next_value = u64::MAX;

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
            next_value = value;
            break;
        }

        last_read_bit_position = iter.last_read_bit_position();
        prev_value = value;
        position += 1;
    }

    // We check that we would be actually able to insert the new value, given the current
    // bit index and the size the new value would require. Note that this is NOT the number
    // of bits that would be required to encode the value, but the number of bits that will
    // be added to the bitstream. This is strictly greater than the number of bits that will
    // be changed in the bitstream.

    let number_of_inserted_bits: usize = if prev_value == u64::MAX {
        // If we are inserting this value as the first value, and there is a next value,
        // we need to take into account that this first value would require 'hash_bits' bits
        // and that the subsequent value would require a variable amount of bits depending
        // of the current prefix-free code employed.
        if next_value == u64::MAX {
            usize::from(hash_bits)
        } else {
            CW::size(encoded - next_value - 1)
        }
    } else {
        // If we are inserting this value in the middle of the list, we need to take into account
        // that the previous value would require a variable amount of bits depending of the current
        // prefix-free code employed and that the subsequent value would require a variable amount of
        // bits depending of the current prefix-free code employed.
        let gap1: u64 = prev_value - encoded - 1;
        if next_value == u64::MAX {
            CW::size(gap1)
        } else {
            let gap2: u64 = encoded - next_value - 1;
            let gap_removed: u64 = prev_value - next_value - 1;

            CW::size(gap1) + CW::size(gap2) - CW::size(gap_removed)
        }
    };

    let new_bit_index = bit_index + number_of_inserted_bits;

    if new_bit_index > hashes_ref.len() * 8 {
        if hash_bits == CH::SMALLEST_VIABLE_HASH_BITS {
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
    let mut bypass: BypassIter<'a> = iter.into();
    // In order to bring the reader a bit more ahead and make more unlikely to get
    // read-write conflicts, we read the next value.
    let mut next = bypass.next();

    // If there is no previos value, we would need to write the encoded value itself but
    // writing such a high value in prefix-free encoding would be inefficient. Therefore,
    // we write the first hash explicitly.
    if prev_value == u64::MAX {
        debug_assert_eq!(
            writer.tell(),
            0,
            "The writer tell must be 0 if there is no previous value"
        );

        writer.write_bits(encoded, usize::from(hash_bits));
    } else {
        if position == 1 {
            debug_assert_eq!(
                writer.tell(),
                usize::from(hash_bits),
                "The writer tell must be {hash_bits} (the hash bits) if there is a single previous value"
            );
        }

        let just_wrote_bits = CW::write(&mut writer, prev_value - encoded - 1);
        debug_assert!(
            bypass.len() == 0 || bypass.last_buffered_bit() >= writer.tell(),
            "{position}/{number_of_hashes}) Reader tell ({}) must be greater than writer tell ({}, wrote {just_wrote_bits}, {prev_value} - {encoded}) in insert at hash size {hash_bits}.",
            bypass.last_buffered_bit(),
            writer.tell(),
        );
    }

    if next_value != u64::MAX {
        position += 1;

        let just_wrote_bits = CW::write(&mut writer, encoded - next_value - 1);
        debug_assert!(
            bypass.len() == 0 || bypass.last_buffered_bit() > writer.tell(),
            "{position}/{number_of_hashes}) Reader tell ({}) must be greater than writer tell ({}, wrote {just_wrote_bits}, {prev_value} - {next_value}) in insert at hash size {hash_bits}.",
            bypass.last_buffered_bit(),
            writer.tell(),
        );

        while let Some((value, n_bits)) = next {
            next = bypass.next();
            writer.write_bits(value, n_bits);
            debug_assert!(
                bypass.len() == 0 || bypass.last_buffered_bit() > writer.tell(),
                "{position}/{number_of_hashes}) Reader tell ({}) must be greater than writer tell ({}, wrote {just_wrote_bits}, {prev_value} - {next_value}) in insert at hash size {hash_bits}.",
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

    debug_assert!(GapHash::<CH>::downgraded(
        hashes,
        number_of_hashes + 1,
        hash_bits,
        new_bit_index,
        0
    )
    .is_sorted_by(|a, b| b < a));
    // We check if the decoded value was insert at position 'insert_position'
    debug_assert_eq!(
        GapHash::<CH>::decoded(hashes, number_of_hashes + 1, hash_bits, new_bit_index)
            .nth(insert_position)
            .unwrap()
            .0,
        register
    );

    Ok(Some(new_bit_index))
}

#[derive(Debug)]
/// Iterator over downgraded hashes.
pub enum DispatchedDowngradedIter<'a, CH: CompositeHash> {
    /// Variants for when the prefix-free codes are used.
    PrefixCodeDowngradedIter(PrefixCodeDowngradedIter<'a, CH>),
    /// Variants for when the prefix-free codes are not used.
    InnerDowngradedIter(CH::Downgraded<'a>),
}

impl<'a, CH: CompositeHash> LastBufferedBit for DispatchedDowngradedIter<'a, CH> {
    fn last_buffered_bit(&self) -> usize {
        match self {
            Self::PrefixCodeDowngradedIter(iter) => iter.last_buffered_bit(),
            Self::InnerDowngradedIter(iter) => iter.last_buffered_bit(),
        }
    }
}

impl<'a, CH: CompositeHash> TryFrom<DispatchedDowngradedIter<'a, CH>>
    for PrefixCodeDowngradedIter<'a, CH>
where
    CH: PrefixFreeCode,
{
    type Error = DispatchedDowngradedIter<'a, CH>;

    fn try_from(value: DispatchedDowngradedIter<'a, CH>) -> Result<Self, Self::Error> {
        match value {
            DispatchedDowngradedIter::PrefixCodeDowngradedIter(iter) => Ok(iter),
            DispatchedDowngradedIter::InnerDowngradedIter(_) => Err(value),
        }
    }
}

impl<'a, CH: CompositeHash> Iterator for DispatchedDowngradedIter<'a, CH>
where
    CH: PrefixFreeCode,
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

impl<'a, CH: CompositeHash> ExactSizeIterator for DispatchedDowngradedIter<'a, CH>
where
    CH: PrefixFreeCode,
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

impl<'a, CH: CompositeHash> From<PrefixCodeDowngradedIter<'a, CH>> for BypassIter<'a> {
    fn from(iter: PrefixCodeDowngradedIter<'a, CH>) -> Self {
        Self {
            bitstream: iter.bitstream,
            bit_index: iter.bit_index,
        }
    }
}

#[derive(Debug)]
/// Iterator over downgraded hashes.
pub struct PrefixCodeDowngradedIter<'a, CH> {
    bitstream: BitReader<'a>,
    previous: u64,
    number_of_hashes: usize,
    /// The expected number of bits to be read.
    bit_index: usize,
    current_iteration: usize,
    hash_bits: u8,
    shift: u8,
    _phantom: PhantomData<CH>,
}

impl<'a, CH: CompositeHash> From<PrefixCodeDowngradedIter<'a, CH>> for &'a [u8] {
    fn from(iter: PrefixCodeDowngradedIter<'a, CH>) -> Self {
        iter.bitstream.into()
    }
}

impl<'a, CH: CompositeHash> LastBufferedBit for PrefixCodeDowngradedIter<'a, CH> {
    fn last_buffered_bit(&self) -> usize {
        self.bitstream.last_buffered_bit_position()
    }
}

impl<'a, CH: CompositeHash> PrefixCodeDowngradedIter<'a, CH> {
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

impl<'a, CH: CompositeHash> Iterator for PrefixCodeDowngradedIter<'a, CH>
where
    CH: PrefixFreeCode,
{
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.number_of_hashes == self.current_iteration {
            return None;
        }
        self.current_iteration += 1;

        if self.current_iteration == 1 {
            self.previous = self.bitstream.read_bits(usize::from(self.hash_bits));
            return Some(CH::downgrade(self.previous, self.hash_bits, self.shift));
        }

        let gap = match self.hash_bits {
            8 => <CH as PrefixFreeCode>::Code8::read(&mut self.bitstream),
            16 => <CH as PrefixFreeCode>::Code16::read(&mut self.bitstream),
            24 => <CH as PrefixFreeCode>::Code24::read(&mut self.bitstream),
            32 => <CH as PrefixFreeCode>::Code32::read(&mut self.bitstream),
            _ => unreachable!(),
        };

        debug_assert!(
            gap.leading_zeros() >= 64 - u32::from(self.hash_bits),
            "A gap {gap} between hash of {} bits cannot have more than {} leading zeros, but got {}.",
            self.hash_bits,
            64 - u32::from(self.hash_bits),
            gap.leading_zeros(),
        );

        debug_assert!(
            gap <= self.previous,
            "{}/{}) Since the hashes are meant to be sorted in descending order, the gap ({gap}) must be less than the previous hash ({}).",
            self.current_iteration,
            self.number_of_hashes,
            self.previous,
        );

        self.previous -= gap + 1;

        debug_assert!(
            self.previous.leading_zeros() >= 64 - u32::from(self.hash_bits),
            "The hash ({}), being theoretically {} bits long, has more than {} leading zeros",
            self.previous,
            self.hash_bits,
            64 - u32::from(self.hash_bits),
        );

        Some(CH::downgrade(self.previous, self.hash_bits, self.shift))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.number_of_hashes, Some(self.number_of_hashes))
    }
}

impl<'a, CH: CompositeHash> ExactSizeIterator for PrefixCodeDowngradedIter<'a, CH>
where
    CH: PrefixFreeCode,
{
    fn len(&self) -> usize {
        self.number_of_hashes
    }
}

#[derive(Debug)]
/// Iterator over decoded hashes.
pub enum DispatchedDecodedIter<'a, CH: CompositeHash> {
    /// Variants for when the prefix-free codes are used.
    PrefixCodeDecodedIter(PrefixCodeDecodedIter<'a, CH>),
    /// Variants for when the prefix-free codes are not used.
    InnerDecodedIter(CH::Decoded<'a>),
}

impl<'a, CH: CompositeHash> LastBufferedBit for DispatchedDecodedIter<'a, CH> {
    fn last_buffered_bit(&self) -> usize {
        match self {
            Self::PrefixCodeDecodedIter(iter) => iter.last_buffered_bit(),
            Self::InnerDecodedIter(iter) => iter.last_buffered_bit(),
        }
    }
}

impl<'a, CH: CompositeHash> TryFrom<DispatchedDecodedIter<'a, CH>> for PrefixCodeDecodedIter<'a, CH>
where
    CH: PrefixFreeCode,
{
    type Error = CH::Decoded<'a>;

    fn try_from(value: DispatchedDecodedIter<'a, CH>) -> Result<Self, Self::Error> {
        match value {
            DispatchedDecodedIter::PrefixCodeDecodedIter(iter) => Ok(iter),
            DispatchedDecodedIter::InnerDecodedIter(inner) => Err(inner),
        }
    }
}

impl<'a, CH: CompositeHash> Iterator for DispatchedDecodedIter<'a, CH>
where
    CH: PrefixFreeCode,
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

impl<'a, CH: CompositeHash> ExactSizeIterator for DispatchedDecodedIter<'a, CH>
where
    CH: PrefixFreeCode,
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
pub struct PrefixCodeDecodedIter<'a, CH> {
    iter: PrefixCodeDowngradedIter<'a, CH>,
}

impl<'a, CH: CompositeHash> LastBufferedBit for PrefixCodeDecodedIter<'a, CH> {
    fn last_buffered_bit(&self) -> usize {
        self.iter.last_buffered_bit()
    }
}

impl<'a, CH: CompositeHash> PrefixCodeDecodedIter<'a, CH> {
    #[allow(unsafe_code)]
    fn new(hashes: &'a [u8], number_of_hashes: usize, hash_bits: u8, bit_index: usize) -> Self {
        Self {
            iter: PrefixCodeDowngradedIter::new(hashes, number_of_hashes, hash_bits, bit_index, 0),
        }
    }
}

impl<'a, CH: CompositeHash> Iterator for PrefixCodeDecodedIter<'a, CH>
where
    CH: PrefixFreeCode,
{
    type Item = (u8, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|hash| CH::decode(hash, self.iter.hash_bits - self.iter.shift))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, CH: CompositeHash> ExactSizeIterator for PrefixCodeDecodedIter<'a, CH>
where
    CH: PrefixFreeCode,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}
