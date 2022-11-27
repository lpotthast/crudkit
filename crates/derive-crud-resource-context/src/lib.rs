use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CrudResourceContext, attributes(crud_resource_context))]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = ast.ident;

    quote! {
        impl crud_rs::context::CrudResourceContext for #ident {}
    }
    .into()
}
