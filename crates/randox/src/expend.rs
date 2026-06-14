use crate::attrs::get_attr;
use proc_macro2::TokenStream;
use quote2::*;
use syn::{spanned::Spanned, *};

pub fn expand(input: &DeriveInput, t: &mut TokenStream, attr_key: &str) {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let body = quote(|t| match data {
        Data::Union(_) => unimplemented!(),
        Data::Struct(DataStruct { fields, .. }) => {
            map_fields(t, ident, attr_key, fields);
        }
        Data::Enum(DataEnum {
            variants,
            brace_token,
            ..
        }) => {
            let len = variants.len();
            let map_variants = quote(|t| {
                for (idx, v) in variants.iter().enumerate() {
                    let is_last = idx == len - 1;
                    let idx = if is_last {
                        Member::Named(Ident::new("_", v.span()))
                    } else {
                        Member::Unnamed(Index {
                            index: idx as u32,
                            span: v.span(),
                        })
                    };

                    let items = quote(|t| {
                        map_fields(t, &v.ident, attr_key, &v.fields);
                    });

                    quote!(t, { #idx => #ident::#items, });
                }
            });

            let len = Index {
                index: len as u32,
                span: brace_token.span.span(),
            };
            quote!(t, {
                match r.random_range(0..#len) {
                    #map_variants
                }
            });
        }
    });

    quote!(t, {
        const _: () = {
            use ::rand::prelude::*;
            impl #impl_generics Distribution<#ident #ty_generics> for ::rand::distr::StandardUniform
            #where_clause {
                fn sample<R: Rng + ?Sized>(&self, r: &mut R) -> #ident {
                    #body
                }
            }
        };
    });
}

fn map_fields(t: &mut TokenStream, ident: &Ident, attr_key: &str, fields: &Fields) {
    let fields = quote(|t| {
        for (idx, field) in fields.iter().enumerate() {
            let idx = match &field.ident {
                Some(name) => Member::Named(name.clone()),
                None => Member::Unnamed(Index {
                    index: idx as u32,
                    span: field.span(),
                }),
            };
            match get_attr(&field.attrs, attr_key) {
                Some(expr) => {
                    quote!(t, { #idx: #expr(r), });
                }
                None => {
                    quote!(t, { #idx: self.sample(r), });
                }
            }
        }
    });
    quote!(t, { #ident { #fields } });
}
