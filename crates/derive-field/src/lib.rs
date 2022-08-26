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

        impl Default for #field_name {
            fn default() -> Self {
                // TODO: select field based on attribute
                Self::Id
            }
        }

        impl crud_yew::CrudFieldNameTrait for #field_name {
            fn get_name(&self) -> &'static str {
                match self {
                    #(#match_field_name_to_str_arms),*
                }
            }
        }

        impl crud_yew::CrudFieldTrait<#field_name, #name> for #name {
            fn get_field(field_name: &str) -> #field_name {
                match field_name {
                    #(#get_field_arms),*,
                    other => panic!("String '{}' can not be parsed as a field name!", other),
                }
            }
        }

        impl crud_yew::CrudIdTrait<#field_name, #name> for #name {
            fn get_id_field() -> #field_name {
                #field_name::Id
            }

            fn get_id(&self) -> u32 {
                self.id
            }
        }

        impl crud_yew::CrudDataTrait for #name {
            type Field = #field_name;
        }
    }
    .into()
}
