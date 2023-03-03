#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use darling::*;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

fn strip_quotes(string: Option<String>) -> Option<String> {
    string.map(|it| it.trim_start_matches('"').trim_end_matches('"').to_string())
}

#[derive(FromDeriveInput)]
#[darling(attributes(crud), forward_attrs(allow, doc, cfg))]
struct Args {
    /// Name of the CRUD resource. Must match the name defined in the backend.
    resource_name: String,

    /// Type of the action payload.
    #[darling(map = "strip_quotes")]
    action_payload: Option<String>, // TODO: Do not require quotes to begin with... require an ident

    /// Type of the authentication data provider.
    #[darling(map = "strip_quotes")]
    auth_provider: Option<String>, // TODO: Do not require quotes to begin with... require an ident

    // TODO: Document
    
    create_model: Option<Ident>,

    read_model_id_field: Option<Ident>,
    read_model_id: Option<Ident>,
    read_model: Option<Ident>,

    update_model_id_field: Option<Ident>,
    update_model_id: Option<Ident>,
    update_model: Option<Ident>,
}

#[proc_macro_derive(CrudResource, attributes(crud))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let args: Args = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return darling::Error::write_errors(err).into(),
    };

    let update_model_ident = args.update_model.unwrap_or_else(|| ast.ident.clone());

    let update_model_id_ident = args.update_model_id.unwrap_or_else(|| {
        Ident::new(format!("{update_model_ident}Id").as_str(), ast.ident.span())
    });

    let update_model_id_field_ident = args.update_model_id_field.unwrap_or_else(|| {
        Ident::new(
            format!("{update_model_ident}IdField").as_str(),
            ast.ident.span(),
        )
    });

    let create_model_ident = args.create_model.unwrap_or_else(|| {
        Ident::new(
            format!("Create{update_model_ident}").as_str(),
            ast.ident.span(),
        )
    });

    let read_model_ident = args.read_model.unwrap_or_else(|| {
        Ident::new(
            format!("Read{update_model_ident}").as_str(),
            ast.ident.span(),
        )
    });

    let read_model_id_ident = args.read_model_id.unwrap_or_else(|| {
        Ident::new(
            format!("Read{update_model_ident}Id").as_str(),
            ast.ident.span(),
        )
    });

    let read_model_id_field_ident = args.read_model_id_field.unwrap_or_else(|| {
        Ident::new(
            format!("Read{update_model_ident}IdField").as_str(),
            ast.ident.span(),
        )
    });

    let resource_ident = Ident::new(
        format!("Crud{update_model_ident}Resource").as_str(),
        ast.ident.span(),
    );

    let resource_name = &args.resource_name;
    let resource_name = quote! { #resource_name };

    let action_payload_type = args
        .action_payload
        .map(|it| {
            let ident = Ident::new(it.as_str(), Span::call_site());
            quote! { #ident }
        })
        .unwrap_or_else(|| quote! { crud_yew::EmptyActionPayload });

    let auth_provider_type = args
        .auth_provider
        .map(|it| match syn::parse_str::<syn::Type>(it.as_ref()) {
            Ok(ty) => quote! { #ty },
            Err(err) => abort!("Given 'auth_provider' is not a valid type: {}", err),
        })
        .unwrap_or_else(|| quote! { crud_yew::services::requests::NoAuthProvider });

    quote! {
        #[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
        pub struct #resource_ident {}

        impl crud_yew::CrudResourceTrait for #resource_ident {
            fn get_resource_name() -> &'static str {
                #resource_name
            }
        }

        impl crud_yew::CrudMainTrait for #resource_ident {
            type CreateModel = #create_model_ident;

            type ReadModelIdField = #read_model_id_field_ident;
            type ReadModelId = #read_model_id_ident;
            type ReadModel = #read_model_ident;

            type UpdateModelIdField = #update_model_id_field_ident;
            type UpdateModelId = #update_model_id_ident;
            type UpdateModel = #update_model_ident;

            type ActionPayload = #action_payload_type;

            type AuthProvider = #auth_provider_type;
        }
    }
    .into()
}
