use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod derive;

#[proc_macro_derive(InjectedIntr, attributes(injected_intr))]
pub fn injected_intr(input: TokenStream) -> TokenStream {
    derive::injected_intr(parse_macro_input!(input as DeriveInput)).into()
}
