use crudkit_core_macro_util::{
    classify_base_type, is_ordered_float, path_to_string, strip_option_path, to_pascal_case,
    ValueKind, ValueKindExt,
};
use darling::*;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::{spanned::Spanned, DeriveInput};

use super::model_type::ModelType;

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
        if let syn::Type::Tuple(syn::TypeTuple { elems, .. }) = ty
            && elems.is_empty()
        {
            return ClassifiedType {
                kind: ValueKind::Void,
                is_optional: false,
                is_ordered_float: false,
            };
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

        ClassifiedType {
            kind,
            is_optional,
            is_ordered_float,
        }
    }

    /// Returns the appropriate `Value` variant name for this type.
    /// Always returns the base variant name (optionality is handled via Null).
    fn value_variant_name(self) -> &'static str {
        self.kind.value_variant_name()
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
fn generate_get_value_arm(field: &CkFieldConfig, field_enum_ident: &Ident) -> TokenStream {
    let field_ident = field.ident.as_ref().expect("Expected named field!");
    let field_name = field_ident.to_string();
    let pascal_case = to_pascal_case(&field_name);
    let field_name_as_type_ident = Ident::new(pascal_case.as_str(), Span::call_site());

    let classified = field.classified_type();

    // Generate the full value expression including the Value:: wrapper.
    let value_expr = generate_get_value_expr(field_ident, classified);

    quote! {
        #field_enum_ident::#field_name_as_type_ident => #value_expr
    }
}

/// Generates the full expression to get a field's value as a `Value`.
/// Includes the `Value::` wrapper and handles optional fields with `Value::Null`.
fn generate_get_value_expr(field_ident: &Ident, classified: ClassifiedType) -> TokenStream {
    use ValueKind::*;

    let value_variant_ident = classified.value_variant_ident();

    // Special cases that need special handling.
    match (classified.kind, classified.is_optional) {
        // Void and Other: always returns unit.
        (Void, _) => return quote! { crudkit_core::Value::Void(()) },
        (Other, _) => return quote! { crudkit_core::Value::Void(()) },

        // Json needs JsonValue wrapper.
        (Json, false) => {
            return quote! {
                crudkit_core::Value::Json(crudkit_web::JsonValue::new(entity.#field_ident.clone()))
            };
        }
        (Json, true) => {
            return quote! {
                match &entity.#field_ident {
                    Some(v) => crudkit_core::Value::Json(crudkit_web::JsonValue::new(v.clone())),
                    None => crudkit_core::Value::Null,
                }
            };
        }

        _ => {}
    }

    // Handle optional fields.
    if classified.is_optional {
        // OrderedFloat needs `.into()` conversion.
        if classified.is_ordered_float {
            return quote! {
                match entity.#field_ident {
                    Some(v) => crudkit_core::Value::#value_variant_ident(v.into()),
                    None => crudkit_core::Value::Null,
                }
            };
        }

        // Types that need cloning.
        let needs_clone = match classified.kind {
            String | PrimitiveDateTime | OffsetDateTime | Duration | Uuid => true,
            _ => false,
        };

        return if needs_clone {
            quote! {
                match &entity.#field_ident {
                    Some(v) => crudkit_core::Value::#value_variant_ident(v.clone()),
                    None => crudkit_core::Value::Null,
                }
            }
        } else {
            quote! {
                match entity.#field_ident {
                    Some(v) => crudkit_core::Value::#value_variant_ident(v),
                    None => crudkit_core::Value::Null,
                }
            }
        };
    }

    // Non-optional fields.

    // OrderedFloat needs `.into()` conversion.
    if classified.is_ordered_float {
        return quote! { crudkit_core::Value::#value_variant_ident(entity.#field_ident.into()) };
    }

    // Types that need cloning (non-Copy types).
    let needs_clone = match classified.kind {
        String | PrimitiveDateTime | OffsetDateTime | Duration | Uuid => true,
        _ => false,
    };

    if needs_clone {
        quote! { crudkit_core::Value::#value_variant_ident(entity.#field_ident.clone()) }
    } else {
        quote! { crudkit_core::Value::#value_variant_ident(entity.#field_ident) }
    }
}

/// Generates the `set_value` match arms for `CrudFieldValueTrait`.
fn generate_set_value_arm(field: &CkFieldConfig, field_enum_ident: &Ident) -> TokenStream {
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

/// Generates the `value_kind` match arm for a field.
fn generate_value_kind_arm(field: &CkFieldConfig, field_enum_ident: &Ident) -> TokenStream {
    let field_ident = field.ident.as_ref().expect("Expected named field!");
    let field_name = field_ident.to_string();
    let pascal_case = to_pascal_case(&field_name);
    let field_name_as_type_ident = Ident::new(pascal_case.as_str(), Span::call_site());

    let classified = field.classified_type();
    let kind_variant = classified.kind.value_variant_ident();

    quote! { #field_enum_ident::#field_name_as_type_ident => crudkit_core::ValueKind::#kind_variant }
}

/// Generates the `is_optional` match arm for a field.
fn generate_is_optional_arm(field: &CkFieldConfig, field_enum_ident: &Ident) -> TokenStream {
    let field_ident = field.ident.as_ref().expect("Expected named field!");
    let field_name = field_ident.to_string();
    let pascal_case = to_pascal_case(&field_name);
    let field_name_as_type_ident = Ident::new(pascal_case.as_str(), Span::call_site());

    let is_optional = field.classified_type().is_optional;
    quote! { #field_enum_ident::#field_name_as_type_ident => #is_optional }
}

/// Generates the expression to set a field's value from a `Value`.
fn generate_set_value_expr(field_ident: &Ident, classified: ClassifiedType) -> TokenStream {
    use ValueKind::*;

    // Special cases that need special handling.
    match (classified.kind, classified.is_optional) {
        // Void and Other: setting not allowed.
        (Void | Other, _) => {
            return quote! { ::tracing::warn!("Setting a custom field is not allowed") };
        }

        // Array: not yet supported for setting.
        (Array, _) => {
            return quote! { ::tracing::warn!("Setting an array field is not yet supported") };
        }

        // Null: should never be a field kind.
        (Null, _) => return quote! { ::tracing::warn!("Null cannot be a field kind") },

        _ => {}
    }

    // Types that need take_* (consume Value) because as_* returns borrowed reference.
    let uses_take_method = matches!(classified.kind, String | Json | Duration);

    if uses_take_method {
        // Generate take_* method usage.
        let method_name = match classified.kind {
            String => "take_string",
            Json => "take_json",
            Duration => "take_duration",
            _ => unreachable!(),
        };
        let method_ident = Ident::new(method_name, Span::call_site());

        if classified.is_optional {
            quote! { entity.#field_ident = value.#method_ident() }
        } else {
            let type_name = classified.kind.value_variant_name();
            quote! { entity.#field_ident = value.#method_ident().expect(concat!("Value is not ", #type_name)) }
        }
    } else {
        // Use as_* accessor methods (return Option<T> for Copy types).
        let method_name = classified
            .kind
            .accessor_method_name()
            .expect("accessor_method_name returned None for non-special type");
        let method_ident = Ident::new(method_name, Span::call_site());

        if classified.is_optional {
            // OrderedFloat needs `.into()` conversion.
            if classified.is_ordered_float {
                quote! { entity.#field_ident = value.#method_ident().map(|v| v.into()) }
            } else {
                quote! { entity.#field_ident = value.#method_ident() }
            }
        } else {
            // Non-optional fields - unwrap the Option.
            let type_name = classified.kind.value_variant_name();
            if classified.is_ordered_float {
                quote! { entity.#field_ident = value.#method_ident().expect(concat!("Value is not ", #type_name)).into() }
            } else {
                quote! { entity.#field_ident = value.#method_ident().expect(concat!("Value is not ", #type_name)) }
            }
        }
    }
}

pub fn expand_derive_field(input: DeriveInput) -> syn::Result<TokenStream> {
    let input_receiver: CkFieldInputReceiver = FromDeriveInput::from_derive_input(&input)
        .map_err(|e| syn::Error::new_spanned(&input, e))?;

    let typified_fields = input_receiver
        .fields()
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().expect("Expected named field!");
            let type_name = to_pascal_case(&name.to_string());
            Ident::new(type_name.as_str(), Span::call_site())
        })
        .collect::<Vec<Ident>>();

    let name = &input_receiver.ident;
    let field_name = Ident::new(format!("{name}Field").as_str(), name.span());

    let direct_field_accessors = input_receiver.fields().iter().map(|field| {
        let name = field.ident.as_ref().expect("Expected named field!");
        let type_name = to_pascal_case(&name.to_string());
        let type_ident = Ident::new(type_name.as_str(), Span::call_site());

        quote! { pub const #type_ident: #field_name = #field_name::#type_ident; }
    });

    let match_field_name_to_str_arms = input_receiver.fields().iter().map(|field| {
        let name = field.ident.as_ref().expect("Expected named field!");
        let name = name.to_string();
        let type_name = to_pascal_case(&name);
        let type_ident = Ident::new(type_name.as_str(), Span::call_site());
        quote! {
            #field_name::#type_ident => #name
        }
    });
    let get_name_impl = match input_receiver.fields().len() {
        0 => quote! { "" },
        _ => quote! {
            match self {
                #(#match_field_name_to_str_arms),*
            }
        },
    };

    let all_field_enum_accessors = input_receiver.fields().iter().map(|field| {
        let name = field.ident.as_ref().expect("Expected named field!");
        let name = name.to_string();
        let type_name = to_pascal_case(&name);
        let type_ident = Ident::new(type_name.as_str(), Span::call_site());
        quote! {
            #field_name::#type_ident
        }
    });

    let get_field_arms = input_receiver.fields().iter().map(|field| {
        let name = field.ident.as_ref().expect("Expected named field!");
        let name = name.to_string();
        let type_name = to_pascal_case(&name);
        let type_ident = Ident::new(type_name.as_str(), Span::call_site());
        quote! {
            #name => #field_name::#type_ident
        }
    });
    let get_field_impl = match input_receiver.fields().len() {
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

    let model_type_based_model_trait_impl = input_receiver.model.gen_erased_model_impl(&name);
    let model_type_based_field_trait_impl =
        input_receiver.model.gen_erased_field_impl(&field_name, &name);

    // Generate CrudFieldValueTrait implementation.
    let get_field_value_arms = input_receiver
        .fields()
        .iter()
        .map(|field| generate_get_value_arm(field, &field_name));
    let get_value_impl = match input_receiver.fields().len() {
        0 => {
            quote! { panic!("Cannot get value. Zero fields available! Should be unreachable. Source-crate: crudkit-web-macros") }
        }
        _ => quote! {
            match self {
                #(#get_field_value_arms),*,
            }
        },
    };

    let set_field_value_arms = input_receiver
        .fields()
        .iter()
        .map(|field| generate_set_value_arm(field, &field_name));
    let set_value_impl = match input_receiver.fields().len() {
        0 => {
            quote! { panic!("Cannot set value. Zero fields available! Should be unreachable. Source-crate: crudkit-web-macros") }
        }
        _ => quote! {
            match self {
                #(#set_field_value_arms),*,
            }
        },
    };

    // Generate value_kind and is_optional match arms.
    let value_kind_arms = input_receiver
        .fields()
        .iter()
        .map(|field| generate_value_kind_arm(field, &field_name));
    let value_kind_impl = match input_receiver.fields().len() {
        0 => quote! { crudkit_core::ValueKind::Void },
        _ => quote! {
            match self {
                #(#value_kind_arms),*
            }
        },
    };

    let is_optional_arms = input_receiver
        .fields()
        .iter()
        .map(|field| generate_is_optional_arm(field, &field_name));
    let is_optional_impl = match input_receiver.fields().len() {
        0 => quote! { false },
        _ => quote! {
            match self {
                #(#is_optional_arms),*
            }
        },
    };

    let field_value_trait_impl = quote! {
        impl crudkit_web::FieldAccess<#name> for #field_name {
            fn value(&self, entity: &#name) -> crudkit_core::Value {
                #get_value_impl
            }

            fn set_value(&self, entity: &mut #name, value: crudkit_core::Value) {
                #set_value_impl
            }

            fn value_kind(&self) -> crudkit_core::ValueKind {
                #value_kind_impl
            }

            fn is_optional(&self) -> bool {
                #is_optional_impl
            }
        }
    };

    Ok(quote! {
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

        impl crudkit_web::Model for #name {
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
                crudkit_web::FieldAccess::set_value(self, model, value);
            }

            fn value_kind(&self) -> crudkit_core::ValueKind {
                crudkit_web::FieldAccess::<#name>::value_kind(self)
            }

            fn is_optional(&self) -> bool {
                crudkit_web::FieldAccess::<#name>::is_optional(self)
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
    })
}
