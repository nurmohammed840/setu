use proc_macro2::{Punct, Spacing, TokenStream};
use quote2::{Quote, ToTokens, quote};
use std::collections::HashSet;
use syn::{spanned::Spanned, *};

use crate::utils::data_ty;
use crate::{errors, utils};
use errors::to_compile_error;

pub fn expand(crate_path: &TokenStream, input: &DeriveInput, t: &mut TokenStream, key_attr: &str) {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let body = quote(|t| {
        match data {
            Data::Struct(DataStruct { fields, .. }) => {
                if let Fields::Unit = fields {
                    quote!(t, {
                        ::std::io::Write::write_all(w, &[__crate::DataType::StructEnd.code()])
                    });
                    return;
                }

                let mut seen: HashSet<&Expr> = HashSet::new();

                for (idx, field) in fields.iter().enumerate() {
                    if let Some(key) = utils::get_attr(&field.attrs, key_attr) {
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

                        let ident = quote(|t| match &field.ident {
                            Some(key) => {
                                quote!(t, { #key });
                            }
                            None => {
                                let idx = Index {
                                    index: idx as u32,
                                    span: field.span(),
                                };
                                quote!(t, { #idx });
                            }
                        });
                        let ref_symbol = match field.ty {
                            Type::Reference(_) => None,
                            _ => Some(Punct::new('&', Spacing::Alone)),
                        };

                        quote!(t, {
                            __crate::encoder::FieldEncoder::encode(#ref_symbol self.#ident, w, #key)?;
                        });
                    }
                }

                quote!(t, {
                    ::std::io::Write::write_all(w, &[__crate::DataType::StructEnd.code()])
                });
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

                        #[allow(unused_mut)] // Incorrect warning: closure mutates `t`
                        let mut get_discriminant = || -> Option<&Expr> {
                            match discriminant {
                                Some((_, expr)) => Some(expr),
                                None => {
                                    let err_span = variant.span();
                                    let err = to_compile_error(
                                        err_span,
                                        format!("missing key at line: {:?}", err_span.start().line),
                                    );
                                    quote!(t, {
                                       _ => { #err ::std::todo!() }
                                    });
                                    None
                                }
                            }
                        };

                        match fields {
                            Fields::Named(fields) => {
                                let err = errors::invalid_enum_named_field(ident, fields);
                                quote!(t, {
                                    Self::#ident { .. } => { #err ::std::todo!() }
                                });
                            }
                            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                                if let Some(key) = get_discriminant() {
                                    match unnamed.len() {
                                        0 => {
                                            quote!(t, { Self::#ident() => __crate::encoder::EnumEncoder::encode(&false, w, #key), });
                                        }
                                        1 => {
                                            quote!(t, { Self::#ident(val) => __crate::encoder::EnumEncoder::encode(val, w, #key), });
                                        }
                                        count => {
                                            let err = errors::exrta_fields(count, unnamed);
                                            quote!(t, {
                                                Self::#ident(..) => { #err ::std::todo!() }
                                            });
                                        }
                                    }
                                }
                            }
                            Fields::Unit => {
                                if let Some(key) = get_discriminant() {
                                    quote!(t, { Self::#ident => __crate::encoder::EnumEncoder::encode(&false, w, #key), });
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
        }
    });

    let generics = add_encode_trait_bounds(generics.clone());
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ty = data_ty(data);

    quote!(t, {
        const _: () = {
            use #crate_path as __crate;
            impl #impl_generics __crate::Encode for #ident #ty_generics #where_clause {
                const TY: __crate::DataType = __crate::#ty;
                fn encode(&self, w: &mut (impl ::std::io::Write + ?::std::marker::Sized)) -> ::std::io::Result<()> {
                    #body
                }
            }
        };
    });
}

// Add a bound `T: __crate::Encode` to every type parameter T.
fn add_encode_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(__crate::Encode));
        }
    }
    generics
}
