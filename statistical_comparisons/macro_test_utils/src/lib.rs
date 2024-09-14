//! Provides a procedural macro to implement all of the methods of a trait for an enum
//! of single-fielded variants, all of which impleent the trait.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, ItemFn};

#[proc_macro_derive(Set)]
pub fn set_derive(input: TokenStream) -> TokenStream {
    // Parse the input TokenStream into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;

    // Handle generic parameters (e.g., <T, U>) and where clauses (e.g., where T: Debug)
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Ensure the input is an enum
    let data_enum = match &input.data {
        Data::Enum(data_enum) => data_enum,
        _ => panic!("Set can only be derived for enums"),
    };

    // Generate match arms for each variant of the enum
    let inserts = data_enum.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        // Ensure the variant has exactly one unnamed field (i.e., a tuple variant)
        let _ = match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0],
            _ => panic!("Each enum variant must have exactly one unnamed field"),
        };

        quote! {
            #name::#variant_name(inner) => {
                inner.insert_element(element);
            },
        }
    });

    let cardinalities = data_enum.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        // Ensure the variant has exactly one unnamed field (i.e., a tuple variant)
        let _ = match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0],
            _ => panic!("Each enum variant must have exactly one unnamed field"),
        };

        quote! {
            #name::#variant_name(inner) => inner.cardinality(),
        }
    });

    let unions = data_enum.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        // Ensure the variant has exactly one unnamed field (i.e., a tuple variant)
        let _ = match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0],
            _ => panic!("Each enum variant must have exactly one unnamed field"),
        };

        quote! {
            (#name::#variant_name(inner), #name::#variant_name(other)) => inner.union(other),
        }
    });

    let model_names = data_enum.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        // Ensure the variant has exactly one unnamed field (i.e., a tuple variant)
        let _ = match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0],
            _ => panic!("Each enum variant must have exactly one unnamed field"),
        };

        quote! {
            #name::#variant_name(inner) => inner.model_name(),
        }
    });

    // Generate the trait implementation, including support for possible
    // generic parameters that the enum may have.

    let expanded = quote! {
        impl #impl_generics Set for #name #ty_generics #where_clause {
            fn insert_element(&mut self, element: u64) {
                match self {
                    #(#inserts)*
                }
            }

            fn cardinality(&self) -> f64 {
                match self {
                    #(#cardinalities)*
                }
            }

            fn union(&self, other: &Self) -> f64 {
                match (self, other) {
                    #(#unions)*
                    _ => panic!("Cannot union different types"),
                }
            }

            fn model_name(&self) -> String {
                match self {
                    #(#model_names)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(TransparentMemSize)]
pub fn transparent_mem_size_derive(input: TokenStream) -> TokenStream {
    // Parse the input TokenStream into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;

    // Handle generic parameters (e.g., <T, U>) and where clauses (e.g., where T: Debug)
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Ensure the input is an enum
    let data_enum = match &input.data {
        Data::Enum(data_enum) => data_enum,
        _ => panic!("TransparentMemSize can only be derived for enums"),
    };

    // Generate match arms for each variant of the enum
    let variants = data_enum.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        // Ensure the variant has exactly one unnamed field (i.e., a tuple variant)
        let _ = match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0],
            _ => panic!("Each enum variant must have exactly one unnamed field"),
        };

        quote! {
            #name::#variant_name(inner) => inner.mem_size(mem_dbg::SizeFlags::default() | mem_dbg::SizeFlags::FOLLOW_REFS),
        }
    });

    // Generate the trait implementation, including support for possible
    // generic parameters that the enum may have.

    let expanded = quote! {
        impl #impl_generics TransparentMemSize for #name #ty_generics #where_clause {
            fn transparent_mem_size(&self) -> usize {
                match self {
                    #(#variants)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn cardinality_benchmark(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input token stream (the function we're deriving for)
    let input = parse_macro_input!(item as ItemFn);

    // Extract the function name
    let fn_name = &input.sig.ident;

    // Define a list of generics we want to cover
    let precisions = (4..=15)
        .map(|precision| {
            (
                precision,
                Ident::new(&format!("Precision{}", precision), fn_name.span()),
            )
        })
        .collect::<Vec<(usize, _)>>();
    let bits = (4..=6)
        .map(|bits| (bits, Ident::new(&format!("Bits{}", bits), fn_name.span())))
        .collect::<Vec<(usize, _)>>();
    let hashers = vec![
        Ident::new("XxHash64", fn_name.span()),
        // Ident::new("WyHash", fn_name.span()),
        // Ident::new("AHasher", fn_name.span()),
        // Ident::new("XxH3", fn_name.span()),
    ];

    // Generate the test functions

    let number_of_hashers = hashers.len();

    let hasher_loading_bar = quote! {
        let hasher_loading_bar = multiprogress.add(indicatif::ProgressBar::new(#number_of_hashers as u64));
        hasher_loading_bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("Hashers: [{elapsed_precise} | {eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
                .unwrap()
                .progress_chars("##-"),
        );
        hasher_loading_bar.tick();
    };
    let benchmarks = hashers.into_iter().flat_map(move |hasher| {
        let precisions = precisions.clone();
        let bits = bits.clone();
        let h2b_calls = quote! {
            HyperTwoVariants::<#hasher>::prepare_cardinality_reports(&multiprogress);
        };
        let number_of_precisions = precisions.len() as u64;
        let precision_loading_bar = quote! {
            let precision_loading_bar = multiprogress.add(indicatif::ProgressBar::new(#number_of_precisions));
            precision_loading_bar.set_style(
                indicatif::ProgressStyle::default_bar()
                    .template("Precision: [{elapsed_precise} | {eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
                    .unwrap()
                    .progress_chars("##-"),
            );
            precision_loading_bar.tick();
        };
        let hll_calls = precisions.into_iter().map(move |(exponent, precision)| {
            let bits = bits.clone();
            let hasher = hasher.clone();
            let number_of_bits = bits.len() as u64;
            let bits_loading_bar = quote! {
                let bits_loading_bar = multiprogress.add(indicatif::ProgressBar::new(#number_of_bits));
                bits_loading_bar.set_style(
                    indicatif::ProgressStyle::default_bar()
                        .template("Bits: [{elapsed_precise} | {eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
                        .unwrap()
                        .progress_chars("##-"),
                );
                bits_loading_bar.tick();
            };
            let bits_benches = bits
                .into_iter()
                .flat_map(move |(bit_num, bit)| {
                    let precision = precision.clone();
                    let hasher = hasher.clone();
                    quote! {
                        HLLVariants::<#exponent, #precision, #hasher, #bit_num, #bit>::prepare_cardinality_reports(&multiprogress);
                        bits_loading_bar.inc(1);
                    }
                });

            quote! {
                #bits_loading_bar
                #(#bits_benches)*
                bits_loading_bar.finish_and_clear();
                precision_loading_bar.inc(1);
            }
        });
        quote! {
            #h2b_calls
            #precision_loading_bar
            #(#hll_calls)*
            precision_loading_bar.finish_and_clear();
            hasher_loading_bar.inc(1);
        }
    });

    // Generate the final token stream
    let expanded = quote! {
        #input

        fn cardinality_benchmarks() {
            let multiprogress = indicatif::MultiProgress::new();
            #hasher_loading_bar
            #(#benchmarks)*
            hasher_loading_bar.finish_and_clear();
            multiprogress.clear().unwrap();
        }
    };

    // Convert the expanded code into a token stream
    TokenStream::from(expanded)
}
