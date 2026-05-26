use proc_macro::TokenStream;
use setu_derive::expend_export;
use syn::parse_macro_input;
use quote2::proc_macro2;

#[proc_macro]
pub fn export(input: TokenStream) -> TokenStream {
    let list = parse_macro_input!(input);
    
    let crate_path = crate_path();
    let mut t = proc_macro2::TokenStream::new();
    expend_export(&crate_path, &list, &mut t);
    t.into()
}

fn crate_path() -> proc_macro2::TokenStream {
    use quote2::*;
    let mut out = proc_macro2::TokenStream::new();
    quote!(out, { ::setu });
    out
}
