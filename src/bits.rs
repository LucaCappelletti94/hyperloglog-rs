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

                    #[inline]
                    #[allow(unsafe_code)]
                    #[expect(clippy::cast_possible_truncation, reason = "The value is guaranteed to be less than 256")]
                    unsafe fn unchecked_from_u64(value: u64) -> Self::Word {
                        debug_assert!(value <= <Self as crate::prelude::VariableWord>::MASK, "The value is too large for the number.");
                        value as u8
                    }
                }

                impl Bits for [<Bits $n>] {}
            }
        )*
    };
}

impl_bits!(4, 5, 6);
