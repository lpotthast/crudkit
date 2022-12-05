use create_id_impl::IdImpl;
use darling::*;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

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

impl create_id_impl::FieldData for &MyFieldReceiver {
    fn get_ident(&self) -> Option<&syn::Ident> {
        self.ident.as_ref()
    }

    fn get_type(&self) -> &syn::Type {
        &self.ty
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

    let id_fields = input
        .fields()
        .iter()
        .filter(|field| field.is_id())
        .collect::<Vec<_>>();

    let id_impl = match id_fields.len() {
        // TODO: Create an error, as every aggregate needs an id?
        0 => quote! {},
        // Implement the `crud_yew::CrudIdTrait` trait if there are id fields in the struct.
        _ => {
            let IdImpl { code, struct_ident, enum_ident: _ } = create_id_impl::create_id_impl(name, &id_fields);

            let init_id_struct_fields = id_fields.iter().map(|field| {
                let ident = field.ident.as_ref().expect("Ident to be present").clone();
                // Example: id: self.id.clone()
                quote! { #ident: self.#ident.clone() } // TODO: Always clone here?
            });

            // Implements the main 'CrudIdTrait' for our base type. Allowing the user to access the ID of the entity.
            quote! {
                #code

                impl crud_yew::CrudIdTrait for #name {
                    type Id = #struct_ident;

                    fn get_id(&self) -> Self::Id {
                        Self::Id {
                            #(#init_id_struct_fields),*
                        }
                    }
                }
            }
        }
    };

    let match_field_name_to_str_arms = input.fields().iter().map(|field| {
        let name = field.ident.as_ref().expect("Expected named field!");
        let name = name.to_string();
        let type_name = field_name_as_type_name(&name);
        let type_ident = Ident::new(type_name.as_str(), Span::call_site());
        quote! {
            #field_name::#type_ident => #name
        }
    });

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

        #id_impl

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
