use proc_macro::TokenStream;
mod builder;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    builder::derive(input.into()).into()
}
