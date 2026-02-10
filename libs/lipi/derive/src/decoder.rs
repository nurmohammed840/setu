use proc_macro2::{Span, TokenStream};
use quote2::{Quote, quote};
use syn::*;

pub fn expand(input: &DeriveInput, crate_path: TokenStream, key_attr: &str) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let body = quote(|t| match data {
        Data::Struct(DataStruct { fields, .. }) => {
            for field in fields {
                match crate::utils::get_attr(field, key_attr) {
                    Some(key) => {
                        let key_name = &field.ident;
                        quote!(t, { #key_name: e.get_and_convert(#key)?, });
                    }
                    None => {
                        let key_name = &field.ident;
                        quote!(t, { #key_name: ::std::default::Default::default(), });
                    }
                }
            }
        }
        Data::Enum(_) => {}
        Data::Union(_) => todo!(),
    });

    let (_, ty_generics, where_clause) = generics.split_for_impl();

    // Add a bound `T: Decode<'de>` to every type parameter of `T`.
    let bound: TypeParamBound = parse_quote!(#crate_path::Decode<'decode>);
    let mut params = generics.params.clone();
    let mut lifetime = LifetimeParam::new(Lifetime::new("'decode", Span::call_site()));

    for param in params.iter_mut() {
        match param {
            GenericParam::Type(ty) => ty.bounds.push(bound.clone()),
            GenericParam::Lifetime(lt) => lifetime.bounds.push(lt.lifetime.clone()),
            _ => {}
        }
    }

    let mut t = TokenStream::new();
    quote!(t, {
        impl <#lifetime, #params> #crate_path::Decoder<'decode> for #ident #ty_generics #where_clause {
            fn decode(e: &#crate_path::Entries<'decode>) -> #crate_path::Result<Self> {
                Ok(Self { #body })
            }
        }
    });
    t
}
