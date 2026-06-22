//! Implementation of `#[derive(Injectable)]`.
//!
//! Implements injectable and describable for a struct

use heck::ToSnakeCase as _;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Fields, Ident, Token, WhereClause, parse_quote, parse2,
    punctuated::Punctuated, spanned::Spanned,
};

pub fn derive_injectable(input: TokenStream) -> syn::Result<TokenStream> {
    let input: DeriveInput = parse2(input)?;

    let fields = named_fields(&input)?;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut injectable_where_clause = where_clause.cloned().unwrap_or_else(|| WhereClause {
        where_token: Default::default(),
        predicates: Punctuated::new(),
    });
    injectable_where_clause
        .predicates
        .push(parse_quote!( Self: ::dices_values::injected::RequiredTraits ));

    // One map entry per field, each referring inside the struct so it stays
    // injectable.
    let inserts: Vec<_> = fields
        .iter()
        .filter_map(|field| {
            // TODO: make this parse fail in a noisier way
            match field
                .attrs
                .iter()
                .filter(|attr| attr.meta.path().is_ident(&"injectable"))
                .map(|attr| {
                    attr.meta.require_list().and_then(|list| {
                        list.parse_args_with(Punctuated::<Ident, Token![,]>::parse_terminated)
                    })
                })
                .process_results(|skipped| skipped.flatten().any(|ident| ident == "skip"))
            {
                Ok(true) => return None,
                Ok(false) => (),
                Err(err) => return Some(Err(err)),
            }

            let ident = field.ident.as_ref().expect("named field");
            let key = ident.to_string();
            let ty = &field.ty;

            injectable_where_clause.predicates.push(parse_quote!(
                #ty: ::dices_values::injected::Injectable
            ));

            Some(Ok(quote! {
                map.insert(
                    ::dices_values::string::ValueString::new_static(#key),
                    ::dices_values::injected::read::ValueOrInject::Inject(
                        &self.#ident as &dyn ::dices_values::injected::Injectable
                    ),
                );
            }))
        })
        .try_collect()?;

    let describable = describable_impl(
        "module",
        name,
        quote! { impl #impl_generics },
        quote! { for #name #ty_generics #where_clause },
    );

    Ok(quote! {
        #describable

        impl #impl_generics ::dices_values::injected::read::Readable for #name #ty_generics #injectable_where_clause {
            fn read(
                &self,
            ) -> ::core::result::Result<
                ::dices_values::injected::read::ReadValue<'_>,
                ::std::boxed::Box<dyn ::std::error::Error>,
            > {
                let mut map = ::std::collections::BTreeMap::new();
                #(#inserts)*
                ::core::result::Result::Ok(::dices_values::injected::read::ReadValue::Map(map))
            }
        }

        impl #impl_generics ::dices_values::injected::Injectable for #name #ty_generics #injectable_where_clause {
            fn as_readable(&self) -> ::core::option::Option<&dyn ::dices_values::injected::read::Readable> {
                ::core::option::Option::Some(self)
            }
        }
    })
}

/// Extract the named fields of a struct, erroring on anything else.
fn named_fields(
    input: &DeriveInput,
) -> syn::Result<&syn::punctuated::Punctuated<syn::Field, syn::token::Comma>> {
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => Ok(&named.named),
            other => Err(syn::Error::new(
                other.span(),
                "`#[derive(Injectable)]` is only supported on structs with named fields",
            )),
        },
        Data::Enum(_) | Data::Union(_) => Err(syn::Error::new(
            input.span(),
            "`#[derive(Injectable)]` is only supported on structs with named fields",
        )),
    }
}

/// Generate a [`Describable`] impl with a fixed prefix and the snake_case name.
///
/// `impl_head` is the `impl <generics>` prefix and `impl_tail` is the
/// `for Type <generics> <where>` suffix, so the same builder is reusable from
/// the attribute macro.
pub fn describable_impl(
    prefix: &str,
    name: &Ident,
    impl_head: TokenStream,
    impl_tail: TokenStream,
) -> TokenStream {
    let desc = format!("{prefix} {}", name.to_string().to_snake_case());
    quote! {
        #impl_head ::dices_values::injected::describable::Describable #impl_tail {
            fn description(&self) -> impl ::core::fmt::Display + '_ {
                #desc
            }
        }
    }
}
