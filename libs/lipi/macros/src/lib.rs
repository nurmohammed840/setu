use lipi_derive::{quote2, syn};
use proc_macro::TokenStream;

#[proc_macro_derive(Encode, attributes(key))]
pub fn encoder(input: TokenStream) -> TokenStream {
    let Ok(derive_input) = syn::parse(input) else {
        return TokenStream::new();
    };
    
    lipi_derive::encoder::expand(&derive_input, crate_path(), "key").into()
}

#[proc_macro_derive(Decode, attributes(key, default, foo))]
pub fn decoder(input: TokenStream) -> TokenStream {
    let Ok(derive_input) = syn::parse(input) else {
        return TokenStream::new();
    };

    lipi_derive::decoder::expand(&derive_input, crate_path(), "key", "default").into()
}

fn crate_path() -> quote2::proc_macro2::TokenStream {
    use quote2::*;
    let mut out = proc_macro2::TokenStream::new();
    quote!(out, { ::lipi });
    out
}
