//! Procedural macros for `dices-values`.
//!
//! This crate only contains the thin `proc-macro` entry points. All the actual
//! logic lives in the [`derive`] and [`attribute`] modules, which work purely
//! in terms of the `proc-macro2` ecosystem (`proc_macro2`, `syn`, `quote`) and
//! never touch the compiler-builtin `proc_macro` crate.

use proc_macro::TokenStream;

mod attribute;
mod derive;

/// Derive [`Injectable`] for a named-field struct.
///
/// Generates a [`Readable`] map view of the struct (one entry per field, each
/// referring inside the struct) plus an [`Injectable`] impl whose `as_readable`
/// returns `Some(self)`. When the struct carries a doc comment, a [`Describable`]
/// impl is generated from it as well.
///
/// [`Injectable`]: dices_values::injected::Injectable
/// [`Readable`]: dices_values::injected::read::Readable
/// [`Describable`]: dices_values::injected::describable::Describable
#[proc_macro_derive(Injectable)]
pub fn derive_injectable(input: TokenStream) -> TokenStream {
    derive::derive_injectable(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Turn a free function into an [`Injectable`] + [`Callable`] unit struct.
///
/// The context parameter (if any) must be marked with `#[cx]`. The remaining
/// parameters are converted from the call arguments, and the return value is
/// converted back into a [`Value`].
///
/// [`Injectable`]: dices_values::injected::Injectable
/// [`Callable`]: dices_values::injected::call::Callable
/// [`Value`]: dices_values::Value
#[proc_macro_attribute]
pub fn injectable(attr: TokenStream, item: TokenStream) -> TokenStream {
    attribute::injectable(attr.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
