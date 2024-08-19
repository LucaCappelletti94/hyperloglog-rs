//! Submodule providing the variable word trait, which is used in combination
//! with a packed array. This allows to define 'virtual' words with sizes that
//! are not a power of two.
use super::PositiveInteger;
use core::fmt::Debug;

/// Trait marker for the variable word.
pub trait VariableWord: Send + Sync + Clone + Copy + Debug + Default{
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

    #[allow(unsafe_code)]
    /// Performs a binary search on the transmuted array.
    unsafe fn transmutative_binary_search(array: &[u64], len: usize, value: Self::Word) -> Result<usize, usize>;

    #[allow(unsafe_code)]
    /// Inserts a value into the transmuted array.
    unsafe fn transmutative_sorted_insert(array: &mut [u64], len: usize, value: Self::Word) -> bool;
}

impl VariableWord for u8 {
    const NUMBER_OF_BITS: u8 = 8;
    type Word = u8;
    const MASK: u64 = 0xFF;

    #[allow(unsafe_code)]
    unsafe fn transmutative_binary_search(array: &[u64], len: usize, value: Self::Word) -> Result<usize, usize> {
        let array_u8: &[Self::Word] = core::slice::from_raw_parts(array.as_ptr() as *const Self::Word, array.len() * 8);
        array_u8[0..len].binary_search(&value)
    }

    #[allow(unsafe_code)]
    unsafe fn transmutative_sorted_insert(array: &mut [u64], len: usize, value: Self::Word) -> bool {
        match Self::transmutative_binary_search(array, len, value) {
            Ok(_) => false,
            Err(index) => {
                let array_u8: &mut [Self::Word] = core::slice::from_raw_parts_mut(array.as_mut_ptr() as *mut Self::Word, array.len() * 8);
                array_u8.copy_within(index..len, index + 1);
                array_u8[index] = value;
                true
            }
        }
    }
}

impl VariableWord for u16 {
    const NUMBER_OF_BITS: u8 = 16;
    type Word = u16;
    const MASK: u64 = 0xFFFF;

    #[allow(unsafe_code)]
    unsafe fn transmutative_binary_search(array: &[u64], len: usize, value: Self::Word) -> Result<usize, usize> {
        let array_u16: &[Self::Word] = core::slice::from_raw_parts(array.as_ptr() as *const Self::Word, array.len() * 4);
        array_u16[0..len].binary_search(&value)
    }

    #[allow(unsafe_code)]
    unsafe fn transmutative_sorted_insert(array: &mut [u64], len: usize, value: Self::Word) -> bool {
        match Self::transmutative_binary_search(array, len, value) {
            Ok(_) => false,
            Err(index) => {
                let array_u16: &mut [Self::Word] = core::slice::from_raw_parts_mut(array.as_mut_ptr() as *mut Self::Word, array.len() * 4);
                array_u16.copy_within(index..len, index + 1);
                array_u16[index] = value;
                true
            }
        }
    }
}

impl VariableWord for u32 {
    const NUMBER_OF_BITS: u8 = 32;
    type Word = u32;
    const MASK: u64 = 0xFFFFFFFF;

    #[allow(unsafe_code)]
    unsafe fn transmutative_binary_search(array: &[u64], len: usize, value: Self::Word) -> Result<usize, usize> {
        let array_u32: &[u32] = core::slice::from_raw_parts(array.as_ptr() as *const u32, array.len() * 2);
        array_u32[0..len].binary_search(&value)
    }

    #[allow(unsafe_code)]
    unsafe fn transmutative_sorted_insert(array: &mut [u64], len: usize, value: Self::Word) -> bool {
        match Self::transmutative_binary_search(array, len, value) {
            Ok(_) => false,
            Err(index) => {
                let array_u32: &mut [u32] = core::slice::from_raw_parts_mut(array.as_mut_ptr() as *mut u32, array.len() * 2);
                debug_assert!(
                    1 + len <= array_u32.len(),
                    "index: {}, len: {}, array_u32.len(): {}",
                    index,
                    len,
                    array_u32.len()
                );
                array_u32.copy_within(index..len, index + 1);
                array_u32[index] = value;
                true
            }
        }
    }
}

impl VariableWord for u64 {
    const NUMBER_OF_BITS: u8 = 64;
    type Word = u64;
    const MASK: u64 = 0xFFFFFFFFFFFFFFFF;

    #[allow(unsafe_code)]
    unsafe fn transmutative_binary_search(array: &[u64], len: usize, value: Self::Word) -> Result<usize, usize> {
        array[0..len].binary_search(&value)
    }

    #[allow(unsafe_code)]
    unsafe fn transmutative_sorted_insert(array: &mut [u64], len: usize, value: Self::Word) -> bool {
        match Self::transmutative_binary_search(array, len, value) {
            Ok(_) => false,
            Err(index) => {
                array.copy_within(index..len, index + 1);
                array[index] = value;
                true
            }
        }
    }
}
