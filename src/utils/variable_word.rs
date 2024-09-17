//! Submodule providing the variable word trait, which is used in combination
//! with a packed array. This allows to define 'virtual' words with sizes that
//! are not a power of two.
use super::PositiveInteger;
use core::fmt::Debug;

/// Trait marker for the variable word.
pub trait VariableWord: Send + Sync + Clone + Copy + Debug + Default + Eq{
    /// The number of bits in the word.
    const NUMBER_OF_BITS: u8;
    /// The number of bits in the word as a usize.
    const NUMBER_OF_BITS_USIZE: usize = Self::NUMBER_OF_BITS as usize;
    /// The number of entries in a usize.
    const NUMBER_OF_ENTRIES: usize = 64 / Self::NUMBER_OF_BITS_USIZE;
    /// The number of entries in u8.
    const NUMBER_OF_ENTRIES_U8: u8 = 64 / Self::NUMBER_OF_BITS;
    /// The mask for the word.
    const MASK: u64;
    /// The word type.
    type Word: PositiveInteger;

    #[allow(unsafe_code)]
    /// Converts the word to a u64.
    ///
    /// # Safety
    /// This method is unsafe because it may return a value that may truncate the word.
    /// It needs to be used with caution and where appropriate.
    unsafe fn unchecked_from_u64(value: u64) -> Self::Word;
}

impl VariableWord for u8 {
    const NUMBER_OF_BITS: u8 = 8;
    type Word = u8;
    const MASK: u64 = 0xFF;

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn unchecked_from_u64(value: u64) -> Self {
        debug_assert!(
            value <= <Self as VariableWord>::MASK,
            "The value is too large for the number."
        );
        value as Self
    }
}

impl VariableWord for u16 {
    const NUMBER_OF_BITS: u8 = 16;
    type Word = u16;
    const MASK: u64 = 0xFFFF;

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn unchecked_from_u64(value: u64) -> Self {
        debug_assert!(
            value <= <Self as VariableWord>::MASK,
            "The value is too large for the number."
        );
        value as Self
    }
}

impl VariableWord for u32 {
    const NUMBER_OF_BITS: u8 = 32;
    type Word = u32;
    const MASK: u64 = 0xFFFF_FFFF;

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn unchecked_from_u64(value: u64) -> Self {
        debug_assert!(
            value <= <Self as VariableWord>::MASK,
            "The value is too large for the number."
        );
        value as Self
    }
}

impl VariableWord for u64 {
    const NUMBER_OF_BITS: u8 = 64;
    type Word = u64;
    const MASK: u64 = 0xFFFF_FFFF_FFFF_FFFF;

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn unchecked_from_u64(value: u64) -> Self {
        value
    }
}
