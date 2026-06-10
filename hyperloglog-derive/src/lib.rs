//! Submodule providing the derive macro for the VariableWord trait.
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn};

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
pub fn test_precisions_and_bits(attr: TokenStream, item: TokenStream) -> TokenStream {
    // We parse the provided attributes, if any, which we expect to be a list of tuples with
    // the precision and bits to exclude from the test. We expect the format to be:
    // [(5, 4), (6,7),(9,5),], as a JSON deserializable string.

    let string_attr = attr.to_string();

    let exclude: Vec<(u8, u8)> = if string_attr.is_empty() {
        vec![]
    } else {
        serde_json::from_str(&string_attr).expect(&format!(
            "Invalid format for the exclude attribute: '{}'",
            &string_attr
        ))
    };

    let exclude_ref = exclude.as_slice();

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
        (4..=6).zip(bits.iter()).flat_map(move |(bit_size, bit)| {
            if exclude_ref.contains(&(precision_exponent as u8, bit_size as u8)) {
                return quote! {};
            }

            let test_fn_name = Ident::new(
                &format!("{}_{}_{}", fn_name, precision, bit).to_lowercase(),
                fn_name.span(),
            );

            quote! {
                #[test]
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
    let test_functions = precisions.iter().flat_map(|precision| {
        let hashers = hashers.clone();
        (bits).iter().flat_map(move |bit| {
            let hashers = hashers.clone();
            hashers.into_iter().flat_map(move |hasher| {
                    let mut feature_constraints =
                        vec![];

                    // If in the name of the function there appears the word MLE, we add the feature mle
                    if fn_name.to_string().contains("mle") {
                        feature_constraints.push(quote! { #[cfg(feature = "mle")] });
                    }

                    let array_test_fn_name = Ident::new(
                        &format!(
                            "{}_{}_{}_{}_array",
                            fn_name, precision, bit, hasher
                        )
                        .to_lowercase(),
                        fn_name.span(),
                    );

                    let vec_test_fn_name = Ident::new(
                        &format!(
                            "{}_{}_{}_{}_vec",
                            fn_name, precision, bit, hasher
                        )
                        .to_lowercase(),
                        fn_name.span(),
                    );

                    quote! {
                        #[test]
                        #(#feature_constraints)*
                        fn #array_test_fn_name() {
                            #fn_name::<#precision, #bit, <#precision as PackedRegister<#bit>>::Array, #hasher>();
                        }
                        #[test]
                        #[cfg(feature = "alloc")]
                        #(#feature_constraints)*
                        fn #vec_test_fn_name() {
                            #fn_name::<#precision, #bit, <#precision as PackedRegister<#bit>>::Vec, #hasher>();
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
