use crate::utils::{data_ty, get_attr, get_attr_or_expr, get_numeric_ty};
use proc_macro2::{Span, TokenStream};
use quote2::*;
use syn::{punctuated::Punctuated, spanned::Spanned, *};

pub fn expand(
    crate_path: &TokenStream,
    input: &DeriveInput,
    t: &mut TokenStream,
    key_attr: &str,
    default_attr: &str,
) {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let body = quote(|t| match data {
        Data::Union(_) => unimplemented!(),
        Data::Struct(DataStruct { fields, .. }) => {
            let struct_fields: Vec<_> = fields
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    let (alias, name) = match &field.ident {
                        Some(name) => (name.clone(), Member::Named(name.clone())),
                        None => (
                            Ident::new(&format!("_{}", idx), field.span()),
                            Member::Unnamed(Index {
                                index: idx as u32,
                                span: field.span(),
                            }),
                        ),
                    };
                    (
                        alias,
                        get_attr(&field.attrs, key_attr).map(|key| {
                            let name_str = match &field.ident {
                                Some(name) => name.to_string(),
                                None => idx.to_string(),
                            };
                            (key, name_str)
                        }),
                        get_attr_or_expr(&field.attrs, default_attr),
                        name,
                    )
                })
                .collect();

            let field_decoder = quote(|t| {
                for (alias, key, _, _) in &struct_fields {
                    let Some((key, name_str)) = key else {
                        continue;
                    };
                    quote!(t, {
                        #key => #alias = __obj__.decode(__ty__, #name_str)?,
                    });
                }
            });

            let field_bind = quote(|t| {
                for (alias, key, default, member) in &struct_fields {
                    match default {
                        Some(default) if default.is_empty() => {
                            quote!(t, { #member: #alias.unwrap_or_else(::std::default::Default::default), });
                        }
                        Some(default) => {
                            quote!(t, { #member: #alias.unwrap_or_else(|| #default), });
                        }
                        None => match key {
                            Some((_, name_str)) => {
                                quote!(t, { #member: __crate::decoder::Optional::convert(#alias, #name_str)?, });
                            }
                            None => {
                                quote!(t, { #member: ::std::default::Default::default(), });
                            }
                        },
                    }
                }
            });

            for (alias, key, _, _) in &struct_fields {
                if key.is_none() {
                    continue;
                }
                quote!(t, {
                    let mut #alias = ::std::option::Option::None;
                });
            }

            quote!(t, {
                let mut __obj__ = __crate::decoder::FieldInfoDecoder::new(__r__);

                while let Some((__key__, __ty__)) = __obj__.next_field_id_and_ty()? {
                    match __key__ {
                        #field_decoder
                        _ => __obj__.skip_field(__key__, __ty__)?
                    }
                }

                Ok(Self { #field_bind })
            });
        }
        Data::Enum(DataEnum { variants, .. }) if let Some(ty) = get_numeric_ty(&input.attrs) => {
            let map_variants = quote(|t| {
                for v in variants {
                    let name = &v.ident;
                    let (_, key) = v.discriminant.as_ref().unwrap();
                    if let Fields::Unit = v.fields {
                        quote!(t, { #key => Self::#name, });
                    }
                }
            });

            let fallback = fallback(default_attr, variants, |t| {
                quote!(t, {
                    return Err(__crate::errors::__unknown_enum_tag(tag, <Self as __crate::Decode>::TY)),
                });
            });

            quote!(t, {
                let tag = <#ty as __crate::Decode>::decode(__r__)?;
                Ok(match tag {
                    #map_variants
                    _ => #fallback
                })
            });
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let map_variants = quote(|t| {
                for v in variants {
                    let name = &v.ident;
                    let (_, key) = v.discriminant.as_ref().unwrap();
                    match &v.fields {
                        Fields::Named(_) => unimplemented!(),
                        Fields::Unit => {
                            quote!(t, {
                                #key => { __obj__.skip_field_value(__ty__)?; Self::#name }
                            });
                        }
                        Fields::Unnamed(_) => {
                            let name_str = format!("{ident}::{name}");
                            quote!(t, {
                                #key => Self::#name(__obj__.decode_field(__ty__, #name_str)?),
                            });
                        }
                    }
                }
            });

            let fallback = fallback(default_attr, variants, |t| {
                quote!(t, {
                    return Err(__crate::errors::__unknown_field(__id__, __ty__)),
                });
            });

            quote!(t, {
                let (__id__, __ty__) = __crate::decoder::decode_field_id_and_ty(__r__)?;
                let mut __obj__ = __crate::decoder::FieldInfoDecoder::new(__r__);

                Ok(match __id__ {
                    #map_variants
                    _ => #fallback
                })
            });
        }
    });

    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let mut params = generics.params.clone();
    let lifetime = add_decoder_trait_bounds(&mut params);

    let ty = data_ty(input);
    quote!(t, {
        const _: () = {
            use #crate_path as __crate;
            impl <#lifetime, #params> __crate::Decode<'decode> for #ident #ty_generics #where_clause {
                const TY: __crate::DataType = #ty;
                fn decode(__r__: &mut &#lifetime [u8]) -> __crate::Result<Self> {
                    #body
                }
            }
        };
    });
}

fn fallback(
    default_attr: &str,
    variants: &Punctuated<Variant, token::Comma>,
    err: impl Fn(&mut TokenStream) + 'static,
) -> QuoteFn<impl Fn(&mut TokenStream)> {
    quote(move |t| {
        let has_default = variants
            .iter()
            .any(|v| get_attr_or_expr(&v.attrs, default_attr).is_some());

        if has_default {
            quote!(t, { <Self as ::std::default::Default>::default() });
        } else {
            err(t);
        }
    })
}

fn add_decoder_trait_bounds(params: &mut Punctuated<GenericParam, Token![,]>) -> LifetimeParam {
    let bound: TypeParamBound = parse_quote!(__crate::Decode<'decode>);
    let mut lifetime = LifetimeParam::new(Lifetime::new("'decode", Span::call_site()));

    for param in params {
        match param {
            GenericParam::Type(ty) => ty.bounds.push(bound.clone()),
            GenericParam::Lifetime(lt) => lifetime.bounds.push(lt.lifetime.clone()),
            _ => {}
        }
    }

    lifetime
}
