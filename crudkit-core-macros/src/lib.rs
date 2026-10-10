#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
// darling's derives generate redundant `continue` expressions in this crate's config types.
#![allow(clippy::needless_continue)]

use darling::{FromDeriveInput, FromField, ast};
use proc_macro::TokenStream;
use proc_macro_type_name::ToTypeName;
use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{DeriveInput, Ident, parse_macro_input};

const SUPPORTED_TYPES_HELP: &str = indoc::indoc! {
    r"
    Supported ID field types:
      - Integers: i8, i16, i32, i64, i128, u8, u16, u32, u64, u128
      - Strings: String
      - Booleans: bool
      - UUIDs: uuid::Uuid
      - Time: time::PrimitiveDateTime, time::OffsetDateTime

    Note:
      - Floating point types (f32, f64) are not supported (not Eq comparable)
      - Optional types (Option<T>) are not supported for ID fields
      - Use exact type paths as shown above"
};

/// Represents a supported ID field type.
///
/// Used internally to avoid duplicating type classification logic.
enum IdValueKind {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    Bool,
    String,
    Uuid,
    PrimitiveDateTime,
    OffsetDateTime,
}

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
/// This struct holds all the `TokenStreams` and identifiers needed to generate
/// the code for one field in both the ID struct and the ID field enum.
struct IdFieldMetadata {
    /// Original field identifier (e.g., `user_id`)
    ident: Ident,

    /// Field name as string (e.g., "`user_id`")
    name: String,

    /// Field name in pascal case (e.g. `UserId`). Usable as a type (or enum variant) name.
    type_name: Ident,

    /// The original field type (e.g., `i32`).
    ty: syn::Type,

    /// The classification of `ty` as a supported ID field type.
    kind: IdValueKind,
}

impl IdFieldMetadata {
    fn new(field: &CkIdFieldConfig) -> syn::Result<Self> {
        let ident = field.get_ident().clone();
        let name = ident.to_string();
        let type_name = (&ident).to_type_ident(ident.span());
        let ty = field.get_type();
        let kind = classify_id_type(ty)?;

        Ok(IdFieldMetadata {
            ident,
            name,
            type_name,
            ty: ty.clone(),
            kind,
        })
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
/// use crudkit_core_macros::CkId;
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
/// - Integers: `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128`
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
pub fn derive_ck_id(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    expand_ck_id(&ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_ck_id(ast: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let input: CkIdInputConfig = FromDeriveInput::from_derive_input(ast)?;

    let id_fields = input
        .fields()
        .iter()
        .filter(|field| field.is_id())
        .collect::<Vec<_>>();

    if id_fields.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            "To derive CkId, at least one id field must exist.\n\
             help: A field is an id field if it is (a) named \"id\" or (b) annotated with \
             `#[ck_id(id)]`, both marking the field as part of the entities id. Specify id fields \
             or remove the derive, if no id fields can be defined for this entity.",
        ));
    }

    let source_struct_name = &input.ident;
    let id_struct_ident = Ident::new(&format!("{source_struct_name}Id"), Span::call_site());
    let id_field_enum_ident =
        Ident::new(&format!("{source_struct_name}IdField"), Span::call_site());

    let field_metadata = id_fields
        .into_iter()
        .map(IdFieldMetadata::new)
        .collect::<syn::Result<Vec<_>>>()?;

    let id_struct = generate_id_struct(&id_struct_ident, &id_field_enum_ident, &field_metadata);
    let id_field_enum = generate_id_field_enum(&id_field_enum_ident, &field_metadata);
    let has_id_impl = generate_has_id_impl(source_struct_name, &id_struct_ident, &field_metadata);

    Ok(quote! {
        #id_struct
        #id_field_enum
        #has_id_impl
    })
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

    // Shows the fields with their values (e.g. `user_id=1, org_id=2`), matching the `Display`
    // output of `SerializableId`.
    let struct_display_format_str = field_metadata
        .iter()
        .map(|it| format!("{}={{}}", it.name))
        .collect::<Vec<_>>()
        .join(", ");
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

    // Generate field extractions for from_serializable_id.
    // Each field gets its own extraction (e.g., `let user_id = extract from id;`).
    let from_serializable_field_extractions = field_metadata
        .iter()
        .map(|it| {
            let ident = &it.ident;
            let name = &it.name;
            let id_value_match = to_id_value_match_extraction(&it.kind);
            quote! {
                let #ident = {
                    let crudkit_core::id::SerializableIdEntry { field_name: _, value } = id.entries().find(|entry| entry.field_name == #name)?;
                    #id_value_match
                };
            }
        })
        .collect::<Vec<_>>();

    // Struct construction from extracted fields.
    let from_serializable_struct_construction = field_metadata
        .iter()
        .map(|it| {
            let ident = &it.ident;
            quote! { #ident }
        })
        .collect::<Vec<_>>();

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

        impl crudkit_core::id::Id for #id_struct_ident {
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

            fn to_serializable_id(&self) -> crudkit_core::id::SerializableId {
                crudkit_core::id::SerializableId(
                    self.fields_iter()
                        .map(|field| crudkit_core::id::SerializableIdEntry {
                            field_name: crudkit_core::id::IdField::name(&field).to_owned(),
                            value: crudkit_core::id::IdField::to_value(&field),
                        })
                        .collect()
                )
            }

            fn from_serializable_id(id: &crudkit_core::id::SerializableId) -> Option<Self> {
                #(#from_serializable_field_extractions)*

                Some(Self {
                    #(#from_serializable_struct_construction),*
                })
            }
        }
    }
}

/// Generates the `HasId` implementation for the source struct.
///
/// This allows accessing the composite ID from an instance of the source struct.
fn generate_has_id_impl(
    source_struct_ident: &Ident,
    id_struct_ident: &Ident,
    field_metadata: &[IdFieldMetadata],
) -> proc_macro2::TokenStream {
    // Generate field initializers: `field_name: self.field_name.clone()`.
    let init_id_struct_fields = field_metadata.iter().map(|it| {
        let ident = &it.ident;
        quote! { #ident: self.#ident.clone() }
    });

    quote! {
        impl crudkit_core::id::HasId for #source_struct_ident {
            type Id = #id_struct_ident;

            fn id(&self) -> Self::Id {
                Self::Id {
                    #(#init_id_struct_fields),*
                }
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
            let id_value_variant = to_id_value_variant(&it.kind);
            quote! { Self::#type_name(value) => #id_value_variant(value.clone()) }
        })
        .collect::<Vec<_>>();

    // Match arms converting to `Display` write impl, showing the field with its value
    // (e.g., `Self::UserId(value) => write!(f, "user_id={}", value)`), matching the `Display`
    // output of `SerializableIdEntry`.
    let self_variant_to_write_arms = field_metadata
        .iter()
        .map(|it| {
            let type_name = &it.type_name;
            let format_str = format!("{}={{}}", it.name);
            quote! { Self::#type_name(value) => f.write_fmt(format_args!(#format_str, value)) }
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

        impl crudkit_core::id::IdField for #id_field_enum_ident {
            fn name(&self) -> &'static str {
                match self {
                    #(#self_variant_to_static_name_arms),*
                }
            }

            fn to_value(&self) -> crudkit_core::id::IdValue {
                match self {
                    #(#self_variant_to_id_value_variant_arms),*
                }
            }
        }
    }
}

/// Returns the `IdValue` variant that must be used for a field of the given `kind`.
///
/// For example: `crudkit_core::id::IdValue::I32` when `kind` is `IdValueKind::I32`.
fn to_id_value_variant(kind: &IdValueKind) -> proc_macro2::TokenStream {
    match kind {
        IdValueKind::I8 => quote! { crudkit_core::id::IdValue::I8 },
        IdValueKind::I16 => quote! { crudkit_core::id::IdValue::I16 },
        IdValueKind::I32 => quote! { crudkit_core::id::IdValue::I32 },
        IdValueKind::I64 => quote! { crudkit_core::id::IdValue::I64 },
        IdValueKind::I128 => quote! { crudkit_core::id::IdValue::I128 },
        IdValueKind::U8 => quote! { crudkit_core::id::IdValue::U8 },
        IdValueKind::U16 => quote! { crudkit_core::id::IdValue::U16 },
        IdValueKind::U32 => quote! { crudkit_core::id::IdValue::U32 },
        IdValueKind::U64 => quote! { crudkit_core::id::IdValue::U64 },
        IdValueKind::U128 => quote! { crudkit_core::id::IdValue::U128 },
        IdValueKind::Bool => quote! { crudkit_core::id::IdValue::Bool },
        IdValueKind::String => quote! { crudkit_core::id::IdValue::String },
        IdValueKind::Uuid => quote! { crudkit_core::id::IdValue::Uuid },
        IdValueKind::PrimitiveDateTime => quote! { crudkit_core::id::IdValue::PrimitiveDateTime },
        IdValueKind::OffsetDateTime => quote! { crudkit_core::id::IdValue::OffsetDateTime },
    }
}

/// Returns code to extract a value from `IdValue` for a field of the given `kind`.
///
/// The generated code is a match expression that extracts the value from `value`.
/// For example: `if let crudkit_core::id::IdValue::I64(x) = value { x.clone() } else { return None }`
fn to_id_value_match_extraction(kind: &IdValueKind) -> proc_macro2::TokenStream {
    match kind {
        IdValueKind::I8 => {
            quote! { if let crudkit_core::id::IdValue::I8(x) = value { x.clone() } else { return None } }
        }
        IdValueKind::I16 => {
            quote! { if let crudkit_core::id::IdValue::I16(x) = value { x.clone() } else { return None } }
        }
        IdValueKind::I32 => {
            quote! { if let crudkit_core::id::IdValue::I32(x) = value { x.clone() } else { return None } }
        }
        IdValueKind::I64 => {
            quote! { if let crudkit_core::id::IdValue::I64(x) = value { x.clone() } else { return None } }
        }
        IdValueKind::I128 => {
            quote! { if let crudkit_core::id::IdValue::I128(x) = value { x.clone() } else { return None } }
        }
        IdValueKind::U8 => {
            quote! { if let crudkit_core::id::IdValue::U8(x) = value { x.clone() } else { return None } }
        }
        IdValueKind::U16 => {
            quote! { if let crudkit_core::id::IdValue::U16(x) = value { x.clone() } else { return None } }
        }
        IdValueKind::U32 => {
            quote! { if let crudkit_core::id::IdValue::U32(x) = value { x.clone() } else { return None } }
        }
        IdValueKind::U64 => {
            quote! { if let crudkit_core::id::IdValue::U64(x) = value { x.clone() } else { return None } }
        }
        IdValueKind::U128 => {
            quote! { if let crudkit_core::id::IdValue::U128(x) = value { x.clone() } else { return None } }
        }
        IdValueKind::Bool => {
            quote! { if let crudkit_core::id::IdValue::Bool(x) = value { x.clone() } else { return None } }
        }
        IdValueKind::String => {
            quote! { if let crudkit_core::id::IdValue::String(x) = value { x.clone() } else { return None } }
        }
        IdValueKind::Uuid => {
            quote! { if let crudkit_core::id::IdValue::Uuid(x) = value { x.clone() } else { return None } }
        }
        IdValueKind::PrimitiveDateTime => {
            quote! { if let crudkit_core::id::IdValue::PrimitiveDateTime(x) = value { x.clone() } else { return None } }
        }
        IdValueKind::OffsetDateTime => {
            quote! { if let crudkit_core::id::IdValue::OffsetDateTime(x) = value { x.clone() } else { return None } }
        }
    }
}

/// Checks if a `syn::Path` represents `Option<T>`.
///
/// Returns `true` if the last path segment is "Option".
/// This handles both `Option<T>` and `std::option::Option<T>`.
fn is_option_path(path: &syn::Path) -> bool {
    path.segments
        .last()
        .is_some_and(|seg| seg.ident == "Option")
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

/// Classifies a type as one of the supported ID field types.
///
/// Returns an error with a helpful message if the type is not supported.
fn classify_id_type(ty: &syn::Type) -> syn::Result<IdValueKind> {
    let unsupported = |message: String| {
        syn::Error::new_spanned(ty, format!("{message}\nhelp: {SUPPORTED_TYPES_HELP}"))
    };

    let syn::Type::Path(type_path) = ty else {
        return Err(unsupported(format!(
            "Expected a type path for ID field, found `{}`",
            ty.to_token_stream()
        )));
    };
    let path = &type_path.path;

    // Reject Option<T> types early.
    if is_option_path(path) {
        return Err(syn::Error::new_spanned(
            ty,
            format!(
                "Option<T> types are not supported for ID fields\n\
                 help: ID fields must have concrete, non-optional values.\n{SUPPORTED_TYPES_HELP}"
            ),
        ));
    }

    // Match primitives (single-segment paths).
    if path.segments.len() == 1
        && let Some(ident) = get_final_segment_ident(path)
    {
        match ident.to_string().as_str() {
            "i8" => return Ok(IdValueKind::I8),
            "i16" => return Ok(IdValueKind::I16),
            "i32" => return Ok(IdValueKind::I32),
            "i64" => return Ok(IdValueKind::I64),
            "i128" => return Ok(IdValueKind::I128),
            "u8" => return Ok(IdValueKind::U8),
            "u16" => return Ok(IdValueKind::U16),
            "u32" => return Ok(IdValueKind::U32),
            "u64" => return Ok(IdValueKind::U64),
            "u128" => return Ok(IdValueKind::U128),
            "bool" => return Ok(IdValueKind::Bool),
            "String" => return Ok(IdValueKind::String),
            float @ ("f32" | "f64") => {
                return Err(unsupported(format!(
                    "{float} is not supported for ID fields (not Eq comparable)"
                )));
            }
            _ => {}
        }
    }

    // Match qualified types.
    let path_str = path_to_string(path);
    match path_str.as_str() {
        "uuid::Uuid" => Ok(IdValueKind::Uuid),
        "time::PrimitiveDateTime" => Ok(IdValueKind::PrimitiveDateTime),
        "time::OffsetDateTime" => Ok(IdValueKind::OffsetDateTime),
        _ => Err(unsupported(format!(
            "Unsupported type '{path_str}' for ID field"
        ))),
    }
}
