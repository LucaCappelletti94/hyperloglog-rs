//! Submodule providing the derive macro for the VariableWord trait.
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, ItemFn, Type};

/// Possible variants for the word size currently supported.
enum WordSize {
    /// 24-bit word.
    U24,
    /// 40-bit word.
    U40,
    /// 48-bit word.
    U48,
    /// 56-bit word.
    U56,
}

impl From<&Ident> for WordSize {
    fn from(ident: &Ident) -> Self {
        if ident.to_string().contains("24") {
            WordSize::U24
        } else if ident.to_string().contains("40") {
            WordSize::U40
        } else if ident.to_string().contains("48") {
            WordSize::U48
        } else if ident.to_string().contains("56") {
            WordSize::U56
        } else {
            panic!("The struct name must contain either 24, 40, 48, or 56");
        }
    }
}

impl WordSize {
    fn number_of_bits(&self) -> u8 {
        match self {
            WordSize::U24 => 24,
            WordSize::U40 => 40,
            WordSize::U48 => 48,
            WordSize::U56 => 56,
        }
    }

    fn mask(&self) -> u64 {
        match self {
            WordSize::U24 => 0xFF_FFFF,
            WordSize::U40 => 0xFF_FFFF_FFFF,
            WordSize::U48 => 0xFFFF_FFFF_FFFF,
            WordSize::U56 => 0xFF_FFFF_FFFF_FFFF,
        }
    }
}

/// The words that can be used underneath the hood.
enum WordType {
    U32,
    U64,
}

impl From<Type> for WordType {
    fn from(ty: Type) -> Self {
        match ty {
            Type::Path(type_path) => {
                let segment = type_path.path.segments.first().unwrap();
                let ident = &segment.ident;
                if ident.to_string() == "u32" {
                    WordType::U32
                } else if ident.to_string() == "u64" {
                    WordType::U64
                } else {
                    panic!("The word type must be either u32 or u64");
                }
            }
            _ => panic!("The word type must be either u32 or u64"),
        }
    }
}

impl WordType {
    fn bits(&self) -> usize {
        match self {
            WordType::U32 => 32,
            WordType::U64 => 64,
        }
    }

    fn bytes(&self) -> usize {
        self.bits() / 8
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
    let field = match &data_struct.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0],
        _ => panic!("The struct must have exactly one unnamed field"),
    };

    // We get the type of the field
    let field_type = &field.ty;
    let word_type: WordType = WordType::from(field_type.clone());

    // Get the word size from the struct name
    let word_size = WordSize::from(name);
    let number_of_bits = word_size.number_of_bits();
    let number_of_bits_usize = number_of_bits as usize;
    let mask = word_size.mask();
    let word_bytes = word_type.bytes();

    // Generate the necessary traits for the word
    let expanded = quote! {
        impl crate::prelude::VariableWord for #name {
            const NUMBER_OF_BITS: u8 = #number_of_bits;
            const MASK: u64 = #mask;
            type Word = Self;
        }

        impl crate::prelude::AsBytes for #name {
            type Bytes = [u8; #number_of_bits_usize / 8];

            #[inline]
            #[must_use]
            fn as_bytes(self) -> Self::Bytes {
                self.into()
            }
        }

        impl From<[u8; #number_of_bits_usize / 8]> for #name {
            #[inline]
            #[must_use]
            fn from(bytes: [u8; #number_of_bits_usize / 8]) -> Self {
                let mut array = [0; #word_bytes];
                array[#word_bytes - #number_of_bits_usize / 8..].copy_from_slice(&bytes);
                Self(#field_type::from_be_bytes(array))
            }
        }

        impl Into<[u8; #number_of_bits_usize / 8]> for #name {
            #[inline]
            #[must_use]
            fn into(self) -> [u8; #number_of_bits_usize / 8] {
                let mut bytes = [0; #number_of_bits_usize / 8];
                bytes.copy_from_slice(&self.0.to_be_bytes()[#word_bytes - #number_of_bits_usize / 8..]);
                bytes
            }
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
                Self((self.0 + rhs.0) & (<Self as crate::prelude::VariableWord>::MASK as #field_type))
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
                Self((self.0.wrapping_sub(rhs.0)) & (<Self as crate::prelude::VariableWord>::MASK as #field_type))
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
                Self((self.0 * rhs.0) & (<Self as crate::prelude::VariableWord>::MASK as #field_type))
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
                Self((self.0 / rhs.0) & (<Self as crate::prelude::VariableWord>::MASK as #field_type))
            }
        }

        impl core::ops::DivAssign for #name {
            #[inline]
            #[must_use]
            fn div_assign(&mut self, rhs: Self) {
                *self = *self / rhs;
            }
        }

        impl core::ops::Rem for #name {
            type Output = Self;

            #[inline]
            #[must_use]
            fn rem(self, rhs: Self) -> Self::Output {
                Self((self.0 % rhs.0) & (<Self as crate::prelude::VariableWord>::MASK as #field_type))
            }
        }

        impl core::ops::RemAssign for #name {
            #[inline]
            #[must_use]
            fn rem_assign(&mut self, rhs: Self) {
                *self = *self % rhs;
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

        impl<A> core::ops::Shl<A> for #name where #field_type: core::ops::Shl<A, Output = #field_type> {
            type Output = Self;

            #[inline]
            #[must_use]
            fn shl(self, rhs: A) -> Self::Output {
                Self(self.0 << rhs)
            }
        }

        impl<A> core::ops::ShlAssign<A> for #name where #field_type: core::ops::ShlAssign<A> {
            #[inline]
            #[must_use]
            fn shl_assign(&mut self, rhs: A) {
                self.0 <<= rhs;
                self.0 &= (<Self as crate::prelude::VariableWord>::MASK as #field_type);
            }
        }

        impl<A> core::ops::Shr<A> for #name where #field_type: core::ops::Shr<A, Output = #field_type> {
            type Output = Self;

            #[inline]
            #[must_use]
            fn shr(self, rhs: A) -> Self::Output {
                Self(self.0 >> rhs)
            }
        }

        impl<A> core::ops::ShrAssign<A> for #name where #field_type: core::ops::ShrAssign<A> {
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
                Self(value as #field_type)
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
                Self(!self.0 & (<Self as crate::prelude::VariableWord>::MASK as #field_type))
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
                    Ok(Self(value as #field_type))
                }
            }
        }

        impl From<u8> for #name {
            #[inline]
            #[must_use]
            fn from(value: u8) -> Self {
                Self(value as #field_type)
            }
        }

        impl From<u16> for #name {
            #[inline]
            #[must_use]
            fn from(value: u16) -> Self {
                Self(value as #field_type)
            }
        }

        impl From<u32> for #name {
            #[inline]
            #[must_use]
            fn from(value: u32) -> Self {
                Self(value as #field_type)
            }
        }

        impl Into<u64> for #name {
            #[inline]
            #[must_use]
            fn into(self) -> u64 {
                self.0 as u64
            }
        }

        impl TryInto<u8> for #name {
            type Error = &'static str;

            #[inline]
            #[must_use]
            fn try_into(self) -> Result<u8, Self::Error> {
                if self.0 > u8::MAX as #field_type {
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
                if self.0 > u16::MAX as #field_type {
                    Err("Value is too large for u16")
                } else {
                    Ok(self.0 as u16)
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

#[proc_macro_attribute]
pub fn test_variable_words(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input token stream (the function we're deriving for)
    let input = parse_macro_input!(item as ItemFn);

    // Extract the function name
    let fn_name = &input.sig.ident;

    // Define a list of generics we want to cover
    let mut generics = vec![
        Ident::new("u8", fn_name.span()),
        Ident::new("u16", fn_name.span()),
        Ident::new("u24", fn_name.span()),
        Ident::new("u32", fn_name.span()),
        Ident::new("u40", fn_name.span()),
        Ident::new("u48", fn_name.span()),
        Ident::new("u56", fn_name.span()),
        Ident::new("u64", fn_name.span()),
    ];

    // We add the Bits{i} for the range 1-8
    for i in 1..=8 {
        generics.push(Ident::new(&format!("Bits{}", i), fn_name.span()));
    }

    // Generate the test functions
    let test_functions = generics.iter().map(|generic| {
        let test_fn_name = Ident::new(
            &format!("{}_{}", fn_name, generic).to_lowercase(),
            fn_name.span(),
        );
        quote! {
            #[test]
            /// Test the #generic type
            fn #test_fn_name() {
                #fn_name::<#generic>();
            }
        }
    });

    // Generate the final token stream
    let expanded = quote! {
        #input

        #(#test_functions)*
    };

    // Convert the expanded code into a token stream
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn test_array(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input token stream (the function we're deriving for)
    let input = parse_macro_input!(item as ItemFn);

    // Extract the function name
    let fn_name = &input.sig.ident;

    // Define a list of generics we want to cover
    let mut generics = vec![
        Ident::new("u8", fn_name.span()),
        Ident::new("u16", fn_name.span()),
        Ident::new("u24", fn_name.span()),
        Ident::new("u32", fn_name.span()),
        Ident::new("u40", fn_name.span()),
        Ident::new("u48", fn_name.span()),
        Ident::new("u56", fn_name.span()),
        Ident::new("u64", fn_name.span()),
    ];

    // We add the Bits{i} for the range 1-8
    for i in 1..=8 {
        generics.push(Ident::new(&format!("Bits{}", i), fn_name.span()));
    }

    // Generate the test functions
    let test_functions = generics.iter().flat_map(|generic| {
        [true, false].into_iter().flat_map(move |packed| {
            [0_usize, 1_usize, 2_usize, 3_usize, 4_usize, 5_usize, 6_usize, 7_usize, 8_usize]
                .into_iter()
                .map(move |number_of_words| {
                    let packed_name = if packed { "packed_" } else { "" };

                    let test_fn_name = Ident::new(
                        &format!("{}_{}_{}{}", fn_name, generic, packed_name, number_of_words)
                            .to_lowercase(),
                        fn_name.span(),
                    );
                    quote! {
                        #[test]
                        /// Test the #generic type
                        fn #test_fn_name() {
                            const NUMBER_OF_WORDS: usize = Array::<#number_of_words, #packed, #generic>::number_of_values() as usize;
                            let mut reference = [<<#generic as VariableWord>::Word as Zero>::ZERO; NUMBER_OF_WORDS];
                            for (value, element) in iter_random_values::<#generic>(NUMBER_OF_WORDS as u64, None, None).zip(reference.iter_mut()) {
                                *element = value;
                            }
                            #fn_name::<NUMBER_OF_WORDS, #number_of_words, #packed, #generic>(reference);
                        }
                    }
                })
        })
    });

    // Generate the final token stream
    let expanded = quote! {
        #input

        #(#test_functions)*
    };

    // Convert the expanded code into a token stream
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn test_all_precisions_and_bits(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input token stream (the function we're deriving for)
    let input = parse_macro_input!(item as ItemFn);

    // Extract the function name
    let fn_name = &input.sig.ident;

    // Define a list of generics we want to cover
    let precisions = (4..=18)
        .map(|precision| Ident::new(&format!("Precision{}", precision), fn_name.span()))
        .collect::<Vec<_>>();
    let bits = (4..=6)
        .map(|bits| Ident::new(&format!("Bits{}", bits), fn_name.span()))
        .collect::<Vec<_>>();

    // Generate the test functions
    let test_functions = precisions.iter().enumerate().flat_map(|(i, precision)| {
        let precision_exponent = i + 4;
        (bits).iter().flat_map(move |bit| {
            let test_fn_name = Ident::new(
                &format!(
                    "{fn_name}_{precision}_{bit}",
                )
                .to_lowercase(),
                fn_name.span(),
            );

            // For each precision, we need to check whether the feature precision_{exponent} is enabled
            let precision_flag = format!("precision_{precision_exponent}");
            let feature_constraints =
                vec![quote! { #[cfg(feature = #precision_flag)] }];

            quote! {
                #[test]
                #(#feature_constraints)*
                fn #test_fn_name() {
                    #fn_name::<#precision, #bit>();
                }
            }
        })
    });

    // Generate the final token stream
    let expanded = quote! {
        #input

        #(#test_functions)*
    };

    // Convert the expanded code into a token stream
    TokenStream::from(expanded)
}


#[proc_macro_attribute]
pub fn test_estimator(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input token stream (the function we're deriving for)
    let input = parse_macro_input!(item as ItemFn);

    // Extract the function name
    let fn_name = &input.sig.ident;

    // Define a list of generics we want to cover
    let precisions = (4..=18)
        .map(|precision| Ident::new(&format!("Precision{}", precision), fn_name.span()))
        .collect::<Vec<_>>();
    let bits = (4..=6)
        .map(|bits| Ident::new(&format!("Bits{}", bits), fn_name.span()))
        .collect::<Vec<_>>();
    let hashers = vec![
        Ident::new("XxHash", fn_name.span()),
        Ident::new("WyHash", fn_name.span()),
    ];

    // Generate the test functions
    let test_functions = precisions.iter().enumerate().flat_map(|(i, precision)| {
        let precision_exponent = i + 4;
        let hashers = hashers.clone();
        (bits).iter().flat_map(move |bit| {
            let hashers = hashers.clone();
            hashers.into_iter().flat_map(move |hasher| {
                [true, false].into_iter().map(move |packed| {
                    let packed_name = if packed { "packed_" } else { "" };

                    let test_fn_name = Ident::new(
                        &format!(
                            "{}_{}_{}{}_{}",
                            fn_name, precision, packed_name, bit, hasher
                        )
                        .to_lowercase(),
                        fn_name.span(),
                    );

                    // For each precision, we need to check whether the feature precision_{exponent} is enabled
                    let precision_flag = format!("precision_{precision_exponent}");
                    let mut feature_constraints =
                        vec![quote! { #[cfg(feature = #precision_flag)] }];

                    // If in the name of the function there appears the word MLE, we add the feature mle
                    if fn_name.to_string().contains("mle") {
                        feature_constraints.push(quote! { #[cfg(feature = "mle")] });
                    }

                    // If in the name of the function there appears the word plusplus, we add the feature plusplus
                    if fn_name.to_string().contains("plusplus") {
                        feature_constraints.push(quote! { #[cfg(feature = "plusplus")] });
                    }

                    // If in the name of the function there appears the word beta, we add the feature beta
                    if fn_name.to_string().contains("beta") {
                        feature_constraints.push(quote! { #[cfg(feature = "beta")] });
                    }
                    
                    if packed {
                        quote! {
                            #[test]
                            #(#feature_constraints)*
                            fn #test_fn_name() {
                                #fn_name::<#precision, #bit, <#precision as ArrayRegister<#bit>>::Packed, #hasher>();
                            }
                        }
                    } else {
                        quote! {
                            #[test]
                            #(#feature_constraints)*
                            fn #test_fn_name() {
                                #fn_name::<#precision, #bit, <#precision as ArrayRegister<#bit>>::Array, #hasher>();
                            }
                        }
                    }
                })
            })
        })
    });

    // Generate the final token stream
    let expanded = quote! {
        #input

        #(#test_functions)*
    };

    // Convert the expanded code into a token stream
    TokenStream::from(expanded)
}
