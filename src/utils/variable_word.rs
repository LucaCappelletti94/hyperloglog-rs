//! Submodule providing the variable word trait, which is used in combination
//! with a packed array. This allows to define 'virtual' words with sizes that
//! are not a power of two.
use super::PositiveInteger;
use core::fmt::Debug;
use hyperloglog_derive::VariableWord;

/// Trait marker for the variable word.
pub trait VariableWord: Send + Sync + Clone + Copy + Debug + Default {
    /// The number of bits in the word.
    const NUMBER_OF_BITS: u8;
    /// The number of bits in the word as a u64.
    const NUMBER_OF_BITS_U64: u64 = Self::NUMBER_OF_BITS as u64;
    /// The number of entries in a u64.
    const NUMBER_OF_ENTRIES: u8 = 64 / Self::NUMBER_OF_BITS;
    /// The number of entries in a u64.
    const NUMBER_OF_ENTRIES_U64: u64 = Self::NUMBER_OF_ENTRIES as u64;
    /// The mask for the word.
    const MASK: u64;
    /// The word type.
    type Word: PositiveInteger + TryInto<u8> + TryInto<u16> + TryInto<u32> + TryInto<u64>;
}

/// Virtual word with 40 bits.
#[allow(non_camel_case_types)]
#[derive(VariableWord)]
pub struct u40(u64);

/// Virtual word with 48 bits.
#[allow(non_camel_case_types)]
#[derive(VariableWord)]
pub struct u48(u64);

/// Virtual word with 56 bits.
#[allow(non_camel_case_types)]
#[derive(VariableWord)]
pub struct u56(u64);

impl VariableWord for u8 {
    const NUMBER_OF_BITS: u8 = 8;
    type Word = u8;
    const MASK: u64 = 0xFF;
}

impl VariableWord for u16 {
    const NUMBER_OF_BITS: u8 = 16;
    type Word = u16;
    const MASK: u64 = 0xFFFF;
}

impl VariableWord for u32 {
    const NUMBER_OF_BITS: u8 = 32;
    type Word = u32;
    const MASK: u64 = 0xFFFF_FFFF;
}

impl VariableWord for u64 {
    const NUMBER_OF_BITS: u8 = 64;
    type Word = u64;
    const MASK: u64 = 0xFFFF_FFFF_FFFF_FFFF;
}