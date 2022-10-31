use proc_macro::TokenStream;
use proc_macro2::{Span, TokenTree};
use proc_macro_error::{abort, emit_error, proc_macro_error};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Field};

// TODO: Automatically derive Eq on new type if source type is already able to derive it!

#[proc_macro_derive(CreateModel, attributes(create_model))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    fn struct_fields(data: &syn::Data) -> impl Iterator<Item = &syn::Field> {
        match data {
            syn::Data::Struct(data) => data.fields.iter(),
            syn::Data::Enum(_) => abort!(syn::Error::new(
                Span::call_site(),
                "Deriving 'CreateModel' for enums is not supported."
            )),
            syn::Data::Union(_) => abort!(syn::Error::new(
                Span::call_site(),
                "Deriving 'CreateModel' for unions is not supported."
            )),
        }
    }

    let struct_field_meta = struct_fields(&ast.data)
        .map(|field| match read_field_meta(field) {
            Ok(meta) => Some(meta),
            Err(err) => {
                emit_error!(err);
                None
            }
        })
        .collect::<Vec<Option<FieldMeta>>>(); // Clippy: Do not remove this! Eagerly collecting everything is required to emit potential error before executing abort_if_dirty.

    // We might have emitted errors while collecting field meta information.
    proc_macro_error::abort_if_dirty();

    let struct_field_meta = struct_field_meta
        .into_iter()
        .map(|it| it.expect("to be present"))
        .collect::<Vec<FieldMeta>>();

    let struct_fields_with_meta = struct_fields(&ast.data)
        .zip(struct_field_meta)
        .collect::<Vec<(&Field, FieldMeta)>>();

    let create_model_fields = struct_fields_with_meta
        .iter()
        .filter(|(_field, meta)| !meta.exclude)
        .map(|(field, meta)| {
            let vis = &field.vis;
            let ident = &field.ident;
            let ty = &field.ty;
            if meta.optional {
                quote! { #vis #ident: Option<#ty> }
            } else {
                quote! { #vis #ident: #ty }
            }
        });

    let into_active_model_arms = struct_fields_with_meta
        .iter()
        .map(|(field, meta)| {
            let ident = field.ident.as_ref().expect("Expected a named field.");
            if meta.exclude {
                if meta.use_default {
                    quote! {
                        #ident: sea_orm::ActiveValue::Set(Default::default())
                    }
                } else {
                    quote! {
                        #ident: sea_orm::ActiveValue::NotSet
                    }
                }
            } else if meta.optional {
                quote! {
                    #ident: match self.#ident {
                        Some(value) => sea_orm::ActiveValue::Set(value),
                        None => sea_orm::ActiveValue::NotSet,
                    }
                }
            } else {
                quote! {
                    #ident: sea_orm::ActiveValue::Set(self.#ident)
                }
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    quote! {
        #[derive(Debug, Clone, PartialEq, Deserialize)]
        pub struct CreateModel {
            #(#create_model_fields),*
        }

        #[async_trait::async_trait]
        impl crud_rs::CreateModelTrait<ActiveModel> for CreateModel {
            async fn into_active_model(self) -> ActiveModel {
                ActiveModel {
                    #(#into_active_model_arms),*
                }
            }
        }
    }
    .into()
}

fn err(span: Span, error: &str, expectation: &str) -> syn::Error {
    syn::Error::new(span, format!("{error} {expectation}"))
}

struct FieldMeta {
    exclude: bool,

    /// The field's type will be wrapped in `Option` if this is evaluated to true.
    /// On a create, the field is only `ActiveValue::Set` if we received a `Option::Some` variant containing the data.
    /// We do not unset data just because we didn't receive on optional field.
    optional: bool,

    use_default: bool,
}

fn read_field_meta(field: &Field) -> Result<FieldMeta, syn::Error> {
    // If not attribute is present, field must not be excluded.
    let mut exclude = false;
    let mut optional = false;
    let mut use_default = false;

    for attr in &field.attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "create_model" {
            let span = attr.span();
            if let Some(TokenTree::Group(group)) = attr.tokens.clone().into_iter().next() {
                let expectation = "Expected 'exclude', 'optional' or 'use_default'";
                for next in group.stream().into_iter() {
                    let span = next.span();
                    match next {
                        proc_macro2::TokenTree::Ident(ident) => {
                            let span = ident.span();
                            match ident.to_string().as_str() {
                                "exclude" => exclude = true,
                                "optional" => optional = true,
                                "use_default" => use_default = true,
                                _ => {
                                    return Err(err(
                                        span,
                                        format!("Found unknown ident '{ident}'.").as_str(),
                                        expectation,
                                    ));
                                }
                            }
                        }
                        proc_macro2::TokenTree::Punct(punct) => {
                            if punct.as_char() != ',' {
                                return Err(err(
                                    span,
                                    format!("Found unknown punctuation '{punct:?}'.").as_str(),
                                    "Expected ','.",
                                ));
                            }
                        }
                        other => {
                            return Err(err(
                            span,
                            format!("Expected a TokenTree::Ident or a TokenTree::Punct, but found: {other}").as_str(),
                            expectation,
                        ));
                        }
                    }
                }
            } else {
                return Err(err(
                    span,
                    "No TokenTree::Group found.",
                    "Expecting create_model attribute to be parsable.",
                ));
            }
        }
    }

    Ok(FieldMeta {
        exclude,
        optional,
        use_default,
    })
}
