use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenTree};
use proc_macro_error::{abort, emit_error, proc_macro_error};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Field};

#[proc_macro_derive(CrudResourceContext, attributes(crud_resource_context))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = ast.ident;

    quote! {
        impl crud_rs::context::CrudResourceContext for #ident {}
    }
    .into()
}
