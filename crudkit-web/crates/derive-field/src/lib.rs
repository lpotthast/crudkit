#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use crudkit_derive_core::to_pascal_case;
use darling::*;
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use proc_macro2::{Ident, Span};
use quote::quote;
use serde::Deserialize;
use syn::{DeriveInput, parse_macro_input, spanned::Spanned};
use types::ModelType;

// Helper to parse explicit type hints from the `ty` attribute.
fn parse_value_type(string: Option<String>) -> Option<ValueType> {
    string.map(|ty| match serde_json::from_str(format!("\"{ty}\"").as_str()) {
        Ok(value_type) => value_type,
        Err(err) => panic!("expected `ck_field(ty = ...)`, where '...' (actual: {ty}) is of a known variant. serde error: {err:?}"),
    })
}

#[derive(Debug, FromField)]
#[darling(attributes(ck_field, ck_id))]
struct CkFieldConfig {
    ident: Option<Ident>,

    /// The Rust type of this field (from syn).
    ty: syn::Type,

    /// Explicit value type override via `#[ck_field(ty = "...")]`.
    #[darling(rename = "ty")]
    #[darling(map = "parse_value_type")]
    value_type: Option<ValueType>,

    /// Determines whether this field is part of the aggregate id.
    id: Option<bool>,
}

impl CkFieldConfig {
    pub fn is_id(&self) -> bool {
        match (self.id, &self.ident) {
            (None, None) => false,
            (None, Some(ident)) => ident == "id",
            (Some(id), None) => id,
            (Some(id), Some(ident)) => id || ident == "id",
        }
    }

    pub fn value_type(&self) -> ValueType {
        self.value_type.unwrap_or_else(|| (&self.ty).into())
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ck_field, ck_id), supports(struct_any))]
struct CkFieldInputReceiver {
    ident: Ident,

    /// Whether this is a `create`, `read` or `update` model.
    /// Depending on this required information, the generated fields enum either implements the
    /// `CreateField`, `ReadField` or `UpdateField` trait.
    model: ModelType,

    data: ast::Data<(), CkFieldConfig>,
}

impl CkFieldInputReceiver {
    pub fn fields(&self) -> &ast::Fields<CkFieldConfig> {
        match &self.data {
            ast::Data::Enum(_) => panic!("Only structs are supported"),
            ast::Data::Struct(fields) => fields,
        }
    }
}

/// Describes the type of value without carrying any actual value.
/// Maps to `crudkit_core::Value` variants.
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromMeta, Deserialize)]
enum ValueType {
    Void,

    Bool,
    OptionalBool,

    U8,
    U16,
    U32,
    U64,
    U128,
    OptionalU8,
    OptionalU16,
    OptionalU32,
    OptionalU64,
    OptionalU128,

    I8,
    I16,
    I32,
    I64,
    I128,
    OptionalI8,
    OptionalI16,
    OptionalI32,
    OptionalI64,
    OptionalI128,

    F32,
    F64,
    OptionalF32,
    OptionalF64,
    OrderedF32,
    OrderedF64,

    String,
    OptionalString,

    Json,
    OptionalJson,

    Uuid,
    OptionalUuid,

    PrimitiveDateTime,
    OffsetDateTime,
    Duration,
    OptionalPrimitiveDateTime,
    OptionalOffsetDateTime,
    OptionalDuration,

    Other,
}

/// Converts to the name of the `crudkit_core::Value` variant which should be used.
impl From<ValueType> for Ident {
    fn from(value_type: ValueType) -> Self {
        Ident::new(
            match value_type {
                ValueType::Void => "Void",

                ValueType::Bool => "Bool",
                ValueType::OptionalBool => "OptionalBool",

                ValueType::U8 => "U8",
                ValueType::U16 => "U16",
                ValueType::U32 => "U32",
                ValueType::U64 => "U64",
                ValueType::U128 => "U128",
                ValueType::OptionalU8 => "OptionalU8",
                ValueType::OptionalU16 => "OptionalU16",
                ValueType::OptionalU32 => "OptionalU32",
                ValueType::OptionalU64 => "OptionalU64",
                ValueType::OptionalU128 => "OptionalU128",

                ValueType::I8 => "I8",
                ValueType::I16 => "I16",
                ValueType::I32 => "I32",
                ValueType::I64 => "I64",
                ValueType::I128 => "I128",
                ValueType::OptionalI8 => "OptionalI8",
                ValueType::OptionalI32 => "OptionalI32",
                ValueType::OptionalI64 => "OptionalI64",
                ValueType::OptionalI16 => "OptionalI16",
                ValueType::OptionalI128 => "OptionalI128",

                ValueType::F32 => "F32",
                ValueType::F64 => "F64",
                ValueType::OrderedF32 => "F32",
                ValueType::OrderedF64 => "F64",
                ValueType::OptionalF32 => "OptionalF32",
                ValueType::OptionalF64 => "OptionalF64",

                ValueType::String => "String",
                ValueType::OptionalString => "OptionalString",

                ValueType::Json => "Json",
                ValueType::OptionalJson => "OptionalJson",

                ValueType::Uuid => "Uuid",
                ValueType::OptionalUuid => "OptionalUuid",

                ValueType::PrimitiveDateTime => "PrimitiveDateTime",
                ValueType::OffsetDateTime => "OffsetDateTime",
                ValueType::Duration => "Duration",
                ValueType::OptionalPrimitiveDateTime => "OptionalPrimitiveDateTime",
                ValueType::OptionalOffsetDateTime => "OptionalOffsetDateTime",
                ValueType::OptionalDuration => "OptionalDuration",

                ValueType::Other => "Other",
            },
            Span::call_site(),
        )
    }
}

impl From<&syn::Type> for ValueType {
    fn from(ty: &syn::Type) -> Self {
        match &ty {
            syn::Type::Tuple(syn::TypeTuple {
                paren_token: _,
                elems,
            }) => match elems.is_empty() {
                true => ValueType::Void,
                false => {
                    let span = ty.span();
                    let message = "crudkit: derive-field: Tuple type with elements is unsupported. Only the `()` tuple is allowed, representing a custom field.";
                    abort!(span, message);
                }
            },
            syn::Type::Path(path) => match join_path(&path.path).as_str() {
                "()" => ValueType::Void,

                "bool" => ValueType::Bool,
                "Option<bool>" => ValueType::OptionalBool,

                "u8" => ValueType::U8,
                "u16" => ValueType::U16,
                "u32" => ValueType::U32,
                "u64" => ValueType::U64,
                "u128" => ValueType::U128,
                "Option<u8>" => ValueType::OptionalU8,
                "Option<u16>" => ValueType::OptionalU16,
                "Option<u32>" => ValueType::OptionalU32,
                "Option<u64>" => ValueType::OptionalU64,
                "Option<u128>" => ValueType::OptionalU128,

                "i8" => ValueType::I8,
                "i16" => ValueType::I16,
                "i32" => ValueType::I32,
                "i64" => ValueType::I64,
                "Option<i8>" => ValueType::OptionalI8,
                "Option<i16>" => ValueType::OptionalI16,
                "Option<i32>" => ValueType::OptionalI32,
                "Option<i64>" => ValueType::OptionalI64,
                "Option<i128>" => ValueType::OptionalI128,

                "f32" => ValueType::F32,
                "f64" => ValueType::F64,
                "OrderedFloat<f32>" => ValueType::OrderedF32,
                "ordered_float::OrderedFloat<f32>" => ValueType::OrderedF32,
                "OrderedFloat<f64>" => ValueType::OrderedF64,
                "ordered_float::OrderedFloat<f64>" => ValueType::OrderedF64,
                "Option<f32>" => ValueType::OptionalF32,
                "Option<f64>" => ValueType::OptionalF64,

                "String" => ValueType::String,
                "Option<String>" => ValueType::OptionalString,

                "serde_json::Value" => ValueType::Json,
                "Option<serde_json::Value>" => ValueType::OptionalJson,

                "Uuid" => ValueType::Uuid,
                "Option<Uuid>" => ValueType::OptionalUuid,

                "time::PrimitiveDateTime" => ValueType::PrimitiveDateTime,
                "time::OffsetDateTime" => ValueType::OffsetDateTime,
                "TimeDuration" => ValueType::Duration,
                "Option<time::PrimitiveDateTime>" => ValueType::OptionalPrimitiveDateTime,
                "Option<time::OffsetDateTime>" => ValueType::OptionalOffsetDateTime,
                "Option<TimeDuration>" => ValueType::OptionalDuration,

                _other => ValueType::Other,
            },
            other => {
                let span = ty.span();
                let message =
                    format!("crudkit: derive-field: Unknown type {other:?}. Not a 'Path' variant.");
                abort!(span, message);
            }
        }
    }
}

fn join_path(path: &syn::Path) -> String {
    quote::quote!(#path).to_string().replace(' ', "")
}

/// Generates the `get_value` match arms for CrudFieldValueTrait.
fn generate_get_value_arm(
    field: &CkFieldConfig,
    field_enum_ident: &Ident,
) -> proc_macro2::TokenStream {
    let field_ident = field.ident.as_ref().expect("Expected named field!");
    let field_name = field_ident.to_string();
    let pascal_case = to_pascal_case(&field_name);
    let field_name_as_type_ident = Ident::new(pascal_case.as_str(), Span::call_site());

    let value_type = field.value_type();
    let value_type_ident: Ident = value_type.into();

    // Code that clones or copies the fields value.
    let value_clone = match value_type {
        ValueType::Void => quote! { () },

        ValueType::Bool => quote! { entity.#field_ident },
        ValueType::OptionalBool => quote! { entity.#field_ident },

        ValueType::U8 => quote! { entity.#field_ident },
        ValueType::U16 => quote! { entity.#field_ident },
        ValueType::U32 => quote! { entity.#field_ident },
        ValueType::U64 => quote! { entity.#field_ident },
        ValueType::U128 => quote! { entity.#field_ident },
        ValueType::OptionalU8 => quote! { entity.#field_ident.clone() },
        ValueType::OptionalU16 => quote! { entity.#field_ident.clone() },
        ValueType::OptionalU32 => quote! { entity.#field_ident.clone() },
        ValueType::OptionalU64 => quote! { entity.#field_ident.clone() },
        ValueType::OptionalU128 => quote! { entity.#field_ident.clone() },

        ValueType::I8 => quote! { entity.#field_ident },
        ValueType::I16 => quote! { entity.#field_ident },
        ValueType::I32 => quote! { entity.#field_ident },
        ValueType::I64 => quote! { entity.#field_ident },
        ValueType::I128 => quote! { entity.#field_ident },
        ValueType::OptionalI8 => quote! { entity.#field_ident.clone() },
        ValueType::OptionalI16 => quote! { entity.#field_ident.clone() },
        ValueType::OptionalI32 => quote! { entity.#field_ident.clone() },
        ValueType::OptionalI64 => quote! { entity.#field_ident.clone() },
        ValueType::OptionalI128 => quote! { entity.#field_ident.clone() },

        ValueType::String => quote! { entity.#field_ident.clone() },
        ValueType::OptionalString => quote! { entity.#field_ident.clone() },

        ValueType::Json => quote! { crudkit_web::JsonValue::new(entity.#field_ident.clone()) },
        ValueType::OptionalJson => {
            quote! { entity.#field_ident.clone().map(|it| crudkit_web::JsonValue::new(it)) }
        }

        ValueType::Uuid => quote! { entity.#field_ident },
        ValueType::OptionalUuid => quote! { entity.#field_ident.clone() },

        ValueType::F32 => quote! { entity.#field_ident },
        ValueType::OrderedF32 => quote! { entity.#field_ident.into() },
        ValueType::F64 => quote! { entity.#field_ident },
        ValueType::OrderedF64 => quote! { entity.#field_ident.into() },
        ValueType::OptionalF32 => quote! { entity.#field_ident.into() },
        ValueType::OptionalF64 => quote! { entity.#field_ident.into() },

        ValueType::PrimitiveDateTime => quote! { entity.#field_ident.clone() },
        ValueType::OffsetDateTime => quote! { entity.#field_ident.clone() },
        ValueType::Duration => quote! { entity.#field_ident.clone() },
        ValueType::OptionalPrimitiveDateTime => quote! { entity.#field_ident.clone() },
        ValueType::OptionalOffsetDateTime => quote! { entity.#field_ident.clone() },
        ValueType::OptionalDuration => quote! { entity.#field_ident.clone() },

        ValueType::Other => quote! { () }, // not important, panics anyway...
    };

    quote! {
        #field_enum_ident::#field_name_as_type_ident => crudkit_core::Value::#value_type_ident(#value_clone)
    }
}

/// Generates the `set_value` match arms for CrudFieldValueTrait.
fn generate_set_value_arm(
    field: &CkFieldConfig,
    field_enum_ident: &Ident,
) -> proc_macro2::TokenStream {
    let field_ident = field.ident.as_ref().expect("Expected named field!");
    let field_name = field_ident.to_string();
    let pascal_case = to_pascal_case(&field_name);
    let field_name_as_type_ident = Ident::new(pascal_case.as_str(), Span::call_site());

    // An expression that, given a `value`, constructs the necessary data type value to be assigned to the field.
    let take_op = match field.value_type() {
        ValueType::Void => {
            quote! { ::tracing::warn!("Setting a custom field is not allowed") }
        }

        ValueType::Bool => quote! { entity.#field_ident = value.take_bool() },
        ValueType::OptionalBool => quote! { entity.#field_ident = value.take_optional_bool() },

        ValueType::U8 => quote! { entity.#field_ident = value.take_u8() },
        ValueType::U16 => quote! { entity.#field_ident = value.take_u16() },
        ValueType::U32 => quote! { entity.#field_ident = value.take_u32() },
        ValueType::U64 => quote! { entity.#field_ident = value.take_u64() },
        ValueType::U128 => quote! { entity.#field_ident = value.take_u128() },
        ValueType::OptionalU8 => quote! { entity.#field_ident = value.take_optional_u8() },
        ValueType::OptionalU16 => quote! { entity.#field_ident = value.take_optional_u16() },
        ValueType::OptionalU32 => quote! { entity.#field_ident = value.take_optional_u32() },
        ValueType::OptionalU64 => quote! { entity.#field_ident = value.take_optional_u64() },
        ValueType::OptionalU128 => quote! { entity.#field_ident = value.take_optional_u128() },

        ValueType::I8 => quote! { entity.#field_ident = value.take_i8() },
        ValueType::I16 => quote! { entity.#field_ident = value.take_i16() },
        ValueType::I32 => quote! { entity.#field_ident = value.take_i32() },
        ValueType::I64 => quote! { entity.#field_ident = value.take_i64() },
        ValueType::I128 => quote! { entity.#field_ident = value.take_i128() },
        ValueType::OptionalI8 => quote! { entity.#field_ident = value.take_optional_i8() },
        ValueType::OptionalI16 => quote! { entity.#field_ident = value.take_optional_i16() },
        ValueType::OptionalI32 => quote! { entity.#field_ident = value.take_optional_i32() },
        ValueType::OptionalI64 => quote! { entity.#field_ident = value.take_optional_i64() },
        ValueType::OptionalI128 => quote! { entity.#field_ident = value.take_optional_i128() },

        ValueType::F32 => quote! { entity.#field_ident = value.take_f32() },
        ValueType::OrderedF32 => quote! { entity.#field_ident = value.take_f32().into() },
        ValueType::F64 => quote! { entity.#field_ident = value.take_f64() },
        ValueType::OrderedF64 => quote! { entity.#field_ident = value.take_f64().into() },
        ValueType::OptionalF32 => quote! { entity.#field_ident = value.take_optional_f32() },
        ValueType::OptionalF64 => quote! { entity.#field_ident = value.take_optional_f64() },

        ValueType::String => quote! { entity.#field_ident = value.take_string() },
        ValueType::OptionalString => quote! { entity.#field_ident = value.take_optional_string() },

        ValueType::Json => quote! { entity.#field_ident = value.take_inner_json_value() },
        ValueType::OptionalJson => {
            quote! { entity.#field_ident = std::option::Option::Some(value.take_inner_json_value()) }
        }

        ValueType::Uuid => quote! { entity.#field_ident = value.to_uuid() },
        ValueType::OptionalUuid => quote! { entity.#field_ident = value.to_optional_uuid() },

        ValueType::PrimitiveDateTime => {
            quote! { entity.#field_ident = value.take_primitive_date_time() }
        }
        ValueType::OffsetDateTime => quote! { entity.#field_ident = value.take_offset_date_time() },
        ValueType::Duration => quote! { entity.#field_ident = value.take_duration() },
        ValueType::OptionalPrimitiveDateTime => {
            quote! { entity.#field_ident = value.take_optional_primitive_date_time() }
        }
        ValueType::OptionalOffsetDateTime => {
            quote! { entity.#field_ident = value.take_optional_offset_date_time() }
        }
        ValueType::OptionalDuration => {
            quote! { entity.#field_ident = value.take_optional_duration() }
        }

        ValueType::Other => {
            quote! { ::tracing::warn!("Setting a custom field is not allowed") }
        }
    };

    quote! {
        #field_enum_ident::#field_name_as_type_ident => #take_op
    }
}

#[proc_macro_derive(CkField, attributes(ck_field, ck_id))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let input: CkFieldInputReceiver = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return Error::write_errors(err).into(),
    };

    let typified_fields = input
        .fields()
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().expect("Expected named field!");
            let type_name = to_pascal_case(&name.to_string());
            Ident::new(type_name.as_str(), Span::call_site())
        })
        .collect::<Vec<Ident>>();

    let name = &input.ident;
    let field_name = Ident::new(format!("{name}Field").as_str(), name.span());

    let direct_field_accessors = input.fields().iter().map(|field| {
        let name = field.ident.as_ref().expect("Expected named field!");
        let type_name = to_pascal_case(&name.to_string());
        let type_ident = Ident::new(type_name.as_str(), Span::call_site());

        quote! { pub const #type_ident: #field_name = #field_name::#type_ident; }
    });

    let id_fields = input
        .fields()
        .iter()
        .filter(|field| field.is_id())
        .collect::<Vec<_>>();

    // TODO: Make this a separate derive macro?
    let id_impl = match id_fields.len() {
        // TODO: Create an error, as every aggregate needs an id?
        0 => quote! {},
        // Implement the `crudkit_web::CrudIdTrait` trait if there are id fields in the struct.
        _ => {
            let id_struct_ident = Ident::new(format!("{}Id", name).as_str(), name.span());

            let init_id_struct_fields = id_fields.iter().map(|field| {
                let ident = field.ident.as_ref().expect("Ident to be present").clone();
                // Example: id: self.id.clone()
                quote! { #ident: self.#ident.clone() } // TODO: Always clone here?
            });

            // Implements the main 'CrudIdTrait' for our base type. Allowing the user to access the ID of the entity.
            quote! {
                impl crudkit_web::CrudIdTrait for #name {
                    type Id = #id_struct_ident;

                    fn get_id(&self) -> Self::Id {
                        Self::Id {
                            #(#init_id_struct_fields),*
                        }
                    }
                }

                impl crudkit_web::model::Identifiable for #name {
                    fn get_id(&self) -> crudkit_id::SerializableId {
                        let id = crudkit_web::CrudIdTrait::get_id(&self);
                        crudkit_id::Id::to_serializable_id(&id)
                    }
                }
            }
        }
    };

    let match_field_name_to_str_arms = input.fields().iter().map(|field| {
        let name = field.ident.as_ref().expect("Expected named field!");
        let name = name.to_string();
        let type_name = to_pascal_case(&name);
        let type_ident = Ident::new(type_name.as_str(), Span::call_site());
        quote! {
            #field_name::#type_ident => #name
        }
    });
    let get_name_impl = match input.fields().len() {
        0 => quote! { "" },
        _ => quote! {
            match self {
                #(#match_field_name_to_str_arms),*
            }
        },
    };

    let all_field_enum_accessors = input.fields().iter().map(|field| {
        let name = field.ident.as_ref().expect("Expected named field!");
        let name = name.to_string();
        let type_name = to_pascal_case(&name);
        let type_ident = Ident::new(type_name.as_str(), Span::call_site());
        quote! {
            #field_name::#type_ident
        }
    });

    let get_field_arms = input.fields().iter().map(|field| {
        let name = field.ident.as_ref().expect("Expected named field!");
        let name = name.to_string();
        let type_name = to_pascal_case(&name);
        let type_ident = Ident::new(type_name.as_str(), Span::call_site());
        quote! {
            #name => #field_name::#type_ident
        }
    });
    let get_field_impl = match input.fields().len() {
        0 => {
            quote! { panic!("String '{}' can not be parsed as a field name! There are zero fields!", field_name) }
        }
        _ => quote! {
            match field_name {
                #(#get_field_arms),*,
                other => panic!("String '{}' can not be parsed as a field name!", other),
            }
        },
    };

    let model_type_based_model_trait_impl = match input.model {
        ModelType::Create => quote! {
            #[typetag::serde]
            impl crudkit_web::model::CreateModel for #name {
            }
        },
        ModelType::Read => quote! {
            #[typetag::serde]
            impl crudkit_web::model::ReadModel for #name {
            }
        },
        ModelType::Update => quote! {
            #[typetag::serde]
            impl crudkit_web::model::UpdateModel for #name {
            }
        },
    };

    let model_type_based_field_trait_impl = match input.model {
        ModelType::Create => quote! {
            #[typetag::serde]
            impl crudkit_web::model::CreateField for #field_name {
                fn set_value(&self, model: &mut crudkit_web::model::AnyCreateModel, value: crudkit_core::Value) {
                    let model = model.downcast_mut::<#name>();
                    crudkit_web::CrudFieldValueTrait::set_value(self, model, value);
                }
            }
        },
        ModelType::Read => quote! {
            #[typetag::serde]
            impl crudkit_web::model::ReadField for #field_name {
                fn set_value(&self, model: &mut crudkit_web::model::AnyReadModel, value: crudkit_core::Value) {
                    let model = model.downcast_mut::<#name>();
                    crudkit_web::CrudFieldValueTrait::set_value(self, model, value);
                }
            }
        },
        ModelType::Update => quote! {
            #[typetag::serde]
            impl crudkit_web::model::UpdateField for #field_name {
                fn set_value(&self, model: &mut crudkit_web::model::AnyUpdateModel, value: crudkit_core::Value) {
                    let model = model.downcast_mut::<#name>();
                    crudkit_web::CrudFieldValueTrait::set_value(self, model, value);
                }
            }
        },
    };

    // Generate CrudFieldValueTrait implementation
    let get_field_value_arms = input
        .fields()
        .iter()
        .map(|field| generate_get_value_arm(field, &field_name));
    let get_value_impl = match input.fields().len() {
        0 => {
            quote! { panic!("Cannot get value. Zero fields available! Should be unreachable. Source-crate: derive-field") }
        }
        _ => quote! {
            match self {
                #(#get_field_value_arms),*,
            }
        },
    };

    let set_field_value_arms = input
        .fields()
        .iter()
        .map(|field| generate_set_value_arm(field, &field_name));
    let set_value_impl = match input.fields().len() {
        0 => {
            quote! { panic!("Cannot set value. Zero fields available! Should be unreachable. Source-crate: derive-field") }
        }
        _ => quote! {
            match self {
                #(#set_field_value_arms),*,
            }
        },
    };

    let field_value_trait_impl = quote! {
        impl crudkit_web::CrudFieldValueTrait<#name> for #field_name {
            fn get_value(&self, entity: &#name) -> crudkit_core::Value {
                #get_value_impl
            }

            fn set_value(&self, entity: &mut #name, value: crudkit_core::Value) {
                #set_value_impl
            }
        }
    };

    quote! {
        impl #name {
            #(#direct_field_accessors)*
        }

        #model_type_based_model_trait_impl

        #[derive(PartialEq, Eq, Hash, Clone, Debug, serde::Serialize, serde::Deserialize)]
        pub enum #field_name {
            #(#typified_fields),*
        }

        impl crudkit_core::Named for #field_name {
            fn get_name(&self) -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#get_name_impl)
            }
        }

        #id_impl

        impl crudkit_web::CrudModel for #name {
            type Field = #field_name;

            fn get_all_fields() -> Vec<#field_name> {
                vec![ #(#all_field_enum_accessors),* ]
            }

            fn get_field(field_name: &str) -> #field_name {
                #get_field_impl
            }
        }

        #[typetag::serde]
        impl crudkit_web::model::Field for #field_name {
            fn set_value(&self, model: &mut crudkit_web::model::AnyModel, value: crudkit_core::Value) {
                let model = model.downcast_mut::<#name>();
                crudkit_web::CrudFieldValueTrait::set_value(self, model, value);
            }
        }

        #model_type_based_field_trait_impl

        impl crudkit_web::model::SerializeAsKey for #field_name {
            fn serialize_as_key(&self) -> String {
                serde_json::to_string(self).unwrap()
            }
        }

        #[typetag::serde]
        impl crudkit_web::model::Model for #name {}

        #field_value_trait_impl
    }
        .into()
}
