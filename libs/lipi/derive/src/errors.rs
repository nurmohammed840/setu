use proc_macro2::TokenStream;
use quote2::ToTokens;
use syn::{punctuated::Punctuated, spanned::Spanned, token::Comma, *};

pub fn to_compile_error(span: proc_macro2::Span, message: impl std::fmt::Display) -> TokenStream {
    syn::Error::new(span, message).to_compile_error()
}

pub fn invalid_enum_named_field(ident: &Ident, fields: &FieldsNamed) -> TokenStream {
    let first_ty = match fields.named.first() {
        Some(field) => field.ty.to_token_stream().to_string(),
        None => "T".to_string(),
    };
    to_compile_error(
        fields.span(),
        format!("unsupported {{ .. }}; use `{ident}({first_ty})` instead"),
    )
}

pub fn exrta_fields(count: usize, unnamed: &Punctuated<Field, Comma>) -> TokenStream {
    let err_span = if count == 2 {
        unnamed.last().span()
    } else {
        let mut err_spans = unnamed.iter();
        err_spans.next(); // skip
        let start = err_spans.next().span();
        let end = err_spans.last().span();
        start.join(end).unwrap_or_else(|| unnamed.last().span())
    };

    to_compile_error(
        err_span,
        format!("remove extra fields; only one field is allowed (found {count})"),
    )
}
