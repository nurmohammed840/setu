use std::mem;

pub use quote2;
pub use quote2::proc_macro2;
pub use syn;

use proc_macro2::TokenStream;
use quote2::{Quote, QuoteFn, quote};
use syn::*;

pub fn expand(crate_path: TokenStream, input: &DeriveInput, output: &mut TokenStream) {
    let DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    } = input;

    let fields = quote(|t| match data {
        Data::Struct(DataStruct { fields, .. }) => {
            if write_fields(t, fields).is_none() {
                panic!("`{ident}` struct needs at most one field")
            }
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let enum_repr = enum_repr(attrs);

            let fields = quote(|t| {
                for Variant {
                    attrs,
                    ident,
                    fields,
                    #[allow(warnings)]
                    discriminant,
                } in variants
                {
                    let attrs = get_attrs(attrs);
                    let field_name = ident.to_string();
                    let discriminant = get_discriminant(discriminant, enum_repr);

                    let field_ty = quote(|t| {
                        if write_fields(t, fields).is_none() {
                            quote!(t, { Unit });
                        }
                    });

                    quote!(t, {
                        (
                            #attrs,
                            __crate::EnumField {
                                name: __crate::Ident::from(#field_name),
                                ty: __crate::EnumFieldType::#field_ty,
                                discriminant: #discriminant
                            }
                        ),
                    });
                }
            });
            quote!(t, { as_enum(::std::vec![#fields]) });
        }
        Data::Union(_) => unimplemented!(),
    });

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let fmt_args = format!("{{}}::{ident}");
    let attrs = get_attrs(attrs);

    quote!(output, {
        const _: () = {
            use #crate_path as __crate;
            impl #impl_generics __crate::TypeId for #ident #ty_generics #where_clause {
                fn ty(__r: &mut __crate::TypeRegistry) -> __crate::Type {
                    __r.register(
                        ::std::format!(#fmt_args, ::std::module_path!()),
                        |__r, name| __crate::ComplexData {
                            name,
                            attrs: #attrs,
                            ty: __crate::ComplexDataType::#fields,
                        },
                    )
                }
            }
        };
    });
}

fn enum_repr(attrs: &[Attribute]) -> Option<&TokenStream> {
    for attr in attrs {
        if let Meta::List(MetaList { path, tokens, .. }) = &attr.meta
            && path.is_ident("repr")
        {
            return Some(tokens);
        }
    }
    None
}

fn write_fields(t: &mut TokenStream, fields: &Fields) -> Option<()> {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            let fields = quote(|t| {
                for Field {
                    attrs, ident, ty, ..
                } in named
                {
                    let attrs = get_attrs(attrs);
                    let field_name = ident.as_ref().map(Ident::to_string);
                    quote!(t, {
                        (
                            #attrs,
                            __crate::StructField {
                                name: __crate::Ident::from(#field_name),
                                ty: <#ty as __crate::TypeId>::ty(__r)
                            }
                        ),
                    });
                }
            });
            quote!(t, { as_struct(::std::vec![#fields]) });
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let fields = quote(|t| {
                for Field { attrs, ty, .. } in unnamed {
                    let attrs = get_attrs(attrs);
                    quote!(t, {
                        (#attrs, <#ty as __crate::TypeId>::ty(__r)),
                    });
                }
            });
            quote!(t, { as_tuple(::std::vec![#fields]) });
        }
        Fields::Unit => return None,
    }
    Some(())
}

fn get_discriminant(
    discriminant: &Option<(Token![=], Expr)>,
    enum_repr: Option<&TokenStream>,
) -> QuoteFn<impl Fn(&mut TokenStream)> {
    quote(move |t| match (discriminant, enum_repr) {
        (Some((_, expr)), Some(ty)) => {
            quote!(t, { __crate::Discriminant::from(#expr as #ty) });
        },
        (Some((_, expr)), None) => {
            quote!(t, { __crate::Discriminant::from(#expr) });
        }
        _ => {
            quote!(t, { __crate::Discriminant::None });
        }
    })
}

fn get_attrs(attrs: &[Attribute]) -> QuoteFn<impl Fn(&mut TokenStream)> {
    quote(move |t| {
        let mut is_first = true;
        let mut string = String::new();

        for attr in attrs {
            if let Meta::NameValue(MetaNameValue { path, value, .. }) = &attr.meta
                && path.is_ident("doc")
                && let Expr::Lit(expr) = value
                && let Lit::Str(data) = &expr.lit
            {
                if !mem::take(&mut is_first) {
                    string.push('\n');
                }
                string += &data.value();
            }
        }

        if string.is_empty() {
            quote!(t, { __crate::Attributes::default() });
        } else {
            quote!(t, { __crate::Attributes::docs(#string) });
        }
    })
}
