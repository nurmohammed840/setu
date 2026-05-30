use proc_macro::TokenStream;
use type_id_derive::{proc_macro2, quote2, syn};

mod utils;

use quote2::quote;

#[proc_macro_derive(TypeId)]
pub fn type_id(input: TokenStream) -> TokenStream {
    let Ok(input) = syn::parse(input) else {
        return TokenStream::new();
    };

    let mut output = proc_macro2::TokenStream::new();
    type_id_derive::expand(&utils::crate_path!(::type_id), &input, &mut output, "key");
    output.into()
}
