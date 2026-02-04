use lipi_derive::syn;
use proc_macro::TokenStream;

#[proc_macro_derive(Encoder, attributes(key))]
pub fn encoder(input: TokenStream) -> TokenStream {
    lipi_derive::encoder::expand(&syn::parse_macro_input!(input)).into()
}

#[proc_macro_derive(Decoder)]
pub fn decoder(input: TokenStream) -> TokenStream {
    lipi_derive::decoder::expand(&syn::parse_macro_input!(input)).into()
}
