#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Shared TokenStream generation logic for Create/Update model derive macros.
//!
//! This library provides storage-agnostic code generation that can be used by
//! both `derive-model` (storage-agnostic) and storage-specific crates like
//! `crudkit-sea-orm-macros`.

use darling::*;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::DeriveInput;

// ============== Field Configuration ==============

/// Field configuration shared between CkCreateModel and CkUpdateModel.
/// Each macro uses its own attribute namespace but the config is identical.
#[derive(Debug, Clone)]
pub struct ModelFieldConfig {
    pub attrs: Vec<syn::Attribute>,
    pub ident: Option<Ident>,
    pub ty: syn::Type,
    pub vis: syn::Visibility,
    /// Excluded fields are not part of the derived model.
    pub exclude: bool,
    /// Optional fields have their model type wrapped in `Option`.
    pub optional: bool,
    /// Fields with use_default will use `Default::default()` for excluded fields.
    pub use_default: bool,
}

/// Metadata about a field for storage-specific implementations.
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub ident: Ident,
    pub ty: syn::Type,
    pub is_excluded: bool,
    pub is_optional: bool,
    pub use_default: bool,
}

impl From<&ModelFieldConfig> for FieldInfo {
    fn from(config: &ModelFieldConfig) -> Self {
        Self {
            ident: config
                .ident
                .clone()
                .expect("Expected a named field in FieldInfo conversion"),
            ty: config.ty.clone(),
            is_excluded: config.exclude,
            is_optional: config.optional,
            use_default: config.use_default,
        }
    }
}

// ============== Create Model Attribute Parsing ==============

/// Internal field config for create model with darling parsing.
#[derive(Debug, Clone, FromField)]
#[darling(attributes(ck_create_model), forward_attrs(schema, serde))]
struct CreateModelFieldConfigInternal {
    attrs: Vec<syn::Attribute>,
    ident: Option<Ident>,
    ty: syn::Type,
    vis: syn::Visibility,
    exclude: Option<bool>,
    optional: Option<bool>,
    use_default: Option<bool>,
}

impl From<CreateModelFieldConfigInternal> for ModelFieldConfig {
    fn from(config: CreateModelFieldConfigInternal) -> Self {
        Self {
            attrs: config.attrs,
            ident: config.ident,
            ty: config.ty,
            vis: config.vis,
            exclude: config.exclude.unwrap_or(false),
            optional: config.optional.unwrap_or(false),
            use_default: config.use_default.unwrap_or(false),
        }
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ck_create_model), supports(struct_any))]
struct CreateModelInputInternal {
    ident: Ident,
    data: ast::Data<(), CreateModelFieldConfigInternal>,
}

impl CreateModelInputInternal {
    fn fields(&self) -> &ast::Fields<CreateModelFieldConfigInternal> {
        match &self.data {
            ast::Data::Enum(_) => panic!("Only structs are supported"),
            ast::Data::Struct(fields) => fields,
        }
    }
}

// ============== Update Model Attribute Parsing ==============

/// Internal field config for update model with darling parsing.
#[derive(Debug, Clone, FromField)]
#[darling(attributes(ck_update_model), forward_attrs(schema, serde))]
struct UpdateModelFieldConfigInternal {
    attrs: Vec<syn::Attribute>,
    ident: Option<Ident>,
    ty: syn::Type,
    vis: syn::Visibility,
    exclude: Option<bool>,
    optional: Option<bool>,
    use_default: Option<bool>,
}

impl From<UpdateModelFieldConfigInternal> for ModelFieldConfig {
    fn from(config: UpdateModelFieldConfigInternal) -> Self {
        Self {
            attrs: config.attrs,
            ident: config.ident,
            ty: config.ty,
            vis: config.vis,
            exclude: config.exclude.unwrap_or(false),
            optional: config.optional.unwrap_or(false),
            use_default: config.use_default.unwrap_or(false),
        }
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ck_update_model), supports(struct_any))]
struct UpdateModelInputInternal {
    ident: Ident,
    data: ast::Data<(), UpdateModelFieldConfigInternal>,
}

impl UpdateModelInputInternal {
    fn fields(&self) -> &ast::Fields<UpdateModelFieldConfigInternal> {
        match &self.data {
            ast::Data::Enum(_) => panic!("Only structs are supported"),
            ast::Data::Struct(fields) => fields,
        }
    }
}

// ============== Output Types ==============

/// Output from create model generation.
pub struct CreateModelOutput {
    /// The generated struct definition with derives and trait implementations.
    pub struct_def: TokenStream,
    /// The storage-agnostic trait implementations.
    pub trait_impls: TokenStream,
    /// The name of the generated CreateModel struct.
    pub create_model_name: Ident,
    /// Field metadata for storage-specific implementations.
    pub fields: Vec<FieldInfo>,
}

/// Output from update model generation.
pub struct UpdateModelOutput {
    /// The generated struct definition with derives and trait implementations.
    pub struct_def: TokenStream,
    /// The storage-agnostic trait implementations.
    pub trait_impls: TokenStream,
    /// The name of the generated UpdateModel struct.
    pub update_model_name: Ident,
    /// Field metadata for storage-specific implementations.
    pub fields: Vec<FieldInfo>,
}

// ============== Shared Generation Logic ==============

/// Generates model struct fields (used for both CreateModel and UpdateModel).
fn generate_model_fields<'a>(
    fields: impl Iterator<Item = &'a ModelFieldConfig> + 'a,
) -> impl Iterator<Item = TokenStream> + 'a {
    fields.filter(|field| !field.exclude).map(|field| {
        let vis = &field.vis;
        let ident = &field.ident;
        let ty = &field.ty;
        let attrs = &field.attrs;
        if field.optional {
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

// ============== Public API ==============

/// Parse and generate CreateModel output from a DeriveInput.
///
/// Returns the struct definition, storage-agnostic trait implementations,
/// and field metadata for storage-specific code generation.
pub fn generate_create_model(input: &DeriveInput) -> darling::Result<CreateModelOutput> {
    let parsed: CreateModelInputInternal = FromDeriveInput::from_derive_input(input)?;

    // Convert to shared config.
    let fields: Vec<ModelFieldConfig> = parsed.fields().iter().cloned().map(Into::into).collect();
    let field_infos: Vec<FieldInfo> = fields.iter().map(Into::into).collect();

    let model_fields = generate_model_fields(fields.iter());

    let name = &parsed.ident;
    let create_model_name = Ident::new(&format!("Create{}", name), name.span());
    let field_enum_name = format_ident!("{}Field", name);

    let struct_def = quote! {
        #[derive(Debug, Clone, PartialEq, utoipa::ToSchema, serde::Deserialize)]
        pub struct #create_model_name {
            #(#model_fields),*
        }
    };

    let trait_impls = quote! {
        impl crudkit_rs::data::Model for #create_model_name {
            type Field = #field_enum_name;
        }

        impl crudkit_rs::data::CreateModelTrait for #create_model_name {}
    };

    Ok(CreateModelOutput {
        struct_def,
        trait_impls,
        create_model_name,
        fields: field_infos,
    })
}

/// Parse and generate UpdateModel output from a DeriveInput.
///
/// Returns the struct definition, storage-agnostic trait implementations,
/// and field metadata for storage-specific code generation.
pub fn generate_update_model(input: &DeriveInput) -> darling::Result<UpdateModelOutput> {
    let parsed: UpdateModelInputInternal = FromDeriveInput::from_derive_input(input)?;

    // Convert to shared config.
    let fields: Vec<ModelFieldConfig> = parsed.fields().iter().cloned().map(Into::into).collect();
    let field_infos: Vec<FieldInfo> = fields.iter().map(Into::into).collect();

    let model_fields = generate_model_fields(fields.iter());

    let name = &parsed.ident;
    let update_model_name = Ident::new(&format!("Update{}", name), name.span());
    let field_enum_name = format_ident!("{}Field", name);

    let struct_def = quote! {
        #[derive(Debug, Clone, PartialEq, utoipa::ToSchema, serde::Deserialize)]
        pub struct #update_model_name {
            #(#model_fields),*
        }
    };

    let trait_impls = quote! {
        impl crudkit_rs::data::Model for #update_model_name {
            type Field = #field_enum_name;
        }
    };

    Ok(UpdateModelOutput {
        struct_def,
        trait_impls,
        update_model_name,
        fields: field_infos,
    })
}
