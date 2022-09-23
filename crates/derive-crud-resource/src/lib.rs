use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::proc_macro_error;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CrudResource, attributes(crud))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let ident = &ast.ident;
    
    let create_ident = Ident::new(format!("Create{}", ident.to_string()).as_str(), ident.span());
    
    let read_ident = Ident::new(format!("Read{}", ident.to_string()).as_str(), ident.span());

    let resource_ident = Ident::new(
        format!("Crud{}Resource", ident.to_string()).as_str(),
        ident.span(),
    );

    let resource_name = match expect_resource_name(&ast) {
        Ok(attr) => {
            let span = attr.span;
            let name = attr.resource_name;
            quote_spanned! {span=>
                #name
            }
        }
        Err(err) => {
            let span = err.span;
            let error_msg = err.error_msg;
            quote_spanned! {span=>
                compile_error!(#error_msg)
            }
        }
    };

    quote! {
        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        pub struct #resource_ident {}

        impl crud_yew::CrudResourceTrait for #resource_ident {
            fn get_resource_name() -> &'static str {
                #resource_name
            }
        }

        impl crud_yew::CrudMainTrait for #resource_ident {
            type CreateModel = #create_ident;
            type ReadModel = #read_ident;
            type UpdateModel = #ident;
        }
    }
    .into()
}

struct ResourceNameAttr {
    pub resource_name: String,
    pub span: Span,
}

struct ExpectedResourceNameAttr {
    pub error_msg: String,
    pub span: Span,
}

fn expect_resource_name(ast: &DeriveInput) -> Result<ResourceNameAttr, ExpectedResourceNameAttr> {
    const EXPECTATION: &'static str = "Expected #[crud(resource_name = \"...\")]";

    fn err(reason: &str, span: Span) -> ExpectedResourceNameAttr {
        ExpectedResourceNameAttr {
            error_msg: format!("{EXPECTATION}. Error: {reason}"),
            span,
        }
    }

    let span = ast.span();
    for attr in &ast.attrs {
        let span = attr.span();
        if attr.path.is_ident("crud") {
            let meta = match attr.parse_meta() {
                Ok(meta) => meta,
                Err(_error) => return Err(err("Expected parsable meta information.", span)),
            };
            let span = meta.span();
            match meta {
                syn::Meta::Path(_) => return Err(err("Expected list as top-level element.", span)),
                syn::Meta::NameValue(_) => {
                    return Err(err("Expected list as top-level element.", span))
                }
                syn::Meta::List(list) => {
                    let nested = match list.nested.first() {
                        Some(nested) => nested,
                        None => return Err(err("Expected at least one nested meta info.", span)),
                    };
                    match nested {
                        syn::NestedMeta::Meta(nested) => {
                            match nested {
                                syn::Meta::Path(_) => {
                                    return Err(err(
                                        "Expected nested meta to be of variant NameValue.",
                                        span,
                                    ))
                                }
                                syn::Meta::List(_) => {
                                    return Err(err(
                                        "Expected nested meta to be of variant NameValue.",
                                        span,
                                    ))
                                }
                                syn::Meta::NameValue(name_value) => {
                                    if !name_value.path.is_ident("resource_name") {
                                        return Err(err("Expected resource_name ident.", span));
                                    }
                                    match &name_value.lit {
                                        syn::Lit::Str(str) => {
                                            return Ok(ResourceNameAttr {
                                                resource_name: str.value(),
                                                span,
                                            })
                                        }
                                        _ => return Err(err(
                                            "Expected a LitStr that contains the resource name.",
                                            span,
                                        )),
                                    }
                                }
                            }
                        }
                        syn::NestedMeta::Lit(_) => {
                            return Err(err(
                                "Expected first nested element to be of variant Meta.",
                                span,
                            ))
                        }
                    }
                }
            }
        }
    }
    Err(err("No matching attribute.", span))
}
