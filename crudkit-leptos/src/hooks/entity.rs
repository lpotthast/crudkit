//! The entity of the caller's surroundings.

use crudkit_core::id::SerializableId;
use leptos::prelude::*;

/// The entity shown by the surrounding table row or entity form: what entity-related atoms act on,
/// e.g. a [`CrudDeleteButton`](crate::atoms::CrudDeleteButton). Read it with [`use_crud_entity`]
/// to build further buttons acting on "this" entity.
///
/// Rows of a [`CrudTable`](crate::atoms::CrudTable) and the form atoms
/// ([`CrudEditForm`](crate::atoms::CrudEditForm), [`CrudDetails`](crate::atoms::CrudDetails))
/// provide it.
#[derive(Debug, Clone, Copy)]
pub struct CrudEntityHandle {
    /// The entity's id, once the entity is shown. A form loading its entity, or showing that it
    /// does not exist, has none.
    pub id: Signal<Option<SerializableId>>,
    /// Asks to delete the entity, which the user then confirms.
    pub delete: Callback<()>,
    /// Whether the entity can be deleted now.
    pub can_delete: Signal<bool>,
}

/// Returns the entity of the nearest table row or entity form, if any. `None` directly inside an
/// instance, even inside the row or form of an outer instance.
#[must_use]
pub fn use_crud_entity() -> Option<CrudEntityHandle> {
    use_context::<Option<CrudEntityHandle>>().flatten()
}
