mod utils;

use proc_macro::TokenStream;
use quote2::*;
use setu_derive::{expend_export, expend_type_definition, check_fn_args_count};
use syn::parse_macro_input;

#[proc_macro]
pub fn export(input: TokenStream) -> TokenStream {
    let list = parse_macro_input!(input);

    let crate_path = utils::crate_path!(::setu);
    let mut t = proc_macro2::TokenStream::new();
    expend_export(&crate_path, &list, &mut t);
    expend_type_definition(&crate_path, &list, &mut t);
    check_fn_args_count(&crate_path, &list, &mut t);
    t.into()
}

#[proc_macro_derive(Input, attributes(key))]
pub fn input(input: TokenStream) -> TokenStream {
    let Ok(input) = syn::parse(input) else {
        return TokenStream::new();
    };

    let lipi_path = utils::crate_path!(::setu::__private::lipi);
    let type_id_path = utils::crate_path!(::setu::__private::type_id);

    let mut t = proc_macro2::TokenStream::new();
    lipi_derive::decoder::expand(&lipi_path, &input, &mut t, "key", "default");
    type_id_derive::expand(&type_id_path, &input, &mut t, "key");
    t.into()
}

#[proc_macro_derive(Output, attributes(key, default))]
pub fn output(input: TokenStream) -> TokenStream {
    let Ok(input) = syn::parse(input) else {
        return TokenStream::new();
    };

    let lipi_path = utils::crate_path!(::setu::__private::lipi);
    let type_id_path = utils::crate_path!(::setu::__private::type_id);

    let mut t = proc_macro2::TokenStream::new();
    lipi_derive::encoder::expand(&lipi_path, &input, &mut t, "key");
    type_id_derive::expand(&type_id_path, &input, &mut t, "key");
    t.into()
}

#[proc_macro_derive(Message, attributes(key, default))]
pub fn message(input: TokenStream) -> TokenStream {
    let Ok(input) = syn::parse(input) else {
        return TokenStream::new();
    };

    let lipi_path = utils::crate_path!(::setu::__private::lipi);
    let type_id_path = utils::crate_path!(::setu::__private::type_id);

    let mut t = proc_macro2::TokenStream::new();
    lipi_derive::encoder::expand(&lipi_path, &input, &mut t, "key");
    lipi_derive::decoder::expand(&lipi_path, &input, &mut t, "key", "default");
    type_id_derive::expand(&type_id_path, &input, &mut t, "key");
    t.into()
}
