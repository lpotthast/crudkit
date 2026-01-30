#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Consolidated derive macros for crudkit-rs.
//!
//! This crate provides storage-agnostic derive macros:
//! - `CkField` - Generates a `{StructName}Field` enum and implements `CrudModel`, `HasId`, etc.
//! - `CkResourceContext` - Implements `CrudResourceContext` marker trait
//! - `CkCreateModel` - Generates a `CreateModel` struct with trait implementations
//! - `CkUpdateModel` - Generates an `UpdateModel` struct with trait implementations

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput, Error};

mod derives;

/// Derives a `{StructName}Field` enum and implements storage-agnostic traits.
///
/// # Generated Code
///
/// - `{StructName}Field` enum with variants for each field (e.g., `Model` → `ModelField`)
/// - `impl FieldTrait for {StructName}Field` - Field name lookup
/// - `impl FieldLookup for {StructName}Field` - Reverse lookup from name to field
/// - `impl ConditionValueConverter for {StructName}Field` - Convert condition values to typed values
/// - `impl CrudModel for T` - Associates the model with its `{StructName}Field` type
/// - `impl HasId for T` - Extracts the entity ID
///
/// # Attributes
///
/// - `#[ck_id(id)]` - Mark field as part of primary key
/// - `#[ck_field(convert_ccv = "fn_name")]` - Custom condition value converter function
#[proc_macro_derive(CkField, attributes(ck_field, ck_id))]
#[proc_macro_error]
pub fn derive_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::expand_derive_field(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Derives the `CrudResourceContext` marker trait.
///
/// This is a simple marker trait that indicates a type can be used as
/// resource-specific context in `CrudContext<R>`.
#[proc_macro_derive(CkResourceContext, attributes(ck_resource_context))]
pub fn derive_resource_context(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::expand_derive_resource_context(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Derives a `CreateModel` struct with storage-agnostic trait implementations.
///
/// # Generated Code
///
/// ```ignore
/// pub struct CreateModel { ... }
/// impl crudkit_rs::data::Model for CreateModel { type Field = Col; }
/// impl crudkit_rs::data::CreateModel for CreateModel {}
/// ```
///
/// # Field Attributes
///
/// - `#[ck_create_model(exclude)]` - Exclude field from CreateModel
/// - `#[ck_create_model(optional)]` - Wrap field in Option
/// - `#[ck_create_model(use_default)]` - Use Default::default() for excluded fields (storage-specific)
#[proc_macro_derive(CkCreateModel, attributes(ck_create_model))]
pub fn derive_create_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::expand_derive_create_model(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Derives an `UpdateModel` struct with storage-agnostic trait implementations.
///
/// # Generated Code
///
/// ```ignore
/// pub struct UpdateModel { ... }
/// impl crudkit_rs::data::CrudModel for UpdateModel { type Field = Col; }
/// ```
///
/// # Field Attributes
///
/// - `#[ck_update_model(exclude)]` - Exclude field from UpdateModel
/// - `#[ck_update_model(optional)]` - Wrap field in Option
/// - `#[ck_update_model(use_default)]` - Use Default::default() for excluded fields (storage-specific)
#[proc_macro_derive(CkUpdateModel, attributes(ck_update_model))]
pub fn derive_update_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::expand_derive_update_model(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
