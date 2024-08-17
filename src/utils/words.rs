//! Submodule for words-arrays
use core::iter::Copied;
use core::slice::Iter;

/// Trait for arrays of words.
pub(crate) trait Words {
    /// The type of the iterator over the words.
    type WordIter<'words>: Iterator<Item = u64>
    where
        Self: 'words;

    /// Returns the number of words in the array, i.e., the length of the array.
    fn number_of_words(&self) -> usize;

    /// Searches a value in the array and returns `true` if the value is found.
    fn find_sorted_with_len(&self, value: u64, len: usize) -> bool;

    /// Inserts a value into the array searching for the correct position within a given length.
    fn sorted_insert_with_len(&mut self, value: u64, len: usize) -> bool;

    /// Returns an iterator over the words.
    fn words(&self) -> Self::WordIter<'_>;
}

impl<const N: usize> Words for [u64; N] {
    type WordIter<'words> = Copied<Iter<'words, u64>> where Self: 'words;

    #[must_use]
    #[inline]
    fn number_of_words(&self) -> usize {
        N
    }

    #[must_use]
    #[inline]
    fn find_sorted_with_len(&self, value: u64, len: usize) -> bool {
        debug_assert!(
            len <= N,
            "The length must be less than or equal to the number of words."
        );
        debug_assert!(
            self[..len].windows(2).all(|w| w[0] <= w[1]),
            "The array must be sorted."
        );
        self[..len].binary_search(&value).is_ok()
    }

    #[must_use]
    #[inline]
    fn sorted_insert_with_len(&mut self, value: u64, len: usize) -> bool {
        debug_assert!(
            len <= N,
            "The length must be less than or equal to the number of words."
        );

        // We check that the array is sorted within a debug assertion.
        debug_assert!(
            self[..len].windows(2).all(|w| w[0] <= w[1]),
            "The array must be sorted."
        );

        match self[..len].binary_search(&value) {
            Ok(_) => false,
            Err(index) => {
                self.copy_within(index..len, index + 1);
                self[index] = value;
                true
            }
        }
    }

    #[must_use]
    #[inline]
    fn words(&self) -> Self::WordIter<'_> {
        self.iter().copied()
    }
}
