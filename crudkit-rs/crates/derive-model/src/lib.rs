#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Storage-agnostic derive macros for Create and Update models.
//!
//! These macros generate:
//! - The model struct (CreateModel/UpdateModel)
//! - Storage-agnostic trait implementations (CrudModel, CreateModelTrait)
//!
//! For SeaORM-specific implementations, use the macros from `crudkit-sea-orm-macros`.

use darling::*;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derives a `CreateModel` struct with storage-agnostic trait implementations.
///
/// # Generated Code
///
/// ```ignore
/// pub struct CreateModel { ... }
/// impl crudkit_rs::data::CrudModel for CreateModel { type Field = Col; }
/// impl crudkit_rs::data::CreateModelTrait for CreateModel {}
/// ```
///
/// # Field Attributes
///
/// - `#[ck_create_model(exclude)]` - Exclude field from CreateModel
/// - `#[ck_create_model(optional)]` - Wrap field in Option
/// - `#[ck_create_model(use_default)]` - Use Default::default() for excluded fields (storage-specific)
#[proc_macro_derive(CkCreateModel, attributes(ck_create_model))]
pub fn ck_create_model(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let output = match derive_model_core::generate_create_model(&ast) {
        Ok(output) => output,
        Err(err) => return Error::write_errors(err).into(),
    };

    let struct_def = output.struct_def;
    let trait_impls = output.trait_impls;

    quote! {
        #struct_def
        #trait_impls
    }
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
pub fn ck_update_model(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let output = match derive_model_core::generate_update_model(&ast) {
        Ok(output) => output,
        Err(err) => return Error::write_errors(err).into(),
    };

    let struct_def = output.struct_def;
    let trait_impls = output.trait_impls;

    quote! {
        #struct_def
        #trait_impls
    }
    .into()
}
