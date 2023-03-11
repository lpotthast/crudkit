#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CrudResourceContext, attributes(crudkit_resource_context))]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = ast.ident;

    quote! {
        impl crudkit_rs::context::CrudResourceContext for #ident {}
    }
    .into()
}
