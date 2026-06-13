use proc_macro2::{Literal, TokenStream};
use quote2::*;

pub fn add_compile_error(t: &mut TokenStream, span: proc_macro2::Span, msg: &str) {
    let mut msg = Literal::string(msg);
    msg.set_span(span);
    quote_spanned!(span, t, {
        ::core::compile_error! { #msg }
    });
}
