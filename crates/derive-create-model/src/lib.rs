#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use darling::*;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

// TODO: Automatically derive Eq on new type if source type is already able to derive it!

#[derive(Debug, FromField)]
#[darling(attributes(ck_create_model))]
struct MyFieldReceiver {
    ident: Option<syn::Ident>,

    ty: syn::Type,

    vis: syn::Visibility,

    /// Excluded fields are not part of the derived `CreateModel`.
    exclude: Option<bool>,

    /// Optional fields have their `CreateModel` type wrapped in `Option`.
    /// On create, the field is only `ActiveValue::Set` if we received a `Option::Some` variant containing the data.
    /// We do not unset data just because we didn't receive an optional field.
    optional: Option<bool>,

    use_default: Option<bool>,
}

impl MyFieldReceiver {
    pub fn is_excluded(&self) -> bool {
        self.exclude.unwrap_or(false)
    }

    pub fn is_optional(&self) -> bool {
        self.optional.unwrap_or(false)
    }

    pub fn use_default(&self) -> bool {
        self.use_default.unwrap_or(false)
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ck_create_model), supports(struct_any))]
struct MyInputReceiver {
    data: ast::Data<(), MyFieldReceiver>,
}

impl MyInputReceiver {
    pub fn fields(&self) -> &ast::Fields<MyFieldReceiver> {
        match &self.data {
            ast::Data::Enum(_) => panic!("Only structs are supported"),
            ast::Data::Struct(fields) => fields,
        }
    }
}

#[proc_macro_derive(CkCreateModel, attributes(ck_create_model))]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let input: MyInputReceiver = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return darling::Error::write_errors(err).into(),
    };

    let create_model_fields = input
        .fields()
        .iter()
        .filter(|field| !field.is_excluded())
        .map(|field| {
            let vis = &field.vis;
            let ident = &field.ident;
            let ty = &field.ty;
            if field.is_optional() {
                quote! { #vis #ident: Option<#ty> }
            } else {
                quote! { #vis #ident: #ty }
            }
        });

    let into_active_model_arms = input.fields().iter().map(|field| {
        let ident = field.ident.as_ref().expect("Expected a named field.");
        if field.is_excluded() {
            if field.use_default() {
                quote! {
                    #ident: sea_orm::ActiveValue::Set(Default::default())
                }
            } else {
                quote! {
                    #ident: sea_orm::ActiveValue::NotSet
                }
            }
        } else if field.is_optional() {
            quote! {
                #ident: match self.#ident {
                    Some(value) => sea_orm::ActiveValue::Set(value),
                    None => sea_orm::ActiveValue::NotSet,
                }
            }
        } else {
            quote! {
                #ident: sea_orm::ActiveValue::Set(self.#ident)
            }
        }
    });

    quote! {
        #[derive(Debug, Clone, PartialEq, utoipa::ToSchema, serde::Deserialize)]
        pub struct CreateModel {
            #(#create_model_fields),*
        }

        #[async_trait::async_trait]
        impl crudkit_rs::CreateModelTrait<ActiveModel> for CreateModel {
            async fn into_active_model(self) -> ActiveModel {
                ActiveModel {
                    #(#into_active_model_arms),*
                }
            }
        }
    }
    .into()
}
