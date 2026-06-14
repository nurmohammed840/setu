mod attrs;
mod expend;

use proc_macro::TokenStream;
use quote2::proc_macro2;

#[proc_macro_derive(Rand, attributes(sample))]
pub fn rand(input: TokenStream) -> TokenStream {
    let Ok(input) = syn::parse(input) else {
        return TokenStream::new();
    };

    let mut output = proc_macro2::TokenStream::new();
    expend::expand(&input, &mut output, "sample");
    output.into()
}
