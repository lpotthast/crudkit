#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use darling::*;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use serde::Deserialize;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

// TODO: Merge FieldValue into Field, only use "field" attribute!

// TODO: This should create a darling error instead of panicking... See https://github.com/TedDriggs/darling/issues/207
fn parse_type(string: Option<String>) -> Option<ValueType> {
    string.map(|ty| match serde_json::from_str(format!("\"{ty}\"").as_str()) {
        Ok(value_type) => value_type,
        Err(err) => panic!("expected `field_value(type = ...)`, where '...' (actual: {ty}) is of a known variant. serde error: {err:?}"),
    })
}

#[derive(Debug, FromField)]
#[darling(attributes(ck_field_value))]
struct MyFieldReceiver {
    ident: Option<syn::Ident>,

    ty: syn::Type,

    #[darling(rename = "ty")]
    #[darling(map = "parse_type")]
    value_type: Option<ValueType>,
}

impl MyFieldReceiver {
    pub fn value_type(&self) -> ValueType {
        self.value_type.unwrap_or_else(|| (&self.ty).into())
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ck_field_value), supports(struct_any))]
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

#[proc_macro_derive(CkFieldValue, attributes(ck_field_value))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let input: MyInputReceiver = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return darling::Error::write_errors(err).into(),
    };

    fn capitalize_first_letter(s: &str) -> String {
        s[0..1].to_uppercase() + &s[1..]
    }

    fn field_name_as_type_name(name: &String) -> String {
        let mut type_name = String::new();
        for part in name.split("_") {
            type_name.push_str(capitalize_first_letter(part).as_str());
        }
        type_name
    }

    let ident = &input.ident;
    let field_enum_ident = Ident::new(format!("{ident}Field").as_str(), ident.span());

    // Self::Id => crudkit_core::Value::U32(entity.id),
    let get_field_value_arms = input.fields().iter().map(|field| {
        let field_ident = field.ident.as_ref().expect("Expected named field!");
        let field_name = field_ident.to_string();
        let field_name_as_type_name = field_name_as_type_name(&field_name);
        let field_name_as_type_ident =
            Ident::new(field_name_as_type_name.as_str(), Span::call_site());

        let value_type = field.value_type();
        let value_type_ident: Ident = value_type.clone().into(); // Uses: impl From<ValueType> for Ident!

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
            ValueType::OptionalJson => quote! { entity.#field_ident.clone().map(|it| crudkit_web::JsonValue::new(it)) },

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
    });
    let get_value_impl = match input.fields().len() {
        0 => {
            quote! { panic!("Cannot get value. Zero fields available! Should be unreachable. Source-crate: derive-field-value") }
        }
        _ => quote! {
            match self {
                #(#get_field_value_arms),*,
            }
        },
    };

    // Self::Id => entity.id = value.take_u32(),
    let set_field_value_arms = input.fields().iter().map(|field| {
        let field_ident = field.ident.as_ref().expect("Expected named field!");
        let field_name = field_ident.to_string();
        let field_name_as_type_name = field_name_as_type_name(&field_name);
        let field_name_as_type_ident =
            Ident::new(field_name_as_type_name.as_str(), Span::call_site());

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

            ValueType::PrimitiveDateTime => quote! { entity.#field_ident = value.take_primitive_date_time() },
            ValueType::OffsetDateTime => quote! { entity.#field_ident = value.take_offset_date_time() },
            ValueType::Duration => quote! { entity.#field_ident = value.take_duration() },
            ValueType::OptionalPrimitiveDateTime => quote! { entity.#field_ident = value.take_optional_primitive_date_time() },
            ValueType::OptionalOffsetDateTime => quote! { entity.#field_ident = value.take_optional_offset_date_time() },
            ValueType::OptionalDuration => quote! { entity.#field_ident = value.take_optional_duration() },

            ValueType::Other => {
                quote! { ::tracing::warn!("Setting a custom field is not allowed") }
            }
        };
        quote! {
            #field_enum_ident::#field_name_as_type_ident => #take_op
        }
    });
    let set_value_impl = match input.fields().len() {
        0 => {
            quote! { panic!("Cannot set value. Zero fields available! Should be unreachable. Source-crate: derive-field-value") }
        }
        _ => quote! {
            match self {
                #(#set_field_value_arms),*,
            }
        },
    };

    quote! {
        impl crudkit_web::CrudFieldValueTrait<#ident> for #field_enum_ident {
            fn get_value(&self, entity: &#ident) -> crudkit_core::Value {
                #get_value_impl
            }

            fn set_value(&self, entity: &mut #ident, value: crudkit_core::Value) {
                #set_value_impl
            }
        }
    }
    .into()
}

/// Describes the type of value without carrying any actual value.
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
                    let message = format!(
                        "crudkit: derive-field-value: Tuple type with elements is unsupported. Only the `()` tuple is allowed, representing a custom field."
                    );
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
                let message = format!(
                    "crudkit: derive-field-value: Unknown type {other:?}. Not a 'Path' variant."
                );
                abort!(span, message);
            }
        }
    }
}

fn join_path(path: &syn::Path) -> String {
    path.to_token_stream().to_string().replace(' ', "")
}
