#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Shared utilities for crudkit derive macros.
//!
//! This crate provides common helper functions used across multiple proc-macro crates
//! in the crudkit ecosystem to reduce code duplication.
//!
//! # Value Kind Classification
//!
//! The [`ValueKind`] type (re-exported from `crudkit_core`) classifies Rust types
//! into their corresponding `crudkit_core::Value` variants. The [`ValueKindExt`]
//! extension trait adds macro-specific helper methods for code generation.

use proc_macro2::{Ident, Span};

// Re-export ValueKind from crudkit-core for use by macro crates.
pub use crudkit_core::ValueKind;

/// Capitalizes the first letter of a string.
///
/// # Examples
/// ```
/// use crudkit_core_macro_util::capitalize_first_letter;
/// assert_eq!(capitalize_first_letter("hello"), "Hello");
/// assert_eq!(capitalize_first_letter("world"), "World");
/// ```
#[must_use]
pub fn capitalize_first_letter(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    s[0..1].to_uppercase() + &s[1..]
}

/// Converts a `snake_case` field name to a `PascalCase` type name.
///
/// # Examples
/// ```
/// use crudkit_core_macro_util::to_pascal_case;
/// assert_eq!(to_pascal_case("user_id"), "UserId");
/// assert_eq!(to_pascal_case("first_name"), "FirstName");
/// assert_eq!(to_pascal_case("id"), "Id");
/// ```
#[must_use]
pub fn to_pascal_case(name: &str) -> String {
    name.split('_').map(capitalize_first_letter).collect()
}

/// Converts a `snake_case` field name to a `PascalCase` identifier.
///
/// This is a convenience function that combines `to_pascal_case` with `Ident::new`.
#[must_use]
pub fn field_name_to_pascal_ident(name: &str) -> Ident {
    Ident::new(&to_pascal_case(name), Span::call_site())
}

/// Converts a `syn::Path` to a normalized string representation.
///
/// Removes spaces that may appear in the path representation.
#[must_use]
pub fn path_to_string(path: &syn::Path) -> String {
    quote::quote!(#path).to_string().replace(' ', "")
}

/// Removes surrounding quotes from a string.
///
/// # Examples
/// ```
/// use crudkit_core_macro_util::strip_quotes;
/// assert_eq!(strip_quotes("\"hello\""), "hello");
/// assert_eq!(strip_quotes("hello"), "hello");
/// ```
#[must_use]
pub fn strip_quotes(s: &str) -> String {
    s.trim_matches('"').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_first_letter() {
        assert_eq!(capitalize_first_letter("hello"), "Hello");
        assert_eq!(capitalize_first_letter("World"), "World");
        assert_eq!(capitalize_first_letter(""), "");
        assert_eq!(capitalize_first_letter("a"), "A");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("user_id"), "UserId");
        assert_eq!(to_pascal_case("first_name"), "FirstName");
        assert_eq!(to_pascal_case("id"), "Id");
        assert_eq!(to_pascal_case("some_long_field_name"), "SomeLongFieldName");
    }

    #[test]
    fn test_strip_quotes() {
        assert_eq!(strip_quotes("\"hello\""), "hello");
        assert_eq!(strip_quotes("hello"), "hello");
        assert_eq!(strip_quotes("\"\""), "");
    }
}

// =============================================================================
// Value Kind Classification
// =============================================================================

/// Extension trait for `ValueKind` providing macro-specific helper methods.
///
/// These methods are only needed during code generation, not at runtime,
/// so they are defined here rather than in `crudkit_core`.
pub trait ValueKindExt {
    /// Returns the `Value` variant name (e.g., "I32" for `I32`).
    fn value_variant_name(&self) -> &'static str;

    /// Creates an `Ident` for the `Value` variant name.
    fn value_variant_ident(&self) -> Ident;

    /// Returns the `ConditionClauseValue::to_*` method name for converting the clause value
    /// to a `Value` of this `ValueKind`.
    ///
    /// Returns `None` for types without a conversion method (`Null`, `Void`, `Array`, `Other`).
    fn condition_method_name(&self) -> Option<&'static str>;

    /// Returns the `Value::as_*` method name for accessing a value of this kind.
    ///
    /// Returns `None` for types without an accessor method (`Null`, `Void`).
    fn accessor_method_name(&self) -> Option<&'static str>;
}

impl ValueKindExt for ValueKind {
    fn value_variant_name(&self) -> &'static str {
        match self {
            ValueKind::Null => "Null",
            ValueKind::Void => "Void",
            ValueKind::Bool => "Bool",
            ValueKind::U8 => "U8",
            ValueKind::U16 => "U16",
            ValueKind::U32 => "U32",
            ValueKind::U64 => "U64",
            ValueKind::U128 => "U128",
            ValueKind::I8 => "I8",
            ValueKind::I16 => "I16",
            ValueKind::I32 => "I32",
            ValueKind::I64 => "I64",
            ValueKind::I128 => "I128",
            ValueKind::F32 => "F32",
            ValueKind::F64 => "F64",
            ValueKind::String => "String",
            ValueKind::Json => "Json",
            ValueKind::Uuid => "Uuid",
            ValueKind::PrimitiveDateTime => "PrimitiveDateTime",
            ValueKind::OffsetDateTime => "OffsetDateTime",
            ValueKind::Duration => "Duration",
            ValueKind::Array => "Array",
            ValueKind::Other => "Other",
        }
    }

    fn value_variant_ident(&self) -> Ident {
        Ident::new(self.value_variant_name(), Span::call_site())
    }

    fn condition_method_name(&self) -> Option<&'static str> {
        match self {
            ValueKind::Bool => Some("to_bool"),
            ValueKind::U8 => Some("to_u8"),
            ValueKind::U16 => Some("to_u16"),
            ValueKind::U32 => Some("to_u32"),
            ValueKind::U64 => Some("to_u64"),
            ValueKind::U128 => Some("to_u128"),
            ValueKind::I8 => Some("to_i8"),
            ValueKind::I16 => Some("to_i16"),
            ValueKind::I32 => Some("to_i32"),
            ValueKind::I64 => Some("to_i64"),
            ValueKind::I128 => Some("to_i128"),
            ValueKind::F32 => Some("to_f32"),
            ValueKind::F64 => Some("to_f64"),
            ValueKind::String => Some("to_string"),
            ValueKind::Json => Some("to_json_value"),
            ValueKind::Uuid => Some("to_uuid"),
            ValueKind::PrimitiveDateTime => Some("to_primitive_date_time"),
            ValueKind::OffsetDateTime => Some("to_offset_date_time"),
            ValueKind::Duration => Some("to_time_duration"),
            ValueKind::Null | ValueKind::Void | ValueKind::Array | ValueKind::Other => None,
        }
    }

    fn accessor_method_name(&self) -> Option<&'static str> {
        match self {
            ValueKind::Bool => Some("as_bool"),
            ValueKind::U8 => Some("as_u8"),
            ValueKind::U16 => Some("as_u16"),
            ValueKind::U32 => Some("as_u32"),
            ValueKind::U64 => Some("as_u64"),
            ValueKind::U128 => Some("as_u128"),
            ValueKind::I8 => Some("as_i8"),
            ValueKind::I16 => Some("as_i16"),
            ValueKind::I32 => Some("as_i32"),
            ValueKind::I64 => Some("as_i64"),
            ValueKind::I128 => Some("as_i128"),
            ValueKind::F32 => Some("as_f32"),
            ValueKind::F64 => Some("as_f64"),
            ValueKind::String => Some("as_string"),
            ValueKind::Json => Some("as_json"),
            ValueKind::Uuid => Some("as_uuid"),
            ValueKind::PrimitiveDateTime => Some("as_primitive_date_time"),
            ValueKind::OffsetDateTime => Some("as_offset_date_time"),
            ValueKind::Duration => Some("as_duration"),
            ValueKind::Array => Some("as_array"),
            ValueKind::Other => Some("as_other"),
            ValueKind::Null | ValueKind::Void => None,
        }
    }
}

/// Checks if a `syn::Path` represents `Option<T>`.
///
/// Returns `true` if the last path segment is "Option".
/// This handles both `Option<T>` and `std::option::Option<T>`.
#[must_use]
pub fn is_option_path(path: &syn::Path) -> bool {
    path.segments
        .last()
        .is_some_and(|seg| seg.ident == "Option")
}

/// Extracts the inner type from `Option<T>` if the path represents an Option.
///
/// Returns `Some(&syn::Type)` containing the inner type if the path is `Option<T>`,
/// or `None` if it's not an Option type.
///
/// This handles both `Option<T>` and `std::option::Option<T>`.
#[must_use]
pub fn strip_option_path(path: &syn::Path) -> Option<&syn::Type> {
    let last_segment = path.segments.last()?;

    if last_segment.ident != "Option" {
        return None;
    }

    // Extract the generic argument from Option<T>.
    match &last_segment.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            // Option has exactly one type argument.
            if args.args.len() == 1 {
                if let syn::GenericArgument::Type(inner_ty) = args.args.first()? {
                    return Some(inner_ty);
                }
            }
            None
        }
        _ => None,
    }
}

/// Classifies a normalized type path string into a `ValueKind`.
///
/// Expects the inner type for `Option<T>` (caller strips `Option` wrapper using [`strip_option_path`]).
/// Returns `ValueKind::Other` for unrecognized types.
///
/// # Type Recognition
///
/// The following types are recognized:
/// - Primitives: `bool`, `u8`..`u128`, `i8`..`i128`, `f32`, `f64`, `String`
/// - Time types: `time::PrimitiveDateTime`, `time::OffsetDateTime`, `TimeDuration`
/// - UUID: `uuid::Uuid`, `Uuid`
/// - JSON: `serde_json::Value`
/// - Ordered floats: `OrderedFloat<f32>`, `OrderedFloat<f64>` (mapped to `F32`, `F64`)
///
/// # Examples
/// ```
/// use crudkit_core_macro_util::{classify_base_type, ValueKind};
/// assert_eq!(classify_base_type("i32"), ValueKind::I32);
/// assert_eq!(classify_base_type("time::PrimitiveDateTime"), ValueKind::PrimitiveDateTime);
/// assert_eq!(classify_base_type("CustomType"), ValueKind::Other);
/// ```
#[must_use]
pub fn classify_base_type(path_str: &str) -> ValueKind {
    match path_str {
        "()" => ValueKind::Void,

        "bool" => ValueKind::Bool,

        "u8" => ValueKind::U8,
        "u16" => ValueKind::U16,
        "u32" => ValueKind::U32,
        "u64" => ValueKind::U64,
        "u128" => ValueKind::U128,

        "i8" => ValueKind::I8,
        "i16" => ValueKind::I16,
        "i32" => ValueKind::I32,
        "i64" => ValueKind::I64,
        "i128" => ValueKind::I128,

        "f32" | "OrderedFloat<f32>" | "ordered_float::OrderedFloat<f32>" => ValueKind::F32,
        "f64" | "OrderedFloat<f64>" | "ordered_float::OrderedFloat<f64>" => ValueKind::F64,

        "String" => ValueKind::String,

        "serde_json::Value" => ValueKind::Json,

        "Uuid" | "uuid::Uuid" => ValueKind::Uuid,

        "time::PrimitiveDateTime" => ValueKind::PrimitiveDateTime,
        "time::OffsetDateTime" => ValueKind::OffsetDateTime,
        "TimeDuration" | "crudkit_sea_orm::newtypes::TimeDuration" => ValueKind::Duration,

        _ => ValueKind::Other,
    }
}

/// Checks if a type path represents an `OrderedFloat` wrapper.
///
/// Returns `true` for `OrderedFloat<f32>`, `OrderedFloat<f64>`, and their fully qualified forms.
#[must_use]
pub fn is_ordered_float(path_str: &str) -> bool {
    matches!(
        path_str,
        "OrderedFloat<f32>"
            | "ordered_float::OrderedFloat<f32>"
            | "OrderedFloat<f64>"
            | "ordered_float::OrderedFloat<f64>"
    )
}

#[cfg(test)]
mod value_kind_tests {
    use super::*;

    #[test]
    fn test_classify_base_type_primitives() {
        assert_eq!(classify_base_type("bool"), ValueKind::Bool);
        assert_eq!(classify_base_type("u8"), ValueKind::U8);
        assert_eq!(classify_base_type("u16"), ValueKind::U16);
        assert_eq!(classify_base_type("u32"), ValueKind::U32);
        assert_eq!(classify_base_type("u64"), ValueKind::U64);
        assert_eq!(classify_base_type("u128"), ValueKind::U128);
        assert_eq!(classify_base_type("i8"), ValueKind::I8);
        assert_eq!(classify_base_type("i16"), ValueKind::I16);
        assert_eq!(classify_base_type("i32"), ValueKind::I32);
        assert_eq!(classify_base_type("i64"), ValueKind::I64);
        assert_eq!(classify_base_type("i128"), ValueKind::I128);
        assert_eq!(classify_base_type("f32"), ValueKind::F32);
        assert_eq!(classify_base_type("f64"), ValueKind::F64);
        assert_eq!(classify_base_type("String"), ValueKind::String);
    }

    #[test]
    fn test_classify_base_type_special() {
        assert_eq!(classify_base_type("()"), ValueKind::Void);
        assert_eq!(classify_base_type("serde_json::Value"), ValueKind::Json);
        assert_eq!(classify_base_type("Uuid"), ValueKind::Uuid);
        assert_eq!(classify_base_type("uuid::Uuid"), ValueKind::Uuid);
        assert_eq!(
            classify_base_type("time::PrimitiveDateTime"),
            ValueKind::PrimitiveDateTime
        );
        assert_eq!(
            classify_base_type("time::OffsetDateTime"),
            ValueKind::OffsetDateTime
        );
        assert_eq!(classify_base_type("TimeDuration"), ValueKind::Duration);
    }

    #[test]
    fn test_classify_base_type_ordered_float() {
        assert_eq!(classify_base_type("OrderedFloat<f32>"), ValueKind::F32);
        assert_eq!(
            classify_base_type("ordered_float::OrderedFloat<f32>"),
            ValueKind::F32
        );
        assert_eq!(classify_base_type("OrderedFloat<f64>"), ValueKind::F64);
        assert_eq!(
            classify_base_type("ordered_float::OrderedFloat<f64>"),
            ValueKind::F64
        );
    }

    #[test]
    fn test_classify_base_type_unknown() {
        assert_eq!(classify_base_type("CustomType"), ValueKind::Other);
        assert_eq!(classify_base_type("my_crate::MyType"), ValueKind::Other);
        // Vec types are now classified as Other (use Array variant instead).
        assert_eq!(classify_base_type("Vec<u8>"), ValueKind::Other);
        assert_eq!(classify_base_type("Vec<i32>"), ValueKind::Other);
    }

    #[test]
    fn test_value_variant_name() {
        assert_eq!(ValueKind::I32.value_variant_name(), "I32");
        assert_eq!(ValueKind::String.value_variant_name(), "String");
        assert_eq!(ValueKind::Void.value_variant_name(), "Void");
        assert_eq!(ValueKind::Null.value_variant_name(), "Null");
        assert_eq!(ValueKind::Array.value_variant_name(), "Array");
    }

    #[test]
    fn test_condition_method_name() {
        assert_eq!(ValueKind::Bool.condition_method_name(), Some("to_bool"));
        assert_eq!(ValueKind::I32.condition_method_name(), Some("to_i32"));
        assert_eq!(ValueKind::String.condition_method_name(), Some("to_string"));
        assert_eq!(
            ValueKind::Json.condition_method_name(),
            Some("to_json_value")
        );
        assert_eq!(ValueKind::Void.condition_method_name(), None);
        assert_eq!(ValueKind::Null.condition_method_name(), None);
        assert_eq!(ValueKind::Array.condition_method_name(), None);
        assert_eq!(ValueKind::Other.condition_method_name(), None);
    }

    #[test]
    fn test_accessor_method_name() {
        assert_eq!(ValueKind::Bool.accessor_method_name(), Some("as_bool"));
        assert_eq!(ValueKind::I32.accessor_method_name(), Some("as_i32"));
        assert_eq!(ValueKind::String.accessor_method_name(), Some("as_string"));
        assert_eq!(ValueKind::Json.accessor_method_name(), Some("as_json"));
        assert_eq!(ValueKind::Uuid.accessor_method_name(), Some("as_uuid"));
        assert_eq!(
            ValueKind::PrimitiveDateTime.accessor_method_name(),
            Some("as_primitive_date_time")
        );
        assert_eq!(ValueKind::Array.accessor_method_name(), Some("as_array"));
        assert_eq!(ValueKind::Other.accessor_method_name(), Some("as_other"));

        // Special types return None.
        assert_eq!(ValueKind::Void.accessor_method_name(), None);
        assert_eq!(ValueKind::Null.accessor_method_name(), None);
    }

    #[test]
    fn test_is_ordered_float() {
        assert!(is_ordered_float("OrderedFloat<f32>"));
        assert!(is_ordered_float("ordered_float::OrderedFloat<f32>"));
        assert!(is_ordered_float("OrderedFloat<f64>"));
        assert!(is_ordered_float("ordered_float::OrderedFloat<f64>"));
        assert!(!is_ordered_float("f32"));
        assert!(!is_ordered_float("f64"));
        assert!(!is_ordered_float("i32"));
    }
}
