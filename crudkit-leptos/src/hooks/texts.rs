//! The user-facing texts of CrudKit's components and notifications.

use crate::config::CrudUiTexts;
use leptos::prelude::*;
use std::sync::Arc;

/// Makes `texts` the texts of CrudKit UI rendered below the current owner.
pub fn provide_crud_texts(texts: CrudUiTexts) {
    provide_context(Arc::new(texts));
}

/// Returns the nearest provided texts, or the defaults.
#[must_use]
pub fn use_crud_texts() -> Arc<CrudUiTexts> {
    use_context::<Arc<CrudUiTexts>>().unwrap_or_default()
}
