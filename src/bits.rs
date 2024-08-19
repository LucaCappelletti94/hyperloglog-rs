//! Submodule providing the trait marker Bits.
use crate::prelude::VariableWord;
use core::fmt::Debug;

#[cfg(feature = "std")]
use crate::utils::Named;

/// Trait marker for the number of bits.
pub trait Bits: VariableWord {}

/// Implementation
macro_rules! impl_bits {
    ($($n: expr),*) => {
        $(
            paste::paste! {
                #[non_exhaustive]
                #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                /// A struct representing the number of bits.
                pub struct [<Bits $n>];

                #[cfg(feature = "std")]
                impl Named for [<Bits $n>] {
                    #[inline]
                    fn name(&self) -> String {
                        format!("B{}", $n)
                    }
                }

                impl VariableWord for [<Bits $n>] {
                    const NUMBER_OF_BITS: u8 = $n;
                    type Word = u8;
                    const MASK: u64 = (1 << $n) - 1;

                    #[allow(unsafe_code)]
                    unsafe fn transmutative_binary_search(_array: &[u64], _len: usize, _value: Self::Word) -> Result<usize, usize> {
                        unimplemented!("It would be highly unusual to use this as struct marker for the number of bits.")
                    }

                    #[allow(unsafe_code)]
                    unsafe fn transmutative_sorted_insert(_array: &mut [u64], _len: usize, _value: Self::Word) -> bool {
                        unimplemented!("It would be highly unusual to use this as struct marker for the number of bits.")
                    }
                }

                impl Bits for [<Bits $n>] {
                }
            }
        )*
    };
}

impl_bits!(1, 2, 3, 4, 5, 6, 7, 8);
