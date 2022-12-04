use darling::*;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Ident, Type};

#[derive(Debug, FromField)]
#[darling(attributes(validation_model))]
struct MyFieldReceiver {
    ident: Option<Ident>,

    ty: Type,

    /// Whether or not this field is part of the entities primary key.
    id: Option<bool>,
}

impl MyFieldReceiver {
    pub fn is_id(&self) -> bool {
        self.id.is_some() || self.ident.as_ref().unwrap() == "id"
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(validation_model), supports(struct_any))]
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

#[proc_macro_derive(ValidationModel, attributes(validation_model))]
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

    fn capitalize_first_letter(s: &str) -> String {
        s[0..1].to_uppercase() + &s[1..]
    }

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

    // vec.push(Column::Id); ...
    let pk_columns = input
        .fields()
        .iter()
        .filter(|field| field.is_id())
        .map(|field| {
            let name = field.ident.as_ref().expect("Expected named field!");
            let mut column_name = String::new();
            for part in name.to_string().split('_') {
                column_name.push_str(capitalize_first_letter(part).as_str());
            }
            let ident = Ident::new(column_name.as_str(), Span::call_site());
            quote! { vec.push(Column::#ident); }
        })
        .collect::<Vec<_>>();
    let pk_columns_len = pk_columns.len();

    quote! {
        pub mod validation_model {
            use sea_orm::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, Eq, sea_orm::DeriveEntityModel, serde::Serialize, serde::Deserialize)]
            #[sea_orm(table_name = #table_name)]
            pub struct Model {
                #[sea_orm(primary_key)]
                #[serde(skip_deserializing)]
                pub id: i32,

                // Fields storing the primary key (might be composite) of a validated entity.
                #(#pk_fields)*

                pub validator_name: String,
                pub validator_version: i32,
                pub violation_severity: crud_rs::validation::ValidationViolationType,
                pub violation_message: String,

                pub created_at: chrono_utc_date_time::UtcDateTime,
                pub updated_at: chrono_utc_date_time::UtcDateTime,
            }

            #[derive(Copy, Clone, Debug, sea_orm::DeriveRelation, sea_orm::EnumIter)]
            pub enum Relation {}

            impl sea_orm::entity::ActiveModelBehavior for ActiveModel {}

            impl core::convert::Into<crud_shared_types::validation::ValidationViolation> for Model {
                fn into(self) -> crud_shared_types::validation::ValidationViolation {
                    match self.violation_severity {
                        crud_rs::validation::ValidationViolationType::Major => crud_shared_types::validation::ValidationViolation::Major(self.violation_message),
                        crud_rs::validation::ValidationViolationType::Critical => crud_shared_types::validation::ValidationViolation::Critical(self.violation_message),
                    }
                }
            }

            impl crud_rs::NewActiveValidationModel<super::Id> for ActiveModel {
                fn new(entity_id: super::Id, validator_name: String, validator_version: i32, violation: crud_rs::validation::PersistableViolation, now: chrono_utc_date_time::UtcDateTime) -> Self {
                    Self {
                        id: sea_orm::ActiveValue::NotSet,

                        #(#set_pk_active_fields)*

                        validator_name: sea_orm::ActiveValue::Set(validator_name.to_owned()),
                        validator_version: sea_orm::ActiveValue::Set(validator_version),

                        violation_severity: sea_orm::ActiveValue::Set(violation.violation_severity),
                        violation_message: sea_orm::ActiveValue::Set(violation.violation_message),

                        created_at: sea_orm::ActiveValue::Set(now.clone()),
                        updated_at: sea_orm::ActiveValue::Set(now.clone()),
                    }
                }
            }

            impl crud_rs::ValidatorModel<super::Id> for Model {
                fn get_id(&self) -> super::Id {
                    super::Id {
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

            impl crud_rs::ValidationColumns for Column {
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

            impl crud_rs::IdColumns for Column {
                fn get_id_columns() -> Vec<Column> {
                    let mut vec = Vec::with_capacity(#pk_columns_len);
                    #(#pk_columns)*
                    vec
                }
            }
        }
    }
    .into()
}
