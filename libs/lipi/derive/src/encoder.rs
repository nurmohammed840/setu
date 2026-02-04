use proc_macro2::{Punct, Spacing, TokenStream};
use quote2::{Quote, ToTokens, quote};
use std::collections::HashSet;
use syn::{spanned::Spanned, *};

pub fn expand(input: &DeriveInput) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let body = quote(|t| match data {
        Data::Struct(DataStruct { fields, .. }) => {
            let mut seen: HashSet<&Expr> = HashSet::new();

            for field in fields {
                if let Some(key) = crate::utils::get_key(field) {
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
                        ::lipi::FieldEncoder::encode(#ref_symbol self.#ident, w, #key)?;
                    });
                }
            }
        }
        Data::Enum(_) => todo!(),
        Data::Union(_) => todo!(),
    });

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut t = TokenStream::new();
    quote!(t, {
        impl #impl_generics ::lipi::Encoder for #ident #ty_generics #where_clause {
            fn encode(&self, w: &mut dyn ::std::io::Write) -> ::std::io::Result<()> {
                #body
                ::std::io::Write::write_all(w, &[10])
            }
        }

        impl #impl_generics ::lipi::FieldEncoder for #ident #ty_generics #where_clause {
            fn encode(&self, w: &mut dyn ::std::io::Write, id: u16) -> ::std::io::Result<()> {
                ::lipi::__private::field_encoder(self, w, id)
            }
        }
    });
    t
}
