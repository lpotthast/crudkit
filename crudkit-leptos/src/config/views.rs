//! Registry of the views an instance can show.

use crudkit_core::id::SerializableId;
use crudkit_web::view::{CREATE_VIEW, CrudView, EDIT_VIEW, READ_VIEW, TABLE_VIEW};
use leptos::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

type ViewRenderer = Arc<dyn Fn(CrudView) -> AnyView + Send + Sync + 'static>;

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

/// Maps the names of [`CrudView`]s to the markup rendering them.
///
/// An instance's [`CrudViewOutlet`](crate::instance::CrudViewOutlet) renders the current view
/// with the renderer registered for its name. A view finds its instance and navigation in context,
/// e.g. through [`use_crud_instance`](crate::hooks::use_crud_instance).
///
/// The four standard views are registered with [`Self::table`], [`Self::create`], [`Self::read`],
/// and [`Self::edit`]. These check that a view is opened with the subject it needs (an entity id
/// for `read` and `edit`, none otherwise) and without a payload, and render a visible error
/// otherwise. Application-defined views are added with [`Self::register`]. A default registry is
/// empty; opening a view without a renderer shows a visible error.
///
/// ```no_run
/// # use crudkit_leptos::prelude::*;
/// # use leptos::prelude::*;
/// let mut registry = CrudViewRegistry::default()
///     .table(|| view! { <p>"All entities"</p> })
///     .edit(|id: SerializableId| view! { <p>{format!("Editing {id:?}")}</p> });
/// registry
///     .register("app.audit", |view: CrudView| {
///         let title = view
///             .typed_payload::<String>()
///             .unwrap_or_else(|_| "Audit".to_owned());
///         view! { <h2>{title}</h2> }
///     })
///     .expect("the view name is unique");
/// ```
#[derive(Clone, Default)]
pub struct CrudViewRegistry {
    renderers: HashMap<String, ViewRenderer>,
}

impl fmt::Debug for CrudViewRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrudViewRegistry")
            .field("names", &self.renderers.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl CrudViewRegistry {
    /// Renders the list view ([`CrudView::table`]) with `render`.
    #[must_use]
    pub fn table<V: IntoView + 'static>(
        self,
        render: impl Fn() -> V + Send + Sync + 'static,
    ) -> Self {
        self.with_standard(TABLE_VIEW, move |view| {
            check_without_subject(&view)?;
            Ok(render().into_any())
        })
    }

    /// Renders the create view ([`CrudView::create`]) with `render`.
    #[must_use]
    pub fn create<V: IntoView + 'static>(
        self,
        render: impl Fn() -> V + Send + Sync + 'static,
    ) -> Self {
        self.with_standard(CREATE_VIEW, move |view| {
            check_without_subject(&view)?;
            Ok(render().into_any())
        })
    }

    /// Renders the read view of an entity ([`CrudView::read`]) with `render`, given its id.
    #[must_use]
    pub fn read<V: IntoView + 'static>(
        self,
        render: impl Fn(SerializableId) -> V + Send + Sync + 'static,
    ) -> Self {
        self.with_standard(READ_VIEW, move |view| {
            Ok(render(entity_subject(view)?).into_any())
        })
    }

    /// Renders the edit view of an entity ([`CrudView::edit`]) with `render`, given its id.
    #[must_use]
    pub fn edit<V: IntoView + 'static>(
        self,
        render: impl Fn(SerializableId) -> V + Send + Sync + 'static,
    ) -> Self {
        self.with_standard(EDIT_VIEW, move |view| {
            Ok(render(entity_subject(view)?).into_any())
        })
    }

    /// Registers the standard view `name`, rendered by `render` unless the opened view does not
    /// fit it, which shows a visible error instead.
    fn with_standard(
        mut self,
        name: &'static str,
        render: impl Fn(CrudView) -> Result<AnyView, String> + Send + Sync + 'static,
    ) -> Self {
        self.renderers.insert(
            name.to_owned(),
            Arc::new(move |view: CrudView| {
                render(view).unwrap_or_else(|message| {
                    view! { <ViewError view=name.to_owned() message /> }.into_any()
                })
            }),
        );
        self
    }

    /// Registers the renderer of the application-defined view `name`. It receives the opened
    /// [`CrudView`], with its subject and payload.
    ///
    /// # Errors
    ///
    /// Returns [`DuplicateCrudViewError`] when `name` already has a renderer.
    pub fn register<V: IntoView + 'static>(
        &mut self,
        name: impl Into<String>,
        render: impl Fn(CrudView) -> V + Send + Sync + 'static,
    ) -> Result<(), DuplicateCrudViewError> {
        let name = name.into();
        if self.renderers.contains_key(&name) {
            return Err(DuplicateCrudViewError { name });
        }
        self.replace(name, render);
        Ok(())
    }

    /// Installs or replaces the renderer of the view `name`.
    pub fn replace<V: IntoView + 'static>(
        &mut self,
        name: impl Into<String>,
        render: impl Fn(CrudView) -> V + Send + Sync + 'static,
    ) {
        self.renderers.insert(
            name.into(),
            Arc::new(move |view: CrudView| render(view).into_any()),
        );
    }

    /// Renders `view` with its registered renderer, or a visible error when it has none.
    pub(crate) fn render(&self, view: CrudView) -> AnyView {
        if let Some(render) = self.renderers.get(&view.name) {
            render(view)
        } else {
            let message = format!("No CrudKit renderer is registered for `{}`", view.name);
            view! { <ViewError view=view.name message /> }.into_any()
        }
    }
}

/// Checks that a standard view without an entity has neither a subject nor a payload.
fn check_without_subject(view: &CrudView) -> Result<(), String> {
    check_without_payload(view)?;
    if view.subject.is_some() {
        return Err(format!(
            "The view `{}` does not accept an entity subject",
            view.name
        ));
    }
    Ok(())
}

/// Returns the entity subject of a standard view of an entity, which has no payload.
fn entity_subject(view: CrudView) -> Result<SerializableId, String> {
    check_without_payload(&view)?;
    view.subject
        .ok_or_else(|| format!("The view `{}` requires an entity subject", view.name))
}

/// Checks that a standard view has no payload.
fn check_without_payload(view: &CrudView) -> Result<(), String> {
    if view.payload.is_null() {
        Ok(())
    } else {
        Err(format!(
            "The view `{}` does not accept a payload",
            view.name
        ))
    }
}

/// A visible error in place of the view named `view`, explained by `message`.
#[component]
fn ViewError(view: String, message: String) -> impl IntoView {
    tracing::error!(view, error = %message, "Could not render CrudKit view");
    view! {
        <div class="crudkit-ViewError" role="alert">
            {message}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use leptos::reactive::owner::Owner;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn duplicate_registration_is_rejected_and_replace_is_explicit() {
        let mut registry = CrudViewRegistry::default();
        registry
            .register("example.custom", |_: CrudView| ())
            .expect("first registration should work");
        let duplicate = registry
            .register("example.custom", |_: CrudView| ())
            .expect_err("duplicate should fail");
        assert_that!(duplicate.name()).is_equal_to("example.custom");

        registry.replace("example.custom", |_: CrudView| ());
        let mut registry = registry.table(|| ());
        assert_that!(registry.register(TABLE_VIEW, |_: CrudView| ()).is_err()).is_true();
    }

    #[test]
    fn views_render_with_their_registered_renderer() {
        Owner::new().with(|| {
            let rendered = Arc::new(AtomicUsize::new(0));
            let (custom, edit) = (rendered.clone(), rendered.clone());
            let mut registry = CrudViewRegistry::default().edit(move |_| {
                edit.fetch_add(1, Ordering::SeqCst);
            });
            registry
                .register("example.custom", move |_: CrudView| {
                    custom.fetch_add(10, Ordering::SeqCst);
                })
                .expect("custom view should register");

            let _ = registry.render(CrudView::new("example.custom"));
            let _ = registry.render(CrudView::edit(SerializableId(Vec::new())));
            let _ = registry.render(CrudView::new("example.unknown"));
            assert_that!(rendered.load(Ordering::SeqCst)).is_equal_to(11);
        });
    }

    #[test]
    fn standard_views_require_their_subject_and_no_payload() {
        Owner::new().with(|| {
            let rendered = Arc::new(AtomicUsize::new(0));
            let (table, edit) = (rendered.clone(), rendered.clone());
            let registry = CrudViewRegistry::default()
                .table(move || {
                    table.fetch_add(1, Ordering::SeqCst);
                })
                .edit(move |_| {
                    edit.fetch_add(1, Ordering::SeqCst);
                });

            let _ = registry.render(CrudView::new(EDIT_VIEW));
            let _ = registry.render(CrudView::table().with_payload(serde_json::json!({})));
            let _ = registry.render(CrudView {
                subject: Some(SerializableId(Vec::new())),
                ..CrudView::table()
            });
            assert_that!(rendered.load(Ordering::SeqCst)).is_equal_to(0);

            let _ = registry.render(CrudView::table());
            assert_that!(rendered.load(Ordering::SeqCst)).is_equal_to(1);
        });
        assert_that!(check_without_subject(&CrudView::create()).is_ok()).is_true();
        assert_that!(entity_subject(CrudView::edit(SerializableId(Vec::new()))).is_ok()).is_true();
        assert_that!(entity_subject(CrudView::new(EDIT_VIEW)).is_err()).is_true();
    }
}
