//! State of one rendered field.

use crate::hooks::instance::use_crud_instance;
use crudkit_core::Value;
use crudkit_web::field::{FieldMode, FieldOptions};
use crudkit_web::prelude::*;
use leptos::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

/// All reactive field values of one entity, keyed by field.
pub type CrudFieldMap<F> = StoredValue<HashMap<F, ReactiveField>>;

/// Returns a unique DOM id for a field's input element.
pub(crate) fn dom_id() -> String {
    format!("f{}", Uuid::new_v4().simple())
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

/// The fields of one read entity, for displaying it, e.g. in a table row.
///
/// The read-only counterpart of a form: it provides values and the [`CrudFieldState`] that
/// registered field renderers expect.
#[derive(Clone, Copy, Debug)]
pub struct CrudRowFields {
    fields: CrudFieldMap<DynReadField>,
}

impl CrudRowFields {
    /// Returns the value of `field`, unless the read model has no such field.
    #[must_use]
    pub fn value(&self, field: &DynReadField) -> Option<Value> {
        self.fields
            .with_value(|fields| fields.get(field).map(ReactiveField::get_untracked))
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
            dom_id: dom_id(),
        })
    }
}

/// Returns the fields of `entity` for display.
///
/// # Panics
///
/// Panics when called outside of a CrudKit instance.
#[must_use]
pub fn use_crud_row_fields(entity: &DynReadModel) -> CrudRowFields {
    let ctx = use_crud_instance();
    let fields = reactive_fields(
        ctx.static_config
            .read_value()
            .model_handler
            .read_model_values(entity),
    );
    CrudRowFields {
        fields: StoredValue::new(fields),
    }
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
