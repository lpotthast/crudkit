//! #[derive(CkFieldSignals)]
//! struct A {
//!     field: String,
//! }
//! let signals = A::to_signals()
//! let a = A::from_signals(signals)

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use darling::*;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use serde::Deserialize;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

// TODO: This should create a darling error instead of panicking... See https://github.com/TedDriggs/darling/issues/207
fn parse_type(string: Option<String>) -> Option<ReactiveValueType> {
    string.map(|ty| match serde_json::from_str(format!("\"{ty}\"").as_str()) {
        Ok(value_type) => value_type,
        Err(err) => panic!("crudkit: derive-field-signals: expected `field_value(type = ...)`, where '...' (actual: {ty}) is of a known variant. serde error: {err:?}"),
    })
}

#[derive(Debug, FromField)]
#[darling(attributes(ck_field_signals))]
struct MyFieldReceiver {
    ident: Option<syn::Ident>,

    ty: syn::Type,

    #[darling(rename = "ty")]
    #[darling(map = "parse_type")]
    reactive_value_type: Option<ReactiveValueType>,

    // TODO: Use this information to automatically fetch referenced data.
    references: Option<syn::Type>,

    /// Determines whether this field is part of the aggregate id.
    // Originates from: crudkit_id
    id: Option<bool>,
}

impl MyFieldReceiver {
    fn get_ident(&self) -> Option<&syn::Ident> {
        self.ident.as_ref()
    }

    fn get_type(&self) -> &syn::Type {
        &self.ty
    }

    pub fn reactive_value_type(&self) -> ReactiveValueType {
        self.reactive_value_type
            .unwrap_or_else(|| (&self.ty).into())
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ck_field_signals), supports(struct_any))]
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

#[proc_macro_derive(CkFieldSignals, attributes(ck_field_signals))]
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

    let name = &input.ident;
    let field_name = Ident::new(format!("{name}Field").as_str(), name.span());

    let all_fields_from_signals = input.fields().iter().map(|field| {
        let name = field.ident.as_ref().expect("Expected named field!");
        let name = name.to_string();
        let name_ident = Ident::new(name.as_str(), Span::call_site());
        let type_name = field_name_as_type_name(&name);
        let type_ident = Ident::new(type_name.as_str(), Span::call_site());
        //let expect_fn = Ident::new(format!("expect_{}", &field.ty.to_token_stream()).as_ref(), Span::call_site());

        // An expression that, given a `value`, constructs the necessary data type value to be assigned to the field.
        let expect_fn = Ident::new(format!("{}", match field.reactive_value_type() {
            ReactiveValueType::String => "expect_string",
            ReactiveValueType::OptionalString => "expect_optional_string",
            ReactiveValueType::I32 => "expect_i32",
            ReactiveValueType::I64 => "expect_i64",
            ReactiveValueType::OptionalI64 => "expect_optional_i64",
            ReactiveValueType::Bool => "expect_bool",
            ReactiveValueType::Select => "expect_select",
            ReactiveValueType::Custom => "expect_custom",
            other => abort!(field.ty.span(), "not implemented"),
        }).as_ref(), Span::call_site());

        let finalize_val = match field.reactive_value_type() {
            ReactiveValueType::Select => {
                let field_ty = field.get_type();
                quote! { val.as_any().downcast_ref::<#field_ty>().unwrap().clone() }
            }
            _ => quote! { val },
        };

        quote! {
            #name_ident: {
                let rw_sig = signals.get(&#field_name::#type_ident).expect("Fully saturated signals map").#expect_fn();
                let val = ::leptos::reactive::traits::Get::get(&rw_sig);
                let ret = #finalize_val;
                ret
            }
        }
    });

    let all_fields_from_signals_untracked = input.fields().iter().map(|field| {
        let name = field.ident.as_ref().expect("Expected named field!");
        let name = name.to_string();
        let name_ident = Ident::new(name.as_str(), Span::call_site());
        let type_name = field_name_as_type_name(&name);
        let type_ident = Ident::new(type_name.as_str(), Span::call_site());
        //let expect_fn = Ident::new(format!("expect_{}", &field.ty.to_token_stream()).as_ref(), Span::call_site());

        // An expression that, given a `value`, constructs the necessary data type value to be assigned to the field.
        let expect_fn = Ident::new(format!("{}", match field.reactive_value_type() {
            ReactiveValueType::String => "expect_string",
            ReactiveValueType::OptionalString => "expect_optional_string",
            ReactiveValueType::I32 => "expect_i32",
            ReactiveValueType::I64 => "expect_i64",
            ReactiveValueType::OptionalI64 => "expect_optional_i64",
            ReactiveValueType::Bool => "expect_bool",
            ReactiveValueType::Select => "expect_select",
            ReactiveValueType::Custom => "expect_custom",
            other => abort!(field.ty.span(), "not implemented"),
        }).as_ref(), Span::call_site());

        let finalize_val = match field.reactive_value_type() {
            ReactiveValueType::Select => {
                let field_ty = field.get_type();
                quote! { val.as_any().downcast_ref::<#field_ty>().unwrap().clone() }
            }
            _ => quote! { val },
        };

        quote! {
            #name_ident: {
                let rw_sig = signals.get(&#field_name::#type_ident).expect("Fully saturated signals map").#expect_fn();
                let val = ::leptos::reactive::traits::GetUntracked::get_untracked(&rw_sig);
                let ret = #finalize_val;
                ret
            }
        }
    });

    quote! {
        impl crudkit_leptos::SignalsTrait for #name {
            type Field = #field_name;

            //fn to_signals(&self) -> std::collections::HashMap<Self::Field, ReactiveValue> {
            //
            //}

            fn from_signals(signals: &std::collections::HashMap<Self::Field, crudkit_leptos::ReactiveValue>) -> Self {
                Self {
                    #(#all_fields_from_signals),*
                }
            }

            fn from_signals_untracked(signals: &std::collections::HashMap<Self::Field, crudkit_leptos::ReactiveValue>) -> Self {
                Self {
                    #(#all_fields_from_signals_untracked),*
                }
            }
        }
    }
        .into()
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, FromMeta, Deserialize)]
enum ReactiveValueType {
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
    U64,
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
    Reference,
    Custom,
}

/// Converts to the name of the `crudkit_web::Value` variant which should be used.
impl From<ReactiveValueType> for Ident {
    fn from(value_type: ReactiveValueType) -> Self {
        Ident::new(
            match value_type {
                ReactiveValueType::String => "String",
                ReactiveValueType::OptionalString => "String",
                ReactiveValueType::Text => "Text",
                ReactiveValueType::OptionalText => "Text",
                ReactiveValueType::Json => "Json",
                ReactiveValueType::OptionalJson => "OptionalJson",
                ReactiveValueType::UuidV4 => "UuidV4",
                ReactiveValueType::UuidV7 => "UuidV7",
                ReactiveValueType::Bool => "Bool",
                ReactiveValueType::ValidationStatus => "ValidationStatus",
                ReactiveValueType::I32 => "I32",
                ReactiveValueType::U64 => "U64",
                ReactiveValueType::I64 => "I64",
                ReactiveValueType::OptionalI64 => "OptionalI64",
                ReactiveValueType::U32 => "U32",
                ReactiveValueType::OptionalI32 => "OptionalI32",
                ReactiveValueType::OptionalU32 => "OptionalU32",
                ReactiveValueType::F32 => "F32",
                ReactiveValueType::F64 => "F64",
                ReactiveValueType::PrimitiveDateTime => "PrimitiveDateTime",
                ReactiveValueType::OffsetDateTime => "OffsetDateTime",
                ReactiveValueType::OptionalPrimitiveDateTime => "OptionalPrimitiveDateTime",
                ReactiveValueType::OptionalOffsetDateTime => "OptionalOffsetDateTime",
                ReactiveValueType::Select => "Select",
                ReactiveValueType::Multiselect => "Multiselect",
                ReactiveValueType::OptionalSelect => "OptionalSelect",
                ReactiveValueType::OptionalMultiselect => "OptionalMultiselect",
                ReactiveValueType::OneToOneRelation => "OneToOneRelation",
                ReactiveValueType::Reference => "Reference",
                ReactiveValueType::Custom => "Custom",
            },
            Span::call_site(),
        )
    }
}

impl From<&syn::Type> for ReactiveValueType {
    fn from(ty: &syn::Type) -> Self {
        match &ty {
            syn::Type::Tuple(syn::TypeTuple {
                paren_token: _,
                elems,
            }) => match elems.is_empty() {
                true => ReactiveValueType::Custom,
                false => {
                    let span = ty.span();
                    let message = format!(
                        "crudkit: derive-field-signals: Tuple type with elements is unsupported. Only the `()` tuple is allowed, representing a custom field."
                    );
                    abort!(span, message);
                }
            },
            syn::Type::Path(path) => match join_path(&path.path).as_str() {
                "()" => ReactiveValueType::Custom,
                "bool" => ReactiveValueType::Bool,
                "u32" => ReactiveValueType::U32,
                "i32" => ReactiveValueType::I32,
                "u64" => ReactiveValueType::U64,
                "i64" => ReactiveValueType::I64,
                "f32" => ReactiveValueType::F32,
                "OrderedFloat<f32>" => ReactiveValueType::F32,
                "ordered_float::OrderedFloat<f32>" => ReactiveValueType::F32,
                "String" => ReactiveValueType::String,
                "serde_json::Value" => ReactiveValueType::Json,
                "UuidV4" => ReactiveValueType::UuidV4,
                "UuidV7" => ReactiveValueType::UuidV7,
                "time::PrimitiveDateTime" => ReactiveValueType::PrimitiveDateTime,
                "time::OffsetDateTime" => ReactiveValueType::OffsetDateTime,
                "Option<i64>" => ReactiveValueType::OptionalI64,
                "Option<i32>" => ReactiveValueType::OptionalI32,
                "Option<u32>" => ReactiveValueType::OptionalU32,
                "Option<String>" => ReactiveValueType::OptionalString,
                "Option<serde_json::Value>" => ReactiveValueType::OptionalJson,
                "Option<time::PrimitiveDateTime>" => ReactiveValueType::OptionalPrimitiveDateTime,
                "Option<time::OffsetDateTime>" => ReactiveValueType::OptionalOffsetDateTime,
                other => {
                    let span = ty.span();
                    let message = format!("crudkit: derive-field-signals: Unknown type {other:?}. Expected a known type.");
                    abort!(
                        span, message;
                        help = "use one of the following types: [...]";
                    );
                }
            },
            other => {
                let span = ty.span();
                let message = format!(
                    "crudkit: derive-field-signals: Unknown type {other:?}. Not a 'Path' variant."
                );
                abort!(span, message);
            }
        }
    }
}

fn join_path(path: &syn::Path) -> String {
    path.to_token_stream().to_string().replace(' ', "")
}
