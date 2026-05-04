use proc_macro2::TokenStream;
use quote2::quote;
use syn::*;

pub fn get_attr<'a>(field: &'a Field, name: &str) -> Option<&'a Expr> {
    field.attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(kv) => kv.path.is_ident(name).then_some(&kv.value),
        _ => None,
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

