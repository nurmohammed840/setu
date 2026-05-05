use proc_macro::TokenStream;
use type_id_derive::{proc_macro2, quote2, syn};

mod utils;

use quote2::quote;
use syn::DeriveInput;

#[proc_macro_derive(TypeId)]
pub fn type_id(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(input);
    let mut output = proc_macro2::TokenStream::new();

    type_id_derive::expand(utils::crate_path!(::type_id_v2), &input, &mut output);

    output.into()
}
