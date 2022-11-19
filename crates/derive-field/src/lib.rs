use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Field, attributes(field))]
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

    let typified_fields = struct_fields(&ast.data)
        .map(|field| {
            let name = field.ident.as_ref().expect("Expected named field!");
            let type_name = field_name_as_type_name(&name.to_string());
            Ident::new(type_name.as_str(), Span::call_site())
        })
        .collect::<Vec<Ident>>();

    let name = &ast.ident;
    let field_name = Ident::new(format!("{name}Field").as_str(), name.span());

    let match_field_name_to_str_arms = struct_fields(&ast.data).map(|field| {
        let name = field.ident.as_ref().expect("Expected named field!");
        let name = name.to_string();
        let type_name = field_name_as_type_name(&name);
        let type_ident = Ident::new(type_name.as_str(), Span::call_site());
        quote! {
            #field_name::#type_ident => #name
        }
    });

    // Find multiple ID field(s)!!!, either by name of by annotation.
    // TODO: Find fields annotated with "id"
    // TODO: Create ID struct use that struct in the ID trait impl.
    let id_field = struct_fields(&ast.data).find(|field| {
        field
            .ident
            .as_ref()
            .and_then(|ident| Some(ident.to_string().as_str() == "id"))
            .unwrap_or(false)
    });

    // Implement the `crud_yew::CrudIdTrait` trait if possible.
    let id_trait_impl = match id_field {
        Some(field) => {
            // "id" - name of original field
            let field_ident = field.ident.as_ref().expect("Ident to be present");
            let field_name = field_ident.to_string();

            // "Id" - enum variant
            let field_type_name = Ident::new(
                field_name_as_type_name(&field_ident.to_string()).as_str(),
                Span::call_site(),
            );

            // i32 - type of original field
            let field_type = &field.ty;

            // "FooIdField" - enum
            let id_field_enum_ident =
                Ident::new(format!("{name}IdField").as_str(), Span::call_site());

            // "FooId" - struct
            let id_struct_ident = Ident::new(format!("{name}Id").as_str(), Span::call_site());

            // FooIdField::Id(value) => f.write_fmt(format_args!("{}", value))
            let id_field_display = vec![
                quote! { #id_field_enum_ident::#field_type_name(value) => f.write_fmt(format_args!("{}", value)) },
            ];

            let id_field_variants = vec![quote! { #field_type_name(#field_type) }];
            let id_field_variants_to_name = vec![quote! { Self::#field_type_name(_) => #field_name }];
            let id_field_variants_into_value = vec![quote! { Self::#field_type_name(value) => crud_yew::Value::I32(*value) }]; // TODO: dynamic!!
            let id_field_variants_into_boxed_value = vec![quote! { Self::#field_type_name(value) => Box::new(crud_yew::Value::I32(*value)) }]; // TODO: dynamic!!

            // Implements the '*IdField' enum as well as the 'IdField' and 'DynIdField' traits.
            let enum_impl = quote! {
                #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
                pub enum #id_field_enum_ident {
                    #(#id_field_variants),*
                }

                impl std::fmt::Display for #id_field_enum_ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self {
                            #(#id_field_display),*
                        }
                    }
                }

                impl crud_shared_types::IdField for #id_field_enum_ident {
                    type Value = crud_yew::Value;

                    fn name(&self) -> &'static str {
                        match self {
                            #(#id_field_variants_to_name),*
                        }
                    }

                    fn into_value(&self) -> Self::Value {
                        match self {
                            #(#id_field_variants_into_value),*
                        }
                    }
                }

                //#[typetag::serde]
                impl crud_shared_types::DynIdField for #id_field_enum_ident {
                    fn dyn_name(&self) -> &'static str {
                        match self {
                            #(#id_field_variants_to_name),*
                        }
                    }

                    fn into_dyn_value(&self) -> Box<dyn crud_shared_types::IdFieldValue> {
                        match self {
                            #(#id_field_variants_into_boxed_value),*
                        }
                    }
                }
            };

            // Implements the '*Id' struct as well as the 'Id' trait.
            let struct_impl = quote! {
                #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
                pub struct #id_struct_ident {
                    // TODO: make this more generic / configurable
                    pub #field_ident: #field_type,
                }

                impl std::fmt::Display for #id_struct_ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        // TODO: make this more generic / configurable
                        f.write_fmt(format_args!("{}", self.#field_ident))
                    }
                }

                impl crud_shared_types::Id for #id_struct_ident {
                    type Field = #id_field_enum_ident;
                    type FieldIter = std::vec::IntoIter<Self::Field>;

                    // TODO: Add all id fields
                    fn fields_iter(&self) -> Self::FieldIter {
                        vec![
                            #id_field_enum_ident::#field_type_name(self.#field_ident),
                        ].into_iter()
                    }

                    fn fields(&self) -> Vec<Box<dyn crud_shared_types::DynIdField>> {
                        vec![
                            Box::new(#id_field_enum_ident::#field_type_name(self.#field_ident)),
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
                            id: self.#field_ident
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
        None => quote! {},
    };

    let get_field_arms = struct_fields(&ast.data).map(|field| {
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
