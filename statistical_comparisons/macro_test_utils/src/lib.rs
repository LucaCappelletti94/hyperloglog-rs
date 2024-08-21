//! Provides a procedural macro to implement all of the methods of a trait for an enum
//! of single-fielded variants, all of which impleent the trait.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Named)]
pub fn my_trait_derive(input: TokenStream) -> TokenStream {
    // Parse the input TokenStream into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;

    // Handle generic parameters (e.g., <T, U>) and where clauses (e.g., where T: Debug)
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Ensure the input is an enum
    let data_enum = match &input.data {
        Data::Enum(data_enum) => data_enum,
        _ => panic!("MyTrait can only be derived for enums"),
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
            #name::#variant_name(inner) => inner.name(),
        }
    });

    // Generate the trait implementation, including support for possible
    // generic parameters that the enum may have.

    let expanded = quote! {
        impl #impl_generics Named for #name #ty_generics #where_clause {
            fn name(&self) -> String {
                match self {
                    #(#variants)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(ExtendableApproximatedSet)]
pub fn extendable_approximated_set_derive(input: TokenStream) -> TokenStream {
    // Parse the input TokenStream into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;

    // Handle generic parameters (e.g., <T, U>) and where clauses (e.g., where T: Debug)
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Ensure the input is an enum
    let data_enum = match &input.data {
        Data::Enum(data_enum) => data_enum,
        _ => panic!("ExtendableApproximatedSet can only be derived for enums"),
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
            #name::#variant_name(inner) => inner.insert(element),
        }
    });

    // Generate the trait implementation, including support for possible
    // generic parameters that the enum may have.

    let expanded = quote! {
        impl #impl_generics ExtendableApproximatedSet<u64> for #name #ty_generics #where_clause {
            fn insert(&mut self, element: &u64) -> bool {
                match self {
                    #(#variants)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Estimator)]
pub fn estimator_derive(input: TokenStream) -> TokenStream {
    // Parse the input TokenStream into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;

    // Handle generic parameters (e.g., <T, U>) and where clauses (e.g., where T: Debug)
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Ensure the input is an enum
    let data_enum = match &input.data {
        Data::Enum(data_enum) => data_enum,
        _ => panic!("Estimator can only be derived for enums"),
    };

    // Generate match arms for each variant of the enum
    let mut estimate_cardinality = Vec::new();
    let mut estimate_union_cardinality = Vec::new();
    let mut estimate_union_cardinality_with_cardinalities = Vec::new();

    data_enum.variants.iter().for_each(|variant| {
        let variant_name = &variant.ident;

        // Ensure the variant has exactly one unnamed field (i.e., a tuple variant)
        let _ = match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0],
            _ => panic!("Each enum variant must have exactly one unnamed field"),
        };

        estimate_cardinality.push(quote! {
            #name::#variant_name(inner) => inner.estimate_cardinality(),
        });

        estimate_union_cardinality.push(quote! {
            (#name::#variant_name(inner), #name::#variant_name(other)) => inner.estimate_union_cardinality(other),
        });

        estimate_union_cardinality_with_cardinalities.push(quote! {
            (#name::#variant_name(inner), #name::#variant_name(other)) => inner.estimate_union_cardinality_with_cardinalities(other, cardinality, other_cardinality),
        });
    });

    // Generate the trait implementation, including support for possible
    // generic parameters that the enum may have.

    let expanded = quote! {
        impl #impl_generics Estimator<f64> for #name #ty_generics #where_clause {
            fn estimate_cardinality(&self) -> f64 {
                match self {
                    #(#estimate_cardinality)*
                }
            }

            fn estimate_union_cardinality_with_cardinalities(&self, other: &Self, cardinality: f64, other_cardinality: f64) -> f64 {
                match (self, other) {
                    #(#estimate_union_cardinality_with_cardinalities)*
                    _ => panic!("Union cardinality with cardinalities not defined for these variants."),
                }
            }

            fn estimate_union_cardinality(&self, other: &Self) -> f64 {
                match (self, other) {
                    #(#estimate_union_cardinality)*
                    _ => panic!("Union cardinality not defined for these variants."),
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
