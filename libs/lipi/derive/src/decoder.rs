use proc_macro2::{Span, TokenStream};
use quote2::{Quote, quote};
use syn::{spanned::Spanned, *};

use crate::utils::data_ty;
use crate::{errors, utils};
use errors::to_compile_error;

pub fn expand(
    input: &DeriveInput,
    crate_path: TokenStream,
    key_attr: &str,
    default_attr: &str,
) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let body = quote(|t| match data {
        Data::Struct(DataStruct { fields, .. }) => {
            let fields = fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref();
                    (
                        name,
                        utils::get_attr(&field.attrs, key_attr)
                            .map(|key| (key, name.map(|s| s.to_string()).unwrap_or_default())),
                        utils::get_attr_or_expr(&field.attrs, default_attr),
                    )
                })
                .collect::<Vec<_>>();

            let field_decoder = quote(|t| {
                for (name, key, _) in &fields {
                    if let Some((key, name_str)) = key {
                        quote!(t, {
                            #key => #name = __obj__.decode(__ty__, #name_str)?,
                        });
                    }
                }
            });

            let field_bind = quote(|t| {
                for (name, key, default) in &fields {
                    match default {
                        Some(default) if default.is_empty() => {
                            quote!(t, { #name: #name.unwrap_or_else(::std::default::Default::default), });
                        }
                        Some(default) => {
                            quote!(t, { #name: #name.unwrap_or_else(|| #default), });
                        }
                        None => match key {
                            Some((_, name_str)) => {
                                quote!(t, { #name: #crate_path::decoder::Optional::convert(#name, #name_str)?, });
                            }
                            None => {
                                quote!(t, { #name: ::std::default::Default::default(), });
                            }
                        },
                    }
                }
            });

            for (name, key, _) in &fields {
                if let Some(_) = key {
                    quote!(t, {
                        let mut #name = ::std::option::Option::None;
                    });
                }
            }

            quote!(t, {
                let mut __obj__ = #crate_path::decoder::FieldInfoDecoder::new(__r__)?;

                while let Some((__key__, __ty__)) = __obj__.next_field_id_and_ty()? {
                    match __key__ {
                        #field_decoder
                        _ => __obj__.skip_field(__key__, __ty__)?
                    }
                }

                Ok(Self { #field_bind })
            });
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let decode_enum_field = quote(|t| {
                for v in variants {
                    match &v.discriminant {
                        Some((_, key)) => {
                            let name = &v.ident;
                            match &v.fields {
                                Fields::Unit => {
                                    quote!(t, {
                                        #key => { __obj__.skip_field_value(__ty__)?; Self::#name }
                                    });
                                }
                                Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                                    match unnamed.len() {
                                        0 => {
                                            quote!(t, {
                                                #key => { __obj__.skip_field_value(__ty__)?; Self::#name() }
                                            });
                                        }
                                        1 => {
                                            let name_str = format!("{ident}::{name}");
                                            quote!(t, {
                                                #key => Self::#name(__obj__.decode_field(__ty__, #name_str)?),
                                            });
                                        }
                                        count => {
                                            let err = errors::exrta_fields(count, unnamed);
                                            quote!(t, {
                                                _ => { #err ::std::todo!() }
                                            });
                                        }
                                    }
                                }
                                Fields::Named(fields) => {
                                    let err = errors::invalid_enum_named_field(name, fields);
                                    quote!(t, {
                                        _ => { #err ::std::todo!() }
                                    });
                                }
                            }
                        }
                        None => {
                            let err_span = v.span();
                            let err = to_compile_error(
                                err_span,
                                format!("missing key at line: {:?}", err_span.start().line),
                            );
                            quote!(t, {
                                _ => { #err ::std::todo!() }
                            });
                        }
                    }
                }
            });

            let has_default = variants
                .iter()
                .any(|v| utils::get_attr_or_expr(&v.attrs, default_attr).is_some());

            let fallback = quote(|t| {
                if has_default {
                    quote!(t, { <Self as ::std::default::Default>::default() });
                } else {
                    quote!(t, {
                        return Err(#crate_path::errors::__unknown_field(__id__, __ty__))
                    });
                }
            });

            quote!(t, {
                let (__id__, __ty__) = #crate_path::decoder::decode_field_id_and_ty(__r__)?;
                let mut __obj__ = #crate_path::decoder::FieldInfoDecoder::new(__r__)?;

                Ok(match __id__ {
                    #decode_enum_field
                    _ => #fallback
                })
            });
        }
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

    let ty = data_ty(data);

    let mut t = TokenStream::new();
    quote!(t, {
        impl <#lifetime, #params> #crate_path::Decode<'decode> for #ident #ty_generics #where_clause {
            const TY: #crate_path::DataType = #crate_path::#ty;
            fn decode(__r__: &mut &#lifetime [u8]) -> #crate_path::Result<Self> {
                #body
            }
        }
    });
    t
}
