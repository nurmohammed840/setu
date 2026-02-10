use syn::*;

pub fn get_attr<'a>(field: &'a Field, name: &str) -> Option<&'a Expr> {
    field.attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(kv) => kv.path.is_ident(name).then_some(&kv.value),
        _ => None,
    })
}
