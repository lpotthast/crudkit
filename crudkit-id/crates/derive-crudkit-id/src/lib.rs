#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use darling::*;
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use proc_macro_type_name::ToTypeName;
use proc_macro2::Span;
use quote::quote;
use syn::{DeriveInput, Ident, parse_macro_input, spanned::Spanned};

#[derive(Debug, FromField)]
#[darling(attributes(ck_id))]
struct CkIdFieldConfig {
    ident: Option<Ident>,

    ty: syn::Type,

    /// Whether this field is part of the entities primary key.
    ///
    /// This can be set by specifying `#[ck_id(id)]` on a field. Only required for fields not
    /// named `id`.
    id: Option<bool>,
}

impl CkIdFieldConfig {
    /// Returns the field identifier.
    ///
    /// # Panics
    /// When called on an unnamed fields.
    pub fn get_ident(&self) -> &syn::Ident {
        self.ident
            .as_ref()
            .expect("Field ident missing - tuple structs are not supported")
    }

    /// Returns the field's type.
    pub fn get_type(&self) -> &syn::Type {
        &self.ty
    }

    /// Checks if this field is part of the entity's set of ID fields.
    ///
    /// A field is an ID field if:
    /// - It has the `#[ck_id(id = true)]` annotation, OR
    /// - It's named "id" (and not explicitly marked `#[ck_id(id = false)]`)
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
#[darling(attributes(ck_id), supports(struct_any))]
struct CkIdInputConfig {
    ident: Ident,

    data: ast::Data<(), CkIdFieldConfig>,
}

impl CkIdInputConfig {
    pub fn fields(&self) -> &ast::Fields<CkIdFieldConfig> {
        match &self.data {
            ast::Data::Enum(_) => {
                unreachable!("darling #[supports(struct_any)] should prevent enums")
            }
            ast::Data::Struct(fields) => fields,
        }
    }
}

/// Metadata for a single ID field, used during code generation.
///
/// This struct holds all the TokenStreams and identifiers needed to generate
/// the code for one field in both the ID struct and the ID field enum.
struct IdFieldMetadata {
    /// Original field identifier (e.g., `user_id`)
    ident: Ident,

    /// Field name as string (e.g., "user_id")
    name: String,

    /// Field name in pascal case (e.g. `UserId`). Usable as a type (or enum variant) name.
    type_name: Ident,

    /// The original field type (e.g., `i32`).
    ty: syn::Type,
}

impl IdFieldMetadata {
    fn from(field: &CkIdFieldConfig) -> Self {
        let ident = field.get_ident().clone();
        let name = ident.to_string();
        let type_name = (&ident).to_type_ident(ident.span());
        let ty = field.get_type();

        IdFieldMetadata {
            ident,
            name,
            type_name,
            ty: ty.clone(),
        }
    }
}

/// Derives ID-related types for the annotated struct.
///
/// A field is an ID field if:
/// - It is named `"id"`, OR
/// - It is annotated with `#[ck_id(id)]`
///
/// At least one ID field must exist, or compilation will fail.
///
/// # Generated Types
///
/// This macro generates two types from a struct `Foo`:
///
/// 1. **`FooId` struct**: Contains only the ID fields of the original struct.
/// 2. **`FooIdField` enum**: One variant per ID field. Each variant carries the fields value.
///
/// # Example
///
/// ```rust,ignore
/// use derive_crudkit_id::CkId;
///
/// #[derive(CkId)]
/// struct User {
///     #[ck_id(id)]
///     user_id: i32,
///     #[ck_id(id)]
///     org_id: i32,
///
///     name: String,
///     email: String,
/// }
///
/// // Generated:
/// //
/// // struct UserId {
/// //     pub user_id: i32,
/// //     pub org_id: i32,
/// // }
/// //
/// // impl Display for UserId { ... }
/// // impl crudkit_id::Id for UserId { ... }
/// //
/// // enum UserIdField {
/// //     UserId(i32),
/// //     OrgId(i32),
/// // }
/// //
/// // impl Display for UserIdField { ... }
/// // impl crudkit_id::IdField for UserIdField { ... }
/// ```
///
/// # Supported ID Field Types
///
/// - Integers: `i32`, `u32`, `i64`, `u64`, `i128`, `u128`
/// - Strings: `String`
/// - Booleans: `bool`
/// - UUIDs: `uuid::Uuid`
/// - Time: `time::PrimitiveDateTime`, `time::OffsetDateTime`
///
/// Note:
/// - Floating point types (`f32`, `f64`) are not supported (not `Eq` comparable).
/// - Optional types (`Option<T>`) are not supported for ID fields.
/// - Use exact type paths as shown above.
#[proc_macro_derive(CkId, attributes(ck_id))]
#[proc_macro_error]
pub fn derive_ck_id(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let input: CkIdInputConfig = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return Error::write_errors(err).into(),
    };

    let id_fields = input
        .fields()
        .iter()
        .filter(|field| field.is_id())
        .collect::<Vec<_>>();

    if id_fields.is_empty() {
        abort!(
            Span::call_site(),
            "To derive CkId, at least one id field must exist.";
            help = "A field is an id field if it is (a) named \"id\" or (b) annotated with `#[ck_id(id)]`, both marking the field as part of the entities id. Specify id fields or remove the derive, if no id fields can be defined for this entity.";
        );
    }

    let source_struct_name = &input.ident;
    let id_struct_ident = Ident::new(&format!("{}Id", source_struct_name), Span::call_site());
    let id_field_enum_ident =
        Ident::new(&format!("{}IdField", source_struct_name), Span::call_site());

    let field_metadata = id_fields
        .into_iter()
        .map(IdFieldMetadata::from)
        .collect::<Vec<_>>();

    let id_struct = generate_id_struct(&id_struct_ident, &id_field_enum_ident, &field_metadata);
    let id_field_enum = generate_id_field_enum(&id_field_enum_ident, &field_metadata);

    quote! {
        #id_struct
        #id_field_enum
    }
    .into()
}

/// Generates the `*Id` struct with its `Display` and `crudkit_id::Id` implementations.
///
/// The struct contains only the ID fields of the original struct.
fn generate_id_struct(
    id_struct_ident: &Ident,
    id_field_enum_ident: &Ident,
    field_metadata: &[IdFieldMetadata],
) -> proc_macro2::TokenStream {
    // Struct field definitions (e.g., `pub user_id: i32,`).
    let struct_fields = field_metadata
        .iter()
        .map(|it| {
            let ident = &it.ident;
            let ty = &it.ty;
            quote! { pub #ident: #ty }
        })
        .collect::<Vec<_>>();

    // Expressions to create enum variant from `self`
    // (e.g., `FooIdField::UserId(self.user_id.clone())`).
    // Note: Clone is required, as `fields()` returns Vec<Field> with owned data.
    let create_enum_variants = field_metadata
        .iter()
        .map(|it| {
            let type_name = &it.type_name;
            let ident = &it.ident;
            quote! { #id_field_enum_ident::#type_name(self.#ident.clone()) }
        })
        .collect::<Vec<_>>();

    let struct_display_format_str = format!(
        "({})",
        field_metadata
            .iter()
            .map(|it| format!("{}: {{}}", it.name))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let struct_display_format_args = field_metadata
        .iter()
        .map(|it| {
            let ident = &it.ident;
            quote! { self.#ident }
        })
        .collect::<Vec<_>>();

    let struct_display_write_call = quote! {
        f.write_fmt(format_args!(#struct_display_format_str, #(#struct_display_format_args),*))
    };

    quote! {
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, serde::Serialize, serde::Deserialize)]
        pub struct #id_struct_ident {
            #(#struct_fields),*
        }

        impl std::fmt::Display for #id_struct_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #struct_display_write_call
            }
        }

        impl crudkit_id::Id for #id_struct_ident {
            type Field = #id_field_enum_ident;
            type FieldIter = std::vec::IntoIter<Self::Field>;

            fn fields_iter(&self) -> Self::FieldIter {
                vec![
                    #(#create_enum_variants),*
                ].into_iter()
            }

            fn fields(&self) -> Vec<Self::Field> {
                vec![
                    #(#create_enum_variants),*
                ]
            }

            fn to_serializable_id(&self) -> crudkit_id::SerializableId {
                crudkit_id::SerializableId(
                    self.fields_iter()
                        .map(|field| (
                            crudkit_id::IdField::name(&field).to_owned(),
                            crudkit_id::IdField::to_value(&field),
                        ))
                        .collect()
                )
            }
        }
    }
}

/// Generates the `*IdField` enum with its `Display` and `crudkit_id::IdField` implementations.
///
/// The enum contains one variant per ID field of the original struct and member of the new `*Id`
/// struct type.
fn generate_id_field_enum(
    id_field_enum_ident: &Ident,
    field_metadata: &[IdFieldMetadata],
) -> proc_macro2::TokenStream {
    // Enum variants with single type (e.g., `UserId(i32)`).
    let variants = field_metadata
        .iter()
        .map(|it| {
            let type_name = &it.type_name;
            let ty = &it.ty;
            quote! { #type_name(#ty) }
        })
        .collect::<Vec<_>>();

    // Match arms mapping variant to name, ignoring values (e.g., `Self::UserId(_) => "user_id"`).
    let self_variant_to_static_name_arms = field_metadata
        .iter()
        .map(|it| {
            let type_name = &it.type_name;
            let name = &it.name;
            quote! { Self::#type_name(_) => #name }
        })
        .collect::<Vec<_>>();

    // Match arms mapping variants to `IdValue` variant
    // (e.g., `Self::UserId(value) => IdValue::I32(value.clone())`).
    // Note: Clone is required, as `to_value()` returns owned IdValue.
    let self_variant_to_id_value_variant_arms = field_metadata
        .iter()
        .map(|it| {
            let type_name = &it.type_name;
            let id_value_variant = to_id_value_variant(&it.ty);
            quote! { Self::#type_name(value) => #id_value_variant(value.clone()) }
        })
        .collect::<Vec<_>>();

    // Match arms converting to `Display` write impl (e.g., `Self::UserId(value) => write!(f, "{}", value)`).
    let self_variant_to_write_arms = field_metadata
        .iter()
        .map(|it| {
            let type_name = &it.type_name;
            quote! { Self::#type_name(value) => f.write_fmt(format_args!("{}", value)) }
        })
        .collect::<Vec<_>>();

    quote! {
        #[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
        pub enum #id_field_enum_ident {
            #(#variants),*
        }

        impl std::fmt::Display for #id_field_enum_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#self_variant_to_write_arms),*
                }
            }
        }

        impl crudkit_id::IdField for #id_field_enum_ident {
            fn name(&self) -> &'static str {
                match self {
                    #(#self_variant_to_static_name_arms),*
                }
            }

            fn to_value(&self) -> crudkit_id::IdValue {
                match self {
                    #(#self_variant_to_id_value_variant_arms),*
                }
            }
        }
    }
}

/// Returns the `IdValue` variant that must be used for the field of type `ty`.
///
/// For example: `crudkit_id::IdValue::I32` when `ty` is `i32`.
fn to_id_value_variant(ty: &syn::Type) -> proc_macro2::TokenStream {
    const SUPPORTED_TYPES_HELP: &str = indoc::indoc! {
        r#"
        Supported ID field types:
          - Integers: i32, u32, i64, u64, i128, u128
          - Strings: String
          - Booleans: bool
          - UUIDs: uuid::Uuid
          - Time: time::PrimitiveDateTime, time::OffsetDateTime

        Note:
          - Floating point types (f32, f64) are not supported (not Eq comparable)
          - Optional types (Option<T>) are not supported for ID fields
          - Use exact type paths as shown above
        "#
    };

    let span = ty.span();

    match ty {
        syn::Type::Path(type_path) => {
            let path = &type_path.path;

            // Reject Option<T> types early.
            if is_option_type(path) {
                abort!(
                    span,
                    "Option<T> types are not supported for ID fields";
                    help = "ID fields must have concrete, non-optional values.\n{}", SUPPORTED_TYPES_HELP;
                );
            }

            // Match primitives (unqualified, single-segment paths).
            if path.segments.len() == 1
                && let Some(ident) = get_final_segment_ident(path)
            {
                match ident.to_string().as_str() {
                    "i32" => return quote! { crudkit_id::IdValue::I32 },
                    "u32" => return quote! { crudkit_id::IdValue::U32 },
                    "i64" => return quote! { crudkit_id::IdValue::I64 },
                    "u64" => return quote! { crudkit_id::IdValue::U64 },
                    "i128" => return quote! { crudkit_id::IdValue::I128 },
                    "u128" => return quote! { crudkit_id::IdValue::U128 },
                    "bool" => return quote! { crudkit_id::IdValue::Bool },
                    "String" => return quote! { crudkit_id::IdValue::String },
                    "f32" => abort!(
                        span,
                        "f32 is not supported for ID fields (not Eq comparable)";
                        help = SUPPORTED_TYPES_HELP;
                    ),
                    "f64" => abort!(
                        span,
                        "f64 is not supported for ID fields (not Eq comparable)";
                        help = SUPPORTED_TYPES_HELP;
                    ),
                    _ => {} // Fall through to qualified type matching.
                }
            }

            // Match qualified types (require exact paths)
            let path_str = path_to_string(path);
            match path_str.as_str() {
                "uuid::Uuid" => {
                    return quote! { crudkit_id::IdValue::Uuid };
                }
                "time::PrimitiveDateTime" => {
                    return quote! { crudkit_id::IdValue::PrimitiveDateTime };
                }
                "time::OffsetDateTime" => {
                    return quote! { crudkit_id::IdValue::OffsetDateTime };
                }
                _ => {}
            }

            // Type not recognized - generate helpful error
            abort!(
                span,
                "Unsupported type '{}' for ID field", path_str;
                help = SUPPORTED_TYPES_HELP;
            );
        }
        _ => {
            abort!(
                span,
                "Expected a type path for ID field, found {:?}", ty;
                help = SUPPORTED_TYPES_HELP;
            );
        }
    }
}

/// Extract the final segment identifier from a path (e.g., "i32", "String" or "Uuid").
fn get_final_segment_ident(path: &syn::Path) -> Option<&syn::Ident> {
    path.segments.last().map(|seg| &seg.ident)
}

/// Convert a `syn::Path` to a `String`, matching how the type would be written in standard code
/// (e.g., `"some_crate::module::Type"`).
fn path_to_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// Check if `path` represents the `Option` type. True if the last path segment has the ident
/// `"Option"`.
fn is_option_type(path: &syn::Path) -> bool {
    path.segments
        .last()
        .map(|seg| seg.ident == "Option")
        .unwrap_or(false)
}
