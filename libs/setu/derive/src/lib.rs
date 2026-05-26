mod parse;
mod utils;

use proc_macro2::{Span, TokenStream};
use quote2::{Quote, quote};

pub use parse::*;
use syn::Ident;

pub fn expend_export(crate_path: &TokenStream, list: &RpcList, t: &mut TokenStream) {
    let rpcs = quote(|t| {
        for rpc in &list.rpcs {
            let Rpc { name, index, .. } = rpc;
            quote!(t, {
                #index => #crate_path::Output::process(#name, req, res),
            });
        }
    });

    let name = match list.name {
        Some(ref name) => name.name.clone(),
        None => Ident::new("App", Span::call_site()),
    };

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
