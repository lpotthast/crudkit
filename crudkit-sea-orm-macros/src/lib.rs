use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error};

mod derives;

/// Derives a `read_view` module containing a copy of the annotated struct with the
/// `pub has_validation_errors: bool` field added to it.
#[proc_macro_derive(ReadView, attributes(read_view))]
pub fn derive_migration_name(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::expand_derive_read_view(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Derives `impl CrudColumns<Column> for Col` which maps the `Col` enum variants
/// to SeaORM `Column` variants.
#[proc_macro_derive(CkSeaOrmBridge, attributes(ck_columns, ck_id))]
pub fn derive_sea_orm_bridge(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::expand_derive_sea_orm_bridge(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Derives a `CreateModel` struct with both storage-agnostic and SeaORM-specific trait implementations.
///
/// # Generated Code
///
/// ```ignore
/// pub struct CreateModel { ... }
/// impl crudkit_rs::data::CrudModel for CreateModel { type Field = Col; }
/// impl crudkit_rs::data::CreateModelTrait for CreateModel {}
/// impl crudkit_sea_orm::CreateModelTrait<ActiveModel> for CreateModel { ... }
/// ```
///
/// # Field Attributes
///
/// - `#[ck_create_model(exclude)]` - Exclude field from CreateModel
/// - `#[ck_create_model(optional)]` - Wrap field in Option
/// - `#[ck_create_model(use_default)]` - Use Default::default() for excluded fields
#[proc_macro_derive(CkSeaOrmCreateModel, attributes(ck_create_model))]
pub fn derive_sea_orm_create_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::expand_derive_sea_orm_create_model(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Derives an `UpdateModel` struct with both storage-agnostic and SeaORM-specific trait implementations.
///
/// # Generated Code
///
/// ```ignore
/// pub struct UpdateModel { ... }
/// impl crudkit_rs::data::CrudModel for UpdateModel { type Field = Col; }
/// impl crudkit_sea_orm::UpdateModelTrait for UpdateModel {}
/// impl crudkit_sea_orm::UpdateActiveModelTrait<UpdateModel> for ActiveModel { ... }
/// ```
///
/// # Field Attributes
///
/// - `#[ck_update_model(exclude)]` - Exclude field from UpdateModel
/// - `#[ck_update_model(optional)]` - Wrap field in Option
/// - `#[ck_update_model(use_default)]` - Use Default::default() for excluded fields
#[proc_macro_derive(CkSeaOrmUpdateModel, attributes(ck_update_model))]
pub fn derive_sea_orm_update_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::expand_derive_sea_orm_update_model(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
