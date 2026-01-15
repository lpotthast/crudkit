#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(CkResourceContext, attributes(ck_resource_context))]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = ast.ident;

    quote! {
        impl crudkit_rs::resource::CrudResourceContext for #ident {}
    }
    .into()
}
