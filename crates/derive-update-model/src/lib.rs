use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenTree};
use proc_macro_error::{abort, emit_error, proc_macro_error};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Field};

#[proc_macro_derive(UpdateModel, attributes(update_model))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    fn struct_fields(data: &syn::Data) -> impl Iterator<Item = &syn::Field> {
        match data {
            syn::Data::Struct(data) => data.fields.iter(),
            syn::Data::Enum(_) => abort!(syn::Error::new(
                Span::call_site(),
                "Deriving 'UpdateModel' for enums is not supported."
            )),
            syn::Data::Union(_) => abort!(syn::Error::new(
                Span::call_site(),
                "Deriving 'UpdateModel' for unions is not supported."
            )),
        }
    }

    // The type of the *CrudContext to use in trait implementations.
    let context_type = match expect_context_type_name(&ast) {
        Ok(ident) => ident,
        Err(err) => abort!(err),
    };

    let struct_field_meta = struct_fields(&ast.data)
        .map(|field| match read_meta(field) {
            Ok(meta) => Some(meta),
            Err(err) => {
                emit_error!(field.span(), err);
                None
            }
        });

    // We might have emitted errors while collecting field meta information.
    proc_macro_error::abort_if_dirty();

    let struct_field_meta = struct_field_meta
        .map(|it| it.unwrap())
        .collect::<Vec<Meta>>();

    let struct_fields_with_meta = struct_fields(&ast.data)
        .zip(struct_field_meta)
        .collect::<Vec<(&Field, Meta)>>();

    let update_model_fields = struct_fields_with_meta
        .iter()
        //.filter(|(_field, meta)| !meta.excluded)
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

    let update_active_model_stmts = struct_fields_with_meta.iter().map(|(field, meta)| {
        let ident = field.ident.as_ref().expect("Expected a named field.");
        if meta.optional {
            quote! {
                match update.#ident {
                    Some(value) => self.#ident = sea_orm::ActiveValue::Set(value),
                    None => {}
                };
            }
        } else {
            quote! {
                self.#ident = sea_orm::ActiveValue::Set(update.#ident);
            }
        }
    });

    quote! {
        #[derive(Debug, Clone, PartialEq, Deserialize)]
        pub struct UpdateModel {
            #(#update_model_fields),*
        }

        impl crud_rs::UpdateModelTrait for UpdateModel {}

        // TODO: This should also receive the crud context.
        impl crud_rs::UpdateActiveModelTrait<UpdateModel> for ActiveModel {
            fn update_with(&mut self, update: UpdateModel) {
                #(#update_active_model_stmts)*
            }
        }
    }
    .into()
}

fn expect_context_type_name(ast: &DeriveInput) -> Result<Ident, syn::Error> {
    const EXPECTATION: &str = "Expected #[create_model(context = \"...\")]";

    fn err(reason: &str, span: Span) -> syn::Error {
        syn::Error::new(span, format!("{EXPECTATION}. Error: {reason}"))
    }

    let span = ast.span();
    for attr in &ast.attrs {
        let span = attr.span();
        if attr.path.is_ident("create_model") {
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
                        syn::NestedMeta::Meta(nested) => match nested {
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
                                if !name_value.path.is_ident("context") {
                                    return Err(err("Expected context ident.", span));
                                }
                                match &name_value.lit {
                                    syn::Lit::Str(str) => {
                                        return Ok(Ident::new(str.value().as_str(), span))
                                    }
                                    _ => return Err(err(
                                        "Expected a LitStr that contains the context type name.",
                                        span,
                                    )),
                                }
                            }
                        },
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

struct Meta {
    exclude: bool,

    /// The field's type will be wrapped in `Option` if this is evaluated to true.
    /// On an update, the field is only `ActiveValue::Set` if we received a `Option::Some` variant containing the data.
    /// We do not unset data just because we didn't receive on optional field.
    optional: bool,
    
    use_default: bool,
}

fn err(span: Span, error: &str, expectation: &str) -> syn::Error {
    syn::Error::new(span, format!("{error} {expectation}"))
}

fn read_meta(field: &Field) -> Result<Meta, syn::Error> {
    // If not attribute is present, field must not be excluded.
    let mut exclude = false;
    let mut optional = false;
    let mut use_default = false;

    for attr in &field.attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "update_model" {
            let span = attr.span();
            if let Some(TokenTree::Group(group)) = attr.tokens.clone().into_iter().next() {
                let mut ts: <proc_macro2::TokenStream as IntoIterator>::IntoIter =
                    group.stream().into_iter();

                let expectation = "Expected 'exclude', 'optional' or 'use_default'";
                match ts
                    .next()
                    .ok_or_else(|| err(span, "Found no tokens.", expectation))?
                {
                    proc_macro2::TokenTree::Ident(ident) => match ident.to_string().as_str() {
                        "exclude" => {
                            exclude = read_exclude(&mut ts, span)?;
                        }
                        "optional" => {
                            optional = read_optional(&mut ts, span)?;
                        }
                        "use_default" => {
                            use_default = read_use_default(&mut ts, span)?;
                        }
                        _ => {
                            return Err(err(
                                span,
                                format!("Found unknown '{ident} ='.").as_str(),
                                expectation,
                            ));
                        }
                    },
                    other => {
                        return Err(err(
                            span,
                            format!("Expected a TokenTree::Ident, but found: {other}").as_str(),
                            expectation,
                        ));
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

    Ok(Meta {
        exclude,
        optional,
        use_default,
    })
}

fn read_exclude(
    ts: &mut <proc_macro2::TokenStream as IntoIterator>::IntoIter,
    span: Span,
) -> Result<bool, syn::Error> {
    const EXPECTATION: &str = "Expected #[update_model(exclude = \"true\")]";
    match ts
        .next()
        .ok_or_else(|| err(span, "Expected '='. Found nothing.", EXPECTATION))?
    {
        proc_macro2::TokenTree::Punct(punct) => assert_eq!(punct.as_char(), '='),
        other => {
            return Err(err(
                span,
                format!("Expected a TokenTree::Punct, but found: {other}").as_str(),
                EXPECTATION,
            ));
        }
    }
    let ty = match ts.next().ok_or_else(|| {
        err(
            span,
            "Expected '= [...]'. Found nothing after the '=' sign.",
            EXPECTATION,
        )
    })? {
        proc_macro2::TokenTree::Literal(literal) => {
            literal.to_string().trim_matches('"').trim().to_string()
        }
        other => {
            return Err(err(
                span,
                format!("Expected a TokenTree::Literal, but found: {other}").as_str(),
                EXPECTATION,
            ));
        }
    };
    if ty.is_empty() {
        return Err(err(
            span,
            "Expected in '= x', that x is not en empty string.",
            EXPECTATION,
        ));
    }
    return match ty.parse::<bool>() {
        Ok(exclude) => Ok(exclude),
        Err(error) => Err(err(span, format!("Value that came after '=' (actual: {ty}) is not parsable to type `bool`: {error:?}. Use either 'true' or 'false'.").as_str(), EXPECTATION)),
    };
}

fn read_optional(
    ts: &mut <proc_macro2::TokenStream as IntoIterator>::IntoIter,
    span: Span,
) -> Result<bool, syn::Error> {
    const EXPECTATION: &str = "Expected #[update_model(optional = \"true\")]";
    match ts
        .next()
        .ok_or_else(|| err(span, "Expected '='. Found nothing.", EXPECTATION))?
    {
        proc_macro2::TokenTree::Punct(punct) => assert_eq!(punct.as_char(), '='),
        other => {
            return Err(err(
                span,
                format!("Expected a TokenTree::Punct, but found: {other}").as_str(),
                EXPECTATION,
            ));
        }
    }
    let ty = match ts.next().ok_or_else(|| {
        err(
            span,
            "Expected '= [...]'. Found nothing after the '=' sign.",
            EXPECTATION,
        )
    })? {
        proc_macro2::TokenTree::Literal(literal) => {
            literal.to_string().trim_matches('"').trim().to_string()
        }
        other => {
            return Err(err(
                span,
                format!("Expected a TokenTree::Literal, but found: {other}").as_str(),
                EXPECTATION,
            ));
        }
    };
    if ty.is_empty() {
        return Err(err(
            span,
            "Expected in '= x', that x is not en empty string.",
            EXPECTATION,
        ));
    }
    return match ty.parse::<bool>() {
        Ok(exclude) => Ok(exclude),
        Err(error) => Err(err(span, format!("Value that came after '=' (actual: {ty}) is not parsable to type `bool`: {error:?}. Use either 'true' or 'false'.").as_str(), EXPECTATION)),
    };
}

fn read_use_default(
    ts: &mut <proc_macro2::TokenStream as IntoIterator>::IntoIter,
    span: Span,
) -> Result<bool, syn::Error> {
    const EXPECTATION: &str = "Expected #[update_model(use_default = \"true\")]";
    match ts
        .next()
        .ok_or_else(|| err(span, "Expected '='. Found nothing.", EXPECTATION))?
    {
        proc_macro2::TokenTree::Punct(punct) => assert_eq!(punct.as_char(), '='),
        other => {
            return Err(err(
                span,
                format!("Expected a TokenTree::Punct, but found: {other}").as_str(),
                EXPECTATION,
            ));
        }
    }
    let ty = match ts.next().ok_or_else(|| {
        err(
            span,
            "Expected '= [...]'. Found nothing after the '=' sign.",
            EXPECTATION,
        )
    })? {
        proc_macro2::TokenTree::Literal(literal) => {
            literal.to_string().trim_matches('"').trim().to_string()
        }
        other => {
            return Err(err(
                span,
                format!("Expected a TokenTree::Literal, but found: {other}").as_str(),
                EXPECTATION,
            ));
        }
    };
    if ty.is_empty() {
        return Err(err(
            span,
            "Expected in '= x', that x is not en empty string.",
            EXPECTATION,
        ));
    }
    return match ty.parse::<bool>() {
        Ok(use_default) => Ok(use_default),
        Err(error) => Err(err(
            span,
            format!("Value that came after '=' (actual: {ty}) is not parsable to type `bool`: {error:?}. Use either 'true' or 'false'.").as_str(),
            EXPECTATION,
        )),
    };
}
