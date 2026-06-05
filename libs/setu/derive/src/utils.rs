use proc_macro2::TokenStream;
use quote2::{Quote, quote_spanned};

pub fn add_compile_error(t: &mut TokenStream, span: proc_macro2::Span, msg: &str) {
    let mut msg = proc_macro2::Literal::string(msg);
    msg.set_span(span);
    quote_spanned!(span, t, {
        ::core::compile_error! { #msg }
    });
}
