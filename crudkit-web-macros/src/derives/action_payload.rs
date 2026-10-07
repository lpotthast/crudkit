use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn expand_derive_action_payload(input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;

    quote! {
        impl crudkit_web::action::ActionPayload for #ident {
        }

        impl crudkit_web::action::ErasedActionPayload for #ident {
        }
    }
}
