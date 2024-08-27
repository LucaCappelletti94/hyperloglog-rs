//! Gap-based composite hash implementation.
use core::{marker::PhantomData, u64};
mod bitreader;
mod bitwriter;
mod optimal_codes;
mod prefix_free_codes;
use bitreader::BitReader;
use bitwriter::BitWriter;
use prefix_free_codes::{CodeRead, CodeWrite, NoPrefixCode};

use super::*;

/// Gap-based composite hash.
pub struct GapHash<CH> {
    _phantom: PhantomData<CH>,
}

/// Trait defining the combination between a given combo of Precision
/// and Bits and which PrefixFreeCode to use for which combination.
pub trait PrefixFreeCode<const HS: u8> {
    /// Prefix-free code for when we are writing an hash of 32 bits.
    type Code: CodeRead + CodeWrite + Default + 'static;
}

impl<CH: CompositeHash> CompositeHash for GapHash<CH>
where
    CH: PrefixFreeCode<8> + PrefixFreeCode<16> + PrefixFreeCode<24> + PrefixFreeCode<32>,
{
    type Precision = <CH as CompositeHash>::Precision;
    type Bits = <CH as CompositeHash>::Bits;

    type Decoded<'a> = DecodedIter<'a, CH>;
    type Downgraded<'a> = DowngradedIter<'a, CH>;

    #[inline]
    #[must_use]
    fn downgraded<'a>(
        hashes: &'a [u8],
        number_of_hashes: usize,
        hash_bits: u8,
        shift: u8,
    ) -> Self::Downgraded<'a> {
        DowngradedIter::new(hashes, number_of_hashes, hash_bits, shift)
    }

    #[inline]
    #[must_use]
    fn decoded<'a>(hashes: &'a [u8], number_of_hashes: usize, hash_bits: u8) -> Self::Decoded<'a> {
        assert!(
            hash_bits >= Self::SMALLEST_VIABLE_HASH_BITS,
            "The hash bits ({hash_bits}) must be greater or equal to the smallest viable hash bits ({})",
            Self::SMALLEST_VIABLE_HASH_BITS,
        );
        DecodedIter::new(hashes, number_of_hashes, hash_bits)
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
        debug_assert!(register > 0);
        debug_assert!(
            index < 1 << Self::Precision::EXPONENT,
            "The index ({index}) must be less than 2^({})",
            Self::Precision::EXPONENT,
        );
        // if hash_bits == 8
        //     && core::any::TypeId::of::<<CH as PrefixFreeCode<8>>::Code>()
        //         == core::any::TypeId::of::<NoPrefixCode<8>>()
        //     || hash_bits == 16
        //         && core::any::TypeId::of::<<CH as PrefixFreeCode<16>>::Code>()
        //             == core::any::TypeId::of::<NoPrefixCode<16>>()
        //     || hash_bits == 24
        //         && core::any::TypeId::of::<<CH as PrefixFreeCode<24>>::Code>()
        //             == core::any::TypeId::of::<NoPrefixCode<24>>()
        //     || hash_bits == 32
        //         && core::any::TypeId::of::<<CH as PrefixFreeCode<32>>::Code>()
        //             == core::any::TypeId::of::<NoPrefixCode<32>>()
        // {
        //     return CH::find(
        //         hashes,
        //         number_of_hashes,
        //         index,
        //         register,
        //         original_hash,
        //         hash_bits,
        //     );
        // }

        let encoded_hash = Self::encode(index, register, original_hash, hash_bits);
        Self::downgraded(hashes, number_of_hashes, hash_bits, 0)
            .position(|hash| hash == encoded_hash)
            .map_or_else(|| Err((index, encoded_hash)), |index| Ok(index))
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
        // if hash_bits == 8
        //     && core::any::TypeId::of::<<CH as PrefixFreeCode<8>>::Code>()
        //         == core::any::TypeId::of::<NoPrefixCode<8>>()
        //     || hash_bits == 16
        //         && core::any::TypeId::of::<<CH as PrefixFreeCode<16>>::Code>()
        //             == core::any::TypeId::of::<NoPrefixCode<16>>()
        //     || hash_bits == 24
        //         && core::any::TypeId::of::<<CH as PrefixFreeCode<24>>::Code>()
        //             == core::any::TypeId::of::<NoPrefixCode<24>>()
        //     || hash_bits == 32
        //         && core::any::TypeId::of::<<CH as PrefixFreeCode<32>>::Code>()
        //             == core::any::TypeId::of::<NoPrefixCode<32>>()
        // {
        //     return CH::insert_sorted_desc(
        //         hashes,
        //         number_of_hashes,
        //         index,
        //         register,
        //         original_hash,
        //         hash_bits,
        //     );
        // }

        match hash_bits {
            8 => insert_sorted_desc_with_writer::<<CH as PrefixFreeCode<8>>::Code, CH>(
                hashes,
                number_of_hashes,
                index,
                register,
                original_hash,
                hash_bits,
            ),
            16 => insert_sorted_desc_with_writer::<<CH as PrefixFreeCode<16>>::Code, CH>(
                hashes,
                number_of_hashes,
                index,
                register,
                original_hash,
                hash_bits,
            ),
            24 => insert_sorted_desc_with_writer::<<CH as PrefixFreeCode<24>>::Code, CH>(
                hashes,
                number_of_hashes,
                index,
                register,
                original_hash,
                hash_bits,
            ),
            32 => insert_sorted_desc_with_writer::<<CH as PrefixFreeCode<32>>::Code, CH>(
                hashes,
                number_of_hashes,
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
    fn downgrade_inplace<'a>(
        hashes: &'a mut [u8],
        number_of_hashes: usize,
        hash_bits: u8,
        shift: u8,
    ) {
        if shift == 0 {
            return;
        }

        match hash_bits - shift {
            8 => downgrade_inplace_with_writer::<<CH as PrefixFreeCode<8>>::Code, CH>(
                hashes,
                number_of_hashes,
                hash_bits,
                shift,
            ),
            16 => downgrade_inplace_with_writer::<<CH as PrefixFreeCode<16>>::Code, CH>(
                hashes,
                number_of_hashes,
                hash_bits,
                shift,
            ),
            24 => downgrade_inplace_with_writer::<<CH as PrefixFreeCode<24>>::Code, CH>(
                hashes,
                number_of_hashes,
                hash_bits,
                shift,
            ),
            32 => downgrade_inplace_with_writer::<<CH as PrefixFreeCode<32>>::Code, CH>(
                hashes,
                number_of_hashes,
                hash_bits,
                shift,
            ),
            _ => unreachable!(),
        }
    }

    const SMALLEST_VIABLE_HASH_BITS: u8 = CH::SMALLEST_VIABLE_HASH_BITS;
}

#[allow(unsafe_code)]
fn downgrade_inplace_with_writer<'a, CT: CodeWrite + 'static, CH: CompositeHash>(
    hashes: &'a mut [u8],
    number_of_hashes: usize,
    hash_bits: u8,
    shift: u8,
) where
    CH: PrefixFreeCode<8> + PrefixFreeCode<16> + PrefixFreeCode<24> + PrefixFreeCode<32>,
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

    let mut iter = GapHash::<CH>::downgraded(hashes, number_of_hashes, hash_bits, shift);

    // We write the first hash explicitly, as otherwise it would be
    // written in a very inefficient way.
    let mut prev_value = if let Some(value) = iter.next() {
        match hash_bits - shift {
            8 => {
                NoPrefixCode::<8>::write(&mut writer, value);
            }
            16 => {
                NoPrefixCode::<16>::write(&mut writer, value);
            }
            24 => {
                NoPrefixCode::<24>::write(&mut writer, value);
            }
            _ => unreachable!(),
        }
        value
    } else {
        return;
    };

    while let Some(value) = iter.next() {
        CT::write(&mut writer, prev_value - value);
        prev_value = value;

        debug_assert!(iter.last_buffered_bit_position() > writer.tell());
    }

    drop(writer);

    debug_assert!(
        GapHash::<CH>::downgraded(hashes, number_of_hashes, hash_bits - shift, 0)
            .is_sorted_by(|a, b| b <= a)
    );
}

#[allow(unsafe_code)]
fn insert_sorted_desc_with_writer<'a, CW: CodeWrite + 'static, CH: CompositeHash>(
    hashes: &'a mut [u8],
    number_of_hashes: usize,
    index: usize,
    register: u8,
    original_hash: u64,
    hash_bits: u8,
) -> bool
where
    CH: PrefixFreeCode<8> + PrefixFreeCode<16> + PrefixFreeCode<24> + PrefixFreeCode<32>,
{
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
    assert!(
        GapHash::<CH>::downgraded(hashes, number_of_hashes, hash_bits, 0)
            .is_sorted_by(|a, b| b <= a)
    );

    let hashes_64 = unsafe {
        core::slice::from_raw_parts_mut(
            hashes.as_mut_ptr() as *mut u64,
            hashes.len() / core::mem::size_of::<u64>(),
        )
    };

    let encoded = CH::encode(index, register, original_hash, hash_bits);

    // iter until we find where we should insert
    let mut iter = GapHash::<CH>::downgraded(hashes, number_of_hashes, hash_bits, 0);
    let mut writer = BitWriter::new(hashes_64);

    let mut prev_value = u64::MAX;
    let mut next_value = u64::MAX;

    let mut position = 0;

    while let Some(value) = iter.next() {
        // The values are sorted in descending order, so we can stop when we find a value
        // that is less than or equal to the value we want to insert
        if encoded >= value {
            // if the value is equal to the encoded value, we don't need to insert it
            if value == encoded {
                return false;
            }
            next_value = value;
            break;
        }

        writer.seek(iter.last_read_bit_position());
        prev_value = value;
        position += 1;
    }

    let insert_position = position;

    // If there is no previos value, we would need to write the encoded value itself but
    // writing such a high value in prefix-free encoding would be inefficient. Therefore,
    // we write the first hash explicitly.
    if prev_value == u64::MAX {
        debug_assert_eq!(
            writer.tell(),
            0,
            "The writer tell must be 0 if there is no previous value"
        );

        match hash_bits {
            8 => {
                NoPrefixCode::<8>::write(&mut writer, encoded);
            }
            16 => {
                NoPrefixCode::<16>::write(&mut writer, encoded);
            }
            24 => {
                NoPrefixCode::<24>::write(&mut writer, encoded);
            }
            32 => {
                NoPrefixCode::<32>::write(&mut writer, encoded);
            }
            _ => unreachable!(),
        }
    } else {
        if position == 1 {
            debug_assert_eq!(
                writer.tell(),
                usize::from(hash_bits),
                "The writer tell must be {hash_bits} (the hash bits) if there is a single previous value"
            );
        }

        CW::write(&mut writer, prev_value - encoded);
        debug_assert!(
            iter.last_buffered_bit_position() > writer.tell(),
            "The reader last buffered bit  ({}) must be greater than the writer tell ({}) after write ops.",
            iter.last_buffered_bit_position(),
            writer.tell()
        );
    }

    if next_value != u64::MAX {
        position += 1;
        prev_value = encoded;

        while let Some(value) = iter.next() {
            CW::write(&mut writer, prev_value - next_value);
            debug_assert!(
                iter.last_buffered_bit_position() > writer.tell(),
                "The reader last buffered bit ({}) must be greater than the writer tell ({}) after write ops.",
                iter.last_buffered_bit_position(),
                writer.tell()
            );
            prev_value = next_value;
            next_value = value;
            position += 1;
        }

        CW::write(&mut writer, prev_value - next_value);
    }

    // We check that all hashes are still ordered in descending order
    drop(writer);

    debug_assert!(
        GapHash::<CH>::downgraded(hashes, number_of_hashes + 1, hash_bits, 0)
            .is_sorted_by(|a, b| b <= a)
    );
    // We check if the decoded value was insert at position 'insert_position'
    debug_assert_eq!(
        GapHash::<CH>::decoded(hashes, number_of_hashes + 1, hash_bits)
            .nth(insert_position)
            .unwrap()
            .0,
        register
    );

    true
}

/// Iterator over downgraded hashes.
pub struct DowngradedIter<'a, CH> {
    bitstream: BitReader<'a>,
    previous: u64,
    number_of_hashes: usize,
    current_iteration: usize,
    hash_bits: u8,
    shift: u8,
    _phantom: PhantomData<CH>,
}

impl<'a, CH: CompositeHash> DowngradedIter<'a, CH> {
    fn last_read_bit_position(&self) -> usize {
        self.bitstream.last_read_bit_position()
    }

    fn last_buffered_bit_position(&self) -> usize {
        self.bitstream.last_buffered_bit_position()
    }

    #[allow(unsafe_code)]
    fn new(hashes: &'a [u8], number_of_hashes: usize, hash_bits: u8, shift: u8) -> Self {
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
            shift,
            _phantom: PhantomData,
        }
    }
}

impl<'a, CH: CompositeHash> Iterator for DowngradedIter<'a, CH>
where
    CH: PrefixFreeCode<8> + PrefixFreeCode<16> + PrefixFreeCode<24> + PrefixFreeCode<32>,
{
    type Item = u64;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.number_of_hashes == self.current_iteration {
            return None;
        }
        self.current_iteration += 1;

        if self.current_iteration == 1 {
            self.previous = match self.hash_bits {
                8 => NoPrefixCode::<8>::read(&mut self.bitstream),
                16 => NoPrefixCode::<16>::read(&mut self.bitstream),
                24 => NoPrefixCode::<24>::read(&mut self.bitstream),
                32 => NoPrefixCode::<32>::read(&mut self.bitstream),
                _ => unreachable!(),
            };
            return Some(CH::downgrade(self.previous, self.hash_bits, self.shift));
        }

        let gap = match self.hash_bits {
            8 => <CH as PrefixFreeCode<8>>::Code::read(&mut self.bitstream),
            16 => <CH as PrefixFreeCode<16>>::Code::read(&mut self.bitstream),
            24 => <CH as PrefixFreeCode<24>>::Code::read(&mut self.bitstream),
            32 => <CH as PrefixFreeCode<32>>::Code::read(&mut self.bitstream),
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

        self.previous -= gap;

        debug_assert!(
            self.previous.leading_zeros() >= 64 - u32::from(self.hash_bits),
            "The hash ({}), being theoretically {} bits long, has more than {} leading zeros",
            self.previous,
            self.hash_bits,
            64 - u32::from(self.hash_bits),
        );

        Some(CH::downgrade(self.previous, self.hash_bits, self.shift))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.number_of_hashes, Some(self.number_of_hashes))
    }
}

impl<'a, CH: CompositeHash> ExactSizeIterator for DowngradedIter<'a, CH>
where
    CH: PrefixFreeCode<8> + PrefixFreeCode<16> + PrefixFreeCode<24> + PrefixFreeCode<32>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.number_of_hashes
    }
}

/// Iterator over decoded hashes.
pub struct DecodedIter<'a, CH> {
    iter: DowngradedIter<'a, CH>,
}

impl<'a, CH: CompositeHash> DecodedIter<'a, CH> {
    #[allow(unsafe_code)]
    fn new(hashes: &'a [u8], number_of_hashes: usize, hash_bits: u8) -> Self {
        Self {
            iter: DowngradedIter::new(hashes, number_of_hashes, hash_bits, 0),
        }
    }
}

impl<'a, CH: CompositeHash> Iterator for DecodedIter<'a, CH>
where
    CH: PrefixFreeCode<8> + PrefixFreeCode<16> + PrefixFreeCode<24> + PrefixFreeCode<32>,
{
    type Item = (u8, usize);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|hash| CH::decode(hash, self.iter.hash_bits - self.iter.shift))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, CH: CompositeHash> ExactSizeIterator for DecodedIter<'a, CH>
where
    CH: PrefixFreeCode<8> + PrefixFreeCode<16> + PrefixFreeCode<24> + PrefixFreeCode<32>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.iter.len()
    }
}
