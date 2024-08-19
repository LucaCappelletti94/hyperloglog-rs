//! Submodule for words-arrays
use super::VariableWord;
use core::iter::Copied;
use core::slice::Iter;

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
    fn iter_variable_words<'words>(&'words self, len: usize) -> Self::Iter<'words>
    where
        W: 'words;
}

/// Trait for arrays of words.
pub trait Words<W: VariableWord>: AsRef<[W::Word]> + AsMut<[W::Word]> {}
impl<T: AsRef<[u8]> + AsMut<[u8]>> Words<u8> for T {}
impl<T: AsRef<[u16]> + AsMut<[u16]>> Words<u16> for T {}
impl<T: AsRef<[u32]> + AsMut<[u32]>> Words<u32> for T {}
impl<T: AsRef<[u64]> + AsMut<[u64]>> Words<u64> for T {}

impl<WS: Words<W>, W: VariableWord> VariableWords<W> for WS {
    type Iter<'words> = Copied<Iter<'words, W::Word>> where Self: 'words, W: 'words;

    fn number_of_words(&self) -> usize {
        self.as_ref().len()
    }

    fn find_sorted_with_len(&self, value: W::Word, len: usize) -> bool {
        debug_assert!(
            self.as_ref().len() >= len,
            "The array must have enough elements."
        );
        debug_assert!(
            self.as_ref()[0..len].windows(2).all(|w| w[0] < w[1]),
            "The array must be strictly sorted, i.e. sorted with no duplicates."
        );
        self.as_ref()[0..len].binary_search(&value).is_ok()
    }

    fn sorted_insert_with_len(&mut self, value: W::Word, len: usize) -> bool {
        debug_assert!(
            self.as_ref().len() >= len + 1,
            "The array must have enough space for the new value. The length is {} and the number of words is {}.",
            len,
            self.as_ref().len()
        );
        debug_assert!(
            self.as_ref()[0..len].windows(2).all(|w| w[0] < w[1]),
            "The array must be strictly sorted, i.e. sorted with no duplicates."
        );

        match self.as_mut()[0..len].binary_search(&value) {
            Ok(_) => false,
            Err(index) => {
                self.as_mut()[index..=len].rotate_right(1);
                self.as_mut()[index] = value;
                true
            }
        }
    }

    fn iter_variable_words<'words>(&'words self, len: usize) -> Self::Iter<'words>
    where
        W: 'words,
    {
        self.as_ref()[0..len].iter().copied()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Macro to generate tests for the `VariableWords` trait for different words.
    macro_rules! generate_variable_words_tests {
        ($($typ:ty),*) => {
            $(
                paste::paste!{
                    #[test]
                    #[should_panic]
                    fn [<test_panic_sorted_insert_with_len_ $typ>]() {
                        let mut words: [$typ; 5] = [1, 2, 3, 4, 5];
                        let mut words = &mut words[..];
                        assert_eq!(words.sorted_insert_with_len(6, 5), true);
                        assert_eq!(words, [1, 2, 3, 4, 6]);
                    }

                    #[test]
                    fn [<test_sorted_insert_with_len_ $typ>]() {
                        let mut words: [$typ; 5] = [1, 2, 3, 4, 5];
                        assert_eq!(words.sorted_insert_with_len(3, 4), false, "We do not insert the value if it is already in the array.");
                        assert_eq!(words, [1, 2, 3, 4, 5], "We do not insert the value if it is already in the array.");

                        let mut words: [$typ; 5] = [1, 2, 3, 5, 0];
                        assert_eq!(words.sorted_insert_with_len(4, 4), true, "We insert the value if it is not in the array.");
                        assert_eq!(words, [1, 2, 3, 4, 5], "We do not insert the value if it is already in the array.");
                    }

                    #[test]
                    fn [<test_find_sorted_with_len_ $typ>]() {
                        let words: [$typ; 5] = [1, 2, 3, 4, 5];
                        assert_eq!(words.find_sorted_with_len(3, 5), true);
                        assert_eq!(words.find_sorted_with_len(6, 5), false);
                        assert_eq!(words.find_sorted_with_len(0, 5), false);
                    }

                    #[test]
                    fn [<test_iter_variable_words_ $typ>]() {
                        let words: [$typ; 5] = [1, 2, 3, 4, 5];
                        let mut iter = words.iter_variable_words(5);
                        assert_eq!(iter.next(), Some(1));
                        assert_eq!(iter.next(), Some(2));
                        assert_eq!(iter.next(), Some(3));
                        assert_eq!(iter.next(), Some(4));
                        assert_eq!(iter.next(), Some(5));
                        assert_eq!(iter.next(), None);
                    }
                }
            )*
        };
    }

    generate_variable_words_tests!(u8, u16, u32, u64);
}
