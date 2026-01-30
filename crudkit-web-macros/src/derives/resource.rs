use darling::*;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::DeriveInput;

fn strip_quotes(string: Option<String>) -> Option<String> {
    string.map(|it| it.trim_start_matches('"').trim_end_matches('"').to_string())
}

#[derive(FromDeriveInput)]
#[darling(attributes(ck_resource), forward_attrs(allow, doc, cfg))]
struct Args {
    /// Name of the CRUD resource. Must match the name defined in the backend.
    resource_name: String,

    /// Type of the action payload.
    #[darling(map = "strip_quotes")]
    action_payload: Option<String>, // TODO: Do not require quotes to begin with... require an ident

    // TODO: Document
    create_model: Option<Ident>,

    read_model_id_field: Option<Ident>,
    read_model_id: Option<Ident>,
    read_model: Option<Ident>,

    update_model_id_field: Option<Ident>,
    update_model_id: Option<Ident>,
    update_model: Option<Ident>,
}

pub fn expand_derive_resource(input: DeriveInput) -> syn::Result<TokenStream> {
    let args: Args = FromDeriveInput::from_derive_input(&input)
        .map_err(|e| syn::Error::new_spanned(&input, e))?;

    let update_model_ident = args.update_model.unwrap_or_else(|| input.ident.clone());

    let update_model_id_ident = args.update_model_id.unwrap_or_else(|| {
        Ident::new(format!("{update_model_ident}Id").as_str(), input.ident.span())
    });

    let update_model_id_field_ident = args.update_model_id_field.unwrap_or_else(|| {
        Ident::new(
            format!("{update_model_ident}IdField").as_str(),
            input.ident.span(),
        )
    });

    let create_model_ident = args.create_model.unwrap_or_else(|| {
        Ident::new(
            format!("Create{update_model_ident}").as_str(),
            input.ident.span(),
        )
    });

    let read_model_ident = args.read_model.unwrap_or_else(|| {
        Ident::new(
            format!("Read{update_model_ident}").as_str(),
            input.ident.span(),
        )
    });

    let read_model_id_ident = args.read_model_id.unwrap_or_else(|| {
        Ident::new(
            format!("Read{update_model_ident}Id").as_str(),
            input.ident.span(),
        )
    });

    let read_model_id_field_ident = args.read_model_id_field.unwrap_or_else(|| {
        Ident::new(
            format!("Read{update_model_ident}IdField").as_str(),
            input.ident.span(),
        )
    });

    let resource_ident = Ident::new(
        format!("Crud{update_model_ident}Resource").as_str(),
        input.ident.span(),
    );

    let resource_name = &args.resource_name;
    let resource_name = quote! { #resource_name };

    let action_payload_type = args
        .action_payload
        .map(|it| {
            let ident = Ident::new(it.as_str(), Span::call_site());
            quote! { #ident }
        })
        .unwrap_or_else(|| quote! { crudkit_web::action::EmptyActionPayload });

    Ok(quote! {
        #[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
        pub struct #resource_ident {}

        impl crudkit_web::Resource for #resource_ident {
            fn resource_name() -> &'static str {
                #resource_name
            }

            type CreateModel = #create_model_ident;

            type ReadModelIdField = #read_model_id_field_ident;
            type ReadModelId = #read_model_id_ident;
            type ReadModel = #read_model_ident;

            type UpdateModelIdField = #update_model_id_field_ident;
            type UpdateModelId = #update_model_id_ident;
            type UpdateModel = #update_model_ident;

            type ActionPayload = #action_payload_type;
        }
    })
}
