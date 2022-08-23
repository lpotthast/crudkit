use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::proc_macro_error;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CrudColumns, attributes(crud_columns))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    fn capitalize_first_letter(s: &str) -> String {
        s[0..1].to_uppercase() + &s[1..]
    }

    fn struct_fields(data: &syn::Data) -> impl Iterator<Item = &syn::Field> {
        match data {
            syn::Data::Struct(data) => data.fields.iter(),
            syn::Data::Enum(_) => panic!("Deriving a builder for enums is not supported."),
            syn::Data::Union(_) => panic!("Deriving a builder for unions is not supported."),
        }
    }

    let column_variants = struct_fields(&ast.data)
        .map(|field| {
            let name = &field.ident.as_ref().expect("Expected named field!");
            let mut column_name = String::new();
            for part in name.to_string().split("_") {
                column_name.push_str(capitalize_first_letter(part).as_str());
            }
            Ident::new(column_name.as_str(), Span::call_site())
        })
        .collect::<Vec<Ident>>();

    let arms = struct_fields(&ast.data)
        .zip(column_variants.iter())
        .map(|(field, variant)| {
            let name = field.ident.as_ref().expect("Expected named field!");
            quote! { stringify!(#name) => Some(Column::#variant) }
        });

    let col_to_column_match_arms = column_variants.iter().map(|variant| {
        quote! { Col::#variant => Column::#variant }
    });

    let extract_ccv_value_by_column_variant_match_arms = struct_fields(&ast.data)
        .zip(column_variants.iter())
        .map(|(field, variant)| match expect_convert_ccv_attr(field) {
            Ok(attr) => {
                let span = attr.span;
                let fun_name = Ident::new(attr.fun_name.as_str(), span);
                quote_spanned! {span=>
                    Column::#variant => crud_rs::#fun_name(value)
                }
            }
            Err((span, err)) => {
                let err = err.to_string();
                quote_spanned! {span=>
                    Column::#variant => compile_error!(#err)
                }
            }
        });

    quote! {
        #[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
        pub enum Col {
            #(#column_variants),*
        }

        impl crud_rs::CrudColumns<Column, ActiveModel> for Col {
            fn to_sea_orm_column(&self) -> Column {
                match self {
                    #(#col_to_column_match_arms),*
                }
            }

            // TODO: make these three functions dynamic based on attribute on one "id" field.
            fn get_id_field() -> Column {
                Column::Id
            }

            fn get_id_field_name() -> std::string::String {
                "id".to_owned()
            }

            fn get_id(model: &ActiveModel) -> std::option::Option<i32> {
                model.id.clone().into_value().map(|v| v.unwrap())
            }
        }

        impl crud_rs::AsColType for Column {
            fn as_col_type(&self, value: crud_shared_types::ConditionClauseValue) -> std::result::Result<crud_shared_types::Value, String> {
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

struct ConvertCcvAttr {
    fun_name: String,
    span: proc_macro2::Span,
}

fn expect_convert_ccv_attr(
    field: &syn::Field,
) -> Result<ConvertCcvAttr, (proc_macro2::Span, Box<dyn std::error::Error>)> {
    for attr in &field.attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "crud_columns" {
            let span = attr.span();
            if let Some(proc_macro2::TokenTree::Group(group)) =
                attr.tokens.clone().into_iter().next()
            {
                let mut ts = group.stream().into_iter();
                match ts.next().expect("Expected 'crud_columns'. Found nothing.") {
                    proc_macro2::TokenTree::Ident(ident) => {
                        if ident != "convert_ccv" {
                            return Err((
                                span,
                                "expected `crud_columns7(convert_ccv = ...)`".into(),
                            ));
                        }
                    }
                    _ => {
                        return Err((span, "expected `crud_columns6(convert_ccv = ...)`".into()));
                    }
                }
                match ts.next().expect("Expected '='. Found nothing.") {
                    proc_macro2::TokenTree::Punct(punct) => assert_eq!(punct.as_char(), '='),
                    _ => {
                        return Err((span, "expected `crud_columns55(convert_ccv = ...)`".into()));
                    }
                }
                let fun_name = match ts.next().unwrap() {
                    proc_macro2::TokenTree::Literal(literal) => {
                        literal.to_string().trim_matches('"').trim().to_string()
                    }
                    _ => {
                        return Err((span, "expected `crud_columns4(convert_ccv = ...)`".into()));
                    }
                };
                if fun_name.is_empty() {
                    return Err((span, "expected `crud_columns3(convert_ccv = ...)`".into()));
                }
                return Ok(ConvertCcvAttr { fun_name, span });
            } else {
                return Err((span, "expected `crud_columns2(convert_ccv = ...)`".into()));
            }
        }
    }
    return Err((
        field.span(),
        "expected `crud_columns1(convert_ccv = ...)`".into(),
    ));
}
