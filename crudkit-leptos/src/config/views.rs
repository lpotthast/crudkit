//! Registry of built-in and application-defined CrudKit views.

#![deny(missing_docs)]

use crate::instance::CrudNavigation;
use crudkit_web::view::CrudView;
use leptos::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

type ViewRenderer = Arc<dyn Fn(CrudView, CrudNavigation) -> AnyView + Send + Sync + 'static>;

#[derive(Clone)]
struct RegisteredRenderer {
    renderer: ViewRenderer,
}

impl RegisteredRenderer {
    fn new(renderer: impl Fn(CrudView, CrudNavigation) -> AnyView + Send + Sync + 'static) -> Self {
        Self {
            renderer: Arc::new(renderer),
        }
    }
}

/// Returned when registering a view name that is already present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateCrudViewError {
    name: String,
}

impl DuplicateCrudViewError {
    /// Returns the registry key that was already occupied.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for DuplicateCrudViewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "a CrudKit renderer is already registered for `{}`",
            self.name
        )
    }
}

impl Error for DuplicateCrudViewError {}

/// Maps open [`CrudView`] names to renderers.
///
/// A default registry contains CrudKit's table, create, read, and edit
/// renderers. [`Self::register`] rejects duplicate names; use [`Self::replace`]
/// when overriding a built-in or replacing an application renderer is
/// intentional.
///
/// ```no_run
/// # use crudkit_leptos::prelude::*;
/// # use leptos::prelude::*;
/// let mut registry = CrudViewRegistry::default();
/// registry.register("app.audit", |view, navigation| {
///     let title = view
///         .typed_payload::<String>()
///         .unwrap_or_else(|_| "Audit".to_owned());
///     view! {
///         <section>
///             <h2>{title}</h2>
///             <button on:click=move |_| navigation.return_from_current()>"Return"</button>
///         </section>
///     }
///     .into_any()
/// }).expect("the view name is unique");
///
/// let navigation = CrudNavigation::new(CrudView::table());
/// navigation.navigate(
///     CrudView::new("app.audit")
///         .with_typed_payload("Entity history".to_owned())
///         .expect("payload serializes"),
/// );
/// ```
#[derive(Clone)]
pub struct CrudViewRegistry {
    renderers: HashMap<String, RegisteredRenderer>,
}

impl fmt::Debug for CrudViewRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrudViewRegistry")
            .field("names", &self.renderers.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl CrudViewRegistry {
    pub(crate) fn empty() -> Self {
        Self {
            renderers: HashMap::new(),
        }
    }

    pub(crate) fn insert_default(
        &mut self,
        name: &'static str,
        renderer: impl Fn(CrudView, CrudNavigation) -> AnyView + Send + Sync + 'static,
    ) {
        let previous = self
            .renderers
            .insert(name.to_owned(), RegisteredRenderer::new(renderer));
        debug_assert!(previous.is_none(), "default renderer names must be unique");
    }

    /// Registers a new application-defined view renderer.
    ///
    /// # Errors
    ///
    /// Returns [`DuplicateCrudViewError`] when `name` is already occupied,
    /// including by one of CrudKit's reserved built-in views.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        renderer: impl Fn(CrudView, CrudNavigation) -> AnyView + Send + Sync + 'static,
    ) -> Result<(), DuplicateCrudViewError> {
        let name = name.into();
        if self.renderers.contains_key(&name) {
            return Err(DuplicateCrudViewError { name });
        }
        self.renderers
            .insert(name, RegisteredRenderer::new(renderer));
        Ok(())
    }

    /// Deliberately installs or replaces a renderer, including a built-in one.
    ///
    /// Replacing a built-in also transfers payload and subject validation to
    /// the replacement renderer.
    pub fn replace(
        &mut self,
        name: impl Into<String>,
        renderer: impl Fn(CrudView, CrudNavigation) -> AnyView + Send + Sync + 'static,
    ) {
        self.renderers
            .insert(name.into(), RegisteredRenderer::new(renderer));
    }

    /// Renders a registered view using the supplied scoped navigation.
    ///
    /// This can be called by custom renderers to compose registered views in a
    /// drawer, panel, or other application-owned host. Missing registrations
    /// render a visible alert instead of invoking a fallback renderer.
    ///
    /// # Panics
    ///
    /// CrudKit's built-in renderers require the
    /// [`CrudInstanceContext`](crate::instance::CrudInstanceContext) that
    /// [`crate::components::instance::CrudInstance`] provides. Rendering a built-in
    /// outside an instance violates that invariant and panics.
    #[must_use]
    pub fn render(&self, view: CrudView, navigation: CrudNavigation) -> AnyView {
        let registered = match self.renderer_for(&view.name) {
            Ok(renderer) => renderer,
            Err(message) => return render_error(&view.name, message),
        };

        (registered.renderer)(view, navigation)
    }

    fn renderer_for(&self, name: &str) -> Result<RegisteredRenderer, String> {
        self.renderers
            .get(name)
            .cloned()
            .ok_or_else(|| format!("No CrudKit renderer is registered for `{name}`"))
    }
}

pub(crate) fn render_error(name: &str, message: String) -> AnyView {
    tracing::error!(view = name, error = %message, "Could not render CrudKit view");
    view! {
        <div class="crud-view-error" role="alert">
            {message}
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use crudkit_web::view::TABLE_VIEW;
    use leptos::reactive::owner::Owner;
    use std::sync::atomic::{AtomicBool, Ordering};

    fn renderer(_: CrudView, _: CrudNavigation) -> AnyView {
        ().into_any()
    }

    #[test]
    fn duplicate_registration_is_rejected_and_replace_is_explicit() {
        let mut registry = CrudViewRegistry::default();
        registry
            .register("example.custom", renderer)
            .expect("first registration should work");
        let duplicate = registry
            .register("example.custom", renderer)
            .expect_err("duplicate should fail");
        assert_that!(duplicate.name()).is_equal_to("example.custom");

        registry.replace("example.custom", renderer);
        registry.replace(TABLE_VIEW, renderer);
    }

    #[test]
    fn unknown_views_are_rejected() {
        let registry = CrudViewRegistry::default();
        assert_that!(registry.renderer_for("example.unknown").is_err()).is_true();
    }

    #[test]
    fn custom_views_use_the_same_renderer_path_as_defaults() {
        let owner = Owner::new();
        owner.with(|| {
            let mut registry = CrudViewRegistry::default();
            let rendered = Arc::new(AtomicBool::new(false));
            let rendered_by_view = rendered.clone();
            registry
                .register("example.custom", move |_, _| {
                    rendered_by_view.store(true, Ordering::SeqCst);
                    ().into_any()
                })
                .expect("custom view should register");
            let navigation = CrudNavigation::new(CrudView::table());

            let _ = registry.render(CrudView::new("example.custom"), navigation);
            assert_that!(rendered.load(Ordering::SeqCst)).is_true();
        });
    }

    #[test]
    fn application_replacements_override_builtin_renderers_and_validation() {
        let owner = Owner::new();
        owner.with(|| {
            let mut registry = CrudViewRegistry::default();
            let replacement_rendered = Arc::new(AtomicBool::new(false));
            let replacement_rendered_by_view = replacement_rendered.clone();
            registry.replace(TABLE_VIEW, move |_, _| {
                replacement_rendered_by_view.store(true, Ordering::SeqCst);
                ().into_any()
            });

            let _ = registry.render(
                CrudView::table().with_payload(serde_json::json!({})),
                CrudNavigation::new(CrudView::table()),
            );

            assert_that!(replacement_rendered.load(Ordering::SeqCst)).is_true();
        });
    }
}
