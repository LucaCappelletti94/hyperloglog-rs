//! Submodule providing the trait [`WordLike`].

use super::Number;
use core::ops::{Shl, Shr, BitAnd, BitOr};

/// Trait marker for types that can be used as words.
pub trait WordLike:
    Copy
    + Ord
    + Number
    + Shl<u8, Output = Self>
    + Shr<u8, Output = Self>
    + BitAnd<Self, Output = Self>
    + BitOr<Self, Output = Self>
{
}

impl WordLike for u64 {}
