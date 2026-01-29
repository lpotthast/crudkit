#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use crudkit_derive_core::{
    classify_base_type, is_ordered_float, path_to_string, strip_option_path, to_pascal_case,
    ValueKind,
};
use darling::*;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};
use types::ModelType;

/// Classified type information for a field.
///
/// This is computed from the Rust type (or explicit override) and contains
/// all the information needed for code generation.
#[derive(Debug, Clone, Copy)]
struct ClassifiedType {
    /// The base value kind (without optionality).
    kind: ValueKind,
    /// Whether the type is `Option<T>`.
    is_optional: bool,
    /// Whether this is an `OrderedFloat` wrapper (requires `.into()` conversion).
    is_ordered_float: bool,
}

impl ClassifiedType {
    /// Classifies a type from a `syn::Type`.
    fn from_syn_type(ty: &syn::Type) -> Self {
        let span = ty.span();

        // Handle unit type `()` specially.
        if let syn::Type::Tuple(syn::TypeTuple { elems, .. }) = ty {
            if elems.is_empty() {
                return ClassifiedType {
                    kind: ValueKind::Void,
                    is_optional: false,
                    is_ordered_float: false,
                };
            }
        }

        // Extract path from type.
        let path = match ty {
            syn::Type::Path(type_path) => &type_path.path,
            other => {
                abort!(
                    span,
                    "crudkit: derive-field: Unsupported type {:?}. Expected a path type.",
                    other
                );
            }
        };

        // Check if this is Option<T> and extract the inner type if so.
        let (inner_path_str, is_optional) = match strip_option_path(path) {
            Some(inner_ty) => {
                // Convert the inner type to a path string for classification.
                let inner_str = match inner_ty {
                    syn::Type::Path(tp) => path_to_string(&tp.path),
                    other => {
                        abort!(
                            span,
                            "crudkit: derive-field: Option inner type {:?} is not a path type.",
                            other
                        );
                    }
                };
                (inner_str, true)
            }
            None => (path_to_string(path), false),
        };

        let kind = classify_base_type(&inner_path_str);
        let is_ordered_float = is_ordered_float(&inner_path_str);

        // Validate: if optional, the kind must support optional variants.
        if is_optional && !kind.has_optional_variant() {
            abort!(
                span,
                "Option<{}> is not supported - no optional variant exists in crudkit_core::Value",
                kind.value_variant_name()
            );
        }

        ClassifiedType {
            kind,
            is_optional,
            is_ordered_float,
        }
    }

    /// Returns the appropriate `Value` variant name for this type.
    fn value_variant_name(self) -> &'static str {
        if self.is_optional {
            self.kind
                .optional_variant_name()
                .expect("ClassifiedType was validated to have optional variant")
        } else {
            self.kind.value_variant_name()
        }
    }

    /// Creates an `Ident` for the `Value` variant.
    fn value_variant_ident(self) -> Ident {
        Ident::new(self.value_variant_name(), Span::call_site())
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(ck_field))]
struct CkFieldConfig {
    ident: Option<Ident>,

    /// The Rust type of this field (from syn).
    ty: syn::Type,
}

impl CkFieldConfig {
    /// Returns the classified type for this field.
    pub fn classified_type(&self) -> ClassifiedType {
        ClassifiedType::from_syn_type(&self.ty)
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ck_field), supports(struct_any))]
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

/// Generates the `get_value` match arms for `CrudFieldValueTrait`.
fn generate_get_value_arm(
    field: &CkFieldConfig,
    field_enum_ident: &Ident,
) -> proc_macro2::TokenStream {
    let field_ident = field.ident.as_ref().expect("Expected named field!");
    let field_name = field_ident.to_string();
    let pascal_case = to_pascal_case(&field_name);
    let field_name_as_type_ident = Ident::new(pascal_case.as_str(), Span::call_site());

    let classified = field.classified_type();
    let value_variant_ident = classified.value_variant_ident();

    // Generate the value access expression based on the type.
    let value_expr = generate_get_value_expr(field_ident, classified);

    quote! {
        #field_enum_ident::#field_name_as_type_ident => crudkit_core::Value::#value_variant_ident(#value_expr)
    }
}

/// Generates the expression to get a field's value for wrapping in a `Value` variant.
fn generate_get_value_expr(field_ident: &Ident, classified: ClassifiedType) -> proc_macro2::TokenStream {
    use ValueKind::*;

    // Special cases that need wrapping.
    match (classified.kind, classified.is_optional) {
        // Void and Other: always returns unit.
        (Void | Other, _) => return quote! { () },

        // Json needs JsonValue wrapper.
        (Json, false) => return quote! { crudkit_web::JsonValue::new(entity.#field_ident.clone()) },
        (Json, true) => {
            return quote! { entity.#field_ident.clone().map(|it| crudkit_web::JsonValue::new(it)) }
        }

        _ => {}
    }

    // OrderedFloat needs `.into()` conversion.
    if classified.is_ordered_float {
        return quote! { entity.#field_ident.into() };
    }

    // Types that need cloning (non-Copy types or optional types).
    let needs_clone = match classified.kind {
        String | PrimitiveDateTime | OffsetDateTime | Duration | Uuid | U8Vec | I32Vec | I64Vec => true,
        // Optional primitives also need clone due to Option wrapper.
        U8 | U16 | U32 | U64 | U128 | I8 | I16 | I32 | I64 | I128 | Bool | F32 | F64
            if classified.is_optional =>
        {
            true
        }
        _ => false,
    };

    if needs_clone {
        quote! { entity.#field_ident.clone() }
    } else {
        quote! { entity.#field_ident }
    }
}

/// Generates the `set_value` match arms for `CrudFieldValueTrait`.
fn generate_set_value_arm(
    field: &CkFieldConfig,
    field_enum_ident: &Ident,
) -> proc_macro2::TokenStream {
    let field_ident = field.ident.as_ref().expect("Expected named field!");
    let field_name = field_ident.to_string();
    let pascal_case = to_pascal_case(&field_name);
    let field_name_as_type_ident = Ident::new(pascal_case.as_str(), Span::call_site());

    let classified = field.classified_type();

    // Generate the assignment expression for setting the field value.
    let take_op = generate_set_value_expr(field_ident, classified);

    quote! {
        #field_enum_ident::#field_name_as_type_ident => #take_op
    }
}

/// Generates the expression to set a field's value from a `Value`.
fn generate_set_value_expr(field_ident: &Ident, classified: ClassifiedType) -> proc_macro2::TokenStream {
    use ValueKind::*;

    // Special cases.
    match (classified.kind, classified.is_optional) {
        // Void and Other: setting not allowed.
        (Void | Other, _) => {
            return quote! { ::tracing::warn!("Setting a custom field is not allowed") }
        }

        // Json uses special methods.
        (Json, false) => return quote! { entity.#field_ident = value.take_inner_json_value() },
        (Json, true) => {
            return quote! { entity.#field_ident = std::option::Option::Some(value.take_inner_json_value()) }
        }

        // Uuid uses to_uuid/to_optional_uuid methods.
        (Uuid, false) => return quote! { entity.#field_ident = value.to_uuid() },
        (Uuid, true) => return quote! { entity.#field_ident = value.to_optional_uuid() },

        _ => {}
    }

    // Get the method name based on kind and optionality.
    let method_name = classified
        .kind
        .take_method_name(classified.is_optional)
        .expect("take_method_name returned None for non-special type");
    let method_ident = Ident::new(method_name, Span::call_site());

    // OrderedFloat needs `.into()` conversion after taking.
    if classified.is_ordered_float {
        quote! { entity.#field_ident = value.#method_ident().into() }
    } else {
        quote! { entity.#field_ident = value.#method_ident() }
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
            impl crudkit_web::model::ErasedCreateModel for #name {
            }
        },
        ModelType::Read => quote! {
            #[typetag::serde]
            impl crudkit_web::model::ErasedReadModel for #name {
            }
        },
        ModelType::Update => quote! {
            #[typetag::serde]
            impl crudkit_web::model::ErasedUpdateModel for #name {
            }
        },
    };

    let model_type_based_field_trait_impl = match input.model {
        ModelType::Create => quote! {
            #[typetag::serde]
            impl crudkit_web::model::ErasedCreateField for #field_name {
                fn set_value(&self, model: &mut crudkit_web::model::DynCreateModel, value: crudkit_core::Value) {
                    let model = model.downcast_mut::<#name>();
                    crudkit_web::CrudFieldValueTrait::set_value(self, model, value);
                }
            }
        },
        ModelType::Read => quote! {
            #[typetag::serde]
            impl crudkit_web::model::ErasedReadField for #field_name {
                fn set_value(&self, model: &mut crudkit_web::model::DynReadModel, value: crudkit_core::Value) {
                    let model = model.downcast_mut::<#name>();
                    crudkit_web::CrudFieldValueTrait::set_value(self, model, value);
                }
            }
        },
        ModelType::Update => quote! {
            #[typetag::serde]
            impl crudkit_web::model::ErasedUpdateField for #field_name {
                fn set_value(&self, model: &mut crudkit_web::model::DynUpdateModel, value: crudkit_core::Value) {
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
            fn value(&self, entity: &#name) -> crudkit_core::Value {
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
            fn name(&self) -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#get_name_impl)
            }
        }

        impl crudkit_web::CrudModel for #name {
            type Field = #field_name;

            fn all_fields() -> Vec<#field_name> {
                vec![ #(#all_field_enum_accessors),* ]
            }

            fn field(field_name: &str) -> #field_name {
                #get_field_impl
            }
        }

        #[typetag::serde]
        impl crudkit_web::model::ErasedField for #field_name {
            fn set_value(&self, model: &mut crudkit_web::model::DynModel, value: crudkit_core::Value) {
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
        impl crudkit_web::model::ErasedModel for #name {}

        #field_value_trait_impl
    }
        .into()
}
