use proc_macro::TokenStream;
use quote::quote;
use std::{fs::read_to_string, path::Path};
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn try_include_str(path: TokenStream) -> TokenStream {
    let path = parse_macro_input!(path as LitStr).value();
    if Path::new(&path).exists() {
        let contents = read_to_string(path).unwrap();
        quote! {
            #contents
        }
    } else {
        quote! {
            ""
        }
    }
    .into()
}
