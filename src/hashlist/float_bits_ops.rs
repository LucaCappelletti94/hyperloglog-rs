//! This module provides functions to encode and decode into the bits that are
//! used in a HyperLogLog as the harmonic sum, while in the HashList we repourpose
//! them to store other metadata.
use crate::prelude::*;

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperLogLog<P, B, R, H> {
    pub(crate) fn set_hash_bits(&mut self, hash_bits: u8) {
        encode_hash_bits(&mut self.harmonic_sum, hash_bits);
    }

    pub(crate) fn get_hash_bits(&self) -> u8 {
        decode_hash_bits(self.harmonic_sum)
    }

    pub(crate) fn add_duplicates(&mut self, new_duplicates: u32) {
        add_duplicates(&mut self.harmonic_sum, new_duplicates);
    }

    pub(crate) fn set_duplicates(&mut self, duplicates: u32) {
        set_duplicates(&mut self.harmonic_sum, duplicates);
    }

    pub(crate) fn get_duplicates(&self) -> u32 {
        decode_duplicates(self.harmonic_sum)
    }

    pub(crate) fn set_writer_tell(&mut self, bit_index: u32) {
        set_writer_tell(&mut self.harmonic_sum, bit_index);
    }

    pub(crate) fn get_writer_tell(&self) -> u32 {
        decode_writer_tell(self.harmonic_sum)
    }

    pub(crate) fn set_number_of_hashes(&mut self, number_of_hashes: u32) {
        set_number_of_hashes(&mut self.harmonic_sum, number_of_hashes);
    }

    pub(crate) fn get_number_of_hashes(&self) -> u32 {
        decode_number_of_hashes(self.harmonic_sum)
    }
}

const BITS_FOR_HASH_BITS: usize = 5;
const HASH_BITS_MASK: u64 = (1 << BITS_FOR_HASH_BITS) - 1;

#[allow(unsafe_code)]
fn encode_hash_bits(float: &mut f64, target_hash: u8) {
    debug_assert!((8..=32).contains(&target_hash));
    let harmonic_sum_as_u64: &mut u64 = unsafe { core::mem::transmute(float) };
    *harmonic_sum_as_u64 = (*harmonic_sum_as_u64 & !HASH_BITS_MASK) | u64::from(target_hash - 8);
}

fn decode_hash_bits(float: f64) -> u8 {
    u8::try_from(float.to_bits() & HASH_BITS_MASK).unwrap() + 8
}

/// The maximum number of duplicates that can we need to store is equal to the
/// number of hashes, but since these values are meant to be extremely unlikely,
/// we can safely use less than the 20 bits needed to store the number of hashes.
/// We use therefore all remaining bits to store the number of duplicates:
///
/// We are currently using, out of the 64 bits of a f64:
/// * 1 bit to represent we are in hash list mode.
/// * 5 bits to represent the hash bits we are using (minus 8).
/// * 20 bits to represent the number of hashes.
/// * 21 bits to represent the bit index of the writer tell.
///
/// Therefore, we have 17 bits left to represent the number of duplicates.
const BITS_FOR_DUPLICATES: usize = 17;
const DUPLICATES_OFFSET: usize = BITS_FOR_HASH_BITS;
const DUPLICATES_MASK: u64 = (1 << BITS_FOR_DUPLICATES) - 1;

#[allow(unsafe_code)]
/// Adds the count of duplicates to the harmonic sum.
fn add_duplicates(float: &mut f64, new_duplicates: u32) {
    set_duplicates(float, decode_duplicates(*float) + new_duplicates);
}

#[allow(unsafe_code)]
fn set_duplicates(float: &mut f64, duplicates: u32) {
    assert!(u64::from(duplicates) <= DUPLICATES_MASK);

    let harmonic_sum_as_u64: &mut u64 = unsafe { core::mem::transmute(float) };
    *harmonic_sum_as_u64 = (*harmonic_sum_as_u64 & !(DUPLICATES_MASK << DUPLICATES_OFFSET))
        | (u64::from(duplicates) << DUPLICATES_OFFSET);
}

fn decode_duplicates(float: f64) -> u32 {
    u32::try_from((float.to_bits() >> DUPLICATES_OFFSET) & DUPLICATES_MASK).unwrap()
}

/// The writer tell represents the largest index of the bits that have been written
/// to in the registers. Given that the largest possible index is the number of registers
/// times the number of bits in a register, we need 2**18 * 6 ~ 2**21
const BITS_FOR_WRITER_TELL: usize = 21;
const WRITER_TELL_OFFSET: usize = BITS_FOR_HASH_BITS + BITS_FOR_DUPLICATES;
const WRITER_TELL_MASK: u64 = (1 << BITS_FOR_WRITER_TELL) - 1;

#[allow(unsafe_code)]
/// Sets the provided bit index to the harmonic sum.
fn set_writer_tell(float: &mut f64, bit_index: u32) {
    assert!(u64::from(bit_index) <= WRITER_TELL_MASK);

    let harmonic_sum_as_u64: &mut u64 = unsafe { core::mem::transmute(float) };
    *harmonic_sum_as_u64 = (*harmonic_sum_as_u64 & !(WRITER_TELL_MASK << WRITER_TELL_OFFSET))
        | u64::from(bit_index) << WRITER_TELL_OFFSET;
}

fn decode_writer_tell(float: f64) -> u32 {
    u32::try_from(float.to_bits() >> WRITER_TELL_OFFSET & WRITER_TELL_MASK).unwrap()
}

/// The largest possible number of hash, given that the largest possible precision
/// of the hyperloglog is 18, with the largest possible number of bits per register
/// being 6, we have 2**18 * 6 / 2 (as the very minimum size of an hash is 2 bits)
/// which is 2**18 * 3 ~ 2**20 - 1 = 0xF_FFFF.
const BITS_FOR_NUMBER_OF_HASHES: usize = 20;
const NUMBER_OF_HASHES_OFFSET: usize = WRITER_TELL_OFFSET + BITS_FOR_WRITER_TELL;
const NUMBER_OF_HASHES_MASK: u64 = (1 << BITS_FOR_NUMBER_OF_HASHES) - 1;

#[allow(unsafe_code)]
/// Sets the provided number of hashes to the harmonic sum.
fn set_number_of_hashes(float: &mut f64, number_of_hashes: u32) {
    assert!(u64::from(number_of_hashes) <= NUMBER_OF_HASHES_MASK);

    let harmonic_sum_as_u64: &mut u64 = unsafe { core::mem::transmute(float) };
    *harmonic_sum_as_u64 = (*harmonic_sum_as_u64
        & !(NUMBER_OF_HASHES_MASK << NUMBER_OF_HASHES_OFFSET))
        | u64::from(number_of_hashes) << NUMBER_OF_HASHES_OFFSET;
}

#[allow(unsafe_code)]
/// Returns the number of hashes stored in the harmonic sum.
fn decode_number_of_hashes(float: f64) -> u32 {
    u32::try_from((float.to_bits() >> NUMBER_OF_HASHES_OFFSET) & NUMBER_OF_HASHES_MASK).unwrap()
}

#[cfg(test)]
mod test_encode_decode_hash_bits {
    use super::*;

    #[test]
    fn test_encode_decode_hash_bits() {
        // The harmonic flag is initialized to minus infinity.
        let mut harmonic_sum = f64::NEG_INFINITY;
        for hash_bits in 8..=32 {
            encode_hash_bits(&mut harmonic_sum, hash_bits);
            assert_eq!(decode_hash_bits(harmonic_sum), hash_bits);
        }

        // We check that the harmonic sum has still a leading number of zeros
        // equal to zero, as we have initialized it to minus infinity and we
        // should not have touched those bits.
        assert_eq!(harmonic_sum.to_bits().leading_zeros(), 0);
    }

    #[test]
    fn test_encode_decode_duplicates() {
        let mut harmonic_sum = f64::NEG_INFINITY;
        add_duplicates(&mut harmonic_sum, 0);
        assert_eq!(decode_duplicates(harmonic_sum), 0);
        add_duplicates(&mut harmonic_sum, 1);
        assert_eq!(decode_duplicates(harmonic_sum), 1);
        add_duplicates(&mut harmonic_sum, 2);
        assert_eq!(decode_duplicates(harmonic_sum), 3);
        add_duplicates(&mut harmonic_sum, 3);
        assert_eq!(decode_duplicates(harmonic_sum), 6);
    }

    #[test]
    fn test_encode_decode_writer_tell() {
        let mut harmonic_sum = f64::NEG_INFINITY;
        set_writer_tell(&mut harmonic_sum, 0);
        assert_eq!(decode_writer_tell(harmonic_sum), 0);
        set_writer_tell(&mut harmonic_sum, 1);
        assert_eq!(decode_writer_tell(harmonic_sum), 1);
        set_writer_tell(&mut harmonic_sum, 2);
        assert_eq!(decode_writer_tell(harmonic_sum), 2);
        set_writer_tell(&mut harmonic_sum, 3);
        assert_eq!(decode_writer_tell(harmonic_sum), 3);
    }

    #[test]
    fn test_encode_decode_number_of_hashes() {
        let mut harmonic_sum = f64::NEG_INFINITY;
        set_number_of_hashes(&mut harmonic_sum, 0);
        assert_eq!(decode_number_of_hashes(harmonic_sum), 0);
        set_number_of_hashes(&mut harmonic_sum, 1);
        assert_eq!(decode_number_of_hashes(harmonic_sum), 1);
        set_number_of_hashes(&mut harmonic_sum, 2);
        assert_eq!(decode_number_of_hashes(harmonic_sum), 2);
        set_number_of_hashes(&mut harmonic_sum, 3);
        assert_eq!(decode_number_of_hashes(harmonic_sum), 3);
    }

    #[test]
    fn test_mixed_encode_decode_operations() {
        let mut harmonic_sum = f64::NEG_INFINITY;
        encode_hash_bits(&mut harmonic_sum, 8);
        assert_eq!(decode_hash_bits(harmonic_sum), 8);
        add_duplicates(&mut harmonic_sum, 1);
        assert_eq!(decode_duplicates(harmonic_sum), 1);
        set_writer_tell(&mut harmonic_sum, 1);
        set_number_of_hashes(&mut harmonic_sum, 1);
        assert_eq!(decode_hash_bits(harmonic_sum), 8);
        assert_eq!(decode_duplicates(harmonic_sum), 1);
        assert_eq!(decode_writer_tell(harmonic_sum), 1);
        assert_eq!(decode_number_of_hashes(harmonic_sum), 1);
        encode_hash_bits(&mut harmonic_sum, 24);
        assert_eq!(decode_hash_bits(harmonic_sum), 24);
        set_writer_tell(&mut harmonic_sum, 100);
        add_duplicates(&mut harmonic_sum, 1);
        assert_eq!(decode_duplicates(harmonic_sum), 2);
        assert_eq!(decode_writer_tell(harmonic_sum), 100);
        set_writer_tell(&mut harmonic_sum, 10);
        assert_eq!(decode_hash_bits(harmonic_sum), 24);
        assert_eq!(decode_duplicates(harmonic_sum), 2);
        assert_eq!(decode_writer_tell(harmonic_sum), 10);
    }
}
