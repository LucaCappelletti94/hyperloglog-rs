//! Submodule providing the multiplicities trait.
use core::fmt::Debug;

use crate::prelude::*;

mod array;

pub use array::ArrayMultiplicities;

pub trait ZeroedMulteplicity<P: Precision, B: Bits>: Sized {
    /// Returns a zeroed value.
    fn zeroed() -> Self;
}

pub trait Multiplicities<P: Precision, B: Bits>:
    ZeroedMulteplicity<P, B> + Debug + Clone + PartialEq + Eq
{
    type Iter<'a>: Iterator<Item = usize> + ExactSizeIterator + DoubleEndedIterator
    where
        Self: 'a;

    #[inline(always)]
    fn initialized() -> Self {
        let mut multiplicities = Self::zeroed();
        multiplicities.set(0, unsafe {
            P::NumberOfZeros::try_from(P::NUMBER_OF_REGISTERS).unwrap_unchecked()
        });
        multiplicities
    }

    fn number_of_multiplicities(&self) -> usize;

    fn iter_multiplicities(&self) -> Self::Iter<'_>;

    fn get(&self, index: usize) -> P::NumberOfZeros;

    fn set(&mut self, index: usize, value: P::NumberOfZeros);

    fn first(&self) -> P::NumberOfZeros {
        self.get(0)
    }

    fn last(&self) -> P::NumberOfZeros {
        self.get(self.number_of_multiplicities() - 1)
    }

    fn increment(&mut self, index: usize) {
        self.set(index, self.get(index) + P::NumberOfZeros::ONE);
    }

    fn decrement(&mut self, index: usize) {
        self.set(index, self.get(index) - P::NumberOfZeros::ONE);
    }
}
