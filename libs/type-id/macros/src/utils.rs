
macro_rules! crate_path {
    [$($tt:tt)*] => ({
        let mut path = proc_macro2::TokenStream::new();
        quote!(path, { $($tt)* });
        path
    });
}

pub(crate) use crate_path;