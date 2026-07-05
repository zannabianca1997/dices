//! Implementation of the `#[injectable]` attribute macro.
//!
//! Converts functions into injectable types

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    FnArg, ItemFn, PatType, ReturnType, Token, Type, TypeReference, TypeSlice, parse2,
    spanned::Spanned,
};

use crate::derive::{describable_impl, manual_page_impl, parse_doc_for_manual_page};

pub fn injectable(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    if !attr.is_empty() {
        return Err(syn::Error::new(
            attr.span(),
            "`#[injectable]` does not take any arguments",
        ));
    }

    let func: ItemFn = parse2(item)?;

    let vis = &func.vis;
    let name = &func.sig.ident;

    // Split the parameters into the (optional) context parameter and the value
    // parameters, keying off the `#[cx]` marker attribute.
    let mut cx_seen = false;
    let mut value_tys: Vec<&Type> = Vec::new();
    // For each parameter, in declaration order: `None` means it is the context,
    // `Some(i)` means it is the `i`-th value parameter.
    let mut order: Vec<Option<usize>> = Vec::new();
    // Variadic parameter position and mutability
    let mut variadic: Option<(&Option<Token![mut]>, &Type)> = None;

    for (position, input) in func.sig.inputs.iter().with_position() {
        match input {
            FnArg::Receiver(receiver) => {
                return Err(syn::Error::new(
                    receiver.span(),
                    "`#[injectable]` only supports free functions, not methods with `self`",
                ));
            }
            FnArg::Typed(PatType { attrs, ty, .. }) => {
                if attrs.iter().any(|a| a.path().is_ident("cx")) {
                    if cx_seen {
                        return Err(syn::Error::new(
                            input.span(),
                            "only one parameter can be marked `#[cx]`",
                        ));
                    }
                    cx_seen = true;
                    order.push(None);
                } else if let Type::Reference(TypeReference {
                    mutability, elem, ..
                }) = &**ty
                    && let Type::Slice(TypeSlice { elem, .. }) = &**elem
                {
                    if !position.is_last() {
                        return Err(syn::Error::new(
                            input.span(),
                            "variadic parameters must be last",
                        ));
                    }
                    variadic = Some((mutability, elem));
                } else {
                    order.push(Some(value_tys.len()));
                    value_tys.push(ty);
                }
            }
        }
    }

    let n_args = value_tys.len();

    // The inner function: the original definition, verbatim, with `#[cx]`
    // stripped from its parameters, re-homed as an inherent associated function.
    let inner = inner_fn(&func);

    // Fetch each required argument, padding missing ones with null.
    let arg_idents: Vec<_> = (0..n_args)
        .map(|i| format_ident!("__arg{i}"))
        .collect();

    let arg_fetches = arg_idents.iter().enumerate().map(|(i, ident)| {
        quote! {
            let #ident = ::core::option::Option::unwrap_or(
                args.get(#i).cloned(),
                ::dices_values::Value::Null(::dices_values::null::ValueNull),
            );
        }
    });

    let variadic_fetch = variadic.map(|_| {
        quote! { let __args = &args[#n_args..]; }
    });

    // Convert each argument, preferring `TryFrom<Value>` then `Deserialize`.
    let conversions = value_tys
        .iter()
        .zip(&arg_idents)
        .enumerate()
        .map(|(i, (ty, ident))| {
            let bind = format_ident!("__val{i}");
            quote! {
                let #bind: #ty = (&&::dices_values::injected::convert::ArgTag::<#ty>::new())
                    .convert(#ident)?;
            }
        })
        .chain(variadic.map(|(mutability, ty)| {
            quote! {
                let #mutability __values: ::std::vec::Vec<#ty> = __args.iter().map(|item| {
                    (&&::dices_values::injected::convert::ArgTag::<#ty>::new())
                        .convert(::core::clone::Clone::clone(item))
                }).collect::<::core::result::Result<_, _>>()?;
            }
        }));

    // Build the inner call argument list, in original declaration order.
    let call_args = order
        .iter()
        .map(|slot| match slot {
            None => quote! { cx },
            Some(i) => {
                let bind = format_ident!("__val{i}");
                quote! { #bind }
            }
        })
        .chain(variadic.map(|(mutability, _)| quote! { & #mutability __values }));

    // The return type, used to drive the return conversion.
    let call_and_return = match &func.sig.output {
        ReturnType::Default => {
            quote! {
                let () = Self::call(#(#call_args),*);
                ::std::result::Result::Ok(::dices_values::Value::Null(::dices_values::null::ValueNull))
            }
        }
        ReturnType::Type(_, ty) if matches!(**ty, Type::Never(_)) => {
            quote! { Self::call(#(#call_args),*) }
        }
        ReturnType::Type(_, ty) => {
            quote! {
                let __ret = Self::call(#(#call_args),*);
                (&&&&::dices_values::injected::convert::RetTag::<#ty>::new()).convert(__ret)
            }
        }
    };

    // The context parameter is named `cx` when the function actually uses one,
    // otherwise `_cx` to avoid an unused-variable warning.
    let cx_ident = if cx_seen {
        format_ident!("cx")
    } else {
        format_ident!("_cx")
    };

    let describable = describable_impl("function", name, quote! { impl }, quote! { for #name });

    let manual_page =
        parse_doc_for_manual_page(&func.attrs).map(|(path, title, content)| {
            manual_page_impl(name, &path, &title, &content)
        });

    Ok(quote! {
        #[derive(
            ::core::fmt::Debug,
            ::core::default::Default,
            ::core::clone::Clone,
            ::core::marker::Copy,
            ::core::cmp::PartialEq,
            ::core::cmp::Eq,
            ::core::cmp::PartialOrd,
            ::core::cmp::Ord,
            ::core::hash::Hash,
        )]
        #vis struct #name;

        impl #name {
            #inner
        }

        #manual_page
        #describable

        impl ::dices_values::injected::call::Callable for #name {
            fn call(
                &self,
                #cx_ident: &mut dyn ::dices_values::injected::call::InjectedContext,
                args: &[::dices_values::Value],
            ) -> ::core::result::Result<
                ::dices_values::Value,
                ::std::boxed::Box<dyn ::std::error::Error>,
            > {
                // Bring the autoref-specialization conversion traits into scope.
                #[allow(unused_imports)]
                use ::dices_values::injected::convert::{
                    ViaTryFrom as _, ViaDeserialize as _,
                    ReturnViaTryInto as _, ReturnViaSerialize as _,
                    ReturnFallibleViaTryInto as _, ReturnFallibleViaSerialize as _,
                };
                #(#arg_fetches)*
                #variadic_fetch
                #(#conversions)*

                #call_and_return
            }
        }

        impl ::dices_values::injected::Injectable for #name {
            fn as_callable(&self) -> ::core::option::Option<&dyn ::dices_values::injected::call::Callable> {
                ::core::option::Option::Some(self)
            }
        }
    })
}

/// Rebuild the original function as the inherent `call` associated function,
/// stripping the `#[cx]` marker from its parameters and dropping the outer doc
/// attributes (those live on the generated `Describable` impl instead).
fn inner_fn(func: &ItemFn) -> ItemFn {
    let mut inner = func.clone();
    inner.attrs.clear();
    inner.vis = syn::Visibility::Inherited;
    inner.sig.ident = format_ident!("call");
    for input in &mut inner.sig.inputs {
        if let FnArg::Typed(pat) = input {
            pat.attrs.retain(|a| !a.path().is_ident("cx"));
        }
    }
    inner
}
