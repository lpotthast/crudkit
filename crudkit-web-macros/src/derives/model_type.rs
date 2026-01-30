use darling::FromMeta;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelType {
    Create,
    Read,
    Update,
}

impl ModelType {
    /// Generates the ErasedModel trait impl (empty body).
    pub fn gen_erased_model_impl(&self, name: &Ident) -> TokenStream {
        let trait_name = self.erased_model_trait();
        quote! {
            #[typetag::serde]
            impl crudkit_web::model::#trait_name for #name {}
        }
    }

    /// Generates the ErasedField trait impl.
    pub fn gen_erased_field_impl(&self, field_name: &Ident, model_name: &Ident) -> TokenStream {
        let trait_name = self.erased_field_trait();
        let dyn_model = self.dyn_model_type();
        quote! {
            #[typetag::serde]
            impl crudkit_web::model::#trait_name for #field_name {
                fn set_value(&self, model: &mut crudkit_web::model::#dyn_model, value: crudkit_core::Value) {
                    let model = model.downcast_mut::<#model_name>();
                    crudkit_web::FieldAccess::set_value(self, model, value);
                }

                fn value_kind(&self) -> crudkit_core::ValueKind {
                    crudkit_web::FieldAccess::<#model_name>::value_kind(self)
                }

                fn is_optional(&self) -> bool {
                    crudkit_web::FieldAccess::<#model_name>::is_optional(self)
                }
            }
        }
    }

    fn erased_model_trait(&self) -> Ident {
        Ident::new(
            match self {
                ModelType::Create => "ErasedCreateModel",
                ModelType::Read => "ErasedReadModel",
                ModelType::Update => "ErasedUpdateModel",
            },
            proc_macro2::Span::call_site(),
        )
    }

    fn erased_field_trait(&self) -> Ident {
        Ident::new(
            match self {
                ModelType::Create => "ErasedCreateField",
                ModelType::Read => "ErasedReadField",
                ModelType::Update => "ErasedUpdateField",
            },
            proc_macro2::Span::call_site(),
        )
    }

    fn dyn_model_type(&self) -> Ident {
        Ident::new(
            match self {
                ModelType::Create => "DynCreateModel",
                ModelType::Read => "DynReadModel",
                ModelType::Update => "DynUpdateModel",
            },
            proc_macro2::Span::call_site(),
        )
    }
}

impl FromMeta for ModelType {
    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        if let syn::Expr::Path(expr_path) = expr {
            let segment = expr_path
                .path
                .segments
                .last()
                .ok_or_else(|| darling::Error::custom("Empty path"))?;

            match segment.ident.to_string().as_str() {
                "Create" => Ok(ModelType::Create),
                "Read" => Ok(ModelType::Read),
                "Update" => Ok(ModelType::Update),
                _ => Err(darling::Error::unknown_value(&segment.ident.to_string())),
            }
        } else {
            Err(darling::Error::unexpected_type("path expression"))
        }
    }
}
