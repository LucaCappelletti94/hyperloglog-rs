//! Submodule providing the derive macro for the VariableWord trait.
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Ident, ItemFn, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Possible variants for the word size currently supported.
enum WordType {
    /// 8-bit word.
    U8,
    /// 16-bit word.
    U16,
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
            WordType::U32,
            WordType::U64,
        ]
    }
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
pub fn test_precisions_and_bits(_attr: TokenStream, item: TokenStream) -> TokenStream {
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
                    "{}_{}_{}",
                    fn_name, precision, bit
                )
                .to_lowercase(),
                fn_name.span(),
            );

            // For each precision, we need to check whether the feature precision_{exponent} is enabled
            let precision_flag = format!("precision_{precision_exponent}");
            quote! {
                #[test]
                #[cfg(feature = #precision_flag)]
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
