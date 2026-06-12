use quote2::ToTokens;
use quote2::proc_macro2::TokenStream;
use syn::*;

pub fn get_attr(attrs: &[Attribute], name: &str) -> Option<TokenStream> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(kv) => kv.path.is_ident(name).then(|| kv.value.to_token_stream()),
        Meta::List(list) => list.path.is_ident(name).then(|| list.tokens.clone()),
        Meta::Path(path) => path.is_ident(name).then(TokenStream::new),
    })
}
