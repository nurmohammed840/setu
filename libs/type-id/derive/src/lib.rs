pub use quote2;
pub use quote2::proc_macro2;
pub use syn;

use proc_macro2::TokenStream;
use syn::DeriveInput;

pub fn expand(crate_path: TokenStream, input: &DeriveInput, output: &mut TokenStream) {
    let DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    } = input;
}
