//! Implementation of the `CkCreateModel` derive macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

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
pub fn expand_derive_create_model(input: DeriveInput) -> syn::Result<TokenStream> {
    let output = crudkit_rs_macros_core::generate_create_model(&input)?;

    let struct_def = output.struct_def;
    let trait_impls = output.trait_impls;

    Ok(quote! {
        #struct_def
        #trait_impls
    })
}
