use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn expand_derive_action_payload(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    Ok(quote! {
        impl crudkit_web::action::CrudActionPayload for #ident {
        }

        impl crudkit_web::action::ActionPayload for #ident {
        }
    })
}
