use darling::{ast, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::quote;

/// Field configuration shared between CkCreateModel and CkUpdateModel.
/// Each macro uses its own attribute namespace but the config is identical.
#[derive(Debug, Clone, FromField)]
#[darling(attributes(read_view))]
#[darling(forward_attrs)] // This forwards all attrs!
struct Field {
    attrs: Vec<syn::Attribute>,
    ident: Option<syn::Ident>,
    ty: syn::Type,
    vis: syn::Visibility,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(read_view), supports(struct_any))]
struct DeriveReadView {
    ident: syn::Ident,

    table_name: String,

    data: ast::Data<(), Field>,
}

impl DeriveReadView {
    pub fn fields(&self) -> &ast::Fields<Field> {
        match &self.data {
            ast::Data::Enum(_) => panic!("Only structs are supported."),
            ast::Data::Struct(fields) => fields,
        }
    }
}

fn generate_model_fields<'a>(
    fields: impl Iterator<Item = &'a Field> + 'a,
) -> impl Iterator<Item = TokenStream> + 'a {
    let excluded_attrs = &["ck_create_model", "ck_update_model"];

    fields.map(|field| {
        let vis = &field.vis;
        let ident = &field.ident;
        let ty = &field.ty;
        let attrs = field.attrs.iter().filter(|attr| {
            !excluded_attrs
                .iter()
                .any(|excluded| attr.path().is_ident(excluded))
        });
        quote! {
            #(#attrs)*
            #vis #ident: #ty
        }
    })
}

impl DeriveReadView {
    fn expand(&self) -> TokenStream {
        let ident = &self.ident;
        let table_name = &self.table_name;

        let fields: Vec<Field> = self.fields().iter().map(|f| f.to_owned()).collect();
        let original_fields = generate_model_fields(fields.iter());

        // TODO: new attrs or forward original attrs?
        quote!(
            pub mod read_view {
                use super::*; // We are inside a new module! This statement allows arbitrary types to be used in the original fields without leading to `unknown type` errors in the model created here. This just brings in all the types the original fields might need.

                use crudkit_sea_orm::{CkColumns, CkId};
                use sea_orm::DerivePrimaryKey;
                use sea_orm::EntityTrait;
                use sea_orm::EnumIter;
                use sea_orm::PrimaryKeyTrait;

                #[derive(
                    Clone,
                    Debug,
                    PartialEq,
                    Eq,
                    sea_orm::DeriveEntityModel,
                    CkId,
                    CkColumns,
                    crudkit_sea_orm::CkSeaOrmBridge,
                    utoipa::ToSchema,
                    serde::Deserialize,
                    serde::Serialize,
                )]
                #[sea_orm(table_name = #table_name)]
                pub struct #ident {
                    #(#original_fields,)*

                    /// Whether there are current validation errors for this entity.
                    pub has_validation_errors: bool,
                }

                #[derive(Debug, Clone, Copy, sea_orm::EnumIter, sea_orm::DeriveRelation)]
                pub enum Relation {}

                impl sea_orm::ActiveModelBehavior for ActiveModel {}
            }
        )
    }
}

pub fn expand_derive_read_view(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let derive: DeriveReadView = FromDeriveInput::from_derive_input(&input)?;
    Ok(derive.expand())
}
