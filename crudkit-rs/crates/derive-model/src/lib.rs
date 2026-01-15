#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use darling::*;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

// TODO: Automatically derive Eq on new type if source type is already able to derive it!

// ============== Shared Field Config ==============

/// Field configuration shared between CkCreateModel and CkUpdateModel.
/// Each macro uses its own attribute namespace but the config is identical.
#[derive(Debug, Clone, FromField)]
#[darling(forward_attrs(schema, serde))]
struct ModelFieldConfig {
    attrs: Vec<syn::Attribute>,
    ident: Option<syn::Ident>,
    ty: syn::Type,
    vis: syn::Visibility,

    /// Excluded fields are not part of the derived model.
    exclude: Option<bool>,

    /// Optional fields have their model type wrapped in `Option`.
    /// On create/update, the field is only `ActiveValue::Set` if we received a `Option::Some` variant containing the data.
    /// We do not unset data just because we didn't receive an optional field.
    optional: Option<bool>,

    use_default: Option<bool>,
}

impl ModelFieldConfig {
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

// ============== Shared Generation Logic ==============

/// Generates model struct fields (used for both CreateModel and UpdateModel).
fn generate_model_fields<'a>(
    fields: impl Iterator<Item = &'a ModelFieldConfig> + 'a,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    fields.filter(|field| !field.is_excluded()).map(|field| {
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
    })
}

/// Generates the into_active_model arms for CreateModel.
fn generate_into_active_model_arms<'a>(
    fields: impl Iterator<Item = &'a ModelFieldConfig> + 'a,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    fields.map(|field| {
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
    })
}

/// Generates the update_with statements for UpdateModel.
fn generate_update_with_stmts<'a>(
    fields: impl Iterator<Item = &'a ModelFieldConfig> + 'a,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    fields.map(|field| {
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
    })
}

// ============== CkCreateModel ==============

/// Wrapper for ModelFieldConfig with ck_create_model attribute namespace.
#[derive(Debug, FromField)]
#[darling(attributes(ck_create_model), forward_attrs(schema, serde))]
struct CreateModelFieldConfig {
    attrs: Vec<syn::Attribute>,
    ident: Option<syn::Ident>,
    ty: syn::Type,
    vis: syn::Visibility,
    exclude: Option<bool>,
    optional: Option<bool>,
    use_default: Option<bool>,
}

impl From<&CreateModelFieldConfig> for ModelFieldConfig {
    fn from(config: &CreateModelFieldConfig) -> Self {
        Self {
            attrs: config.attrs.clone(),
            ident: config.ident.clone(),
            ty: config.ty.clone(),
            vis: config.vis.clone(),
            exclude: config.exclude,
            optional: config.optional,
            use_default: config.use_default,
        }
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ck_create_model), supports(struct_any))]
struct CreateModelInput {
    data: ast::Data<(), CreateModelFieldConfig>,
}

impl CreateModelInput {
    pub fn fields(&self) -> &ast::Fields<CreateModelFieldConfig> {
        match &self.data {
            ast::Data::Enum(_) => panic!("Only structs are supported"),
            ast::Data::Struct(fields) => fields,
        }
    }
}

#[proc_macro_derive(CkCreateModel, attributes(ck_create_model))]
pub fn ck_create_model(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let input: CreateModelInput = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return darling::Error::write_errors(err).into(),
    };

    // Convert to shared config
    let fields: Vec<ModelFieldConfig> = input.fields().iter().map(|f| f.into()).collect();

    let model_fields = generate_model_fields(fields.iter());
    let into_active_model_arms = generate_into_active_model_arms(fields.iter());

    quote! {
        #[derive(Debug, Clone, PartialEq, utoipa::ToSchema, serde::Deserialize)]
        pub struct CreateModel {
            #(#model_fields),*
        }

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

// ============== CkUpdateModel ==============

/// Wrapper for ModelFieldConfig with ck_update_model attribute namespace.
#[derive(Debug, FromField)]
#[darling(attributes(ck_update_model), forward_attrs(schema, serde))]
struct UpdateModelFieldConfig {
    attrs: Vec<syn::Attribute>,
    ident: Option<syn::Ident>,
    ty: syn::Type,
    vis: syn::Visibility,
    exclude: Option<bool>,
    optional: Option<bool>,
    use_default: Option<bool>,
}

impl From<&UpdateModelFieldConfig> for ModelFieldConfig {
    fn from(config: &UpdateModelFieldConfig) -> Self {
        Self {
            attrs: config.attrs.clone(),
            ident: config.ident.clone(),
            ty: config.ty.clone(),
            vis: config.vis.clone(),
            exclude: config.exclude,
            optional: config.optional,
            use_default: config.use_default,
        }
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ck_update_model), supports(struct_any))]
struct UpdateModelInput {
    data: ast::Data<(), UpdateModelFieldConfig>,
}

impl UpdateModelInput {
    pub fn fields(&self) -> &ast::Fields<UpdateModelFieldConfig> {
        match &self.data {
            ast::Data::Enum(_) => panic!("Only structs are supported"),
            ast::Data::Struct(fields) => fields,
        }
    }
}

#[proc_macro_derive(CkUpdateModel, attributes(ck_update_model))]
pub fn ck_update_model(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let input: UpdateModelInput = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return darling::Error::write_errors(err).into(),
    };

    // Convert to shared config
    let fields: Vec<ModelFieldConfig> = input.fields().iter().map(|f| f.into()).collect();

    let model_fields = generate_model_fields(fields.iter());
    let update_with_stmts = generate_update_with_stmts(fields.iter());

    quote! {
        #[derive(Debug, Clone, PartialEq, utoipa::ToSchema, serde::Deserialize)]
        pub struct UpdateModel {
            #(#model_fields),*
        }

        impl crudkit_rs::UpdateModelTrait for UpdateModel {}

        impl crudkit_rs::UpdateActiveModelTrait<UpdateModel> for ActiveModel {
            fn update_with(&mut self, update: UpdateModel) {
                #(#update_with_stmts)*
            }
        }
    }
    .into()
}
