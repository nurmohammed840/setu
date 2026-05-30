use syn::*;

pub fn get_attr<'a>(attrs: &'a [Attribute], name: &str) -> Option<&'a Expr> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(kv) => kv.path.is_ident(name).then_some(&kv.value),
        _ => None,
    })
}
