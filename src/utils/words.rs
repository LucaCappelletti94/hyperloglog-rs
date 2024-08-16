//! Submodule for words-arrays
use crate::utils::WordLike;

/// Trait for arrays of words.
pub trait Words {
    /// The type of the words.
    type Word: WordLike;
    /// The type of the iterator over the words.
    type WordIter<'a>: Iterator<Item = Self::Word>
    where
        Self: 'a;

    /// Returns the number of words in the array, i.e., the length of the array.
    fn number_of_words(&self) -> usize;

    /// Searches a value in the array and returns `true` if the value is found.
    fn find_sorted_with_len(&self, value: Self::Word, len: usize) -> bool;

    /// Inserts a value into the array searching for the correct position within a given length.
    fn sorted_insert_with_len(&mut self, value: Self::Word, len: usize) -> bool;

    /// Returns an iterator over the words.
    fn words(&self) -> Self::WordIter<'_>;
}

impl<T: WordLike, const N: usize> Words for [T; N] {
    type Word = T;
    type WordIter<'a> = core::iter::Copied<core::slice::Iter<'a, Self::Word>> where Self: 'a;

    #[must_use]
    fn number_of_words(&self) -> usize {
        N
    }

    #[must_use]
    fn find_sorted_with_len(&self, value: Self::Word, len: usize) -> bool {
        debug_assert!(len <= N);
        debug_assert!(self[..len].windows(2).all(|w| w[0] <= w[1]));
        self[..len].binary_search(&value).is_ok()
    }

    #[must_use]
    fn sorted_insert_with_len(&mut self, value: Self::Word, len: usize) -> bool {
        debug_assert!(len <= N);

        // We check that the array is sorted within a debug assertion.
        debug_assert!(self[..len].windows(2).all(|w| w[0] <= w[1]));

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
    fn words(&self) -> Self::WordIter<'_> {
        self.iter().copied()
    }
}
