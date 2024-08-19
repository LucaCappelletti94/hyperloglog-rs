//! Submodule for words-arrays
use super::VariableWord;

/// Trait for arrays of words.
pub trait VariableWords<W: VariableWord> {

    /// The type of the iterator over the words.
    type Iter<'words>: Iterator<Item = W::Word>
    where
        W: 'words,
        Self: 'words;

    /// Returns the number of words in the array, i.e., the length of the array.
    fn number_of_words(&self) -> usize;

    /// Searches a value in the array and returns `true` if the value is found.
    fn find_sorted_with_len(&self, value: W::Word, len: usize) -> bool;

    /// Inserts a value into the array searching for the correct position within a given length.
    fn sorted_insert_with_len(&mut self, value: W::Word, len: usize) -> bool;

    /// Returns an iterator over the words.
    fn variable_words(&self, len: usize) -> Self::Iter<'_>;
}
