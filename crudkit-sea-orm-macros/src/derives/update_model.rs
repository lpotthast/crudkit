//! SeaORM-specific UpdateModel derive macro.

use crudkit_rs_macros_core::FieldInfo;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// Generate SeaORM-specific ActiveModel update for UpdateModel.
fn generate_sea_orm_update_model_impl(
    update_model_name: &proc_macro2::Ident,
    fields: &[FieldInfo],
) -> TokenStream {
    let stmts = fields.iter().map(|field| {
        let ident = &field.ident;
        if field.is_excluded {
            if field.use_default {
                quote! {
                    self.#ident = sea_orm::ActiveValue::Set(Default::default());
                }
            } else {
                // Intentionally left blank. We will not set the field at all, keeping the value that is already stored.
                quote! {}
            }
        } else if field.is_optional {
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
        impl crudkit_sea_orm::SeaOrmUpdateModel<#update_model_name> for ActiveModel {
            fn update_with(&mut self, update: #update_model_name) {
                #(#stmts)*
            }
        }
    }
}

/// Expand CkSeaOrmUpdateModel derive.
///
/// Generates:
/// - UpdateModel struct
/// - Storage-agnostic trait implementations (CrudModel)
/// - SeaORM-specific impls (UpdateModelTrait, UpdateActiveModelTrait)
pub fn expand_derive_sea_orm_update_model(input: DeriveInput) -> syn::Result<TokenStream> {
    let output = crudkit_rs_macros_core::generate_update_model(&input)
        .map_err(|e| syn::Error::new_spanned(&input, e.to_string()))?;

    let struct_def = output.struct_def;
    let trait_impls = output.trait_impls;
    let sea_orm_impl = generate_sea_orm_update_model_impl(&output.update_model_name, &output.fields);

    Ok(quote! {
        #struct_def
        #trait_impls
        #sea_orm_impl
    })
}
