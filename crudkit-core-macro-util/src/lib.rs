#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Shared utilities for crudkit derive macros.
//!
//! This crate provides common helper functions used across multiple proc-macro crates
//! in the crudkit ecosystem to reduce code duplication.
//!
//! # Value Kind Classification
//!
//! The [`ValueKind`] enum provides a shared abstraction for classifying Rust types
//! into their corresponding `crudkit_core::Value` variants. This is used by both
//! frontend (`crudkit-web-macros`) and backend (`crudkit-rs-macros`) derive macros.

use proc_macro2::{Ident, Span};

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

/// Represents the base variant families of `crudkit_core::Value`.
///
/// Each variant corresponds to a non-optional `Value` variant.
/// Optionality is tracked separately by callers using [`is_option_path`].
///
/// # Usage
///
/// Use [`classify_base_type`] to convert a type path string into a `ValueKind`.
/// Then use the various methods to get variant names and method names for code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Void,
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    String,
    Json,
    Uuid,
    PrimitiveDateTime,
    OffsetDateTime,
    Duration,
    // Vec types (no optional variants in Value).
    U8Vec,
    I32Vec,
    I64Vec,
    // Fallback (no optional variant in Value).
    Other,
}

impl ValueKind {
    /// Returns the `Value` variant name (e.g., "I32" for `I32`).
    #[must_use]
    pub fn value_variant_name(&self) -> &'static str {
        match self {
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
            ValueKind::U8Vec => "U8Vec",
            ValueKind::I32Vec => "I32Vec",
            ValueKind::I64Vec => "I64Vec",
            ValueKind::Other => "Other",
        }
    }

    /// Returns the optional variant name (e.g., `OptionalI32` for `I32`).
    ///
    /// Returns `None` for kinds without optional variants: `Void`, `U8Vec`, `I32Vec`, `I64Vec`, `Other`.
    #[must_use]
    pub fn optional_variant_name(&self) -> Option<&'static str> {
        match self {
            ValueKind::Bool => Some("OptionalBool"),
            ValueKind::U8 => Some("OptionalU8"),
            ValueKind::U16 => Some("OptionalU16"),
            ValueKind::U32 => Some("OptionalU32"),
            ValueKind::U64 => Some("OptionalU64"),
            ValueKind::U128 => Some("OptionalU128"),
            ValueKind::I8 => Some("OptionalI8"),
            ValueKind::I16 => Some("OptionalI16"),
            ValueKind::I32 => Some("OptionalI32"),
            ValueKind::I64 => Some("OptionalI64"),
            ValueKind::I128 => Some("OptionalI128"),
            ValueKind::F32 => Some("OptionalF32"),
            ValueKind::F64 => Some("OptionalF64"),
            ValueKind::String => Some("OptionalString"),
            ValueKind::Json => Some("OptionalJson"),
            ValueKind::Uuid => Some("OptionalUuid"),
            ValueKind::PrimitiveDateTime => Some("OptionalPrimitiveDateTime"),
            ValueKind::OffsetDateTime => Some("OptionalOffsetDateTime"),
            ValueKind::Duration => Some("OptionalDuration"),
            ValueKind::Void
            | ValueKind::U8Vec
            | ValueKind::I32Vec
            | ValueKind::I64Vec
            | ValueKind::Other => None,
        }
    }

    /// Whether `crudkit_core::Value` has an `Optional*` variant for this kind.
    ///
    /// Returns `false` for: `Void`, `U8Vec`, `I32Vec`, `I64Vec`, `Other`.
    #[must_use]
    pub fn has_optional_variant(&self) -> bool {
        self.optional_variant_name().is_some()
    }

    /// Returns the method name on `ConditionClauseValue` for this kind.
    ///
    /// Returns `None` for types without a conversion method (`Void`, `Other`, `I32Vec`, `I64Vec`).
    #[must_use]
    pub fn condition_method_name(&self) -> Option<&'static str> {
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
            ValueKind::U8Vec => Some("to_byte_vec"),
            ValueKind::Void | ValueKind::I32Vec | ValueKind::I64Vec | ValueKind::Other => None,
        }
    }

    /// Returns the `Value::take_*` method name for extracting a value of this kind.
    ///
    /// The `is_optional` parameter determines whether to return the optional variant
    /// (e.g., `take_optional_i32` vs `take_i32`).
    ///
    /// Returns `None` for types without a take method (`Void`, `Json`, `Uuid`, `Other`).
    /// These types require special handling in code generation.
    #[must_use]
    pub fn take_method_name(&self, is_optional: bool) -> Option<&'static str> {
        match (self, is_optional) {
            (ValueKind::Bool, false) => Some("take_bool"),
            (ValueKind::Bool, true) => Some("take_optional_bool"),

            (ValueKind::U8, false) => Some("take_u8"),
            (ValueKind::U8, true) => Some("take_optional_u8"),
            (ValueKind::U16, false) => Some("take_u16"),
            (ValueKind::U16, true) => Some("take_optional_u16"),
            (ValueKind::U32, false) => Some("take_u32"),
            (ValueKind::U32, true) => Some("take_optional_u32"),
            (ValueKind::U64, false) => Some("take_u64"),
            (ValueKind::U64, true) => Some("take_optional_u64"),
            (ValueKind::U128, false) => Some("take_u128"),
            (ValueKind::U128, true) => Some("take_optional_u128"),

            (ValueKind::I8, false) => Some("take_i8"),
            (ValueKind::I8, true) => Some("take_optional_i8"),
            (ValueKind::I16, false) => Some("take_i16"),
            (ValueKind::I16, true) => Some("take_optional_i16"),
            (ValueKind::I32, false) => Some("take_i32"),
            (ValueKind::I32, true) => Some("take_optional_i32"),
            (ValueKind::I64, false) => Some("take_i64"),
            (ValueKind::I64, true) => Some("take_optional_i64"),
            (ValueKind::I128, false) => Some("take_i128"),
            (ValueKind::I128, true) => Some("take_optional_i128"),

            (ValueKind::F32, false) => Some("take_f32"),
            (ValueKind::F32, true) => Some("take_optional_f32"),
            (ValueKind::F64, false) => Some("take_f64"),
            (ValueKind::F64, true) => Some("take_optional_f64"),

            (ValueKind::String, false) => Some("take_string"),
            (ValueKind::String, true) => Some("take_optional_string"),

            (ValueKind::PrimitiveDateTime, false) => Some("take_primitive_date_time"),
            (ValueKind::PrimitiveDateTime, true) => Some("take_optional_primitive_date_time"),
            (ValueKind::OffsetDateTime, false) => Some("take_offset_date_time"),
            (ValueKind::OffsetDateTime, true) => Some("take_optional_offset_date_time"),
            (ValueKind::Duration, false) => Some("take_duration"),
            (ValueKind::Duration, true) => Some("take_optional_duration"),

            // Vec types don't have optional variants.
            (ValueKind::U8Vec, _) => Some("take_u8_vec"),
            (ValueKind::I32Vec, _) => Some("take_i32_vec"),
            (ValueKind::I64Vec, _) => Some("take_i64_vec"),

            // These require special handling - return None.
            (ValueKind::Void | ValueKind::Json | ValueKind::Uuid | ValueKind::Other, _) => None,
        }
    }

    /// Creates an `Ident` for the `Value` variant name.
    #[must_use]
    pub fn value_variant_ident(&self) -> Ident {
        Ident::new(self.value_variant_name(), Span::call_site())
    }

    /// Creates an `Ident` for the optional `Value` variant name.
    ///
    /// # Panics
    /// Panics if this kind has no optional variant.
    #[must_use]
    pub fn optional_variant_ident(&self) -> Ident {
        Ident::new(
            self.optional_variant_name()
                .expect("ValueKind has no optional variant"),
            Span::call_site(),
        )
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
/// - Vec types: `Vec<u8>`, `Vec<i32>`, `Vec<i64>`
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

        "Vec<u8>" => ValueKind::U8Vec,
        "Vec<i32>" => ValueKind::I32Vec,
        "Vec<i64>" => ValueKind::I64Vec,

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
    fn test_classify_base_type_vec() {
        assert_eq!(classify_base_type("Vec<u8>"), ValueKind::U8Vec);
        assert_eq!(classify_base_type("Vec<i32>"), ValueKind::I32Vec);
        assert_eq!(classify_base_type("Vec<i64>"), ValueKind::I64Vec);
    }

    #[test]
    fn test_classify_base_type_unknown() {
        assert_eq!(classify_base_type("CustomType"), ValueKind::Other);
        assert_eq!(classify_base_type("my_crate::MyType"), ValueKind::Other);
    }

    #[test]
    fn test_value_variant_name() {
        assert_eq!(ValueKind::I32.value_variant_name(), "I32");
        assert_eq!(ValueKind::String.value_variant_name(), "String");
        assert_eq!(ValueKind::Void.value_variant_name(), "Void");
    }

    #[test]
    fn test_optional_variant_name() {
        assert_eq!(ValueKind::I32.optional_variant_name(), Some("OptionalI32"));
        assert_eq!(
            ValueKind::String.optional_variant_name(),
            Some("OptionalString")
        );
        assert_eq!(ValueKind::Void.optional_variant_name(), None);
        assert_eq!(ValueKind::U8Vec.optional_variant_name(), None);
        assert_eq!(ValueKind::Other.optional_variant_name(), None);
    }

    #[test]
    fn test_has_optional_variant() {
        assert!(ValueKind::I32.has_optional_variant());
        assert!(ValueKind::String.has_optional_variant());
        assert!(!ValueKind::Void.has_optional_variant());
        assert!(!ValueKind::U8Vec.has_optional_variant());
        assert!(!ValueKind::Other.has_optional_variant());
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
        assert_eq!(
            ValueKind::U8Vec.condition_method_name(),
            Some("to_byte_vec")
        );
        assert_eq!(ValueKind::Void.condition_method_name(), None);
        assert_eq!(ValueKind::Other.condition_method_name(), None);
        assert_eq!(ValueKind::I32Vec.condition_method_name(), None);
    }

    #[test]
    fn test_take_method_name() {
        // Non-optional
        assert_eq!(ValueKind::Bool.take_method_name(false), Some("take_bool"));
        assert_eq!(ValueKind::I32.take_method_name(false), Some("take_i32"));
        assert_eq!(
            ValueKind::String.take_method_name(false),
            Some("take_string")
        );
        assert_eq!(
            ValueKind::PrimitiveDateTime.take_method_name(false),
            Some("take_primitive_date_time")
        );

        // Optional
        assert_eq!(
            ValueKind::Bool.take_method_name(true),
            Some("take_optional_bool")
        );
        assert_eq!(
            ValueKind::I32.take_method_name(true),
            Some("take_optional_i32")
        );
        assert_eq!(
            ValueKind::String.take_method_name(true),
            Some("take_optional_string")
        );

        // Vec types (no optional variants)
        assert_eq!(
            ValueKind::U8Vec.take_method_name(false),
            Some("take_u8_vec")
        );
        assert_eq!(ValueKind::U8Vec.take_method_name(true), Some("take_u8_vec"));

        // Special types return None
        assert_eq!(ValueKind::Void.take_method_name(false), None);
        assert_eq!(ValueKind::Json.take_method_name(false), None);
        assert_eq!(ValueKind::Uuid.take_method_name(false), None);
        assert_eq!(ValueKind::Other.take_method_name(false), None);
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
