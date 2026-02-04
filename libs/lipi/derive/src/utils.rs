use syn::*;

pub fn get_key(field: &Field) -> Option<&Expr> {
    field.attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(kv) => kv.path.is_ident("key").then_some(&kv.value),
        _ => None,
    })
}
