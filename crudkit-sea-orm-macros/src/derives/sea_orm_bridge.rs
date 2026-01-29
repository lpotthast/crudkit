use darling::{ast, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use proc_macro_type_name::ToTypeName;
use quote::{format_ident, quote};

/// Field configuration for CkSeaOrmBridge.
#[derive(Debug, Clone, FromField)]
#[darling(attributes(ck_field, ck_id))]
#[darling(forward_attrs)]
struct Field {
    ident: Option<syn::Ident>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ck_field, ck_id), supports(struct_any))]
struct DeriveCkSeaOrmBridge {
    ident: syn::Ident,
    data: ast::Data<(), Field>,
}

impl DeriveCkSeaOrmBridge {
    pub fn fields(&self) -> &ast::Fields<Field> {
        match &self.data {
            ast::Data::Enum(_) => panic!("Only structs are supported."),
            ast::Data::Struct(fields) => fields,
        }
    }
}

impl DeriveCkSeaOrmBridge {
    fn expand(&self) -> TokenStream {
        let fields: Vec<Field> = self.fields().iter().map(|f| f.to_owned()).collect();
        let field_enum_name = format_ident!("{}Field", self.ident);

        // Generate {StructName}Field::Variant => Column::Variant match arms.
        let col_to_column_match_arms = fields.iter().map(|field| {
            let ident = field.ident.as_ref().expect("Expected named field!");
            let span = ident.span();
            let variant = ident.to_type_ident(span);
            quote! { #field_enum_name::#variant => Column::#variant }
        });

        quote!(
            impl crudkit_sea_orm::CrudColumns<Column> for #field_enum_name {
                fn to_sea_orm_column(&self) -> Column {
                    match self {
                        #(#col_to_column_match_arms),*
                    }
                }
            }
        )
    }
}

pub fn expand_derive_sea_orm_bridge(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let derive: DeriveCkSeaOrmBridge = FromDeriveInput::from_derive_input(&input)?;
    Ok(derive.expand())
}
