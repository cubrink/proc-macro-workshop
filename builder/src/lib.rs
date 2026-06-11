use proc_macro::TokenStream;
mod builder;
mod diagnostics;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    builder::derive(input.into()).into()
}
