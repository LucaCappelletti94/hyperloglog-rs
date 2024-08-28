//! Struct markers for Prefix-Free Codes.
use super::bitreader::{len_exp_golomb, len_golomb, len_rice, BitReader};
use super::bitwriter::BitWriter;

/// Trait for determining the size of a code.
pub trait CodeSize {
    /// Get the size of a code.
    fn size(value: u64) -> usize;
}

/// Trait for reading a code from a bit stream.
pub trait CodeRead {
    /// Read a code from a bit stream.
    fn read(reader: &mut BitReader) -> u64;
}

/// Trait for writing a code to a bit stream.
pub trait CodeWrite {
    /// Write a code to a bit stream.
    fn write(writer: &mut BitWriter, value: u64) -> usize;
}

#[derive(Default)]
/// Golomb code with a given parameter B.
pub struct Golomb<const B: usize>;

impl<const B: usize> CodeSize for Golomb<B> {
    fn size(value: u64) -> usize {
        len_golomb(value, B as u64)
    }
}

impl<const B: usize> CodeRead for Golomb<B> {
    fn read(reader: &mut BitReader) -> u64 {
        let value = reader.read_golomb(B);
        debug_assert!(value.leading_zeros() >= 32, "All the considered values encoded are hashes with at most 32 bits, and therefore at least 32 leading zeros. Read an hash {value:064b} with {} leading zeros", value.leading_zeros());
        value
    }
}

impl<const B: usize> CodeWrite for Golomb<B> {
    fn write(writer: &mut BitWriter, value: u64) -> usize {
        debug_assert!(
            value.leading_zeros() >= 32,
            "All the considered values encoded are hashes with at most 32 bits, and therefore at least 32 leading zeros. Got an hash {value:064b} with {} leading zeros",
            value.leading_zeros()
        );
        writer.write_golomb(value, B as u64)
    }
}

#[derive(Default)]
/// Rice code with a given parameter B.
pub struct Rice<const B: usize>;

impl<const B: usize> CodeSize for Rice<B> {
    fn size(value: u64) -> usize {
        len_rice(value, B)
    }
}

impl<const B: usize> CodeRead for Rice<B> {
    fn read(reader: &mut BitReader) -> u64 {
        let value = reader.read_rice(B);
        debug_assert!(value.leading_zeros() >= 32, "All the considered values encoded are hashes with at most 32 bits, and therefore at least 32 leading zeros. Read an hash {value:064b} with {} leading zeros", value.leading_zeros());
        value
    }
}

impl<const B: usize> CodeWrite for Rice<B> {
    fn write(writer: &mut BitWriter, value: u64) -> usize {
        debug_assert!(
            value.leading_zeros() >= 32,
            "All the considered values encoded are hashes with at most 32 bits, and therefore at least 32 leading zeros. Got an hash {value:064b} with {} leading zeros",
            value.leading_zeros()
        );
        writer.write_rice(value, B as u64)
    }
}

#[derive(Default)]
/// Exponential Golomb code with a given parameter B.
pub struct ExpGolomb<const B: usize>;

impl<const B: usize> CodeSize for ExpGolomb<B> {
    fn size(value: u64) -> usize {
        len_exp_golomb(value, B)
    }
}

impl<const B: usize> CodeRead for ExpGolomb<B> {
    fn read(reader: &mut BitReader) -> u64 {
        let value = reader.read_exp_golomb(B);
        debug_assert!(value.leading_zeros() >= 32, "All the considered values encoded are hashes with at most 32 bits, and therefore at least 32 leading zeros. Read an hash {value:064b} with {} leading zeros", value.leading_zeros());
        value
    }
}

impl<const B: usize> CodeWrite for ExpGolomb<B> {
    fn write(writer: &mut BitWriter, value: u64) -> usize {
        debug_assert!(
            value.leading_zeros() >= 32,
            "All the considered values encoded are hashes with at most 32 bits, and therefore at least 32 leading zeros. Got an hash {value:064b} with {} leading zeros",
            value.leading_zeros()
        );
        writer.write_exp_golomb(value, B as u64)
    }
}

#[derive(Default)]
/// No prefix code.
pub struct NoPrefixCode<const HS: u8>;

impl<const HS: u8> CodeSize for NoPrefixCode<HS> {
    fn size(_: u64) -> usize {
        usize::from(HS)
    }
}

impl<const HS: u8> CodeRead for NoPrefixCode<HS> {
    fn read(reader: &mut BitReader) -> u64 {
        let value = reader.read_bits(HS as usize);
        debug_assert!(value.leading_zeros() >= 32, "All the considered values encoded are hashes with at most 32 bits, and therefore at least 32 leading zeros. Read an hash {value:064b} with {} leading zeros", value.leading_zeros());
        value
    }
}

impl<const HS: u8> CodeWrite for NoPrefixCode<HS> {
    fn write(writer: &mut BitWriter, value: u64) -> usize {
        debug_assert!(
            value.leading_zeros() >= 32,
            "All the considered values encoded are hashes with at most 32 bits, and therefore at least 32 leading zeros. Got an hash {value:064b} with {} leading zeros",
            value.leading_zeros()
        );
        writer.write_bits(value, HS as usize)
    }
}
