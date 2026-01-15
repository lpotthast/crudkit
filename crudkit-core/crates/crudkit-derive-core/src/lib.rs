#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Shared utilities for crudkit derive macros.
//!
//! This crate provides common helper functions used across multiple proc-macro crates
//! in the crudkit ecosystem to reduce code duplication.

use proc_macro2::{Ident, Span};

/// Capitalizes the first letter of a string.
///
/// # Examples
/// ```
/// use crudkit_derive_core::capitalize_first_letter;
/// assert_eq!(capitalize_first_letter("hello"), "Hello");
/// assert_eq!(capitalize_first_letter("world"), "World");
/// ```
pub fn capitalize_first_letter(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    s[0..1].to_uppercase() + &s[1..]
}

/// Converts a snake_case field name to a PascalCase type name.
///
/// # Examples
/// ```
/// use crudkit_derive_core::to_pascal_case;
/// assert_eq!(to_pascal_case("user_id"), "UserId");
/// assert_eq!(to_pascal_case("first_name"), "FirstName");
/// assert_eq!(to_pascal_case("id"), "Id");
/// ```
pub fn to_pascal_case(name: &str) -> String {
    name.split('_').map(capitalize_first_letter).collect()
}

/// Converts a snake_case field name to a PascalCase identifier.
///
/// This is a convenience function that combines `to_pascal_case` with `Ident::new`.
pub fn field_name_to_pascal_ident(name: &str) -> Ident {
    Ident::new(&to_pascal_case(name), Span::call_site())
}

/// Converts a `syn::Path` to a normalized string representation.
///
/// Removes spaces that may appear in the path representation.
pub fn path_to_string(path: &syn::Path) -> String {
    quote::quote!(#path).to_string().replace(' ', "")
}

/// Removes surrounding quotes from a string.
///
/// # Examples
/// ```
/// use crudkit_derive_core::strip_quotes;
/// assert_eq!(strip_quotes("\"hello\""), "hello");
/// assert_eq!(strip_quotes("hello"), "hello");
/// ```
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
