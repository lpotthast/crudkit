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
/// context, so components rendered by the renderer can use [`crate::hooks::field::use_crud_field`]
/// and the input hooks in [`crate::hooks::inputs`].
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
    /// The renderer runs in its own reactive owner with `state` provided as context, so the input
    /// hooks work inside it and contexts it provides do not reach sibling fields.
    #[must_use]
    pub fn render(&self, state: CrudFieldState<F>) -> AnyView {
        let render = self.render.clone();
        (move || {
            provide_context(state.clone());
            render(state.clone())
        })
        .into_any()
    }
}

/// Per-field renderer overrides of an instance.
///
/// A field is rendered with the renderer registered for it here, or otherwise with CrudKit's
/// default renderer for its value kind, see [`Self::resolve`]. Fields of custom types have no
/// default renderer and render a visible configuration error until one is registered.
#[derive(Debug, Clone)]
pub struct FieldRendererRegistry<F: TypeErasedField> {
    pub(crate) reg: HashMap<F, FieldRenderer<F>>,
}
impl<F: TypeErasedField> FieldRendererRegistry<F> {
    /// Starts an empty registry. Building it without registrations yields a registry that
    /// resolves every field to its default renderer.
    #[must_use]
    pub fn builder() -> FieldRendererRegistryBuilder<F> {
        FieldRendererRegistryBuilder::new()
    }

    /// Returns the renderer registered for `field`, if any.
    #[must_use]
    pub fn get(&self, field: &F) -> Option<&FieldRenderer<F>> {
        self.reg.get(field)
    }

    /// Returns the renderer registered for `field`, or CrudKit's default renderer for its value
    /// kind.
    #[must_use]
    pub fn resolve(&self, field: &F) -> FieldRenderer<F> {
        self.reg
            .get(field)
            .cloned()
            .unwrap_or_else(|| FieldRenderer::default_for(field.value_kind()))
    }
}
/// Collects per-field renderer overrides for a [`FieldRendererRegistry`].
#[derive(Debug)]
pub struct FieldRendererRegistryBuilder<F: TypeErasedField> {
    reg: HashMap<F, FieldRenderer<F>>,
}
impl<F: TypeErasedField> FieldRendererRegistryBuilder<F> {
    fn new() -> Self {
        Self {
            reg: HashMap::new(),
        }
    }

    /// Registers `renderer` for `field`, replacing an earlier registration of the same field.
    #[must_use]
    pub fn register(mut self, field: impl Into<F>, renderer: FieldRenderer<F>) -> Self {
        self.reg.insert(field.into(), renderer);
        self
    }

    /// Finishes the registry.
    #[must_use]
    pub fn build(self) -> FieldRendererRegistry<F> {
        FieldRendererRegistry { reg: self.reg }
    }
}
