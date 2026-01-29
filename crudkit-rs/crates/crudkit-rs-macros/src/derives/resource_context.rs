//! Implementation of the `CkResourceContext` derive macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn expand_derive_resource_context(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident;

    Ok(quote! {
        impl crudkit_rs::resource::CrudResourceContext for #ident {}
    })
}
