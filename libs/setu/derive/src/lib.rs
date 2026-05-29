mod parse;
mod utils;

use proc_macro2::{Span, TokenStream};
use quote2::{Quote, quote};

pub use parse::*;
use syn::Ident;

pub fn expend_export(crate_path: &TokenStream, list: &RpcList, t: &mut TokenStream) {
    let rpcs = quote(|t| {
        for Rpc { name, index, .. } in &list.rpcs {
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

pub fn expend_type_definition(crate_path: &TokenStream, list: &RpcList, t: &mut TokenStream) {
    let body = quote(|t| {
        for Rpc { name, index, args, .. } in &list.rpcs {
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
                    ::std::vec![ #body ]
                }
            }
        };
    });
}

fn interface_name(list: &RpcList) -> Ident {
    match list.name {
        Some(ref name) => name.name.clone(),
        None => Ident::new("App", Span::call_site()),
    }
}
