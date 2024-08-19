//! Submodule providing the derive macro for the VariableWord trait.
//!
//! A variable word is a trait that allows to define 'virtual' words, which
//! we use to define custom words that are multiples of 8 bits but not a power
//! of two, such as 40, 48, or 56 bits. The VariableWord derive not only derives
//! the variable word trait, but also all of the necessary traits for the word,
//! assuming that the underlying word is an u64 (we only cover 40, 48, and 56 bits).
//! These include for instance the Display, Debug, Add, Sub, Mul, Div, BitAnd, BitOr,
//! BitXor, Shl, Shr, AddAssign, SubAssign, MulAssign, DivAssign, BitAndAssign, BitOrAssign,
//! BitXorAssign, ShlAssign, ShrAssign, PartialEq, Eq, PartialOrd, Ord, Hash, and Default traits.
//!
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident};

/// Possible variants for the word size currently supported.
enum WordSize {
    /// 40-bit word.
    U40,
    /// 48-bit word.
    U48,
    /// 56-bit word.
    U56,
}

impl From<&Ident> for WordSize {
    fn from(ident: &Ident) -> Self {
        if ident.to_string().contains("40") {
            WordSize::U40
        } else if ident.to_string().contains("48") {
            WordSize::U48
        } else if ident.to_string().contains("56") {
            WordSize::U56
        } else {
            panic!("The struct name must contain either 40, 48, or 56");
        }
    }
}

impl WordSize {
    fn number_of_bits(&self) -> u8 {
        match self {
            WordSize::U40 => 40,
            WordSize::U48 => 48,
            WordSize::U56 => 56,
        }
    }

    fn mask(&self) -> u64 {
        match self {
            WordSize::U40 => 0xFF_FFFF_FFFF,
            WordSize::U48 => 0xFFFF_FFFF_FFFF,
            WordSize::U56 => 0xFF_FFFF_FFFF_FFFF,
        }
    }
}

#[proc_macro_derive(VariableWord)]
pub fn derive_variable_word(input: TokenStream) -> TokenStream {
    // Parse the input TokenStream into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Ensure the input is a struct
    let data_struct = match &input.data {
        Data::Struct(data_struct) => data_struct,
        _ => panic!("VariableWord can only be derived for structs"),
    };

    // Ensure the struct has exactly one unnamed field (i.e., a tuple struct)
    let _field = match &data_struct.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0],
        _ => panic!("The struct must have exactly one unnamed field"),
    };

    // Get the word size from the struct name
    let word_size = WordSize::from(name);
    let number_of_bits = word_size.number_of_bits();
    let mask = word_size.mask();

    // Generate the necessary traits for the word
    let expanded = quote! {
        impl crate::prelude::VariableWord for #name {
            const NUMBER_OF_BITS: u8 = #number_of_bits;
            const MASK: u64 = #mask;
            type Word = Self;
        }

        impl core::fmt::Display for #name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl core::ops::Add for #name {
            type Output = Self;

            #[inline]
            #[must_use]
            fn add(self, rhs: Self) -> Self::Output {
                Self((self.0 + rhs.0) & <Self as crate::prelude::VariableWord>::MASK)
            }
        }

        impl core::ops::AddAssign for #name {
            #[inline]
            #[must_use]
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        impl core::ops::Sub for #name {
            type Output = Self;

            #[inline]
            #[must_use]
            fn sub(self, rhs: Self) -> Self::Output {
                Self((self.0.wrapping_sub(rhs.0)) & <Self as crate::prelude::VariableWord>::MASK)
            }
        }

        impl core::ops::SubAssign for #name {
            #[inline]
            #[must_use]
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }

        impl core::ops::Mul for #name {
            type Output = Self;

            #[inline]
            #[must_use]
            fn mul(self, rhs: Self) -> Self::Output {
                Self((self.0 * rhs.0) & <Self as crate::prelude::VariableWord>::MASK)
            }
        }

        impl core::ops::MulAssign for #name {
            #[inline]
            #[must_use]
            fn mul_assign(&mut self, rhs: Self) {
                *self = *self * rhs;
            }
        }

        impl core::ops::Div for #name {
            type Output = Self;

            #[inline]
            #[must_use]
            fn div(self, rhs: Self) -> Self::Output {
                Self((self.0 / rhs.0) & <Self as crate::prelude::VariableWord>::MASK)
            }
        }

        impl core::ops::DivAssign for #name {
            #[inline]
            #[must_use]
            fn div_assign(&mut self, rhs: Self) {
                *self = *self / rhs;
            }
        }

        impl core::ops::BitAnd for #name {
            type Output = Self;

            #[inline]
            #[must_use]
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }

        impl core::ops::BitAndAssign for #name {
            #[inline]
            #[must_use]
            fn bitand_assign(&mut self, rhs: Self) {
                *self = *self & rhs;
            }
        }

        impl core::ops::BitOr for #name {
            type Output = Self;

            #[inline]
            #[must_use]
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        impl core::ops::BitOrAssign for #name {
            #[inline]
            #[must_use]
            fn bitor_assign(&mut self, rhs: Self) {
                *self = *self | rhs;
            }
        }

        impl core::ops::BitXor for #name {
            type Output = Self;

            #[inline]
            #[must_use]
            fn bitxor(self, rhs: Self) -> Self::Output {
                Self(self.0 ^ rhs.0)
            }
        }

        impl core::ops::BitXorAssign for #name {
            #[inline]
            #[must_use]
            fn bitxor_assign(&mut self, rhs: Self) {
                *self = *self ^ rhs;
            }
        }

        impl<A> core::ops::Shl<A> for #name where u64: core::ops::Shl<A, Output = u64> {
            type Output = Self;

            #[inline]
            #[must_use]
            fn shl(self, rhs: A) -> Self::Output {
                Self(self.0 << rhs)
            }
        }

        impl<A> core::ops::ShlAssign<A> for #name where u64: core::ops::ShlAssign<A> {
            #[inline]
            #[must_use]
            fn shl_assign(&mut self, rhs: A) {
                self.0 <<= rhs;
                self.0 &= <Self as crate::prelude::VariableWord>::MASK;
            }
        }

        impl<A> core::ops::Shr<A> for #name where u64: core::ops::Shr<A, Output = u64> {
            type Output = Self;

            #[inline]
            #[must_use]
            fn shr(self, rhs: A) -> Self::Output {
                Self(self.0 >> rhs)
            }
        }

        impl<A> core::ops::ShrAssign<A> for #name where u64: core::ops::ShrAssign<A> {
            #[inline]
            #[must_use]
            fn shr_assign(&mut self, rhs: A) {
                self.0 >>= rhs;
            }
        }

        impl core::cmp::PartialEq for #name {
            #[inline]
            #[must_use]
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl core::cmp::Eq for #name {}

        impl core::cmp::PartialOrd for #name {
            #[inline]
            #[must_use]
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                self.0.partial_cmp(&other.0)
            }
        }

        impl core::cmp::Ord for #name {
            #[inline]
            #[must_use]
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                self.0.cmp(&other.0)
            }
        }

        impl core::hash::Hash for #name {
            #[inline]
            #[must_use]
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }

        impl core::default::Default for #name {
            #[inline]
            #[must_use]
            fn default() -> Self {
                Self(0)
            }
        }

        impl crate::prelude::Number for #name {
            #[inline]
            #[must_use]
            fn saturating_zero_sub(self, other: Self) -> Self {
                if self < other {
                    <Self as crate::prelude::Zero>::ZERO
                } else {
                    self - other
                }
            }
        }

        impl crate::prelude::PositiveInteger for #name {
            type TryFromU64Error = <Self as core::convert::TryFrom<u64>>::Error;

            #[inline]
            #[must_use]
            fn try_from_u64(value: u64) -> Result<Self, Self::TryFromU64Error> {
                Self::try_from(value)
            }

            #[inline]
            #[must_use]
            unsafe fn unchecked_from_u64(value: u64) -> Self {
                Self(value)
            }

            #[inline]
            #[must_use]
            fn to_usize(self) -> usize {
                self.0 as usize
            }
        }

        impl crate::prelude::Zero for #name {
            const ZERO: Self = Self(0);

            #[inline]
            #[must_use]
            fn is_zero(&self) -> bool {
                self.0 == 0
            }
        }

        impl crate::prelude::One for #name {
            const ONE: Self = Self(1);

            #[inline]
            #[must_use]
            fn is_one(&self) -> bool {
                self.0 == 1
            }
        }

        impl core::ops::Not for #name {
            type Output = Self;

            #[inline]
            #[must_use]
            fn not(self) -> Self::Output {
                Self(!self.0 & <Self as crate::prelude::VariableWord>::MASK)
            }
        }

        impl Clone for #name {
            #[inline]
            #[must_use]
            fn clone(&self) -> Self {
                Self(self.0)
            }
        }

        impl Copy for #name {}

        impl core::fmt::Debug for #name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl TryFrom<u64> for #name {
            type Error = &'static str;

            #[inline]
            #[must_use]
            fn try_from(value: u64) -> Result<Self, Self::Error> {
                if value > <Self as crate::prelude::VariableWord>::MASK {
                    Err("Value is too large for the word size")
                } else {
                    Ok(Self(value))
                }
            }
        }

        impl From<u8> for #name {
            #[inline]
            #[must_use]
            fn from(value: u8) -> Self {
                Self(value as u64)
            }
        }

        impl From<u16> for #name {
            #[inline]
            #[must_use]
            fn from(value: u16) -> Self {
                Self(value as u64)
            }
        }

        impl From<u32> for #name {
            #[inline]
            #[must_use]
            fn from(value: u32) -> Self {
                Self(value as u64)
            }
        }

        impl Into<u64> for #name {
            #[inline]
            #[must_use]
            fn into(self) -> u64 {
                self.0
            }
        }

        impl TryInto<u8> for #name {
            type Error = &'static str;

            #[inline]
            #[must_use]
            fn try_into(self) -> Result<u8, Self::Error> {
                if self.0 > u8::MAX as u64 {
                    Err("Value is too large for u8")
                } else {
                    Ok(self.0 as u8)
                }
            }
        }

        impl TryInto<u16> for #name {
            type Error = &'static str;

            #[inline]
            #[must_use]
            fn try_into(self) -> Result<u16, Self::Error> {
                if self.0 > u16::MAX as u64 {
                    Err("Value is too large for u16")
                } else {
                    Ok(self.0 as u16)
                }
            }
        }

        impl TryInto<u32> for #name {
            type Error = &'static str;

            #[inline]
            #[must_use]
            fn try_into(self) -> Result<u32, Self::Error> {
                if self.0 > u32::MAX as u64 {
                    Err("Value is too large for u32")
                } else {
                    Ok(self.0 as u32)
                }
            }
        }

        #[cfg(feature = "std")]
        impl crate::prelude::Named for #name {
            #[inline]
            #[must_use]
            fn name(&self) -> String {
                "#name".to_owned()
            }
        }
    };

    // Return the generated impl
    TokenStream::from(expanded)
}
