use convert_case::{Case, Casing};
use darling::{ast::Data, FromDeriveInput, FromVariant};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Generics, Ident, LitStr, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(injected_intr), supports(enum_unit))]
struct InjectedIntrDeriveInput {
    ident: Ident,
    generics: Generics,
    data: Data<InjectedIntrVariantInput, ()>,

    #[darling(rename = "data")]
    data_ty: Type,
    #[darling(rename = "error")]
    error_ty: Type,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(injected_intr), supports(unit))]
struct InjectedIntrVariantInput {
    ident: Ident,

    /// Name of the intrisic
    name: Option<LitStr>,

    /// Path of the called function
    calls: syn::Expr,

    /// Injection paths in the standard library
    #[darling(default)]
    std: Vec<LitStr>,

    /// Add to the prelude
    #[darling(default)]
    prelude: bool,
}

pub fn injected_intr(input: &DeriveInput) -> TokenStream {
    let InjectedIntrDeriveInput {
        ident,
        generics,
        data: Data::Enum(mut variants),
        data_ty,
        error_ty,
    } = (match InjectedIntrDeriveInput::from_derive_input(input) {
        Ok(v) => v,
        Err(e) => return e.write_errors(),
    })
    else {
        unreachable!()
    };

    for InjectedIntrVariantInput {
        ident,
        name,
        calls: _,
        std,
        prelude,
    } in &mut variants
    {
        if name.is_none() {
            *name = Some(LitStr::new(
                &ident.to_string().to_case(Case::Snake),
                ident.span(),
            ));
        }
        if *prelude {
            std.push(LitStr::new("prelude.", Span::call_site()));
        }
    }

    let variants = variants;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    if variants.is_empty() {
        // Special impl for empty enums

        return quote! {
            #[automatically_derived]
            impl #impl_generics ::dices_ast::intrisics::InjectedIntr for #ident #ty_generics #where_clause {
                type Data = #data_ty;
                type Error = #error_ty;

                fn iter() -> impl ::core::iter::IntoIterator<Item = Self> {
                    []
                }

                fn name(&self) -> &'static str {
                    match *self {}
                }

                fn named(_name: &str) -> ::core::option::Option<Self> {
                    ::core::option::Option::None
                }

                fn std_paths(&self) -> &[&[&'static str]] {
                    match *self {}
                }

                fn call(
                    &self,
                    _data: &mut Self::Data,
                    _params: Box<[::dices_ast::Value<Self>]>,
                ) -> Result<::dices_ast::Value<Self>, Self::Error> {
                    match *self {}
                }
            }
        };
    }

    let variants_idents = variants.iter().map(|v| &v.ident).collect::<Vec<_>>();

    let variants_names = variants
        .iter()
        .map(|v| v.name.as_ref().unwrap())
        .collect::<Vec<_>>();

    let variants_paths = variants.iter().map(|v| {
        let paths = v.std.iter().map(|p| {
            let mut path = p.value();
            if path.ends_with('.') {
                path.push_str(&v.name.as_ref().unwrap().value());
            }
            let components = path.split('.').map(|s| LitStr::new(s, p.span()));
            quote! {
                &[#(#components),*]
            }
        });
        quote! {
            &[#(#paths),*]
        }
    });

    let variants_calls = variants.iter().map(|v| &v.calls);

    quote! {
        #[automatically_derived]
        impl #impl_generics ::dices_ast::intrisics::InjectedIntr for #ident #ty_generics #where_clause {
            type Data = #data_ty;
            type Error = #error_ty;

            fn iter() -> impl ::core::iter::IntoIterator<Item = Self> {
                [#(Self::#variants_idents),*]
            }

            fn name(&self) -> &'static str {
                match self {
                  #(
                    Self::#variants_idents => #variants_names
                  ),*
                }
            }

            fn named(name: &str) -> ::core::option::Option<Self> {
                match name {
                    #(
                        #variants_names => ::core::option::Option::Some(Self::#variants_idents),
                    )*
                    _ => ::core::option::Option::None,
                }
            }

            fn std_paths(&self) -> &[&[&'static str]] {
                match self {
                    #(
                        Self::#variants_idents => #variants_paths
                    ),*
                }
            }

            fn call(
                &self,
                data: &mut Self::Data,
                params: Box<[Value<Self>]>,
            ) -> Result<Value<Self>, Self::Error> {
                match self {
                    #(
                        Self::#variants_idents => (#variants_calls) (data, params)
                    ),*
                }
            }
        }
    }
}
