//! Submodule for words-arrays
use super::{u24, VariableWord};
use core::fmt::Debug;
use core::iter::{Copied, Map};
use core::slice::Iter;

/// Trait for arrays of words.
pub trait VariableWords<W: VariableWord> {
    /// The type of the iterator over the words.
    type Words<'words>: ExactSizeIterator<Item = W::Word> + DoubleEndedIterator
    where
        W: 'words,
        Self: 'words;

    /// Searches a value in the array and returns `true` if the value is found.
    fn find_sorted_with_len(&self, value: W::Word, len: usize) -> bool;

    /// Inserts a value into the array searching for the correct position within a given length.
    fn sorted_insert_with_len(&mut self, value: W::Word, len: usize) -> bool;

    /// Returns an iterator over the words.
    fn iter_variable_words<'words>(&'words self, len: usize) -> Self::Words<'words>
    where
        W: 'words;
}

/// Trait for arrays of words.
pub trait Words<W: VariableWord>:
    AsRef<[<Self as Words<W>>::SliceType]> + AsMut<[<Self as Words<W>>::SliceType]>
where
    W::Word: From<<Self as Words<W>>::SliceType>,
{
    type SliceType: Ord + Copy + Debug;
}
impl<T: AsRef<[u8]> + AsMut<[u8]>> Words<u8> for T {
    type SliceType = u8;
}
impl<T: AsRef<[u16]> + AsMut<[u16]>> Words<u16> for T {
    type SliceType = u16;
}
impl<T: AsRef<[[u8; 3]]> + AsMut<[[u8; 3]]>> Words<u24> for T {
    type SliceType = [u8; 3];
}
impl<T: AsRef<[u32]> + AsMut<[u32]>> Words<u32> for T {
    type SliceType = u32;
}

impl<WS: Words<W>, W: VariableWord> VariableWords<W> for WS
where
    W::Word: From<<Self as Words<W>>::SliceType>,
    W::Word: Into<<Self as Words<W>>::SliceType>,
{
    type Words<'words> = Map<
        Copied<Iter<'words, <Self as Words<W>>::SliceType>>,
        fn(<Self as Words<W>>::SliceType) -> W::Word,
    > where
        W: 'words,
        Self: 'words;

    fn find_sorted_with_len(&self, value: W::Word, len: usize) -> bool {
        debug_assert!(
            self.as_ref().len() >= len,
            "The array must have enough elements."
        );
        debug_assert!(
            self.as_ref()[0..len].is_sorted_by_key(|x| W::Word::from(*x)),
            "The array with len ({len}) must be sorted but got {:?}",
            &self.as_ref()[0..len],
        );
        self.as_ref()[0..len].binary_search_by_key(&value, |x| W::Word::from(*x)).is_ok()
    }

    fn sorted_insert_with_len(&mut self, value: W::Word, len: usize) -> bool {
        debug_assert!(
            self.as_ref().len() > len,
            "The array must have enough space for the new value. The length is {len} and the number of words is {}.",
            self.as_ref().len()
        );
        debug_assert!(
            self.as_ref()[0..len].is_sorted_by_key(|x| W::Word::from(*x)),
            "The array with len ({len}) must be sorted but got {:?}",
            &self.as_ref()[0..len],
        );

        let slice_type: <Self as Words<W>>::SliceType = value.into();
        match self.as_mut()[0..len].binary_search_by_key(&value, |x| W::Word::from(*x)) {
            Ok(_) => false,
            Err(index) => {
                self.as_mut().copy_within(index..len, index + 1);
                self.as_mut()[index] = slice_type;

                debug_assert!(
                    self.as_ref()[0..=len].is_sorted_by_key(|x| W::Word::from(*x)),
                    "The array must be sorted."
                );

                true
            }
        }
    }

    fn iter_variable_words<'words>(&'words self, len: usize) -> Self::Words<'words>
    where
        W: 'words,
    {
        self.as_ref()[0..len].iter().copied().map(W::Word::from)
    }
}