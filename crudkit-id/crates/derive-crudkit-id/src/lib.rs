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
struct MyFieldReceiver {
    ident: Option<Ident>,

    ty: syn::Type,

    /// Whether this field is part of the entities primary key.
    ///
    /// This can be set by specifying `#[ck_id(id)]` on a field. Only required for fields not
    /// named `id`.
    id: Option<bool>,
}

impl MyFieldReceiver {
    pub fn get_ident(&self) -> Option<&syn::Ident> {
        self.ident.as_ref()
    }

    pub fn get_type(&self) -> &syn::Type {
        &self.ty
    }

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
struct MyInputReceiver {
    ident: Ident,

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

/// Derives a type containing only the id fields of the annotated struct.
/// A field is an id field if
///   - it is named "id" or
///   - it is annotated with `#[ck_id(id)]`
/// both marking it as part of the entities id.
/// A compile error is created if the annotated struct does not contain any "id" fields.
///
/// TODO: Describe created types.
#[proc_macro_derive(CkId, attributes(ck_id))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let input: MyInputReceiver = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return darling::Error::write_errors(err).into(),
    };

    let id_fields = input
        .fields()
        .iter()
        .filter(|field| field.is_id())
        .collect::<Vec<_>>();

    if id_fields.len() == 0 {
        let message = format!("To derive CkId, at least one id field must exist.");
        abort!(Span::call_site(), message; help = "A field is an id field if it is (a) named \"id\" or (b) annotated with `#[ck_id(id)]`, both marking the field as part of the entities id. Specify id fields or remove the derive, if no id fields can be defined for this entity.";);
        // TODO: remove this constraint? rename error message?
    }

    let source_struct_name = &input.ident;

    // "FooId" - struct
    let id_struct_ident = Ident::new(
        format!("{source_struct_name}Id").as_str(),
        Span::call_site(),
    );

    // "FooIdField" - enum
    let id_field_enum_ident = Ident::new(
        format!("{source_struct_name}IdField").as_str(),
        Span::call_site(),
    );

    struct IdField {
        ident: Ident,
        name: String,

        variant: proc_macro2::TokenStream,
        variant_to_name_arm: proc_macro2::TokenStream,
        variant_to_value_arm: proc_macro2::TokenStream,
        display_arm: proc_macro2::TokenStream,
        struct_field: proc_macro2::TokenStream,
        create_enum_variant: proc_macro2::TokenStream,
    }

    let f = id_fields.into_iter().map(|field| {
        // "id" - name of original field
        let ident = field.get_ident().expect("Ident to be present").clone();
        let name = ident.to_string();

        // "Id" - enum variant
        let type_name = (&ident).to_type_ident(ident.span());

        // i32 - type of original field
        let ty = &field.get_type().clone();

        // Example: Id(i32)
        let variant = quote! { #type_name(#ty) };

        // Example: Self::Id(_) => "id"
        let variant_to_name_arm = quote! { Self::#type_name(_) => #name };

        let crudkit_value = to_id_value(ty);

        // Example: Self::Id(value) => crudkit_id::IdValue::I32(*value)
        let variant_to_value_arm = quote! { Self::#type_name(value) => #crudkit_value(value.clone()) }; // TODO: always call clone?

        // Example: FooIdField::Id(value) => f.write_fmt(format_args!("{}", value))
        let display_arm = quote! { #id_field_enum_ident::#type_name(value) => f.write_fmt(format_args!("{}", value)) };

        // Example: pub id: i32,
        let struct_field = quote! { pub #ident: #ty };

        // Example: FooIdField::Id(self.id)
        let create_enum_variant = quote! { #id_field_enum_ident::#type_name(self.#ident.clone()) }; // TODO: Always clone here?

        IdField { ident, name, variant, variant_to_name_arm, variant_to_value_arm, display_arm, struct_field, create_enum_variant }
    }).collect::<Vec<_>>();

    let variants = f.iter().map(|it| it.variant.clone()).collect::<Vec<_>>();
    let variants_to_name_arms = f
        .iter()
        .map(|it| it.variant_to_name_arm.clone())
        .collect::<Vec<_>>();
    let variants_to_value_arms = f
        .iter()
        .map(|it| it.variant_to_value_arm.clone())
        .collect::<Vec<_>>();
    let display_arms = f
        .iter()
        .map(|it| it.display_arm.clone())
        .collect::<Vec<_>>();

    let struct_fields = f
        .iter()
        .map(|it| it.struct_field.clone())
        .collect::<Vec<_>>();
    let struct_display_format_str = format!(
        "({})",
        f.iter()
            .map(|it| format!("{}: {{}}", it.name))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let struct_display_format_args = f
        .iter()
        .map(|it| {
            let ident = &it.ident;
            quote! { self.#ident }
        })
        .collect::<Vec<_>>();
    let struct_display_write_call = quote! { f.write_fmt(format_args!(#struct_display_format_str, #(#struct_display_format_args),*)) };
    let create_enum_variants = f
        .iter()
        .map(|it| it.create_enum_variant.clone())
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
                    //#(#create_boxed_enum_variants),*
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

        // -----------------

        // Implements the '*IdField' enum as well as the 'IdField' traits.
        #[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
        pub enum #id_field_enum_ident {
            #(#variants),*
        }

        impl std::fmt::Display for #id_field_enum_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#display_arms),*
                }
            }
        }

        impl crudkit_id::IdField for #id_field_enum_ident {
            fn name(&self) -> &'static str {
                match self {
                    #(#variants_to_name_arms),*
                }
            }

            fn to_value(&self) -> crudkit_id::IdValue {
                match self {
                    #(#variants_to_value_arms),*
                }
            }
        }
    }
    .into()
}

fn to_id_value(ty: &syn::Type) -> proc_macro2::TokenStream {
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
            if path.segments.len() == 1 {
                if let Some(ident) = get_final_segment_ident(path) {
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

/// Extract the final segment identifier from a path (e.g., "i32", "String", "UuidV4")
fn get_final_segment_ident(path: &syn::Path) -> Option<&syn::Ident> {
    path.segments.last().map(|seg| &seg.ident)
}

/// Convert path to display string for error messages
fn path_to_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// Check if path represents Option<T>
fn is_option_type(path: &syn::Path) -> bool {
    path.segments
        .last()
        .map(|seg| seg.ident == "Option")
        .unwrap_or(false)
}
