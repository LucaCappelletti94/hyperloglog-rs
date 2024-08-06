//! Submodule for words-arrays
use crate::utils::WordLike;

pub trait Words {
    type Word: WordLike;
    type WordIter<'a>: Iterator<Item = Self::Word>
    where
        Self: 'a;

    fn words(&self) -> Self::WordIter<'_>;
}

impl<T: WordLike, const N: usize> Words for [T; N] {
    type Word = T;
    type WordIter<'a> = core::iter::Copied<core::slice::Iter<'a, Self::Word>> where Self: 'a;

    fn words(&self) -> Self::WordIter<'_> {
        self.iter().copied()
    }
}
