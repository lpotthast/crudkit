//! SeaORM-specific CreateModel derive macro.

use crudkit_rs_macros_core::FieldInfo;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// Generate SeaORM-specific ActiveModel conversion for CreateModel.
fn generate_sea_orm_create_model_impl(
    create_model_name: &proc_macro2::Ident,
    fields: &[FieldInfo],
) -> TokenStream {
    let arms = fields.iter().map(|field| {
        let ident = &field.ident;
        if field.is_excluded {
            if field.use_default {
                quote! {
                    #ident: sea_orm::ActiveValue::Set(Default::default())
                }
            } else {
                quote! {
                    #ident: sea_orm::ActiveValue::NotSet
                }
            }
        } else if field.is_optional {
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
    });

    quote! {
        impl crudkit_sea_orm::SeaOrmCreateModel<ActiveModel> for #create_model_name {
            async fn into_active_model(self) -> ActiveModel {
                ActiveModel {
                    #(#arms),*
                }
            }
        }
    }
}

/// Expand CkSeaOrmCreateModel derive.
///
/// Generates:
/// - CreateModel struct
/// - Storage-agnostic trait implementations (CrudModel, CreateModelTrait)
/// - SeaORM-specific impl (CreateModelTrait<ActiveModel>)
pub fn expand_derive_sea_orm_create_model(input: DeriveInput) -> syn::Result<TokenStream> {
    let output = crudkit_rs_macros_core::generate_create_model(&input)
        .map_err(|e| syn::Error::new_spanned(&input, e.to_string()))?;

    let struct_def = output.struct_def;
    let trait_impls = output.trait_impls;
    let sea_orm_impl = generate_sea_orm_create_model_impl(&output.create_model_name, &output.fields);

    Ok(quote! {
        #struct_def
        #trait_impls
        #sea_orm_impl
    })
}
