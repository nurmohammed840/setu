use lipi_derive::{quote2, syn};
use proc_macro::TokenStream;

#[proc_macro_derive(Encoder, attributes(key))]
pub fn encoder(input: TokenStream) -> TokenStream {
    lipi_derive::encoder::expand(&syn::parse_macro_input!(input), crate_path(), "key").into()
}

#[proc_macro_derive(Decoder)]
pub fn decoder(input: TokenStream) -> TokenStream {
    lipi_derive::decoder::expand(&syn::parse_macro_input!(input), crate_path(), "key").into()
}

fn crate_path() -> quote2::proc_macro2::TokenStream {
    use quote2::*;
    let mut out = proc_macro2::TokenStream::new();
    quote!(out, { ::lipi });
    out
}
