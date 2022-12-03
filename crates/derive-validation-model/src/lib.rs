use darling::*;
use proc_macro::TokenStream;
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

            impl core::convert::From<Model> for crud_shared_types::validation::ValidationViolation {
                fn from(val: Model) -> Self {
                    match val.violation_severity {
                        crud_rs::validation::ValidationViolationType::Major => crud_shared_types::validation::ValidationViolation::Major(val.violation_message),
                        crud_rs::validation::ValidationViolationType::Critical => crud_shared_types::validation::ValidationViolation::Critical(val.violation_message),
                    }
                }
            }
        }
    }
    .into()
}
