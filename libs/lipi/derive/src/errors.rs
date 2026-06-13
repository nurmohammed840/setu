use std::collections::HashSet;

use crate::utils::{self, add_compile_error};
use proc_macro2::TokenStream;
use quote2::ToTokens;
use std::format as fmt;
use syn::{spanned::Spanned, *};

type VerifyResult = std::result::Result<(), TokenStream>;

pub fn verify(input: &DeriveInput, key_attr: &str) -> VerifyResult {
    let DeriveInput { ident, data, .. } = input;
    let mut err = TokenStream::new();

    match data {
        Data::Union(_) => unimplemented!(),
        Data::Struct(DataStruct { fields, .. }) => {
            let mut seen = HashSet::new();

            for field in fields {
                let Some(key) = utils::get_attr(&field.attrs, key_attr) else {
                    continue;
                };

                let Some(key_0) = seen.get(key) else {
                    seen.insert(key);
                    continue;
                };

                let loc = key.span().start();

                add_compile_error(
                    &mut err,
                    key_0.span(),
                    &fmt!("duplicate key at line {}", loc.line),
                );

                add_compile_error(
                    &mut err,
                    key.span(),
                    &fmt!(
                        "duplicate key `{}` later defined here",
                        key_0.to_token_stream()
                    ),
                );
            }
        }
        Data::Enum(DataEnum { variants, .. }) => {
            for v in variants {
                if v.discriminant.is_none() {
                    let span = v.span();
                    add_compile_error(
                        &mut err,
                        span,
                        &fmt!("missing key at line: {:?}", span.start().line),
                    );
                }

                match &v.fields {
                    Fields::Named(fields) => {
                        let first_ty = match fields.named.first() {
                            Some(field) => field.ty.to_token_stream().to_string(),
                            None => "T".into(),
                        };

                        add_compile_error(
                            &mut err,
                            fields.span(),
                            &fmt!("unsupported {{ .. }}; use `{ident}({first_ty})` instead"),
                        );
                    }
                    Fields::Unnamed(FieldsUnnamed { unnamed, .. }) if unnamed.len() != 1 => {
                        let count = unnamed.len();
                        let span = if count == 2 {
                            unnamed.last().span()
                        } else {
                            let mut spans = unnamed.iter();
                            spans.next(); // skip
                            let start = spans.next().span();
                            let end = spans.next_back().span();
                            start.join(end).unwrap_or_else(|| unnamed.last().span())
                        };

                        add_compile_error(
                            &mut err,
                            span,
                            &fmt!("remove extra fields; only one field is allowed (found {count})"),
                        )
                    }
                    _ => {}
                }
            }
        }
    }

    if err.is_empty() { Ok(()) } else { Err(err) }
}
