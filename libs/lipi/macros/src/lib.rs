mod utils;

use lipi_derive::{quote2, syn};
use proc_macro::TokenStream;
use quote2::*;

#[proc_macro_derive(Encode, attributes(key))]
pub fn encoder(input: TokenStream) -> TokenStream {
    let Ok(input) = syn::parse(input) else {
        return TokenStream::new();
    };

    let mut t = proc_macro2::TokenStream::new();
    lipi_derive::encoder::expand(&utils::crate_path!(::lipi), &input, &mut t, "key");
    t.into()
}

#[proc_macro_derive(Decode, attributes(key, default))]
pub fn decoder(input: TokenStream) -> TokenStream {
    let Ok(input) = syn::parse(input) else {
        return TokenStream::new();
    };

    let mut t = proc_macro2::TokenStream::new();
    lipi_derive::decoder::expand(
        &utils::crate_path!(::lipi),
        &input,
        &mut t,
        "key",
        "default",
    );
    t.into()
}
