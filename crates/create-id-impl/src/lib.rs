use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::spanned::Spanned;

pub trait IdInfo {
    fn get_ident(&self) -> Option<&syn::Ident>;
    fn get_ty(&self) -> &syn::Type;
}

pub struct IdImpl {
    pub code: TokenStream,
    pub struct_ident: Ident,
    pub enum_ident: Ident,
}

/// Assumption: id_fields is not empty! The functions `abort!`s if the vec is empty.
pub fn create_id_impl<F>(source_struct_name: &syn::Ident, id_fields: &Vec<F>) -> IdImpl
where
    F: IdInfo,
{
    if id_fields.len() == 0 {
        let message = format!("Error in usage of create_id_impl: id_fields vec must not be empty!");
        abort!(Span::call_site(), message; help = "Only call create_id_impl if id fields exist!";);
    }

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
        variant_to_boxed_value_arm: proc_macro2::TokenStream,
        display_arm: proc_macro2::TokenStream,
        struct_field: proc_macro2::TokenStream,
        create_enum_variant: proc_macro2::TokenStream,
        create_boxed_enum_variant: proc_macro2::TokenStream,
    }

    let f = id_fields.into_iter().map(|field| {
        // "id" - name of original field
        let ident = field.get_ident().expect("Ident to be present").clone();
        let name = ident.to_string();

        // "Id" - enum variant
        let type_name = Ident::new(
            field_name_as_type_name(&ident.to_string()).as_str(),
            Span::call_site(),
        );

        // i32 - type of original field
        let ty = &field.get_ty().clone();

        // Example: Id(i32)
        let variant = quote! { #type_name(#ty) };

        // Example: Self::Id(_) => "id"
        let variant_to_name_arm = quote! { Self::#type_name(_) => #name };

        let crud_value = to_id_value(ty);

        // Example: Self::Id(value) => crud_shared_types::IdValue::I32(*value)
        let variant_to_value_arm = quote! { Self::#type_name(value) => #crud_value(value.clone()) }; // TODO: always call clone?

        // Example: Self::Id(value) => Box::new(crud_shared_types::IdValue::I32(*value))
        let variant_to_boxed_value_arm = quote! { Self::#type_name(value) => Box::new(#crud_value(value.clone())) }; // TODO: always call clone?

        // Example: FooIdField::Id(value) => f.write_fmt(format_args!("{}", value))
        let display_arm = quote! { #id_field_enum_ident::#type_name(value) => f.write_fmt(format_args!("{}", value)) };

        // Example: pub id: i32,
        let struct_field = quote! { pub #ident: #ty };

        // Example: FooIdField::Id(self.id)
        let create_enum_variant = quote! { #id_field_enum_ident::#type_name(self.#ident.clone()) }; // TODO: Always clone here?

        // Example: Box::new(FooIdField::Id(self.id))
        let create_boxed_enum_variant = quote! { Box::new(#id_field_enum_ident::#type_name(self.#ident.clone())) }; // TODO: Always clone here?

        IdField { ident, name, variant, variant_to_name_arm, variant_to_value_arm, variant_to_boxed_value_arm, display_arm, struct_field, create_enum_variant, create_boxed_enum_variant }
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
    let variants_to_boxed_value_arms = f
        .iter()
        .map(|it| it.variant_to_boxed_value_arm.clone())
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
    let create_boxed_enum_variants = f
        .iter()
        .map(|it| it.create_boxed_enum_variant.clone())
        .collect::<Vec<_>>();

    let code = quote! {
        // Implements the '*Id' struct as well as the 'Id' trait.
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
        pub struct #id_struct_ident {
            #(#struct_fields),*
        }

        impl std::fmt::Display for #id_struct_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #struct_display_write_call
            }
        }

        impl crud_shared_types::id::Id for #id_struct_ident {
            type Field = #id_field_enum_ident;
            type FieldIter = std::vec::IntoIter<Self::Field>;

            fn fields_iter(&self) -> Self::FieldIter {
                vec![
                    #(#create_enum_variants),*
                ].into_iter()
            }

            fn fields(&self) -> Vec<Box<dyn crud_shared_types::id::DynIdField>> {
                vec![
                    #(#create_boxed_enum_variants),*
                ]
            }

            fn into_serializable_id(&self) -> crud_shared_types::id::SerializableId {
                crud_shared_types::id::SerializableId(
                    self.fields_iter()
                        .map(|field| (
                            field.name().to_owned(),
                            field.into_value().into_serializable_value()
                        ))
                        .collect()
                )
            }
        }

        // -----------------

        // Implements the '*IdField' enum as well as the 'IdField' and 'DynIdField' traits.
        #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

        impl crud_shared_types::id::IdField for #id_field_enum_ident {
            type Value = crud_shared_types::IdValue;

            fn name(&self) -> &'static str {
                match self {
                    #(#variants_to_name_arms),*
                }
            }

            fn into_value(&self) -> Self::Value {
                match self {
                    #(#variants_to_value_arms),*
                }
            }
        }

        //#[typetag::serde]
        impl crud_shared_types::id::DynIdField for #id_field_enum_ident {
            fn dyn_name(&self) -> &'static str {
                match self {
                    #(#variants_to_name_arms),*
                }
            }

            fn into_dyn_value(&self) -> Box<dyn crud_shared_types::id::IdFieldValue> {
                match self {
                    #(#variants_to_boxed_value_arms),*
                }
            }
        }
    };

    IdImpl {
        code,
        struct_ident: id_struct_ident,
        enum_ident: id_field_enum_ident,
    }
}

fn to_id_value(ty: &syn::Type) -> proc_macro2::TokenStream {
    let span = ty.span();
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
            "bool" => quote! { crud_shared_types::IdValue::Bool },
            "u32" => quote! { crud_shared_types::IdValue::U32 },
            "i32" => quote! { crud_shared_types::IdValue::I32 },
            "i64" => quote! { crud_shared_types::IdValue::I64 },
            "f32" => abort!(
                span, "f32 is an invalid type for an ID field as it is not `Eq` comparable!";
                help = "use one of the following types: [...]";
            ),
            "String" => quote! { crud_shared_types::IdValue::String },
            "UtcDateTime" => quote! { crud_shared_types::IdValue::UtcDateTime },
            "Option" => match &path.path.segments[0].arguments {
                syn::PathArguments::None => todo!(),
                syn::PathArguments::AngleBracketed(args) => {
                    match args.args.iter().next().unwrap() {
                        syn::GenericArgument::Lifetime(_) => todo!(),
                        syn::GenericArgument::Type(ty) => {
                            if let syn::Type::Path(path) = ty {
                                match path.path.segments[0].ident.to_string().as_str() {
                                    "i64" => quote! { crud_shared_types::IdValue::OptionalI64 },
                                    "u32" => quote! { crud_shared_types::IdValue::OptionalU32 },
                                    "String" => quote! { crud_shared_types::IdValue::OptionalString },
                                    "UtcDateTime" => {
                                        quote! { crud_shared_types::IdValue::OptionalUtcDateTime }
                                    }
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
