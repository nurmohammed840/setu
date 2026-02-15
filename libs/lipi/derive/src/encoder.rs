use proc_macro2::{Punct, Spacing, TokenStream};
use quote2::{Quote, ToTokens, quote};
use std::collections::HashSet;
use syn::{spanned::Spanned, *};

pub fn expand(input: &DeriveInput, crate_path: TokenStream, key_attr: &str) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let body = quote(|t| match data {
        Data::Struct(DataStruct { fields, .. }) => {
            let mut seen: HashSet<&Expr> = HashSet::new();

            let field_count = fields
                .iter()
                .filter_map(|field| crate::utils::get_attr(field, key_attr))
                .count();

            quote!(t, {
                #crate_path::__private::encode_length(w, #field_count)?;
            });

            for field in fields {
                if let Some(key) = crate::utils::get_attr(field, key_attr) {
                    match seen.get(key) {
                        Some(key_0) => {
                            let loc = key.span().start();
                            let mut err = Error::new(
                                key_0.span(),
                                format!("duplicate key at line {}", loc.line),
                            );
                            err.combine(Error::new(
                                key.span(),
                                format!(
                                    "duplicate key `{}` later defined here",
                                    key_0.to_token_stream()
                                ),
                            ));
                            let err = err.to_compile_error();
                            quote!(t, { #err });
                        }
                        None => {
                            seen.insert(key);
                        }
                    }

                    let ident = &field.ident;
                    let ref_symbol = match field.ty {
                        Type::Reference(_) => None,
                        _ => Some(Punct::new('&', Spacing::Alone)),
                    };

                    quote!(t, {
                        #crate_path::__private::FieldEncoder::encode(#ref_symbol self.#ident, w, #key)?;
                    });
                }
            }
            quote!(t, { ::std::io::Result::Ok(()) });
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let encode_field = quote(|t| {
                for variant in variants {
                    let Variant {
                        ident,
                        fields,
                        discriminant,
                        ..
                    } = variant;

                    let mut get_discriminant = || -> Option<&Expr> {
                        match discriminant {
                            Some((_, expr)) => Some(expr),
                            None => {
                                let loc = variant.span().start();
                                let err = Error::new(
                                    variant.span(),
                                    format!("missing key at line: {:?}", loc.line),
                                );
                                let err = err.to_compile_error();
                                quote!(t, {
                                   _ => { #err ::std::todo!() }
                                });
                                None
                            }
                        }
                    };

                    match fields {
                        Fields::Named(fields) => {
                            let err = Error::new(fields.span(), format!("unsupported {{ .. }}"));
                            let err = err.to_compile_error();
                            quote!(t, {
                                Self::#ident { .. } => { #err ::std::todo!() }
                            });
                        }
                        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                            if let Some(key) = get_discriminant() {
                                let mut iter = unnamed.iter();
                                match iter.next() {
                                    None => {
                                        quote!(t, { Self::#ident() => ::lipi::__private::EnumEncoder::encode(&false, w, #key), });
                                    }
                                    Some(_) => {
                                        quote!(t, { Self::#ident(val) => ::lipi::__private::EnumEncoder::encode(val, w, #key), });
                                    }
                                }
                            }
                        }
                        Fields::Unit => {
                            if let Some(key) = get_discriminant() {
                                quote!(t, { Self::#ident => ::lipi::__private::EnumEncoder::encode(&false, w, #key), });
                            }
                        }
                    }
                }
            });

            quote!(t, {
                match self {
                    #encode_field
                }
            });
        }
        Data::Union(_) => unimplemented!(),
    });

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut t = TokenStream::new();
    quote!(t, {
        impl #impl_generics #crate_path::Encode for #ident #ty_generics #where_clause {
            const TY: u8 = 9;
            fn encode(&self, w: &mut dyn ::std::io::Write) -> ::std::io::Result<()> {
                #body

            }
        }
    });
    t
}
