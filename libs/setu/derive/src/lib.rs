mod parse;
mod utils;

use proc_macro2::{Span, TokenStream};
use quote2::{Quote, quote, quote_spanned};

pub use parse::*;
use syn::Ident;

use crate::utils::add_compile_error;

pub fn expend_export(crate_path: &TokenStream, list: &FnList, t: &mut TokenStream) {
    let rpcs = quote(|t| {
        for Rpc { name, index, .. } in &list.fns {
            quote!(t, {
                #index => #crate_path::Output::process(#name, req, res),
            });
        }
    });

    let name = interface_name(list);

    quote!(t, {
        #[derive(::std::clone::Clone)]
        pub struct #name;

        impl #crate_path::Application for #name {
            fn execute(
                id: u32,
                req: #crate_path::transport::http::HttpRequest,
                res: #crate_path::transport::http::HttpResponse,
            ) {
                match id {
                    #rpcs
                    id => #crate_path::__private::unknown_rpc(id, res)
                }
            }
        }
    });
}

pub fn expend_type_definition(crate_path: &TokenStream, list: &FnList, t: &mut TokenStream) {
    let maybe_errs = quote(|t| {
        for Rpc { args, .. } in &list.fns {
            let mut seen = Vec::with_capacity(args.len());
            for name in args {
                if seen.contains(&name) {
                    add_compile_error(t, name.span(), &format!("duplicate: `{name}`"));
                } else {
                    seen.push(name);
                }
            }
        }
    });

    let body = quote(|t| {
        for Rpc {
            name, index, args, ..
        } in &list.fns
        {
            let raw = name.to_string();
            let args = quote(|t| {
                for arg in args {
                    let arg = arg.to_string();
                    quote!(t, { #arg, });
                }
            });
            quote!(t, {
                Func::with_meta(r, "", &#name, #index, #raw, &[#args]),
            });
        }
    });

    let name = interface_name(list);
    quote!(t, {
        const _: () = {
            use #crate_path::__private::setu_type_info::{FnMetaData, Func, TypeDefinition};
            use #crate_path::__private::type_id::TypeRegistry;

            impl TypeDefinition for #name {
                fn type_definition(r: &mut TypeRegistry) -> ::std::vec::Vec<Func<FnMetaData>> {
                    #maybe_errs
                    ::std::vec![ #body ]
                }
            }
        };
    });
}

pub fn check_fn_args_count(crate_path: &TokenStream, list: &FnList, t: &mut TokenStream) {
    let body = quote(|t| {
        for rpc in &list.fns {
            let ident = &rpc.name;
            let args_len = rpc.args.len();

            let panic_msg = quote(|t| {
                let span = rpc.name.span();
                let args = format!("`{ident}` expected {args_len} arguments");
                quote_spanned!(span, t, { ::std::panic!(#args) });
            });

            quote!(t, {
                if __crate::fn_args_count(&#ident) != #args_len {
                    #panic_msg;
                }
            });
        }
    });
    quote!(t, {
        const _: () = {
            use #crate_path::__private::setu_type_info as __crate;
            #body
        };
    });
}

fn interface_name(list: &FnList) -> Ident {
    match list.name {
        Some(ref name) => name.name.clone(),
        None => Ident::new("App", Span::call_site()),
    }
}
