use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenTree};
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

    // The type of the *CrudContext to use in trait implementations.
    let struct_meta = match read_struct_meta(&ast) {
        Ok(struct_meta) => struct_meta,
        Err(err) => abort!(err),
    };
    let context_type = struct_meta.context_type;

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

    let into_active_model_arms = struct_fields_with_meta.iter().map(|(field, meta)| {
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
    }).collect::<Vec<proc_macro2::TokenStream>>();

    let simple_conversion = match struct_meta.add_simple_conversion {
        true => quote! {
            #[async_trait::async_trait]
            impl crud_rs::SimpleCreateModelTrait<ActiveModel> for CreateModel {
                async fn into_active_model_without_context(self) -> ActiveModel {
                    ActiveModel {
                        #(#into_active_model_arms),*
                    }
                }
            }
        },
        false => quote! {
            // Implementation should not be present! The users does not want this conversion to take place without having the context.
        }
    };

    quote! {
        #[derive(Debug, Clone, PartialEq, Deserialize)]
        pub struct CreateModel {
            #(#create_model_fields),*
        }

        #[async_trait::async_trait]
        impl crud_rs::CreateModelTrait<ActiveModel, #context_type> for CreateModel {
            async fn into_active_model(self, _context: &#context_type) -> ActiveModel {
                ActiveModel {
                    #(#into_active_model_arms),*
                }
            }
        }

        #simple_conversion
    }
    .into()
}

fn err(span: Span, error: &str, expectation: &str) -> syn::Error {
    syn::Error::new(span, format!("{error} {expectation}"))
}

struct StructMeta {
    /// Path of a struct implementing the CrudResourceContext that should be used in creation of a model.
    context_type: Ident,

    /// If true, an implementation for converting from the CreateModel into an ActiveModel is given which does not require a context.
    /// This is an opt-in. The impl must not be generated if not requested by the user.
    add_simple_conversion: bool,
}

fn read_struct_meta(ast: &DeriveInput) -> Result<StructMeta, syn::Error> {          
    let expectation = "Expected `#[create_model(...)]` with `context = \"...\")` and `simple_conversion` (optional).";

    // If not attribute is present, field must not be excluded.
    let mut context_type = None;
    let mut add_simple_conversion = false;

    let span = ast.span();
    for attr in &ast.attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "create_model" {
            let span = attr.span();
            if let Some(TokenTree::Group(group)) = attr.tokens.clone().into_iter().next() {

                let mut ts = group.stream().into_iter();
                while let Some(next) = ts.next() {
                    let span = next.span();
                    match next {
                        proc_macro2::TokenTree::Ident(ident) => {
                            let span = ident.span();
                            match ident.to_string().as_str() {
                                "context" => {
                                    match ts.next().ok_or_else(|| err(span, "Expected '='. Found nothing.", expectation))? {
                                        proc_macro2::TokenTree::Punct(punct) => if punct.as_char() != '=' {
                                            return Err(err(span, format!("Expected '='. Found {}.", punct.as_char()).as_str(), expectation));
                                        },
                                        _ => {
                                            return Err(err(span, "The `context` requires a type to be specified.", expectation));
                                        }
                                    }
                                    let ty_string = match ts.next().ok_or_else(|| err(span, "Expected a literal. Found nothing.", expectation))? {
                                        proc_macro2::TokenTree::Literal(literal) => {
                                            literal.to_string().trim_matches('"').trim().to_string()
                                        }
                                        _ => {
                                            return Err(err(span, "The `context` requires a type to be specified.", expectation));
                                        }
                                    };
                                    if ty_string.is_empty() {
                                        return Err(err(span, "The `context` must not be empty.", expectation));
                                    }
                                    context_type = Some(Ident::new(ty_string.as_str(), span));
                                },
                                "add_simple_conversion" => add_simple_conversion = true,
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

    if context_type.is_none() {
        return Err(err(span, "Did not find CreateModel 'context_type' specification.", expectation));
    }

    Ok(StructMeta {
        context_type: context_type.unwrap(),
        add_simple_conversion,
    })

    // for attr in &ast.attrs {
    //     let span = attr.span();
    //     if attr.path.is_ident("create_model") {
    //         let meta = match attr.parse_meta() {
    //             Ok(meta) => meta,
    //             Err(_error) => return Err(err(span, "Expected parsable meta information.", EXPECTATION)),
    //         };
    //         let span = meta.span();
    //         match meta {
    //             syn::Meta::Path(_) => return Err(err("Expected list as top-level element.", span)),
    //             syn::Meta::NameValue(_) => {
    //                 return Err(err("Expected list as top-level element.", span))
    //             }
    //             syn::Meta::List(list) => {
    //                 let nested = match list.nested.first() {
    //                     Some(nested) => nested,
    //                     None => return Err(err("Expected at least one nested meta info.", span)),
    //                 };
    //                 match nested {
    //                     syn::NestedMeta::Meta(nested) => match nested {
    //                         syn::Meta::Path(_) => {
    //                             return Err(err(
    //                                 "Expected nested meta to be of variant NameValue.",
    //                                 span,
    //                             ))
    //                         }
    //                         syn::Meta::List(_) => {
    //                             return Err(err(
    //                                 "Expected nested meta to be of variant NameValue.",
    //                                 span,
    //                             ))
    //                         }
    //                         syn::Meta::NameValue(name_value) => {
    //                             if !name_value.path.is_ident("context") {
    //                                 return Err(err("Expected context ident.", span));
    //                             }
    //                             match &name_value.lit {
    //                                 syn::Lit::Str(str) => {
    //                                     return Ok(Ident::new(str.value().as_str(), span))
    //                                 }
    //                                 _ => return Err(err(
    //                                     "Expected a LitStr that contains the context type name.",
    //                                     span,
    //                                 )),
    //                             }
    //                         }
    //                     },
    //                     syn::NestedMeta::Lit(_) => {
    //                         return Err(err(
    //                             "Expected first nested element to be of variant Meta.",
    //                             span,
    //                         ))
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }
    // Err(err("No matching attribute.", span))
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
