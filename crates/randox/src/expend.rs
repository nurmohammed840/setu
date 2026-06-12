use crate::attrs::get_attr;
use proc_macro2::TokenStream;
use quote2::*;
use syn::*;

pub fn expand(input: &DeriveInput, t: &mut TokenStream, attr_key: &str) {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let body = quote(|t| match data {
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
        Data::Struct(DataStruct { fields, .. }) => {
            for Field { ident, attrs, .. } in fields {
                match get_attr(attrs, attr_key) {
                    Some(expr) => {
                        quote!(t, { #ident: #expr(r), });
                    }
                    None => {
                        quote!(t, { #ident: self.sample(r), });
                    }
                }
            }
        }
    });

    quote!(t, {
        const _: () = {
            use ::rand::prelude::*;
            impl #impl_generics Distribution<#ident #ty_generics> for ::rand::distr::StandardUniform
            #where_clause {
                fn sample<R: Rng + ?Sized>(&self, r: &mut R) -> #ident {
                    #ident { #body }
                }
            }
        };
    });
}
