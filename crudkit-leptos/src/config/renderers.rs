//! Field renderers and the per-instance registries resolving them.

use crate::hooks::field::CrudFieldState;
use crudkit_web::prelude::*;
use leptos::prelude::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

/// Renders one field of type `F`.
///
/// The renderer receives the field's [`CrudFieldState`]. The same state is also provided as
/// context, so the atoms and hooks rendered by the renderer bind to the field, e.g.
/// [`CrudTextField`](crate::atoms::CrudTextField) or [`crate::hooks::field::use_crud_field`].
pub struct FieldRenderer<F: TypeErasedField> {
    render: Arc<dyn Fn(CrudFieldState<F>) -> AnyView + Send + Sync>,
}
impl<F: TypeErasedField> Clone for FieldRenderer<F> {
    fn clone(&self) -> Self {
        Self {
            render: self.render.clone(),
        }
    }
}
impl<F: TypeErasedField> Debug for FieldRenderer<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldRenderer").finish_non_exhaustive()
    }
}

impl<F: TypeErasedField> FieldRenderer<F> {
    /// Creates a renderer from a view function.
    pub fn new<C: IntoView + 'static>(
        view_fn: impl Fn(CrudFieldState<F>) -> C + Send + Sync + 'static,
    ) -> Self {
        Self {
            render: Arc::new(move |state| view_fn(state).into_any()),
        }
    }

    /// Renders the field described by `state`.
    ///
    /// The renderer runs once, in its own reactive owner with `state` provided as context, so the
    /// input hooks work inside it and contexts it provides do not reach sibling fields. Signals it
    /// reads while running are not tracked: a renderer showing changing values reads them in the
    /// view it returns.
    #[must_use]
    pub fn render(&self, state: CrudFieldState<F>) -> AnyView {
        let render = self.render.clone();
        (move || {
            untrack(|| {
                crate::hooks::field::provide_field(&state);
                render(state.clone())
            })
        })
        .into_any()
    }
}

/// Per-field renderer overrides of an instance. Empty by default.
///
/// A field is rendered with the renderer registered for it here, or otherwise with the markup the
/// application chose for its input kind (see [`CrudFieldControl`](crate::atoms::CrudFieldControl)).
///
/// ```no_run
/// # crudkit_leptos::__doc_example!();
/// # fn renderers() -> FieldRendererRegistry<DynReadField> {
/// FieldRendererRegistry::default().register(
///     ReadClub::Name,
///     FieldRenderer::new(|_| view! { <strong><CrudFieldValue/></strong> }),
/// )
/// # }
/// # fn main() {}
/// ```
#[derive(Debug, Clone)]
pub struct FieldRendererRegistry<F: TypeErasedField> {
    pub(crate) reg: HashMap<F, FieldRenderer<F>>,
}

impl<F: TypeErasedField> Default for FieldRendererRegistry<F> {
    fn default() -> Self {
        Self {
            reg: HashMap::new(),
        }
    }
}

impl<F: TypeErasedField> FieldRendererRegistry<F> {
    /// Registers `renderer` for `field`, replacing an earlier registration of the same field.
    #[must_use]
    pub fn register(mut self, field: impl Into<F>, renderer: FieldRenderer<F>) -> Self {
        self.reg.insert(field.into(), renderer);
        self
    }

    /// Returns the renderer registered for `field`, if any.
    #[must_use]
    pub fn get(&self, field: &F) -> Option<&FieldRenderer<F>> {
        self.reg.get(field)
    }
}
