#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use darling::*;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, proc_macro_error};
use proc_macro_type_name::ToTypeName;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

#[derive(Debug, FromField)]
#[darling(attributes(ck_columns, ck_id))]
struct MyFieldReceiver {
    ident: Option<syn::Ident>,

    ty: syn::Type,

    /// Whether or not this field is part of the entities primary key.
    id: Option<bool>,

    convert_ccv: Option<String>,
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
#[darling(attributes(ck_columns, ck_id), supports(struct_any))]
struct MyInputReceiver {
    ident: syn::Ident,

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

#[proc_macro_derive(CkColumns, attributes(ck_columns, ck_id))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let input: MyInputReceiver = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return darling::Error::write_errors(err).into(),
    };

    let fields = input.fields().iter().collect::<Vec<_>>();

    let id_fields = input
        .fields()
        .iter()
        .filter(|field| field.is_id())
        .collect::<Vec<_>>();

    let column_variants = fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().expect("Expected named field!");
            let span = ident.span();
            ident.to_type_ident(span)
        })
        .collect::<Vec<Ident>>();

    let col_to_column_match_arms = column_variants.iter().map(|variant| {
        quote! { Col::#variant => Column::#variant }
    });

    let init_id_struct_fields = id_fields.iter().map(|field| {
        let ident = field.ident.as_ref().expect("Ident to be present").clone();
        // Example: id: self.id.clone()
        quote! { #ident: model.#ident.clone() }
        // TODO: Always clone here?
    });

    let init_id_struct_fields_self = id_fields.iter().map(|field| {
        let ident = field.ident.as_ref().expect("Ident to be present").clone();
        // Example: id: self.id.clone()
        quote! { #ident: self.#ident.clone() }
        // TODO: Always clone here?
    });

    let init_active_id_struct_fields = id_fields.iter().map(|field| {
        let ident = field.ident.as_ref().expect("Ident to be present").clone();
        // Example: id: self.id.clone()
        quote! { #ident: active_model.#ident.clone().into_value().map(|v| v.unwrap()).unwrap() }
        // TODO: Always clone here?
    });

    let extract_ccv_value_by_column_variant_match_arms = fields
        .iter()
        .zip(column_variants.iter())
        .map(|(field, variant)| {
            let fun_name = match &field.convert_ccv {
                Some(fun_name) => Ident::new(fun_name.as_str(), field.ident.span()),
                None => convert_field_type_to_function_name(&field.ty),
            };
            quote! {
                Column::#variant => crudkit_rs::#fun_name(value)
            }
        });

    let get_col_arms = fields
        .iter()
        .zip(column_variants.iter())
        .map(|(field, variant)| {
            let ident = field.ident.as_ref().expect("Expected named field!");
            quote! { stringify!(#ident) => Some(Column::#variant) }
        });

    // TODO: Use given id ident or fall back to expectable default...
    let id_struct_ident = Ident::new(format!("{}Id", input.ident).as_str(), Span::call_site());

    quote! {
        #[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
        pub enum Col {
            #(#column_variants),*
        }

        impl crudkit_rs::CrudColumns<Column, Model, ActiveModel> for Col {
            type Id = #id_struct_ident;

            fn to_sea_orm_column(&self) -> Column {
                match self {
                    #(#col_to_column_match_arms),*
                }
            }

            // We use #struct_ident instead of Self::Id, as `for Col`, Col being an enum, can lead to indistinguishable types.
            fn get_id(model: &Model) -> #id_struct_ident {
                #id_struct_ident {
                    #(#init_id_struct_fields),*
                }
            }

            // We use #struct_ident instead of Self::Id, as `for Col`, Col being an enum, can lead to indistinguishable types.
            fn get_id_active(active_model: &ActiveModel) -> std::result::Result<#id_struct_ident, String> {
                // TODO: The init_active_id_struct_fields code unwraps() and therefore panics. Catch missing data and return an error.
                Ok(#id_struct_ident {
                    #(#init_active_id_struct_fields),*
                })
            }
        }

        impl crudkit_rs::GetIdFromModel for Model {
            type Id = #id_struct_ident;

            // We use #struct_ident instead of Self::Id, as `for Col`, Col being an enum, can lead to indistinguishable types.
            fn get_id(&self) -> #id_struct_ident {
                #id_struct_ident {
                    #(#init_id_struct_fields_self),*
                }
            }
        }

        // TODO: Can we not just convert From<ConditionClauseValue> for std::result::Result<crudkit_shared::Value, String> by using the From trait?
        impl crudkit_rs::AsColType for Column {
            fn as_col_type(&self, value: crudkit_condition::ConditionClauseValue) -> std::result::Result<crudkit_shared::Value, String> {
                match self {
                    #(#extract_ccv_value_by_column_variant_match_arms),*
                }
            }
        }

        impl crudkit_rs::MaybeColumnTrait for Entity {
            type Column = Column;

            fn get_col(name: &str) -> std::option::Option<Self::Column> {
                match name {
                    #(#get_col_arms),*,
                    _ => None,
                }
            }
        }
    }.into()
}

fn convert_field_type_to_function_name(ty: &syn::Type) -> Ident {
    let span = ty.span();
    // TODO: This should dynamically check types by absolute (resolved) path!
    let fun_name = match ty {
        syn::Type::Path(path) => match join_path(&path.path).as_str() {
            "bool" => "to_bool",
            "Vec<u8>" => "to_byte_vec",
            "u32" => "to_u32",
            "i32" => "to_i32",
            "i64" => "to_i64",
            "f32" => "to_f32",
            "OrderedFloat<f32>" => "to_f32",
            "ordered_float::OrderedFloat<f32>" => "to_f32",
            "f64" => "to_f64",
            "OrderedFloat<f64>" => "to_f64",
            "ordered_float::OrderedFloat<f64>" => "to_f64",
            "String" => "to_string",
            "serde_json::Value" => "to_json_value",
            "crudkit_shared::UuidV4" => "to_uuid_v4",
            "crudkit_shared::UuidV7" => "to_uuid_v7",
            "time::PrimitiveDateTime" => "to_primitive_date_time",
            "time::OffsetDateTime" => "to_offset_date_time",
            "time::Time" => "to_time",
            "Option<u32>" => "to_u32",
            "Option<i32>" => "to_i32",
            "Option<i64>" => "to_i64",
            "Option<String>" => "to_string",
            "Option<serde_json::Value>" => "to_json_value",
            "Option<time::PrimitiveDateTime>" => "to_primitive_date_time",
            "Option<time::OffsetDateTime>" => "to_offset_date_time",
            "Option<TimeDuration>" => "to_time_duration",
            other => {
                let message =
                    format!("derive-crud-columns: Unknown type {other:?}. Expected a known type.");
                abort!(
                    span, message;
                    help = "use one of the following types: [...]";
                );
            }
        },
        other => {
            let message = format!(
                "derive-crud-columns: Unknown type {other:?}. Not a 'Path' type. Expected a known type."
            );
            abort!(
                span, message;
                help = "use one of the following types: [...]";
            );
        }
    };
    Ident::new(fun_name, span)
}

fn join_path(path: &syn::Path) -> String {
    path.to_token_stream().to_string().replace(' ', "")
}
