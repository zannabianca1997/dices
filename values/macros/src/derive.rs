//! Implementation of `#[derive(Injectable)]`.
//!
//! Implements injectable and describable for a struct

use heck::ToSnakeCase as _;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Fields, Ident, Meta, Token, WhereClause, parse_quote, parse2,
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

    let manual_page = parse_doc_for_manual_page(&input.attrs)
        .map(|(path, title, content)| manual_page_impl(name, &path, &title, &content));

    Ok(quote! {
        #manual_page
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

/// Parse doc attributes (`/// ...`) on a function or struct to extract a
/// manual page section.
///
/// Expected format:
///
/// ```
/// /// 2.3. Title of the page
/// ///
/// /// Content of the page
/// ```
///
/// The section numbers form the path (`[2, 3]`), the rest of the first line
/// after the version is the title, and subsequent lines form the content.
/// Returns `None` if there are no doc comments or the first line doesn't
/// match the expected pattern.
pub fn parse_doc_for_manual_page(attrs: &[Attribute]) -> Option<(Vec<u16>, String, String)> {
    let docs: Vec<String> = attrs
        .iter()
        .filter_map(|attr| {
            if !attr.path().is_ident("doc") {
                return None;
            }
            match &attr.meta {
                Meta::NameValue(nv) => match &nv.value {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) => Some(s.value()),
                    _ => None,
                },
                _ => None,
            }
        })
        .collect();

    if docs.is_empty() {
        return None;
    }

    let doc_text = docs
        .iter()
        .map(|d| d.strip_prefix(' ').unwrap_or(d))
        .join("\n");

    let mut lines = doc_text.lines();
    let first_line = lines.next()?.trim();

    // Find the ". " that separates the version path from the title.
    //
    // Valid examples: "5.", "2.3.", "1.2.3." all followed by "<title>"
    // Invalid:        "5.Standard" (no space), "foo. bar" (non-digit version)
    let separator = find_title_separator(first_line)?;

    let version_str = &first_line[..separator];
    let components: Vec<u16> = version_str
        .split('.')
        .map(|s| s.parse::<u16>().ok())
        .collect::<Option<Vec<_>>>()?;

    if components.is_empty() {
        return None;
    }

    let title = first_line[separator + 2..].to_string();

    let content = lines.collect::<Vec<_>>().join("\n").trim().to_string();

    Some((components, title, content))
}

/// Find the offset of the `. ` separator between the numeric version path and
/// the page title.  Returns `None` if the first line does not contain a valid
/// version followed by `". "`.
fn find_title_separator(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Consume a digit sequence
        if !bytes[i].is_ascii_digit() {
            return None;
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        // After digits, we expect either ". " (title separator) or "." (more numbers)
        if i >= bytes.len() || bytes[i] != b'.' {
            return None;
        }
        if i + 1 < bytes.len() && bytes[i + 1] == b' ' {
            return Some(i);
        }
        // Skip the '.' and continue to next version component
        i += 1;
    }
    None
}

/// Generate a [`LinkedPage`] registration in the manual system.
///
/// Produces a `#[distributed_slice(LINKED_PAGES)]` static that inserts a
/// manual page derived from doc comments.
pub fn manual_page_impl(name: &Ident, path: &[u16], title: &str, content: &str) -> TokenStream {
    let static_ident = format_ident!("__MANUAL_PAGE_{}", name);
    let path_elems = path.iter().map(|&c| {
        let lit = proc_macro2::Literal::u16_suffixed(c);
        quote! { #lit }
    });

    quote! {
        #[::dices_man::registry::linked::distributed_slice(
            ::dices_man::registry::linked::LINKED_PAGES
        )]
        #[allow(non_upper_case_globals)]
        static #static_ident: ::dices_man::registry::linked::LinkedPage =
            ::dices_man::registry::linked::LinkedPage {
                path: &[#(#path_elems),*],
                title: #title,
                content: #content,
            };
    }
}
