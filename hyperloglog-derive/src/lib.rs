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
        serde_json::from_str(&string_attr).unwrap_or_else(|error| {
            panic!("Invalid format for the exclude attribute: '{string_attr}': {error}")
        })
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

    // Default CI matrix. Non-default combinations are gated behind
    // `#[cfg(feature = "exhaustive-tests")]` so a nightly / pre-release sweep
    // still exercises the entire cross product.
    const DEFAULT_PRECISIONS: &[u8] = &[4, 8, 12, 16];
    const DEFAULT_BITS: &[u8] = &[4, 6];

    // Generate the test functions
    let test_functions = precisions.iter().enumerate().flat_map(|(i, precision)| {
        let precision_exponent = (i + 4) as u8;
        (4u8..=6).zip(bits.iter()).flat_map(move |(bit_size, bit)| {
            if exclude_ref.contains(&(precision_exponent, bit_size)) {
                return quote! {};
            }

            let test_fn_name = Ident::new(
                &format!("{}_{}_{}", fn_name, precision, bit).to_lowercase(),
                fn_name.span(),
            );

            let is_default = DEFAULT_PRECISIONS.contains(&precision_exponent)
                && DEFAULT_BITS.contains(&bit_size);
            let feature_gate = if is_default {
                quote! {}
            } else {
                quote! { #[cfg(feature = "exhaustive-tests")] }
            };

            quote! {
                #[test]
                #feature_gate
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
    // Each hasher carries a short name (used to build the generated test function names) and a
    // fully-qualified path (used as the type argument), so the test crate does not need to import
    // the hasher type itself.
    let hashers: Vec<(&str, proc_macro2::TokenStream)> = vec![
        ("xxhash", quote! { twox_hash::XxHash }),
        // ("wyhash", quote! { wyhash::WyHash }),
        // ("ahasher", quote! { ahash::AHasher }),
    ];

    // Default CI matrix. Non-default combinations are gated behind
    // `#[cfg(feature = "exhaustive-tests")]` so a pre-release sweep still
    // exercises every precision and bit width. The vec-backed variant was
    // dropped entirely; the vec register backend is covered by the unit tests
    // under `registers::packed_array`.
    const DEFAULT_PRECISIONS: &[usize] = &[4, 8, 12, 16];
    const DEFAULT_BITS: &[u8] = &[4, 6];

    // Generate the test functions
    let test_functions = precisions.iter().enumerate().flat_map(|(idx, precision)| {
        let precision_exponent = idx + 4;
        let hashers = hashers.clone();
        (4u8..=6).zip(bits.iter()).flat_map(move |(bit_size, bit)| {
            let hashers = hashers.clone();
            let is_default = DEFAULT_PRECISIONS.contains(&precision_exponent)
                && DEFAULT_BITS.contains(&bit_size);
            let feature_gate = if is_default {
                quote! {}
            } else {
                quote! { #[cfg(feature = "exhaustive-tests")] }
            };
            hashers.into_iter().map(move |(hasher_name, hasher_path)| {
                let array_test_fn_name = Ident::new(
                    &format!(
                        "{}_{}_{}_{}_array",
                        fn_name, precision, bit, hasher_name
                    )
                    .to_lowercase(),
                    fn_name.span(),
                );

                let feature_gate = feature_gate.clone();
                quote! {
                    #[test]
                    #feature_gate
                    fn #array_test_fn_name() {
                        #fn_name::<#precision, #bit, <#precision as PackedRegister<#bit>>::Array, #hasher_path>();
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
