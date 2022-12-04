use create_id_impl::IdImpl;
use darling::*;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug, FromField)]
#[darling(attributes(crud_columns))]
struct MyFieldReceiver {
    ident: Option<syn::Ident>,

    ty: syn::Type,

    /// Whether or not this field is part of the entities primary key.
    id: Option<bool>,

    convert_ccv: Option<String>,
}

impl create_id_impl::IdInfo for &MyFieldReceiver {
    fn get_ident(&self) -> Option<&syn::Ident> {
        self.ident.as_ref()
    }

    fn get_ty(&self) -> &syn::Type {
        &self.ty
    }
}

impl MyFieldReceiver {
    pub fn is_id(&self) -> bool {
        self.id.is_some() || self.ident.as_ref().unwrap() == "id"
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(crud_columns), supports(struct_any))]
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

#[proc_macro_derive(CrudColumns, attributes(crud_columns))]
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

    let column_variants = input
        .fields()
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().expect("Expected named field!");
            let mut column_name = String::new();
            for part in name.to_string().split('_') {
                column_name.push_str(capitalize_first_letter(part).as_str());
            }
            Ident::new(column_name.as_str(), Span::call_site())
        })
        .collect::<Vec<Ident>>();

    let arms = input
        .fields()
        .iter()
        .zip(column_variants.iter())
        .map(|(field, variant)| {
            let name = field.ident.as_ref().expect("Expected named field!");
            quote! { stringify!(#name) => Some(Column::#variant) }
        });

    let col_to_column_match_arms = column_variants.iter().map(|variant| {
        quote! { Col::#variant => Column::#variant }
    });

    let extract_ccv_value_by_column_variant_match_arms = input
        .fields()
        .iter()
        .zip(column_variants.iter())
        .map(|(field, variant)| {
            let fun_name = match &field.convert_ccv {
                Some(fun_name) => Ident::new(fun_name.as_str(), field.ident.span()),
                None => extract_convert_function_name(&field),
            };
            quote! {
                Column::#variant => crud_rs::#fun_name(value)
            }
        });

    let id_fields = input
        .fields()
        .iter()
        .filter(|field| field.is_id())
        .collect::<Vec<_>>();

    let IdImpl {
        code,
        struct_ident,
        enum_ident: _,
    } = create_id_impl::create_id_impl(&input.ident, &id_fields);

    let init_id_struct_fields = id_fields.iter().map(|field| {
        let ident = field.ident.as_ref().expect("Ident to be present").clone();
        // Example: id: self.id.clone()
        quote! { #ident: model.#ident.clone() }
        // TODO: Always clone here?
    });

    let init_active_id_struct_fields = id_fields.iter().map(|field| {
        let ident = field.ident.as_ref().expect("Ident to be present").clone();
        // Example: id: self.id.clone()
        quote! { #ident: active_model.#ident.clone().into_value().map(|v| v.unwrap()).unwrap() }
        // TODO: Always clone here?
    });

    quote! {
        #[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
        pub enum Col {
            #(#column_variants),*
        }

        #code

        impl crud_rs::CrudColumns<Column, Model, ActiveModel> for Col {
            type Id = #struct_ident;

            fn to_sea_orm_column(&self) -> Column {
                match self {
                    #(#col_to_column_match_arms),*
                }
            }

            // We use #struct_ident instead of Self::Id, as `for Col`, Col being an enum, can lead to indistinguishable types.
            fn get_id(model: &Model) -> #struct_ident {
                #struct_ident {
                    #(#init_id_struct_fields),*
                }
            }

            // We use #struct_ident instead of Self::Id, as `for Col`, Col being an enum, can lead to indistinguishable types.
            fn get_id_active(active_model: &ActiveModel) -> std::result::Result<#struct_ident, String> {
                // TODO: The init_active_id_struct_fields code unwraps() and therefore panics. Catch missing data and return an error.
                Ok(#struct_ident {
                    #(#init_active_id_struct_fields),*
                })
            }
        }

        // TODO: Can we not just convert From<ConditionClauseValue> for std::result::Result<crud_shared_types::Value, String> by using the From trait?
        impl crud_rs::AsColType for Column {
            fn as_col_type(&self, value: crud_shared_types::condition::ConditionClauseValue) -> std::result::Result<crud_shared_types::Value, String> {
                match self {
                    #(#extract_ccv_value_by_column_variant_match_arms),*
                }
            }
        }

        impl crud_rs::MaybeColumnTrait for Entity {
            type Column = Column;

            fn get_col(name: &str) -> std::option::Option<Self::Column> {
                match name {
                    #(#arms),*,
                    _ => None,
                }
            }
        }
    }.into()
}

fn extract_convert_function_name(field: &MyFieldReceiver) -> Ident {
    let span = field.ident.span();
    let fun_name = match &field.ty {
        syn::Type::Array(_) => todo!(),
        syn::Type::BareFn(_) => todo!(),
        syn::Type::Group(_) => todo!(),
        syn::Type::ImplTrait(_) => todo!(),
        syn::Type::Infer(_) => todo!(),
        syn::Type::Macro(_) => todo!(),
        syn::Type::Never(_) => todo!(),
        syn::Type::Paren(_) => todo!(),
        syn::Type::Path(path) => match path.path.segments[0].ident.to_string().as_str() {
            "bool" => "to_bool",
            "u32" => "to_u32",
            "i32" => "to_i32",
            "i64" => "to_i64",
            "f32" => "to_f32",
            "String" => "to_string",
            "UtcDateTime" => "to_date_time",
            "Option" => match &path.path.segments[0].arguments {
                syn::PathArguments::None => todo!(),
                syn::PathArguments::AngleBracketed(args) => {
                    match args.args.iter().next().unwrap() {
                        syn::GenericArgument::Lifetime(_) => todo!(),
                        syn::GenericArgument::Type(ty) => {
                            if let syn::Type::Path(path) = ty {
                                match path.path.segments[0].ident.to_string().as_str() {
                                    "u32" => "to_u32",
                                    "i32" => "to_i32",
                                    "i64" => "to_i64",
                                    "f32" => "to_f32",
                                    "String" => "to_string",
                                    "UtcDateTime" => "to_date_time",
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
    };
    Ident::new(fun_name, span)
}
