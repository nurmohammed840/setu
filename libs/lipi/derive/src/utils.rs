use proc_macro2::TokenStream;
use quote2::{ToTokens, quote};
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
        Meta::Path(path) => path.is_ident(name).then(|| TokenStream::new()),
    })
}

pub fn data_ty(data: &Data) -> quote2::QuoteFn<impl Fn(&mut TokenStream)> {
    quote(move |t| match data {
        Data::Struct(_) => {
            quote!(t, { DataType::Struct });
        }
        Data::Enum(_) => {
            quote!(t, { DataType::Union });
        }
        Data::Union(_) => unimplemented!(),
    })
}
