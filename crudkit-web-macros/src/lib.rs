#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Consolidated derive macros for crudkit-web.
//!
//! This crate provides derive macros for frontend CRUD models:
//! - `CkResource` - Generates a resource struct implementing `Resource` (CrudMainTrait)
//! - `CkField` - Generates a `{StructName}Field` enum and implements `Model`, field access traits
//! - `CkActionPayload` - Implements `CrudActionPayload` and `ActionPayload` traits

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput, Error};

mod derives;

/// Derives a resource struct implementing the `Resource` trait.
///
/// # Generated Code
///
/// - `Crud{StructName}Resource` struct
/// - `impl Resource for Crud{StructName}Resource` with type associations
///
/// # Attributes
///
/// - `#[ck_resource(resource_name = "...")]` - Required: the API resource name
/// - `#[ck_resource(action_payload = "TypeName")]` - Optional: custom action payload type
/// - `#[ck_resource(create_model = Ident)]` - Optional: custom create model type
/// - `#[ck_resource(read_model = Ident)]` - Optional: custom read model type
/// - `#[ck_resource(update_model = Ident)]` - Optional: custom update model type (defaults to struct name)
#[proc_macro_derive(CkResource, attributes(ck_resource))]
#[proc_macro_error]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::expand_derive_resource(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Derives a `{StructName}Field` enum and implements model traits.
///
/// # Generated Code
///
/// - `{StructName}Field` enum with variants for each field
/// - `impl Model for {StructName}` - Field enumeration and lookup
/// - `impl FieldAccess<{StructName}> for {StructName}Field` - Value get/set
/// - Type-erased trait implementations based on `model` attribute
///
/// # Attributes
///
/// - `#[ck_field(model = Create|Read|Update)]` - Required: specifies which model type this is
#[proc_macro_derive(CkField, attributes(ck_field, ck_id))]
#[proc_macro_error]
pub fn derive_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::expand_derive_field(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Derives the `CrudActionPayload` and `ActionPayload` traits.
///
/// # Generated Code
///
/// - `impl CrudActionPayload for {StructName}`
/// - `impl ActionPayload for {StructName}`
#[proc_macro_derive(CkActionPayload, attributes(ck_action_payload))]
#[proc_macro_error]
pub fn derive_action_payload(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::expand_derive_action_payload(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
