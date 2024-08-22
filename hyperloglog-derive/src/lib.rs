//! Submodule providing the derive macro for the VariableWord trait.
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, ItemFn, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Possible variants for the word size currently supported.
enum WordType {
    /// 8-bit word.
    U8,
    /// 16-bit word.
    U16,
    /// 24-bit word.
    U24,
    /// 32-bit word.
    U32,
    /// 64-bit word.
    U64,
}

impl core::fmt::Display for WordType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WordType::U8 => "u8",
                WordType::U16 => "u16",
                WordType::U24 => "u24",
                WordType::U32 => "u32",
                WordType::U64 => "u64",
            }
        )
    }
}

impl From<&str> for WordType {
    fn from(s: &str) -> Self {
        match s {
            "u8" => WordType::U8,
            "u16" => WordType::U16,
            "u24" => WordType::U24,
            "u32" => WordType::U32,
            "u64" => WordType::U64,
            _ => panic!("Unsupported word size"),
        }
    }
}

impl From<&Ident> for WordType {
    fn from(ident: &Ident) -> Self {
        if ident.to_string().contains("u8") {
            WordType::U8
        } else if ident.to_string().contains("u16") {
            WordType::U16
        } else if ident.to_string().contains("u24") {
            WordType::U24
        } else if ident.to_string().contains("u32") {
            WordType::U32
        } else if ident.to_string().contains("u64") {
            WordType::U64
        } else {
            panic!("Unsupported word size");
        }
    }
}

impl ToTokens for WordType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            WordType::U8 => quote! { u8 },
            WordType::U16 => quote! { u16 },
            WordType::U24 => quote! { u24 },
            WordType::U32 => quote! { u32 },
            WordType::U64 => quote! { u64 },
        }
        .to_tokens(tokens);
    }
}

impl From<Type> for WordType {
    fn from(ty: Type) -> Self {
        match ty {
            Type::Path(type_path) => {
                let segment = type_path.path.segments.first().unwrap();
                let ident = &segment.ident;
                if *ident == "u8" {
                    WordType::U8
                } else if *ident == "u16" {
                    WordType::U16
                } else if *ident == "u24" {
                    WordType::U24
                } else if *ident == "u32" {
                    WordType::U32
                } else if *ident == "u64" {
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
    /// Returns all the variants.
    fn variants() -> Vec<Self> {
        vec![
            WordType::U8,
            WordType::U16,
            WordType::U24,
            WordType::U32,
            WordType::U64,
        ]
    }

    /// Returns whether the word is a power of two.
    fn is_power_of_two(&self) -> bool {
        match self {
            WordType::U8 => true,
            WordType::U16 => true,
            WordType::U24 => false,
            WordType::U32 => true,
            WordType::U64 => true,
        }
    }

    /// Returns the underlying field type.
    fn field_type(&self) -> Type {
        match self {
            WordType::U8 => syn::parse_quote! { u8 },
            WordType::U16 => syn::parse_quote! { u16 },
            WordType::U24 => syn::parse_quote! { u32 },
            WordType::U32 => syn::parse_quote! { u32 },
            WordType::U64 => syn::parse_quote! { u64 },
        }
    }

    /// Returns all the smaller variants of the word size.
    fn smaller_variants(&self) -> Vec<Self> {
        Self::variants()
            .into_iter()
            .filter(|variant| variant.number_of_bits() < self.number_of_bits())
            .collect()
    }

    /// Returns all variants that are strictly larger than the current one, and are a power of two.
    fn larger_power_of_two_variants(&self) -> Vec<Self> {
        Self::variants()
            .into_iter()
            .filter(|variant| {
                (variant.number_of_bits() > self.number_of_bits()) && variant.is_power_of_two()
            })
            .collect()
    }

    fn number_of_bits(&self) -> u8 {
        match self {
            WordType::U8 => 8,
            WordType::U16 => 16,
            WordType::U24 => 24,
            WordType::U32 => 32,
            WordType::U64 => 64,
        }
    }

    fn mask(&self) -> u64 {
        match self {
            WordType::U8 => 0xFF,
            WordType::U16 => 0xFFFF,
            WordType::U24 => 0xFF_FFFF,
            WordType::U32 => 0xFFFF_FFFF,
            WordType::U64 => 0xFFFF_FFFF_FFFF_FFFF,
        }
    }

    fn bytes(&self) -> usize {
        usize::from(self.number_of_bits() / 8)
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
    let word_size = WordType::from(name);
    let number_of_bits = word_size.number_of_bits();
    let number_of_bits_usize = number_of_bits as usize;
    let mask = word_size.mask();
    let word_bytes = word_type.bytes();

    let test_bytes_symmetry_for_name =
        Ident::new(&format!("test_bytes_symmetry_for_{name}"), name.span());

    // Generate the necessary traits for the word
    let mut expanded = quote! {
        impl crate::prelude::VariableWord for #name {
            const NUMBER_OF_BITS: u8 = #number_of_bits;
            const MASK: u64 = #mask;
            type Word = Self;

            #[inline]
            #[allow(unsafe_code)]
            unsafe fn unchecked_from_u64(value: u64) -> Self::Word {
                debug_assert!(value <= <Self as crate::prelude::VariableWord>::MASK, "The value is too large for the number.");
                Self(value as #field_type)
            }
        }

        impl From<[u8; #number_of_bits_usize / 8]> for #name {
            #[inline]
            #[must_use]
            fn from(bytes: [u8; #number_of_bits_usize / 8]) -> Self {
                let mut array = [0; #word_bytes];
                array[..#number_of_bits_usize / 8].copy_from_slice(&bytes);
                Self(<#field_type>::from_le_bytes(array))
            }
        }

        impl Into<[u8; #number_of_bits_usize / 8]> for #name {
            #[inline]
            #[must_use]
            fn into(self) -> [u8; #number_of_bits_usize / 8] {
                let array = self.0.to_le_bytes();
                let mut bytes = [0; #number_of_bits_usize / 8];
                bytes.copy_from_slice(&array[..#number_of_bits_usize / 8]);
                bytes
            }
        }

        #[cfg(test)]
        mod #test_bytes_symmetry_for_name {
            use super::*;

            #[test]
            fn test_bytes_symmetry() {
                for value in crate::prelude::iter_random_values::<#name>(1_000, Some(#name::try_from(#name::MASK).unwrap()), None) {
                    let bytes: [u8; #number_of_bits_usize / 8] = value.into();
                    let value_from_bytes: #name = <#name>::from(bytes);
                    assert_eq!(value, value_from_bytes);
                }
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
                Self(self.0 - rhs.0)
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
                Self(self.0 / rhs.0)
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
                Self(self.0 % rhs.0)
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
                let result = self.0 << rhs;
                debug_assert!(result <= <Self as crate::prelude::VariableWord>::MASK as #field_type, "The value {result} is too large for the number.");
                Self(result)
            }
        }

        impl<A: core::fmt::Debug> core::ops::ShlAssign<A> for #name where #field_type: core::ops::ShlAssign<A> {
            #[inline]
            #[must_use]
            fn shl_assign(&mut self, rhs: A) {
                self.0 <<= rhs;
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

        impl Copy for #name {}

        impl core::fmt::Debug for #name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<bool> for #name {
            #[inline]
            #[must_use]
            fn from(value: bool) -> Self {
                Self(#field_type::from(value))
            }
        }

        #[cfg(feature = "std")]
        impl crate::prelude::Named for #name {
            #[inline]
            #[must_use]
            fn name(&self) -> String {
                stringify!(#name).to_string()
            }
        }
    };

    // We iterate the variant that are smaller than the current one and implement the From trait
    // for each one. Next, we iterate all variants that are larger and we implement the TryFrom.

    // We iterate the smaller variants
    for smaller_variant in word_size.smaller_variants() {
        let smaller_variant_name = smaller_variant.to_string();
        let smaller_variant_ident = Ident::new(&smaller_variant_name, name.span());

        let field_type_of_smaller_variant = smaller_variant.field_type();

        if smaller_variant.is_power_of_two() {
            expanded.extend(quote! {
                impl TryInto<#smaller_variant_ident> for #name {
                    type Error = &'static str;

                    #[inline]
                    #[must_use]
                    fn try_into(self) -> Result<#smaller_variant_ident, Self::Error> {
                        if self.0 > <Self as crate::prelude::VariableWord>::MASK as #field_type{
                            Err("The value is too large for the number.")
                        } else {
                            Ok(self.0 as #field_type_of_smaller_variant)
                        }
                    }
                }

                impl From<#smaller_variant_ident> for #name {
                    #[inline]
                    #[must_use]
                    fn from(value: #smaller_variant_ident) -> Self {
                        Self(value as #field_type)
                    }
                }
            });
        } else {
            expanded.extend(quote! {
                impl TryInto<#smaller_variant_ident> for #name {
                    type Error = &'static str;

                    #[inline]
                    #[must_use]
                    fn try_into(self) -> Result<#smaller_variant_ident, Self::Error> {
                        if self.0 > <Self as crate::prelude::VariableWord>::MASK as #field_type{
                            Err("The value is too large for the number.")
                        } else {
                            Ok(#smaller_variant_ident(self.0 as #field_type_of_smaller_variant))
                        }
                    }
                }

                impl From<#smaller_variant_ident> for #name {
                    #[inline]
                    #[must_use]
                    fn from(value: #smaller_variant_ident) -> Self {
                        Self(value.0 as #field_type)
                    }
                }
            });
        }
    }

    // We iterate the larger variants
    for larger_variant in word_size.larger_power_of_two_variants() {
        let larger_variant_name = larger_variant.to_string();
        let larger_variant_ident = Ident::new(&larger_variant_name, name.span());

        let field_type_of_larger_variant = larger_variant.field_type();

        if larger_variant.is_power_of_two() {
            expanded.extend(quote! {
                impl TryFrom<#larger_variant_ident> for #name {
                    type Error = &'static str;

                    #[inline]
                    #[must_use]
                    fn try_from(value: #larger_variant_ident) -> Result<Self, Self::Error> {
                        if value > <Self as crate::prelude::VariableWord>::MASK as #field_type_of_larger_variant {
                            Err("The value is too large for the number.")
                        } else {
                            Ok(Self(value as #field_type))
                        }
                    }
                }

                impl Into<#larger_variant_ident> for #name {
                    #[inline]
                    #[must_use]
                    fn into(self) -> #field_type_of_larger_variant {
                        self.0 as #field_type_of_larger_variant
                    }
                }
            });
        } else {
            expanded.extend(quote! {
                impl TryFrom<#larger_variant_ident> for #name {
                    type Error = &'static str;

                    #[inline]
                    #[must_use]
                    fn try_from(value: #larger_variant_ident) -> Result<Self, Self::Error> {
                        if value.0 > <Self as crate::prelude::VariableWord>::MASK as #field_type_of_larger_variant {
                            Err("The value is too large for the number.")
                        } else {
                            Ok(Self(value.0 as #field_type))
                        }
                    }
                }

                impl Into<#larger_variant_ident> for #name {
                    #[inline]
                    #[must_use]
                    fn into(self) -> #larger_variant_ident {
                        #larger_variant_ident(self.0 as #field_type_of_larger_variant)
                    }
                }
            });
        }
    }

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
    ];

    // We add the Bits{i} for the range 1-8
    for i in 4..=6 {
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
/// Derive the test functions for the Array struct
pub fn test_array(attr: TokenStream, item: TokenStream) -> TokenStream {
    // We see whether there any attributes
    let string_attr = attr.to_string();
    let deny: Vec<WordType> = if string_attr.is_empty() {
        vec![]
    } else {
        string_attr
            .split(',')
            .map(|word| word.trim())
            .map(WordType::from)
            .collect::<Vec<_>>()
    };

    // Parse the input token stream (the function we're deriving for)
    let input = parse_macro_input!(item as ItemFn);

    // Extract the function name
    let fn_name = &input.sig.ident;

    // Define a list of generics we want to cover
    let mut generics: Vec<Ident> = WordType::variants()
        .into_iter()
        .filter(|word| !deny.contains(word))
        .map(|word| Ident::new(&format!("{word}",), fn_name.span()))
        .collect::<Vec<_>>();

    // We check that u8 was not denied
    if !deny.contains(&WordType::U8) {
        // We add the Bits{i} for the range 1-8
        for i in 4..=6 {
            generics.push(Ident::new(&format!("Bits{}", i), fn_name.span()));
        }
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
        // Ident::new("WyHash", fn_name.span()),
        // Ident::new("AHasher", fn_name.span()),
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
