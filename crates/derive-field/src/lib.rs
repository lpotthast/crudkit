use darling::*;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{proc_macro_error, abort};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, spanned::Spanned};

#[derive(Debug, FromField)]
#[darling(attributes(field))]
struct MyFieldReceiver {
    ident: Option<syn::Ident>,

    ty: syn::Type,

    /// Determines whether this field is part of the aggregate id.
    id: Option<bool>,
}

impl MyFieldReceiver {
    pub fn is_id(&self) -> bool {
        self.id.is_some() || self.ident.as_ref().unwrap() == "id"
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(field), supports(struct_any))]
struct MyInputReceiver {
    ident: syn::Ident,

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

#[proc_macro_derive(Field, attributes(field))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let input: MyInputReceiver = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return darling::Error::write_errors(err).into(),
    };

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

    let typified_fields = input
        .fields()
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().expect("Expected named field!");
            let type_name = field_name_as_type_name(&name.to_string());
            Ident::new(type_name.as_str(), Span::call_site())
        })
        .collect::<Vec<Ident>>();

    let name = &input.ident;
    let field_name = Ident::new(format!("{name}Field").as_str(), name.span());

    let match_field_name_to_str_arms = input.fields().iter().map(|field| {
        let name = field.ident.as_ref().expect("Expected named field!");
        let name = name.to_string();
        let type_name = field_name_as_type_name(&name);
        let type_ident = Ident::new(type_name.as_str(), Span::call_site());
        quote! {
            #field_name::#type_ident => #name
        }
    });

    // "FooId" - struct
    let id_struct_ident = Ident::new(format!("{name}Id").as_str(), Span::call_site());

    // "FooIdField" - enum
    let id_field_enum_ident = Ident::new(format!("{name}IdField").as_str(), Span::call_site());

    let id_fields = input
        .fields()
        .iter()
        .filter(|field| field.is_id())
        .collect::<Vec<_>>();

    // Implement the `crud_yew::CrudIdTrait` trait if there are id fields in the struct.
    let id_trait_impl = match id_fields.len() {
        0 => quote! {}, // TODO: Create an error, as every aggregate needs an id?
        _ => {
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
                init_struct_field: proc_macro2::TokenStream,
            }

            let f = id_fields.into_iter().map(|field| {
                // "id" - name of original field
                let ident = field.ident.as_ref().expect("Ident to be present").clone();
                let name = ident.to_string();

                // "Id" - enum variant
                let type_name = Ident::new(
                    field_name_as_type_name(&ident.to_string()).as_str(),
                    Span::call_site(),
                );

                // i32 - type of original field
                let ty = &field.ty.clone();

                // Example: Id(i32)
                let variant = quote! { #type_name(#ty) };

                // Example: Self::Id(_) => "id"
                let variant_to_name_arm = quote! { Self::#type_name(_) => #name };

                let crud_value = to_crud_value(&field.ty);

                // Example: Self::Id(value) => crud_yew::Value::I32(*value)
                let variant_to_value_arm = quote! { Self::#type_name(value) => #crud_value(value.clone()) }; // TODO: always call clone?

                // Example: Self::Id(value) => Box::new(crud_yew::Value::I32(*value))
                let variant_to_boxed_value_arm = quote! { Self::#type_name(value) => Box::new(#crud_value(value.clone())) }; // TODO: always call clone?

                // Example: FooIdField::Id(value) => f.write_fmt(format_args!("{}", value))
                let display_arm = quote! { #id_field_enum_ident::#type_name(value) => f.write_fmt(format_args!("{}", value)) };

                // Example: pub id: i32,
                let struct_field = quote! { pub #ident: #ty };

                // Example: FooIdField::Id(self.id)
                let create_enum_variant = quote! { #id_field_enum_ident::#type_name(self.#ident.clone()) }; // TODO: Always clone here?

                // Example: Box::new(FooIdField::Id(self.id))
                let create_boxed_enum_variant = quote! { Box::new(#id_field_enum_ident::#type_name(self.#ident.clone())) }; // TODO: Always clone here?

                // Example: id: self.id.clone()
                let init_struct_field = quote! { #ident: self.#ident.clone() }; // TODO: Always clone here?

                IdField { ident, name, variant, variant_to_name_arm, variant_to_value_arm, variant_to_boxed_value_arm, display_arm, struct_field, create_enum_variant, create_boxed_enum_variant, init_struct_field }
            }).collect::<Vec<_>>();

            let variants = f.iter().map(|it| it.variant.clone()).collect::<Vec<_>>();
            let variants_to_name_arms = f.iter().map(|it| it.variant_to_name_arm.clone()).collect::<Vec<_>>();
            let variants_to_value_arms = f.iter().map(|it| it.variant_to_value_arm.clone()).collect::<Vec<_>>();
            let variants_to_boxed_value_arms = f.iter().map(|it| it.variant_to_boxed_value_arm.clone()).collect::<Vec<_>>();
            let display_arms = f.iter().map(|it| it.display_arm.clone()).collect::<Vec<_>>();

            let struct_fields = f.iter().map(|it| it.struct_field.clone()).collect::<Vec<_>>();
            let struct_display_format_str = format!("({})", f.iter().map(|it| format!("{}: {{}}", it.name)).collect::<Vec<_>>().join(", "));
            let struct_display_format_args = f.iter().map(|it| {
                let ident = &it.ident;
                quote! { self.#ident }
            }).collect::<Vec<_>>();
            let struct_display_write_call = quote! { f.write_fmt(format_args!(#struct_display_format_str, #(#struct_display_format_args),*)) };
            let create_enum_variants = f.iter().map(|it| it.create_enum_variant.clone()).collect::<Vec<_>>();
            let create_boxed_enum_variants = f.iter().map(|it| it.create_boxed_enum_variant.clone()).collect::<Vec<_>>();
            let init_struct_fields = f.iter().map(|it| it.init_struct_field.clone()).collect::<Vec<_>>();

            // Implements the '*IdField' enum as well as the 'IdField' and 'DynIdField' traits.
            let enum_impl = quote! {
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
                    type Value = crud_yew::Value;

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

            // Implements the '*Id' struct as well as the 'Id' trait.
            let struct_impl = quote! {
                #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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
                }
            };

            // Implements the main 'CrudIdTrait' for our base type. Allowing the user to access the ID of the entity.
            let id_trait_impl = quote! {
                impl crud_yew::CrudIdTrait for #name {
                    type Id = #id_struct_ident;

                    fn get_id(&self) -> Self::Id {
                        Self::Id {
                            #(#init_struct_fields),*
                        }
                    }
                }
            };

            quote! {
                #struct_impl
                #enum_impl
                #id_trait_impl
            }
        }
    };

    let get_field_arms = input.fields().iter().map(|field| {
        let name = field.ident.as_ref().expect("Expected named field!");
        let name = name.to_string();
        let type_name = field_name_as_type_name(&name);
        let type_ident = Ident::new(type_name.as_str(), Span::call_site());
        quote! {
            #name => #field_name::#type_ident
        }
    });

    quote! {
        #[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
        pub enum #field_name {
            #(#typified_fields),*
        }

        impl crud_yew::CrudFieldNameTrait for #field_name {
            fn get_name(&self) -> &'static str {
                match self {
                    #(#match_field_name_to_str_arms),*
                }
            }
        }

        #id_trait_impl

        impl crud_yew::CrudDataTrait for #name {
            type Field = #field_name;

            fn get_field(field_name: &str) -> #field_name {
                match field_name {
                    #(#get_field_arms),*,
                    other => panic!("String '{}' can not be parsed as a field name!", other),
                }
            }
        }
    }
    .into()
}


fn to_crud_value(ty: &syn::Type) -> proc_macro2::TokenStream {
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
            "bool" => quote! { crud_yew::Value::Bool },
            "u32" => quote! { crud_yew::Value::U32 },
            "i32" => quote! { crud_yew::Value::I32 },
            "i64" => quote! { crud_yew::Value::I64 },
            "f32" => quote! { crud_yew::Value::F32 },
            "String" => quote! { crud_yew::Value::String },
            "UtcDateTime" => quote! { crud_yew::Value::UtcDateTime },
            "Option" => match &path.path.segments[0].arguments {
                syn::PathArguments::None => todo!(),
                syn::PathArguments::AngleBracketed(args) => {
                    match args.args.iter().next().unwrap() {
                        syn::GenericArgument::Lifetime(_) => todo!(),
                        syn::GenericArgument::Type(ty) => {
                            if let syn::Type::Path(path) = ty {
                                match path.path.segments[0].ident.to_string().as_str() {
                                    "i64" => quote! { crud_yew::Value::OptionalI64 },
                                    "u32" => quote! { crud_yew::Value::OptionalU32 },
                                    "String" => quote! { crud_yew::Value::OptionalString },
                                    "UtcDateTime" => quote! { crud_yew::Value::OptionalUtcDateTime },
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