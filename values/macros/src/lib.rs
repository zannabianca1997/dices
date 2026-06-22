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
/// referring inside the struct), an [`Injectable`] impl whose `as_readable`
/// returns `Some(self)`, and a [`Describable`] impl whose description starts
/// with `"module "` followed by the snake_case of the struct name.
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

/// Turn a free function into an [`Injectable`] + [`Callable`] + [`Describable`]
/// unit struct.
///
/// The context parameter (if any) must be marked with `#[cx]`. The remaining
/// parameters are converted from the call arguments, and the return value is
/// converted back into a [`Value`]. The [`Describable`] description starts with
/// `"function "` followed by the snake_case of the function name.
///
/// [`Injectable`]: dices_values::injected::Injectable
/// [`Callable`]: dices_values::injected::call::Callable
/// [`Describable`]: dices_values::injected::describable::Describable
/// [`Value`]: dices_values::Value
#[proc_macro_attribute]
pub fn injectable(attr: TokenStream, item: TokenStream) -> TokenStream {
    attribute::injectable(attr.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
