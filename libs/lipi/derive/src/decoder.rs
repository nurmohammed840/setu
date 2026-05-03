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
            let fields = fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref();
                    (
                        name,
                        crate::utils::get_attr(field, key_attr)
                            .map(|key| (key, name.map(|s| s.to_string()).unwrap_or_default())),
                    )
                })
                .collect::<Vec<_>>();

            let field_decoder = quote(|t| {
                for (name, key) in &fields {
                    if let Some((key, name_str)) = key {
                        quote!(t, {
                            #key => #name = ___obj___.decode(___ty___, #name_str)?,
                        });
                    }
                }
            });

            let field_bind = quote(|t| {
                for (name, key) in &fields {
                    match key {
                        Some((_, name_str)) => {
                            quote!(t, {
                                #name: #crate_path::decoder::Optional::convert(#name, #name_str)?,
                            });
                        }
                        None => {
                            quote!(t, {
                                #name: ::std::default::Default::default(),
                            });
                        }
                    }
                }
            });

            for (name, key) in &fields {
                if let Some(_) = key {
                    quote!(t, {
                        let mut #name = ::std::option::Option::None;
                    });
                }
            }

            quote!(t, {
                let mut ___obj___ = #crate_path::decoder::FieldInfoDecoder::new(___r___)?;

                while let Some((___key___, ___ty___)) = ___obj___.next_field_id_and_ty()? {
                    match ___key___ {
                        #field_decoder
                        _ => ___obj___.skip_field(___key___, ___ty___)?
                    }
                }

                Ok(Self { #field_bind })
            });
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
        impl <#lifetime, #params> #crate_path::Decode<'decode> for #ident #ty_generics #where_clause {
            const TY: #crate_path::DataType = #crate_path::DataType::Struct;
            fn decode(___r___: &mut &#lifetime [u8]) -> #crate_path::Result<Self> {
                #body
            }
        }
    });
    t
}
