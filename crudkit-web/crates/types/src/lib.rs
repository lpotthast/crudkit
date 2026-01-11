use darling::FromMeta;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelType {
    Create,
    Read,
    Update,
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
