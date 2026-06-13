use proc_macro2::{Punct, Spacing, TokenStream};
use quote2::{Quote, ToTokens, quote};
use syn::{spanned::Spanned, *};

use crate::utils;
use crate::utils::data_ty;

pub fn expand(crate_path: &TokenStream, input: &DeriveInput, t: &mut TokenStream, key_attr: &str) {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let body = quote(|t| match data {
        Data::Union(_) => unimplemented!(),
        Data::Struct(DataStruct { fields, .. }) => {
            match fields {
                Fields::Named(FieldsNamed { named, .. }) => {
                    for Field {
                        attrs, ty, ident, ..
                    } in named
                    {
                        let Some(key) = utils::get_attr(attrs, key_attr) else {
                            continue;
                        };
                        encode_field(t, ty, ident, key);
                    }
                }
                Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                    for (idx, f) in unnamed.iter().enumerate() {
                        let Some(key) = utils::get_attr(&f.attrs, key_attr) else {
                            continue;
                        };
                        let idx = Index {
                            index: idx as u32,
                            span: f.span(),
                        };
                        encode_field(t, &f.ty, idx, key);
                    }
                }
                Fields::Unit => {}
            }
            quote!(t, {
                ::std::io::Write::write_all(w, &[__crate::DataType::StructEnd.code()])
            });
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let body = quote(|t| {
                for v in variants {
                    let name = &v.ident;
                    let (_, key) = v.discriminant.as_ref().unwrap();

                    match &v.fields {
                        Fields::Named(_) => unimplemented!(),
                        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                            assert_eq!(unnamed.len(), 1);
                            quote!(t, { Self::#name(val) => __crate::encoder::Field::encode(val, w, #key), });
                        }
                        Fields::Unit => {
                            quote!(t, { Self::#name => __crate::encoder::Field::encode(&false, w, #key), });
                        }
                    }
                }
            });
            quote!(t, {
                match self {
                    #body
                }
            });
        }
    });

    let ty = data_ty(data);

    let generics = add_encode_trait_bounds(generics.clone());
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
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

fn encode_field(t: &mut TokenStream, ty: &Type, ident: impl ToTokens, key: &Expr) {
    let ref_symbol = ref_symbol(ty);
    quote!(t, {
        __crate::encoder::OptionalField::encode(#ref_symbol self.#ident, w, #key)?;
    });
}

fn ref_symbol(ty: &Type) -> Option<Punct> {
    match ty {
        Type::Reference(_) => None,
        _ => Some(Punct::new('&', Spacing::Alone)),
    }
}

// Add a bound `T: __crate::Encode` to every type parameter T.
fn add_encode_trait_bounds(mut generics: Generics) -> Generics {
    let bound: TypeParamBound = parse_quote!(__crate::Encode);
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(bound.clone());
        }
    }
    generics
}
