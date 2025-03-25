#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use darling::*;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

// TODO: Automatically derive Eq on new type if source type is already able to derive it!

#[derive(Debug, FromField)]
#[darling(attributes(ck_update_model), forward_attrs(schema, serde))]
struct MyFieldReceiver {
    attrs: Vec<syn::Attribute>,

    ident: Option<syn::Ident>,

    ty: syn::Type,

    vis: syn::Visibility,

    /// Excluded fields are not part of the derived `UpdateModel`.
    exclude: Option<bool>,

    /// Optional fields have their `UpdateModel` type wrapped in `Option`.
    /// On update, the field is only `ActiveValue::Set` if we received a `Option::Some` variant containing the data.
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
#[darling(attributes(ck_update_model), supports(struct_any))]
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

#[proc_macro_derive(CkUpdateModel, attributes(ck_update_model))]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let input: MyInputReceiver = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return darling::Error::write_errors(err).into(),
    };

    let update_model_fields = input
        .fields()
        .iter()
        .filter(|field| !field.is_excluded())
        .map(|field| {
            let vis = &field.vis;
            let ident = &field.ident;
            let ty = &field.ty;
            let attrs = &field.attrs;
            if field.is_optional() {
                quote! {
                    #(#attrs)*
                    #vis #ident: Option<#ty>
                }
            } else {
                quote! {
                    #(#attrs)*
                    #vis #ident: #ty
                }
            }
        });

    let update_active_model_stmts = input.fields().iter().map(|field| {
        let ident = field.ident.as_ref().expect("Expected a named field.");
        if field.is_excluded() {
            if field.use_default() {
                quote! {
                    self.#ident = sea_orm::ActiveValue::Set(Default::default());
                }
            } else {
                quote! {
                    // Intentionally left blank. We will not set the field at all, keeping the value that is already stored.
                }
            }
        } else if field.is_optional() {
            quote! {
                match update.#ident {
                    Some(value) => self.#ident = sea_orm::ActiveValue::Set(value),
                    None => {}
                };
            }
        } else {
            quote! {
                self.#ident = sea_orm::ActiveValue::Set(update.#ident);
            }
        }
    });

    quote! {
        #[derive(Debug, Clone, PartialEq, utoipa::ToSchema, Deserialize)]
        pub struct UpdateModel {
            #(#update_model_fields),*
        }

        impl crudkit_rs::UpdateModelTrait for UpdateModel {}

        impl crudkit_rs::UpdateActiveModelTrait<UpdateModel> for ActiveModel {
            fn update_with(&mut self, update: UpdateModel) {
                #(#update_active_model_stmts)*
            }
        }
    }
    .into()
}
