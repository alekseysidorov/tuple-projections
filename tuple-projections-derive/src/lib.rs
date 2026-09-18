//! Procedural macros for `tuple-projections`.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, LitInt, parse_macro_input};

/// Generates the finite set of left projections for tuples up to `max_arity`.
#[proc_macro]
pub fn impl_left_projections(input: TokenStream) -> TokenStream {
    let max_arity = parse_macro_input!(input as LitInt);
    let max_arity = match max_arity.base10_parse::<usize>() {
        Ok(value) => value,
        Err(error) => {
            return syn::Error::new(max_arity.span(), error)
                .to_compile_error()
                .into();
        }
    };

    let implementations = (0..=max_arity)
        .flat_map(tuple_projection_implementations)
        .collect::<TokenStream2>();

    quote!(#implementations).into()
}

/// Derives [`tuple_projections::TupleRepr`] and the corresponding projections.
#[proc_macro_derive(TupleProjection)]
pub fn derive_tuple_projection(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_tuple_projection_impl(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn tuple_projection_implementations(arity: usize) -> impl Iterator<Item = TokenStream2> {
    let types = (0..arity)
        .map(|index| format_ident!("T{index}"))
        .collect::<Vec<_>>();

    (0..=arity).map(move |prefix_len| {
        let target = tuple_type(&types);
        let prefix = tuple_type(&types[..prefix_len]);
        let remainder = tuple_type(&types[prefix_len..]);

        quote! {
            impl<#(#types),*> ::tuple_projections::LeftProjectionOf<#target> for #prefix {
                type Remainder = #remainder;
            }
        }
    })
}

fn tuple_type<T: quote::ToTokens>(types: &[T]) -> TokenStream2 {
    match types {
        [] => quote!(()),
        [single] => quote!((#single,)),
        many => quote!((#(#many),*)),
    }
}

fn derive_tuple_projection_impl(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let (field_types, into_body, from_body) = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let names = fields
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().expect("named field"))
                    .collect::<Vec<_>>();
                let types = fields
                    .named
                    .iter()
                    .map(|field| &field.ty)
                    .collect::<Vec<_>>();

                (
                    types,
                    quote! {
                        let Self { #(#names),* } = self;
                        (#(#names),*)
                    },
                    quote! {
                        let (#(#names),*) = tuple;
                        Self { #(#names),* }
                    },
                )
            }
            Fields::Unnamed(fields) => {
                let names = (0..fields.unnamed.len())
                    .map(|index| format_ident!("field_{index}"))
                    .collect::<Vec<_>>();
                let types = fields
                    .unnamed
                    .iter()
                    .map(|field| &field.ty)
                    .collect::<Vec<_>>();

                (
                    types,
                    quote! {
                        let Self(#(#names),*) = self;
                        (#(#names),*)
                    },
                    quote! {
                        let (#(#names),*) = tuple;
                        Self(#(#names),*)
                    },
                )
            }
            Fields::Unit => (Vec::new(), quote!({}), quote!(Self)),
        },
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "`TupleProjection` can only be derived for structs, not enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "`TupleProjection` can only be derived for structs, not unions",
            ));
        }
    };

    let tuple = tuple_type(&field_types);
    let projection_impls = (0..=field_types.len()).map(|prefix_len| {
        let prefix = tuple_type(&field_types[..prefix_len]);
        let remainder = tuple_type(&field_types[prefix_len..]);

        quote! {
            impl #impl_generics ::tuple_projections::LeftProjectionOf<#name #ty_generics>
                for #prefix #where_clause
            {
                type Remainder = #remainder;
            }
        }
    });

    Ok(quote! {
        impl #impl_generics ::tuple_projections::TupleRepr for #name #ty_generics #where_clause {
            type Tuple = #tuple;

            fn into_tuple(self) -> Self::Tuple {
                #into_body
            }

            fn from_tuple(tuple: Self::Tuple) -> Self {
                #from_body
            }
        }

        #(#projection_impls)*
    })
}
