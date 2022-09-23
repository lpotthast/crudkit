use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use serde::Deserialize;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Field};

#[proc_macro_derive(FieldValue, attributes(field_value))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

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

    fn struct_fields(data: &syn::Data) -> impl Iterator<Item = &syn::Field> {
        match data {
            syn::Data::Struct(data) => data.fields.iter(),
            syn::Data::Enum(_) => panic!("Deriving a builder for enums is not supported."),
            syn::Data::Union(_) => panic!("Deriving a builder for unions is not supported."),
        }
    }

    let ident = &ast.ident;
    let field_enum_ident = Ident::new(format!("{ident}Field").as_str(), ident.span());

    fn field_type_to_value_type(ty: &syn::Type) -> ValueType {
        match &ty {
            syn::Type::Array(_) => todo!(),
            syn::Type::BareFn(_) => todo!(),
            syn::Type::Group(_) => todo!(),
            syn::Type::ImplTrait(_) => todo!(),
            syn::Type::Infer(_) => todo!(),
            syn::Type::Macro(_) => todo!(),
            syn::Type::Never(_) => todo!(),
            syn::Type::Paren(_) => todo!(),
            syn::Type::Path(path) => match path.path.segments[0].ident.to_string().as_str() {
                "bool" => ValueType::Bool,
                "u32" => ValueType::U32,
                "i32" => ValueType::I32,
                "String" => ValueType::String,
                "UtcDateTime" => ValueType::UtcDateTime,
                "Option" => match &path.path.segments[0].arguments {
                    syn::PathArguments::None => todo!(),
                    syn::PathArguments::AngleBracketed(args) => {
                        match args.args.iter().next().unwrap() {
                            syn::GenericArgument::Lifetime(_) => todo!(),
                            syn::GenericArgument::Type(ty) => {
                                if let syn::Type::Path(path) = ty {
                                    match path.path.segments[0].ident.to_string().as_str() {
                                        "u32" => ValueType::OptionalU32,
                                        "String" => ValueType::OptionalString,
                                        "UtcDateTime" => ValueType::OptionalUtcDateTime,
                                        other => {
                                            let span = ty.span();
                                            let message = format!("Unknown argument to Option type: {other:?}. Expected a known type.");
                                            abort!(
                                                span, message;
                                                help = "use one of the following types: [...]";
                                            );
                                        }
                                    }
                                } else {
                                    let span = ty.span();
                                    let message = format!("Option did not contain a 'Type'.");
                                    abort!(
                                        span, message;
                                        help = "Use Option<String> or other type...";
                                    );
                                }
                            }
                            syn::GenericArgument::Binding(_) => todo!(),
                            syn::GenericArgument::Constraint(_) => todo!(),
                            syn::GenericArgument::Const(_) => todo!(),
                        }
                    }
                    syn::PathArguments::Parenthesized(_) => todo!(),
                },
                other => {
                    let span = ty.span();
                    let message = format!("Unknown type {other:?}. Expected a known type.");
                    abort!(
                        span, message;
                        help = "use one of the following types: [...]";
                    );
                }
            },
            syn::Type::Ptr(_) => todo!(),
            syn::Type::Reference(_) => todo!(),
            syn::Type::Slice(_) => todo!(),
            syn::Type::TraitObject(_) => todo!(),
            syn::Type::Tuple(_) => todo!(),
            syn::Type::Verbatim(_) => todo!(),
            _ => todo!(),
        }
    }

    // Self::Id => Value::U32(entity.id),
    let get_field_value_arms = struct_fields(&ast.data).map(|field| {
        let field_ident = field.ident.as_ref().expect("Expected named field!");
        let field_name = field_ident.to_string();
        let field_name_as_type_name = field_name_as_type_name(&field_name);
        let field_name_as_type_ident =
            Ident::new(field_name_as_type_name.as_str(), Span::call_site());

        let value_type = match type_attr(&field) {
            Ok(ok) => match ok {
                Some(attr) => attr.ty,
                None => field_type_to_value_type(&field.ty),
            },
            Err(err) => abort!(err),
        };
        let value_type_ident: Ident = value_type.clone().into();

        let id_field_name = "id";
        let id_field_ident = Ident::new(id_field_name, Span::call_site());

        // Code that clones or copies the fields value.
        let value_clone = match value_type {
            ValueType::String => quote! { entity.#field_ident.clone() },
            // We use .unwrap_or_default(), as we feed that string into Value::String (see From<ValueType>). We should get rid of this.
            ValueType::OptionalString => quote! { entity.#field_ident.clone().unwrap_or_default() },
            ValueType::Bool => quote! { entity.#field_ident },
            ValueType::I32 => quote! { entity.#field_ident },
            ValueType::U32 => quote! { entity.#field_ident },
            ValueType::OptionalU32 => quote! { entity.#field_ident.clone() },
            ValueType::F32 => quote! { entity.#field_ident },
            ValueType::F64 => quote! { entity.#field_ident },
            ValueType::UtcDateTime => quote! { entity.#field_ident.clone() },
            ValueType::OptionalUtcDateTime => quote! { entity.#field_ident.clone() },
            ValueType::Select => quote! { entity.#field_ident.clone().into() },
            ValueType::Multiselect => quote! { entity.#field_ident.clone().into() },
            ValueType::OptionalSelect => quote! { entity.#field_ident.clone().map(Into::into) },
            ValueType::OptionalMultiselect => quote! { entity.#field_ident.clone().map(|it| it.map(Into::into)) },
            ValueType::OneToOneRelation => quote! { entity.#field_ident },
            ValueType::NestedTable => quote! { entity.#id_field_ident }, // not important, panics anyway...
        };

        quote! {
            #field_enum_ident::#field_name_as_type_ident => Value::#value_type_ident(#value_clone)
        }
    });

    // Self::Id => entity.id = value.take_u32(),
    let set_field_value_arms = struct_fields(&ast.data).map(|field| {
        let field_ident = field.ident.as_ref().expect("Expected named field!");
        let field_name = field_ident.to_string();
        let field_name_as_type_name = field_name_as_type_name(&field_name);
        let field_name_as_type_ident =
            Ident::new(field_name_as_type_name.as_str(), Span::call_site());

        let value_type = match type_attr(&field) {
            Ok(ok) => match ok {
                Some(attr) => attr.ty,
                None => field_type_to_value_type(&field.ty),
            },
            Err(err) => abort!(err),
        };
        // An expression that, given a `value`, constructs the necessary data type value to be assigned to the field.
        let take_op = match value_type {
            ValueType::String => quote! { value.take_string() },
            // TODO: value should contain Option. do not force Some type...
            ValueType::OptionalString => quote! { std::option::Option::Some(value.take_string()) },
            ValueType::Bool => quote! { value.take_bool() },    
            ValueType::I32 => quote! { value.take_i32() },
            ValueType::U32 => quote! { value.take_u32() },
            ValueType::OptionalU32 => quote! { value.take_optional_u32() },
            ValueType::F32 => quote! { value.take_f32() },
            ValueType::F64 => quote! { value.take_f64() },
            ValueType::UtcDateTime => quote! { value.take_date_time() },
            ValueType::OptionalUtcDateTime => quote! { value.take_optional_date_time() },
            ValueType::Select => quote! { value.take_select_downcast_to().into() },
            ValueType::Multiselect => quote! { value.take_multiselect_downcast_to().into() },
            ValueType::OptionalSelect => quote! { value.take_optional_select_downcast_to().into() },
            ValueType::OptionalMultiselect => quote! { value.take_optional_multiselect_downcast_to().into() },
            ValueType::OneToOneRelation => quote! { value.take_one_to_one_relation() },
            ValueType::NestedTable => {
                quote! { panic!("Setting a nested table dummy field is not allowed") }
            }
        };
        quote! {
            #field_enum_ident::#field_name_as_type_ident => entity.#field_ident = #take_op
        }
    });

    quote! {
        impl crud_yew::CrudFieldValueTrait<#ident> for #field_enum_ident {
            fn get_value(&self, entity: &#ident) -> crud_yew::Value {
                match self {
                    #(#get_field_value_arms),*,
                }
            }

            fn set_value(&self, entity: &mut #ident, value: crud_yew::Value) {
                match self {
                    #(#set_field_value_arms),*,
                }
            }
        }
    }
    .into()
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize)]
enum ValueType {
    String,
    OptionalString,
    Bool,
    I32,
    U32,
    OptionalU32,
    F32,
    F64,
    UtcDateTime,
    OptionalUtcDateTime,
    Select,
    Multiselect,
    OptionalSelect,
    OptionalMultiselect,
    OneToOneRelation,
    NestedTable,
}

/// Converts to the name of the `Value` variant which should be used.
impl From<ValueType> for Ident {
    fn from(value_type: ValueType) -> Self {
        Ident::new(
            match value_type {
                ValueType::String => "String",
                ValueType::OptionalString => "String",
                ValueType::Bool => "Bool",
                ValueType::I32 => "I32",
                ValueType::U32 => "U32",
                ValueType::OptionalU32 => "OptionalU32",
                ValueType::F32 => "F32",
                ValueType::F64 => "F64",
                ValueType::UtcDateTime => "UtcDateTime",
                ValueType::OptionalUtcDateTime => "OptionalUtcDateTime",
                ValueType::Select => "Select",
                ValueType::Multiselect => "Multiselect",
                ValueType::OptionalSelect => "OptionalSelect",
                ValueType::OptionalMultiselect => "OptionalMultiselect",
                ValueType::OneToOneRelation => "OneToOneRelation",
                ValueType::NestedTable => "NestedTable",
            },
            Span::call_site(),
        )
    }
}

struct TypeAttr {
    ty: ValueType,
}

fn type_attr(field: &Field) -> Result<Option<TypeAttr>, syn::Error> {
    for attr in &field.attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "field_value" {
            let span = attr.span();
            if let Some(proc_macro2::TokenTree::Group(group)) =
                attr.tokens.clone().into_iter().next()
            {
                let mut ts = group.stream().into_iter();
                match ts.next().expect("Expected 'type'. Found nothing.") {
                    proc_macro2::TokenTree::Ident(ident) => {
                        if ident != "type" {
                            return Err(syn::Error::new(
                                span,
                                format!("expected `field_value(type = ...)`, found '{ident} =' instead of 'type ='"),
                            ));
                        }
                    }
                    _ => {
                        return Err(syn::Error::new(span, "expected `field_value(type = ...)`"));
                    }
                }
                match ts.next().expect("Expected '='. Found nothing.") {
                    proc_macro2::TokenTree::Punct(punct) => assert_eq!(punct.as_char(), '='),
                    _ => {
                        return Err(syn::Error::new(span, "expected `field_value(type = ...)`"));
                    }
                }
                let ty = match ts.next().unwrap() {
                    proc_macro2::TokenTree::Literal(literal) => {
                        literal.to_string().trim_matches('"').trim().to_string()
                    }
                    _ => {
                        return Err(syn::Error::new(span, "expected `field_value(type = ...)`"));
                    }
                };
                if ty.is_empty() {
                    return Err(syn::Error::new(span, "expected `field_value(type = ...)`"));
                }
                return match serde_json::from_str(format!("\"{ty}\"").as_str()) {
                    Ok(ty) => Ok(Some(TypeAttr { ty })),
                    Err(err) => Err(syn::Error::new(
                        span,
                        format!("expected `field_value(type = ...)`, where '...' (actual: {ty}) is of a known variant. serde error: {err:?}"),
                    )),
                };
            } else {
                return Err(syn::Error::new(span, "expected `field_value(type = ...)` "));
            }
        }
    }
    return Ok(None);
}
