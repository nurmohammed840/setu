use proc_macro2::{Literal, TokenStream};
use quote2::*;
use syn::*;

pub fn get_attr<'a>(attrs: &'a [Attribute], name: &str) -> Option<&'a Expr> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(kv) => kv.path.is_ident(name).then_some(&kv.value),
        _ => None,
    })
}

pub fn get_attr_or_expr(attrs: &[Attribute], name: &str) -> Option<TokenStream> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(kv) => kv.path.is_ident(name).then(|| kv.value.to_token_stream()),
        Meta::List(list) => list.path.is_ident(name).then(|| list.tokens.clone()),
        Meta::Path(path) => path.is_ident(name).then(TokenStream::new),
    })
}

pub fn is_numeric(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|attr| matches!(&attr.meta, Meta::Path(path) if path.is_ident("numeric")))
}

pub fn get_repr(attrs: &[Attribute]) -> Option<TokenStream> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::List(list) => list.path.is_ident("repr").then(|| list.tokens.clone()),
        _ => None,
    })
}

pub fn get_numeric_ty(attrs: &[Attribute]) -> Option<TokenStream> {
    is_numeric(attrs).then(|| get_repr(attrs))?
}

pub fn data_ty(input: &DeriveInput) -> QuoteFn<impl Fn(&mut TokenStream)> {
    quote(move |t| match input.data {
        Data::Struct(_) => {
            quote!(t, { __crate::DataType::Struct });
        }
        Data::Enum(_) => {
            if let Some(ty) = get_numeric_ty(&input.attrs) {
                quote!(t, { <#ty as __crate::Encode>::TY });
            } else {
                quote!(t, { __crate::DataType::Union });
            }
        }
        Data::Union(_) => unimplemented!(),
    })
}

pub fn add_compile_error(t: &mut TokenStream, span: proc_macro2::Span, msg: &str) {
    let mut msg: Literal = Literal::string(msg);
    msg.set_span(span);
    quote_spanned!(span, t, {
        ::core::compile_error! { #msg }
    });
}
