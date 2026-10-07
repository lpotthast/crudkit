//! Hosting of fields: CrudKit's field renderers, or application markup using the input hooks.

use crate::config::FieldRendererRegistry;
use crate::hooks::field::{CrudFieldInputError, CrudFieldState, dom_id};
use crate::hooks::form::CrudFormState;
use crudkit_core::Value;
use crudkit_web::field::{FieldMode, FieldOptions};
use crudkit_web::prelude::*;
use leptos::prelude::*;

/// Returns the state of `field` in `form`, or `None` if the form has no value for it.
fn form_field_state<F: TypeErasedField>(
    form: CrudFormState<F>,
    field: &F,
    options: FieldOptions,
    mode: FieldMode,
) -> Option<CrudFieldState<F>> {
    let Some(value) = form.field(field) else {
        tracing::error!(field = %field.name(), "the layout references a field the model does not have");
        return None;
    };
    let target = field.clone();
    Some(CrudFieldState {
        field: field.clone(),
        value,
        mode,
        options,
        fields: form.fields.get_untracked(),
        set: Callback::new(move |result: Result<Value, CrudFieldInputError>| {
            form.set_field(target.clone(), result.map_err(|err| err.to_string()));
        }),
        error: form.field_error(field.clone()),
        dom_id: dom_id(),
    })
}

/// Renders `state` with its registered renderer, or the default renderer for its value kind,
/// inside `div.crudkit-field`.
pub(crate) fn hosted_field<F: TypeErasedField>(
    field_renderer_registry: Signal<FieldRendererRegistry<F>>,
    state: CrudFieldState<F>,
) -> impl IntoView {
    let mode = match state.mode {
        FieldMode::Display => "display",
        FieldMode::Readable => "readable",
        FieldMode::Editable => "editable",
    };
    let field = state.field.clone();
    let renderer = Signal::derive(move || field_renderer_registry.read().resolve(&field));
    let error = state.error;
    view! {
        <div
            class="crudkit-field"
            data-mode=mode
            data-invalid=move || error.read().is_some().then_some("true")
        >
            {move || renderer.get().render(state.clone())}
        </div>
    }
}

/// Renders `field` of `form` with its registered renderer, or the default renderer for its value
/// kind, inside `div.crudkit-field` with `data-mode` (`display`, `readable`, `editable`) and
/// `data-invalid`.
pub(crate) fn form_field<F: TypeErasedField>(
    field_renderer_registry: Signal<FieldRendererRegistry<F>>,
    form: CrudFormState<F>,
    field: &F,
    options: FieldOptions,
    mode: FieldMode,
) -> AnyView {
    match form_field_state(form, field, options, mode) {
        Some(state) => hosted_field(field_renderer_registry, state).into_any(),
        None => view! {
            <div class="crudkit-field-error" role="alert">
                {format!("Unknown field: {}", field.name())}
            </div>
        }
        .into_any(),
    }
}

/// Makes `field` of `form` the field of `children`, without rendering any markup.
///
/// Inside, [`crate::hooks::field::use_crud_field`] and the input hooks in [`crate::hooks::inputs`]
/// bind application markup to the field. Children are rendered in their own reactive owner and are
/// recreated when the form loads another entity.
#[component]
pub fn CrudFormField<F: TypeErasedField>(
    /// The form holding the field's value.
    form: CrudFormState<F>,
    /// The field to bind. Nothing is rendered if the form has no value for it.
    field: F,
    /// Label, disabled state, and date-time formatting of the field. Defaults to an enabled field
    /// without label.
    #[prop(optional)]
    options: FieldOptions,
    /// Whether the field is shown as plain value, read-only input, or editable input.
    mode: FieldMode,
    children: ChildrenFn,
) -> impl IntoView {
    move || {
        form_field_state(form, &field, options.clone(), mode).map(|state| {
            provide_context(state);
            children()
        })
    }
}
