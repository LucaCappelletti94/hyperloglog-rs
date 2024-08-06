//! Implementations of the multiplicities trait for array objects.
use super::*;
use crate::utils::*;

macro_rules! impl_zeroed_multiplicities_for_array_by_exponent {
    ($exponent: expr, $($bits: expr),*) => {
        $(
            paste::paste! {
                impl ZeroedMulteplicity<[<Precision $exponent>], [<Bits $bits>]> for [<[<Precision $exponent>] as Precision>::NumberOfZeros; maximal_multeplicity($exponent, $bits)] {
                    fn zeroed() -> Self {
                        [<[<Precision $exponent>] as Precision>::NumberOfZeros::ZERO; maximal_multeplicity($exponent, $bits)]
                    }
                }
            }
        )*
    };
}

macro_rules! impl_zeroed_multiplicities_for_array {
    ($($exponent: expr),*) => {
        $(
            impl_zeroed_multiplicities_for_array_by_exponent!($exponent, 1, 2, 3, 4, 5, 6);
        )*
    };
}

impl_zeroed_multiplicities_for_array!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

/// Trait marker to associate a specific multiplicity array with a combination of precision and bits.
///
/// Meant to be associated with a specific Precision.
pub trait ArrayMultiplicities<B: Bits>: Precision {
    type ArrayMultiplicities: Multiplicities<Self, B>;
}

macro_rules! impl_multiplicities_for_array_by_exponent {
    ($exponent: expr, $($bits: expr),*) => {
        $(
            paste::paste! {
                impl ArrayMultiplicities<[<Bits $bits>]> for [<Precision $exponent>] {
                    type ArrayMultiplicities = [<[<Precision $exponent>] as Precision>::NumberOfZeros; maximal_multeplicity($exponent, $bits)];
                }

                impl Multiplicities<[<Precision $exponent>], [<Bits $bits>]> for [<[<Precision $exponent>] as Precision>::NumberOfZeros; maximal_multeplicity($exponent, $bits)] {
                    type Iter<'a> = core::iter::Map<core::iter::Copied<core::slice::Iter<'a, <[<Precision $exponent>] as Precision>::NumberOfZeros>>, fn(<[<Precision $exponent>] as Precision>::NumberOfZeros) -> usize>;

                    fn iter_multiplicities(&self) -> Self::Iter<'_> {
                        self.as_slice().iter().copied().map(|value| value as usize)
                    }

                    fn number_of_multiplicities(&self) -> usize {
                        maximal_multeplicity($exponent, $bits)
                    }

                    fn get(&self, index: usize) -> <[<Precision $exponent>] as Precision>::NumberOfZeros {
                        self[index]
                    }

                    fn set(&mut self, index: usize, value: <[<Precision $exponent>] as Precision>::NumberOfZeros) {
                        self[index] = value;
                    }

                    fn increment(&mut self, index: usize) {
                        self[index] += 1;
                    }

                    fn decrement(&mut self, index: usize) {
                        self[index] -= 1;
                    }
                }
            }
        )*
    };
}

macro_rules! impl_multiplicities_for_array {
    ($($exponent: expr),*) => {
        $(
            impl_multiplicities_for_array_by_exponent!($exponent, 1, 2, 3, 4, 5, 6);
        )*
    };
}

impl_multiplicities_for_array!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
