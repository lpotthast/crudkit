#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CkActionPayload, attributes(ck_action_payload))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let ident = &ast.ident;

    quote! {
        impl ::crudkit_web::CrudActionPayload for #ident {
        }
    }
    .into()
}
