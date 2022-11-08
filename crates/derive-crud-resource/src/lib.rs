use darling::*;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

fn strip_quotes(string: Option<String>) -> Option<String> {
    string.map(|it| it.trim_start_matches('"').trim_end_matches('"').to_string())
}

#[derive(FromDeriveInput)]
#[darling(attributes(crud), forward_attrs(allow, doc, cfg))]
struct Args {
    resource_name: String,
    #[darling(map = "strip_quotes")]
    action_payload: Option<String>,
}

#[proc_macro_derive(CrudResource, attributes(crud))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let ident = &ast.ident;

    let create_ident = Ident::new(
        format!("Create{}", ident.to_string()).as_str(),
        ident.span(),
    );

    let read_ident = Ident::new(format!("Read{}", ident.to_string()).as_str(), ident.span());

    let resource_ident = Ident::new(
        format!("Crud{}Resource", ident.to_string()).as_str(),
        ident.span(),
    );

    let args: Args = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return darling::Error::write_errors(err).into(),
    };

    let resource_name = &args.resource_name;
    let resource_name = quote! { #resource_name };

    let action_payload_type = args
        .action_payload
        .map(|it| {
            let ident = Ident::new(it.as_str(), Span::call_site());
            quote! { #ident }
        })
        .unwrap_or_else(|| quote! { crud_yew::EmptyActionPayload });

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

            type ActionPayload = #action_payload_type;
        }
    }
    .into()
}
