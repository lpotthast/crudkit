//! State of one rendered field.

use crate::hooks::instance::use_crud_instance;
use crudkit_core::{Value, ValueKind};
use crudkit_web::field::{FieldMode, FieldOptions};
use crudkit_web::prelude::*;
use leptonic::utils::id::use_id;
use leptos::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// All reactive field values of one entity, keyed by field.
pub type CrudFieldMap<F> = StoredValue<HashMap<F, ReactiveField>>;

/// Returns a document-unique DOM id for a field's input element, the same on the server and while
/// hydrating. Call it while creating a component, see Leptonic's `use_id`.
pub(crate) fn dom_id() -> String {
    use_id("crudkit-field")
}

/// Error reported by an input that could not turn user input into a [`Value`].
pub type CrudFieldInputError = Arc<dyn std::error::Error>;

/// Everything a field renderer needs to render and edit one field.
///
/// Field renderers receive this state as their argument. Components rendered below a field can
/// retrieve it with [`use_crud_field`].
#[derive(Clone)]
pub struct CrudFieldState<F: TypeErasedField> {
    /// The rendered field.
    pub field: F,
    /// The field's current value.
    pub value: ReactiveField,
    /// How the field is presented.
    pub mode: FieldMode,
    /// Layout-supplied options such as the label.
    pub options: FieldOptions,
    /// Values of all fields of the same entity, for renderers that depend on other fields.
    pub fields: CrudFieldMap<F>,
    /// Reports a new value, or an input that could not be converted into one.
    pub set: Callback<Result<Value, CrudFieldInputError>>,
    /// The field's current input error, if any.
    pub error: Signal<Option<String>>,
    /// Whether the field's form is saving. Inputs are read-only meanwhile.
    pub is_saving: Signal<bool>,
    /// Unique DOM id for the field's input element.
    pub dom_id: String,
}

impl<F: TypeErasedField> fmt::Debug for CrudFieldState<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrudFieldState")
            .field("field", &self.field)
            .field("mode", &self.mode)
            .field("options", &self.options)
            .field("dom_id", &self.dom_id)
            .finish_non_exhaustive()
    }
}

impl<F: TypeErasedField> CrudFieldState<F> {
    /// Returns whether the field accepts `Value::Null`.
    pub fn is_optional(&self) -> bool {
        self.field.is_optional()
    }

    /// Returns whether the user may change the value.
    pub fn is_editable(&self) -> bool {
        self.mode == FieldMode::Editable && !self.options.disabled
    }

    /// Reports `value` as the field's new value.
    pub fn set_value(&self, value: Value) {
        self.set.run(Ok(value));
    }
}

/// Returns the state of the field currently being rendered.
///
/// # Panics
///
/// Panics when called outside of a field renderer for fields of type `F`.
#[must_use]
pub fn use_crud_field<F: TypeErasedField>() -> CrudFieldState<F> {
    use_context::<CrudFieldState<F>>()
        .expect("`use_crud_field` must be called while rendering a CrudKit field of this type")
}

/// The field being rendered, whatever its model: what inputs bind to.
///
/// Provided next to the [`CrudFieldState`] of a field, e.g. by
/// [`CrudField`](crate::atoms::CrudField) and while a field renderer runs. Read it with
/// [`use_crud_field_binding`]; the input hooks (e.g.
/// [`use_crud_text_input`](crate::hooks::use_crud_text_input)) build on it.
#[derive(Clone)]
pub struct CrudFieldBinding {
    /// The field's name.
    pub name: String,
    /// The kind of value the field holds.
    pub value_kind: ValueKind,
    /// Whether the field accepts `Value::Null`.
    pub is_optional: bool,
    /// The field's current value.
    pub value: ReactiveField,
    /// How the field is presented.
    pub mode: FieldMode,
    /// Layout-supplied options such as the label.
    pub options: FieldOptions,
    /// Reports a new value, or an input that could not be converted into one.
    pub set: Callback<Result<Value, CrudFieldInputError>>,
    /// Why the field's current input could not be read, if it could not.
    pub error: Signal<Option<String>>,
    /// Whether the field's form is saving. Inputs are read-only meanwhile.
    pub is_saving: Signal<bool>,
    /// Unique DOM id for the field's input element.
    pub dom_id: String,
    /// Whether the instance configuration registers a
    /// [`FieldRenderer`](crate::config::FieldRenderer) for the field, e.g. to give such fields
    /// more room.
    pub has_renderer: bool,
}

impl fmt::Debug for CrudFieldBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrudFieldBinding")
            .field("name", &self.name)
            .field("value_kind", &self.value_kind)
            .field("mode", &self.mode)
            .field("dom_id", &self.dom_id)
            .finish_non_exhaustive()
    }
}

impl CrudFieldBinding {
    /// Returns whether the user may change the value.
    #[must_use]
    pub fn is_editable(&self) -> bool {
        self.mode == FieldMode::Editable && !self.options.disabled
    }

    /// Returns the field's label: the one its layout gives, else its name.
    #[must_use]
    pub fn label(&self) -> String {
        self.options
            .label
            .as_ref()
            .map_or_else(|| self.name.clone(), |label| label.name.clone())
    }
}

impl<F: TypeErasedField> From<&CrudFieldState<F>> for CrudFieldBinding {
    fn from(state: &CrudFieldState<F>) -> Self {
        Self {
            name: state.field.name().to_string(),
            value_kind: state.field.value_kind(),
            is_optional: state.field.is_optional(),
            value: state.value,
            mode: state.mode,
            options: state.options.clone(),
            set: state.set,
            error: state.error,
            is_saving: state.is_saving,
            dom_id: state.dom_id.clone(),
            has_renderer: false,
        }
    }
}

/// Provides `state` and its [`CrudFieldBinding`] to the current owner of a field renderer.
pub(crate) fn provide_field<F: TypeErasedField>(state: &CrudFieldState<F>) {
    provide_context(CrudFieldBinding {
        has_renderer: true,
        ..CrudFieldBinding::from(state)
    });
    provide_context(state.clone());
}

/// Returns the field being rendered, whatever its model.
///
/// # Panics
///
/// Panics when called outside of a field, e.g. a [`CrudField`](crate::atoms::CrudField) or a
/// field renderer.
#[must_use]
pub fn use_crud_field_binding() -> CrudFieldBinding {
    use_context::<CrudFieldBinding>()
        .expect("`use_crud_field_binding` must be called inside a CrudKit field")
}

/// The fields of one read entity, for displaying it, e.g. in a table row.
///
/// The read-only counterpart of a form: it provides values and the [`CrudFieldState`] that
/// registered field renderers expect.
#[derive(Clone, Copy, Debug)]
pub struct CrudRowFields {
    fields: CrudFieldMap<DynReadField>,
}

impl CrudRowFields {
    /// Returns the value of `field`, unless the read model has no such field. Tracks the value.
    #[must_use]
    pub fn value(&self, field: &DynReadField) -> Option<Value> {
        self.fields
            .with_value(|fields| fields.get(field).copied())
            .map(|value| value.get())
    }

    /// Returns the state for displaying `field`, e.g. with
    /// [`crate::config::FieldRenderer::render`], unless the read model has no such field.
    #[must_use]
    pub fn display_state(
        &self,
        field: &DynReadField,
        options: FieldOptions,
    ) -> Option<CrudFieldState<DynReadField>> {
        let value = self
            .fields
            .with_value(|fields| fields.get(field).copied())?;
        Some(CrudFieldState {
            field: field.clone(),
            value,
            mode: FieldMode::Display,
            options,
            fields: self.fields,
            set: Callback::new(|_| {}),
            error: Signal::stored(None),
            is_saving: Signal::stored(false),
            dom_id: dom_id(),
        })
    }
}

/// Returns the fields of `entity` for display. Their values follow `entity` when it changes, e.g.
/// after a reload, so rendered fields update in place.
///
/// # Panics
///
/// Panics when called outside of a CrudKit instance.
#[must_use]
pub fn use_crud_row_fields(entity: impl Into<Signal<DynReadModel>>) -> CrudRowFields {
    let entity = entity.into();
    let ctx = use_crud_instance();
    let values_of = move |entity: &DynReadModel| {
        ctx.static_config
            .read_value()
            .model_handler
            .read_model_values(entity)
    };
    let initial = entity.get_untracked();
    let fields = StoredValue::new(reactive_fields(values_of(&initial)));
    Effect::watch(
        move || entity.get(),
        move |entity, previous, _| {
            // Until the entity changes, the fields hold its values already.
            if previous.is_none() && *entity == initial {
                return;
            }
            fields.with_value(|fields| {
                for (field, value) in values_of(entity) {
                    if let Some(reactive) = fields.get(&field) {
                        reactive.set(value);
                    }
                }
            });
        },
        true,
    );
    CrudRowFields { fields }
}

/// The reactive value of one form field.
///
/// This provides fine-grained reactivity for individual fields. `Value::Null` represents an absent
/// value of an optional field. Applications read it; changes go through the form
/// ([`crate::hooks::field::CrudFieldState::set`] or [`crate::hooks::form::CrudFormState::set_field`]),
/// which keeps the field and the form's draft consistent.
#[derive(Debug, Clone, Copy)]
pub struct ReactiveField {
    value: RwSignal<Value>,
}

impl ReactiveField {
    /// Creates a new `ReactiveField` with the given initial value.
    #[must_use]
    pub fn new(value: Value) -> Self {
        Self {
            value: RwSignal::new(value),
        }
    }

    /// Sets the value. Only the form may do so, after updating its draft.
    pub(crate) fn set(&self, v: Value) {
        self.value.set(v);
    }

    /// Gets the current value.
    #[must_use]
    pub fn get(&self) -> Value {
        self.value.get()
    }

    /// Gets the current value without tracking.
    #[must_use]
    pub fn get_untracked(&self) -> Value {
        self.value.get_untracked()
    }

    /// Applies `f` to the current value without cloning it.
    pub fn with<T>(&self, f: impl FnOnce(&Value) -> T) -> T {
        self.value.with(f)
    }
}

impl From<ReactiveField> for Signal<Value> {
    fn from(field: ReactiveField) -> Self {
        field.value.into()
    }
}

/// Creates one [`ReactiveField`] per field, initialized with the field's value.
///
/// Use it with the field values of a [`crudkit_web::model_handler::ModelHandler`].
pub(crate) fn reactive_fields<F: Eq + std::hash::Hash>(
    values: Vec<(F, Value)>,
) -> HashMap<F, ReactiveField> {
    values
        .into_iter()
        .map(|(field, value)| (field, ReactiveField::from(value)))
        .collect()
}

impl From<Value> for ReactiveField {
    fn from(v: Value) -> Self {
        ReactiveField::new(v)
    }
}
