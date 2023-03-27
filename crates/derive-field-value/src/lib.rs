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

    #[darling(rename = "type")]
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

    // Self::Id => crudkit_yew::Value::U32(entity.id),
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
            ValueType::String => quote! { entity.#field_ident.clone() },
            ValueType::Text => quote! { entity.#field_ident.clone() },
            ValueType::Json => quote! { crudkit_yew::JsonValue::new(entity.#field_ident.clone()) },
            ValueType::OptionalText => quote! { entity.#field_ident.clone().unwrap_or_default() },
            // We use .unwrap_or_default(), as we feed that string into Value::String (see From<ValueType>). We should get rid of this.
            ValueType::OptionalString => quote! { entity.#field_ident.clone().unwrap_or_default() },
            ValueType::OptionalJson => quote! { entity.#field_ident.clone().map(|it| crudkit_yew::JsonValue::new(it)) },
            ValueType::UuidV4 => quote! { entity.#field_ident },
            ValueType::UuidV7 => quote! { entity.#field_ident },
            ValueType::Bool => quote! { entity.#field_ident },
            ValueType::ValidationStatus => quote! { entity.#field_ident },
            ValueType::I32 => quote! { entity.#field_ident },
            ValueType::I64 => quote! { entity.#field_ident },
            ValueType::OptionalI64 => quote! { entity.#field_ident.clone() },
            ValueType::U32 => quote! { entity.#field_ident },
            ValueType::OptionalI32 => quote! { entity.#field_ident.clone() },
            ValueType::OptionalU32 => quote! { entity.#field_ident.clone() },
            ValueType::F32 => quote! { entity.#field_ident },
            ValueType::F64 => quote! { entity.#field_ident },
            ValueType::PrimitiveDateTime => quote! { entity.#field_ident.clone() },
            ValueType::OffsetDateTime => quote! { entity.#field_ident.clone() },
            ValueType::OptionalPrimitiveDateTime => quote! { entity.#field_ident.clone() },
            ValueType::OptionalOffsetDateTime => quote! { entity.#field_ident.clone() },
            ValueType::Select => quote! { entity.#field_ident.clone().into() },
            ValueType::Multiselect => quote! { entity.#field_ident.clone().into() },
            ValueType::OptionalSelect => quote! { entity.#field_ident.clone().map(Into::into) },
            ValueType::OptionalMultiselect => {
                quote! { entity.#field_ident.clone().map(|it| it.map(Into::into)) }
            }
            ValueType::OneToOneRelation => quote! { entity.#field_ident },
            ValueType::NestedTable => quote! {
                crudkit_id::Id::fields(&entity.get_id()).into_iter().map(|it| Box::new(it) as Box<dyn crudkit_id::IdField>).collect::<Vec<_>>()
            }, // not important, panics anyway...
            ValueType::Custom => quote! { () }, // not important, panics anyway...
        };

        quote! {
            #field_enum_ident::#field_name_as_type_ident => crudkit_yew::Value::#value_type_ident(#value_clone)
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
        let field_ty = &field.ty;
        let field_ident = field.ident.as_ref().expect("Expected named field!");
        let field_name = field_ident.to_string();
        let field_name_as_type_name = field_name_as_type_name(&field_name);
        let field_name_as_type_ident =
            Ident::new(field_name_as_type_name.as_str(), Span::call_site());

        // An expression that, given a `value`, constructs the necessary data type value to be assigned to the field.
        let take_op = match field.value_type() {
            ValueType::String => quote! { entity.#field_ident = value.take_string() },
            ValueType::Text => quote! { entity.#field_ident = value.take_string() },
            ValueType::Json => quote! { entity.#field_ident = value.take_inner_json_value() },
            ValueType::OptionalText => quote! { entity.#field_ident = std::option::Option::Some(value.take_string()) },
            // TODO: value should contain Option. do not force Some type...
            ValueType::OptionalString => quote! { entity.#field_ident = std::option::Option::Some(value.take_string()) },
            ValueType::OptionalJson => {
                quote! { entity.#field_ident = std::option::Option::Some(value.take_inner_json_value()) }
            }
            ValueType::UuidV4 => quote! { entity.#field_ident = value.to_uuid_v4() },
            ValueType::UuidV7 => quote! { entity.#field_ident = value.to_uuid_v7() },
            ValueType::Bool => quote! { entity.#field_ident = value.take_bool() },
            ValueType::ValidationStatus => quote! { entity.#field_ident = value.take_bool() },
            ValueType::I32 => quote! { entity.#field_ident = value.take_i32() },
            ValueType::I64 => quote! { entity.#field_ident = value.take_i64() },
            ValueType::OptionalI64 => quote! { entity.#field_ident = value.take_optional_i64() },
            ValueType::U32 => quote! { entity.#field_ident = value.take_u32() },
            ValueType::OptionalI32 => quote! { entity.#field_ident = value.take_optional_i32() },
            ValueType::OptionalU32 => quote! { entity.#field_ident = value.take_optional_u32() },
            ValueType::F32 => quote! { entity.#field_ident = value.take_f32() },
            ValueType::F64 => quote! { entity.#field_ident = value.take_f64() },
            ValueType::PrimitiveDateTime => quote! { entity.#field_ident = value.take_primitive_date_time() },
            ValueType::OffsetDateTime => quote! { entity.#field_ident = value.take_offset_date_time() },
            ValueType::OptionalPrimitiveDateTime => quote! { entity.#field_ident = value.take_optional_primitive_date_time() },
            ValueType::OptionalOffsetDateTime => quote! { entity.#field_ident = value.take_optional_offset_date_time() },
            ValueType::Select => quote! { entity.#field_ident = value.take_select_downcast_to::<#field_ty>().into() },
            ValueType::Multiselect => quote! { entity.#field_ident = value.take_multiselect_downcast_to().into() },
            ValueType::OptionalSelect => quote! { entity.#field_ident = value.take_optional_select_downcast_to().into() },
            ValueType::OptionalMultiselect => {
                quote! { entity.#field_ident = value.take_optional_multiselect_downcast_to().into() }
            }
            ValueType::OneToOneRelation => quote! { entity.#field_ident = value.take_one_to_one_relation() },
            ValueType::NestedTable => {
                quote! { tracing::warn!("Setting a nested table dummy field is not allowed") }
            }
            ValueType::Custom => {
                quote! { tracing::warn!("Setting a custom field is not allowed") }
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
        impl crudkit_yew::CrudFieldValueTrait<#ident> for #field_enum_ident {
            fn get_value(&self, entity: &#ident) -> crudkit_yew::Value {
                #get_value_impl
            }

            fn set_value(&self, entity: &mut #ident, value: crudkit_yew::Value) {
                #set_value_impl
            }
        }
    }
    .into()
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, FromMeta, Deserialize)]
enum ValueType {
    String,
    OptionalString,
    Text,
    OptionalText,
    Json,
    OptionalJson,
    UuidV4,
    UuidV7,
    Bool,
    ValidationStatus,
    I32,
    I64,
    OptionalI64,
    U32,
    OptionalI32,
    OptionalU32,
    F32,
    F64,
    PrimitiveDateTime,
    OffsetDateTime,
    OptionalPrimitiveDateTime,
    OptionalOffsetDateTime,
    Select,
    Multiselect,
    OptionalSelect,
    OptionalMultiselect,
    OneToOneRelation,
    NestedTable,
    Custom,
}

/// Converts to the name of the `crudkit_yew::Value` variant which should be used.
impl From<ValueType> for Ident {
    fn from(value_type: ValueType) -> Self {
        Ident::new(
            match value_type {
                ValueType::String => "String",
                ValueType::OptionalString => "String",
                ValueType::Text => "Text",
                ValueType::OptionalText => "Text",
                ValueType::Json => "Json",
                ValueType::OptionalJson => "OptionalJson",
                ValueType::UuidV4 => "UuidV4",
                ValueType::UuidV7 => "UuidV7",
                ValueType::Bool => "Bool",
                ValueType::ValidationStatus => "ValidationStatus",
                ValueType::I32 => "I32",
                ValueType::I64 => "I64",
                ValueType::OptionalI64 => "OptionalI64",
                ValueType::U32 => "U32",
                ValueType::OptionalI32 => "OptionalI32",
                ValueType::OptionalU32 => "OptionalU32",
                ValueType::F32 => "F32",
                ValueType::F64 => "F64",
                ValueType::PrimitiveDateTime => "PrimitiveDateTime",
                ValueType::OffsetDateTime => "OffsetDateTime",
                ValueType::OptionalPrimitiveDateTime => "OptionalPrimitiveDateTime",
                ValueType::OptionalOffsetDateTime => "OptionalOffsetDateTime",
                ValueType::Select => "Select",
                ValueType::Multiselect => "Multiselect",
                ValueType::OptionalSelect => "OptionalSelect",
                ValueType::OptionalMultiselect => "OptionalMultiselect",
                ValueType::OneToOneRelation => "OneToOneRelation",
                ValueType::NestedTable => "NestedTable",
                ValueType::Custom => "Custom",
            },
            Span::call_site(),
        )
    }
}

impl From<&syn::Type> for ValueType {
    fn from(ty: &syn::Type) -> Self {
        match &ty {
            syn::Type::Path(path) => match join_path(&path.path).as_str() {
                "bool" => ValueType::Bool,
                "u32" => ValueType::U32,
                "i32" => ValueType::I32,
                "i64" => ValueType::I64,
                "f32" => ValueType::F32,
                "String" => ValueType::String,
                "serde_json::Value" => ValueType::Json,
                "UuidV4" => ValueType::UuidV4,
                "UuidV7" => ValueType::UuidV7,
                "time::PrimitiveDateTime" => ValueType::PrimitiveDateTime,
                "time::OffsetDateTime" => ValueType::OffsetDateTime,
                "Option<i64>" => ValueType::OptionalI64,
                "Option<i32>" => ValueType::OptionalI32,
                "Option<u32>" => ValueType::OptionalU32,
                "Option<String>" => ValueType::OptionalString,
                "Option<serde_json::Value>" => ValueType::OptionalJson,
                "Option<time::PrimitiveDateTime>" => ValueType::OptionalPrimitiveDateTime,
                "Option<time::OffsetDateTime>" => ValueType::OptionalOffsetDateTime,
                other => {
                    let span = ty.span();
                    let message = format!("Unknown type {other:?}. Expected a known type.");
                    abort!(
                        span, message;
                        help = "use one of the following types: [...]";
                    );
                }
            },
            other => {
                let span = ty.span();
                let message = format!("Unknown type {other:?}. Not a 'Path' variant.");
                abort!(span, message);
            }
        }
    }
}

fn join_path(path: &syn::Path) -> String {
    path.to_token_stream().to_string().replace(' ', "")
}
