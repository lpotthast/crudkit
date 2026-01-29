//! Implementation of the `CkUpdateModel` derive macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

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
pub fn expand_derive_update_model(input: DeriveInput) -> syn::Result<TokenStream> {
    let output = crudkit_rs_macros_core::generate_update_model(&input)?;

    let struct_def = output.struct_def;
    let trait_impls = output.trait_impls;

    Ok(quote! {
        #struct_def
        #trait_impls
    })
}
