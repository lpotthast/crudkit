#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use darling::*;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug, FromDeriveInput)]
struct MyInputReceiver {
    ident: Ident,
}

#[proc_macro_derive(CkDynamic)]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let input: MyInputReceiver = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return Error::write_errors(err).into(),
    };

    let ident = &input.ident;
    let field_ident = Ident::new(format!("{ident}Field").as_str(), ident.span());

    quote! {
        #[typetag::serde]
        impl crudkit_web::dynamic::Field for #field_ident {
            fn set_value(&self, model: &mut crudkit_web::dynamic::AnyModel, value: crudkit_web::value::Value) {
                let model = model.downcast_mut::<#ident>();
                crudkit_web::CrudFieldValueTrait::set_value(self, model, value);
            }
        }

        impl crudkit_web::dynamic::SerializeAsKey for #field_ident {
            fn serialize_as_key(&self) -> String {
                serde_json::to_string(self).unwrap()
            }
        }

        impl crudkit_web::dynamic::NamedProperty for #field_ident {
            fn get_name(&self) -> String {
                crudkit_web::CrudFieldNameTrait::get_name(self).to_string()
            }
        }

        #[typetag::serde]
        impl crudkit_web::dynamic::Model for #ident {}
    }
    .into()
}
