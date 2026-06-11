use proc_macro::TokenStream;
use quote2::proc_macro2;

mod expend;

#[proc_macro_derive(Randox)]
pub fn randox(input: TokenStream) -> TokenStream {
    let Ok(input) = syn::parse(input) else {
        return TokenStream::new();
    };

    let mut output = proc_macro2::TokenStream::new();
    expend::expand(&input, &mut output);
    output.into()
}
