#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use darling::*;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Ident, Type};

#[derive(Debug, FromField)]
#[darling(attributes(ck_validation_model))]
struct MyFieldReceiver {
    ident: Option<Ident>,

    ty: Type,

    /// Whether or not this field is part of the entities primary key.
    id: Option<bool>,
}

impl MyFieldReceiver {
    pub fn is_id(&self) -> bool {
        match (self.id, &self.ident) {
            (None, None) => false,
            (None, Some(ident)) => ident == "id",
            (Some(id), None) => id,
            (Some(id), Some(ident)) => id || ident == "id",
        }
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ck_validation_model), supports(struct_any))]
struct MyInputReceiver {
    table_name: String,

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

#[proc_macro_derive(CkValidationModel, attributes(ck_validation_model))]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let input: MyInputReceiver = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return darling::Error::write_errors(err).into(),
    };

    let table_name = &input.table_name;

    let pk_fields = input
        .fields()
        .iter()
        .filter(|field| field.is_id())
        .map(|field| {
            let ident = Ident::new(
                format!("entity_{}", field.ident.as_ref().expect("Named field")).as_str(),
                field.ident.span(),
            );
            let ty = &field.ty;
            quote! { pub #ident: #ty, }
        });

    let set_pk_active_fields = input
        .fields()
        .iter()
        .filter(|field| field.is_id())
        .map(|field| {
            let original_ident = field.ident.as_ref().expect("Named field");
            let ident = Ident::new(
                format!("entity_{original_ident}").as_str(),
                field.ident.span(),
            );

            quote! { #ident: sea_orm::ActiveValue::Set(entity_id.#original_ident.clone()), }
        });

    // id: self.entity_id.clone(),
    let super_id_field_init = input
        .fields()
        .iter()
        .filter(|field| field.is_id())
        .map(|field| {
            let original_ident = field.ident.as_ref().expect("Named field");
            let ident = Ident::new(
                format!("entity_{original_ident}").as_str(),
                field.ident.span(),
            );
            quote! { #original_ident: self.#ident.clone(), }
        });

    quote! {
        pub mod validation_model {
            use crudkit_sea_orm::CrudColumns;
            use sea_orm::entity::prelude::*;
            use serde::{Deserialize, Serialize};

            type ParentId = <super::Col as CrudColumns<super::Column, super::Model, super::ActiveModel>>::Id;

            #[derive(
                Clone,
                Debug,
                PartialEq,
                Eq,
                crudkit_sea_orm::crudkit_id::CkId,
                crudkit_sea_orm::CkColumns,
                sea_orm::DeriveEntityModel,
                serde::Serialize,
                serde::Deserialize
            )]
            #[sea_orm(table_name = #table_name)]
            pub struct Model {
                #[sea_orm(primary_key)]
                #[serde(skip_deserializing)]
                pub id: i64,

                // Fields storing the primary key (might be composite) of a validated entity.
                #(#pk_fields)*

                pub validator_name: String,
                pub validator_version: i32,
                #[ck_columns(convert_ccv = "to_string")]
                pub violation_severity: crudkit_sea_orm::validation::PersistedViolationSeverity,
                pub violation_message: String,

                pub created_at: time::OffsetDateTime,
                pub updated_at: time::OffsetDateTime,
            }

            #[derive(Copy, Clone, Debug, sea_orm::DeriveRelation, sea_orm::EnumIter)]
            pub enum Relation {}

            impl sea_orm::entity::ActiveModelBehavior for ActiveModel {}

            impl core::convert::Into<crudkit_validation::ValidationViolation> for Model {
                fn into(self) -> crudkit_validation::ValidationViolation {
                    match self.violation_severity {
                        crudkit_sea_orm::validation::PersistedViolationSeverity::Major => crudkit_validation::Violation::major(self.violation_message),
                        crudkit_sea_orm::validation::PersistedViolationSeverity::Critical => crudkit_validation::Violation::critical(self.violation_message),
                    }
                }
            }

            impl crudkit_sea_orm::NewActiveValidationModel<ParentId> for ActiveModel {
                fn new(entity_id: ParentId, validator_name: String, validator_version: i32, violation: crudkit_sea_orm::PersistableViolation, now: time::OffsetDateTime) -> Self {
                    Self {
                        id: sea_orm::ActiveValue::NotSet,

                        #(#set_pk_active_fields)*

                        validator_name: sea_orm::ActiveValue::Set(validator_name.to_owned()),
                        validator_version: sea_orm::ActiveValue::Set(validator_version),

                        violation_severity: sea_orm::ActiveValue::Set(violation.severity()),
                        violation_message: sea_orm::ActiveValue::Set(violation.into_message()),

                        created_at: sea_orm::ActiveValue::Set(now.clone()),
                        updated_at: sea_orm::ActiveValue::Set(now.clone()),
                    }
                }
            }

            impl crudkit_sea_orm::ValidatorModel<ParentId> for Model {
                fn get_id(&self) -> ParentId {
                    ParentId {
                        #(#super_id_field_init)*
                    }
                }

                fn get_validator_name(&self) -> String {
                    self.validator_name.clone()
                }

                fn get_validator_version(&self) -> i32 {
                    self.validator_version
                }
            }

            impl crudkit_sea_orm::ValidationColumns for Column {
                fn get_validator_name_column() -> Self {
                    Self::ValidatorName
                }

                fn get_validator_version_column() -> Self {
                    Self::ValidatorVersion
                }

                fn get_violation_severity_column() -> Self {
                    Self::ViolationSeverity
                }
            }

            // Note: This impl returns the ID columns of this validation model (statically known from above), not the parent model!
            // For that ID, see the `impl crudkit_sea_orm::ValidatorModel<ParentId> for Model` implementation.
            impl crudkit_sea_orm::IdColumns for Column {
                fn get_id_columns() -> Vec<Column> {
                    let mut vec = Vec::with_capacity(1);
                    vec.push(Column::Id);
                    vec
                }
            }
        }
    }
    .into()
}
