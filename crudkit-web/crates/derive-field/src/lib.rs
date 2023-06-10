#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use darling::*;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug, FromField)]
#[darling(attributes(field, ck_id))]
struct MyFieldReceiver {
    ident: Option<syn::Ident>,

    ty: syn::Type,

    /// Determines whether this field is part of the aggregate id.
    // Originates from: crudkit_id
    id: Option<bool>,
}

impl MyFieldReceiver {
    fn get_ident(&self) -> Option<&syn::Ident> {
        self.ident.as_ref()
    }

    fn get_type(&self) -> &syn::Type {
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
#[darling(attributes(field, ck_id), supports(struct_any))]
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

#[proc_macro_derive(CkField, attributes(field, ck_id))]
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
        // Implement the `crudkit_web::CrudIdTrait` trait if there are id fields in the struct.
        _ => {
            let id_struct_ident = Ident::new(format!("{}Id", name).as_str(), name.span());

            let init_id_struct_fields = id_fields.iter().map(|field| {
                let ident = field.ident.as_ref().expect("Ident to be present").clone();
                // Example: id: self.id.clone()
                quote! { #ident: self.#ident.clone() } // TODO: Always clone here?
            });

            // Implements the main 'CrudIdTrait' for our base type. Allowing the user to access the ID of the entity.
            quote! {
                impl crudkit_web::CrudIdTrait for #name {
                    type Id = #id_struct_ident;

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
        let type_name = field_name_as_type_name(&name);
        let type_ident = Ident::new(type_name.as_str(), Span::call_site());
        quote! {
            #field_name::#type_ident
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

    quote! {
        #[derive(PartialEq, Eq, Hash, Clone, Debug, serde::Serialize, serde::Deserialize)]
        pub enum #field_name {
            #(#typified_fields),*
        }

        impl crudkit_web::CrudFieldNameTrait for #field_name {
            fn get_name(&self) -> &'static str {
                #get_name_impl
            }
        }

        #id_impl

        impl crudkit_web::CrudDataTrait for #name {
            type Field = #field_name;

            fn get_all_fields() -> Vec<#field_name> {
                vec![ #(#all_field_enum_accessors),* ]
            }

            fn get_field(field_name: &str) -> #field_name {
                #get_field_impl
            }
        }
    }
    .into()
}
