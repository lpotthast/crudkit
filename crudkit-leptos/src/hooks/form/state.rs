//! The state of a form: its draft, the baseline the draft is compared against, the values of its
//! fields, why its last save failed, and where its fields are rendered.

use crate::config::CrudUiTexts;
use crate::hooks::field::{CrudFieldMap, ReactiveField, reactive_fields};
use crate::hooks::tabs::TabLocation;
use crate::hooks::texts::use_crud_texts;
use crudkit_core::Value;
use crudkit_web::data_source::{
    CrudFieldProblem, apply_merge_patch, merge_patch, merge_patch_conflicts,
};
use crudkit_web::http::RequestError;
use crudkit_web::json_pointer::split_first_token;
use crudkit_web::model_handler::ModelHandler;
use crudkit_web::prelude::*;
use leptonic::Id;
use leptos::prelude::*;
use leptos::reactive::owner::Owner;
use std::collections::HashMap;
use std::fmt;

/// A member of an entity that both the user and someone else changed, found when a reload applied
/// the user's unsaved changes to the reloaded version. The draft keeps the user's value, so saving
/// replaces the other change.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CrudFieldConflict {
    /// The name of the field, as the model's field enum names it, e.g. `tags`.
    pub field: String,
    /// The JSON pointer (RFC 6901) of the member into the entity as the model serializes it, e.g.
    /// `/tags/team`. It starts with the field.
    pub pointer: String,
    /// The member's value in the reloaded version. `None` if someone else removed the member.
    pub theirs: Option<serde_json::Value>,
}

impl CrudFieldConflict {
    /// Returns the part of the pointer below the field, e.g. `/team` for `/tags/team`. Empty for
    /// a conflict about the field as a whole.
    #[must_use]
    pub fn path_in_field(&self) -> &str {
        split_first_token(&self.pointer).map_or("", |(_, rest)| rest)
    }
}

/// Where a field of a form is rendered: the element its error summary links to, its label, and
/// the tabs to select to show it.
#[derive(Debug, Clone)]
pub(crate) struct FieldTarget {
    pub(crate) dom_id: Id,
    pub(crate) label: Option<String>,
    pub(crate) tabs: Vec<TabLocation>,
}

/// The owners of a form's state: the form's own, and the one of the values of the fields of the
/// entity it shows, which the form disposes when it loads another entity.
#[derive(Debug, Default)]
struct FieldOwners {
    form: Option<Owner>,
    values: Option<Owner>,
}

/// Why the last save of a form failed, and the problems and conflicts of fields that still apply.
#[derive(Debug, Clone, Default, PartialEq)]
struct SaveFailure {
    /// `None` after a successful save, and once a reload resolved a save refused as outdated.
    error: Option<RequestError>,
    problems: Vec<CrudFieldProblem>,
    conflicts: Vec<CrudFieldConflict>,
}

/// Draft state of a form editing one entity, with one [`ReactiveField`] per field.
///
/// The form keeps the complete draft model for saving and dirty checks, and one reactive value per
/// field for fine-grained rendering. [`Self::set_field`] keeps both in sync.
pub struct CrudFormState<F: TypeErasedField> {
    /// The reactive values of all fields. Changes when another entity loads.
    pub fields: Signal<CrudFieldMap<F>>,
    /// The current draft, or `None` while no entity is loaded.
    pub draft: Signal<Option<F::Model>>,
    /// Input errors keyed by field.
    pub errors: Signal<HashMap<F, String>>,
    /// Why the last save failed, while it applies. A successful save and loading another entity
    /// clear it, and so does reloading the entity after a save was refused as
    /// [outdated](RequestError::Outdated).
    pub save_error: Signal<Option<RequestError>>,
    /// The problems of fields that the last save reported ([`RequestError::InvalidFields`]),
    /// without those of fields the user changed since. They do not keep the form from saving.
    pub problems: Signal<Vec<CrudFieldProblem>>,
    /// The members that someone else changed too, found when reloads applied the user's unsaved
    /// changes to the reloaded versions, without those of fields the user changed since. A
    /// successful save and loading another entity clear them.
    pub conflicts: Signal<Vec<CrudFieldConflict>>,
    /// Whether the draft differs from the loaded or default entity.
    pub is_dirty: Signal<bool>,
    /// Whether the form holds an entity. Field values exist only while this is `true`.
    pub is_ready: Signal<bool>,
    /// Whether some field holds input that could not be converted into a value. Such a draft is
    /// not saved, because it does not reflect what the user sees.
    pub has_errors: Signal<bool>,
    /// Whether a save request is in flight. Fields are read-only meanwhile, so that the saved
    /// entity, which becomes the form's new state, cannot overwrite input made during the request.
    pub is_saving: Signal<bool>,
    field_values: RwSignal<CrudFieldMap<F>>,
    draft_model: RwSignal<Option<F::Model>>,
    /// The entity as the data source returned it last, which the draft is compared against.
    pub(super) baseline: RwSignal<Option<F::Model>>,
    input_errors: RwSignal<HashMap<F, String>>,
    failure: RwSignal<SaveFailure>,
    owners: StoredValue<FieldOwners>,
    /// Where the fields of the form are rendered, by the names of the fields.
    targets: RwSignal<HashMap<String, FieldTarget>>,
    /// The number of mounted error summaries that show why the form's saves failed.
    error_summaries: StoredValue<usize>,
}

// Deriving `Clone` and `Copy` would require `F: Clone` and `F: Copy`. The handle is `Copy` for every
// field type, so both are implemented manually.
#[allow(clippy::expl_impl_clone_on_copy)]
impl<F: TypeErasedField> Clone for CrudFormState<F> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<F: TypeErasedField> Copy for CrudFormState<F> {}

impl<F: TypeErasedField> fmt::Debug for CrudFormState<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrudFormState").finish_non_exhaustive()
    }
}

impl<F: TypeErasedField> CrudFormState<F> {
    /// Creates the state of a form in the current owner, which also owns the values of its fields.
    /// The form never saves until [`Self::saving_while`] gives it its saving state.
    pub(super) fn new() -> Self {
        let draft = RwSignal::new(None::<F::Model>);
        let baseline = RwSignal::new(None::<F::Model>);
        let errors = RwSignal::new(HashMap::new());
        let fields = RwSignal::new(StoredValue::new(HashMap::new()));
        let failure = RwSignal::new(SaveFailure::default());
        Self {
            fields: fields.into(),
            draft: draft.into(),
            errors: errors.into(),
            save_error: Memo::new(move |_| failure.read().error.clone()).into(),
            problems: Memo::new(move |_| failure.read().problems.clone()).into(),
            conflicts: Memo::new(move |_| failure.read().conflicts.clone()).into(),
            is_dirty: Memo::new(move |_| match (&*draft.read(), &*baseline.read()) {
                (Some(draft), Some(baseline)) => draft != baseline,
                _ => false,
            })
            .into(),
            // A memo, so that views keyed on readiness do not re-render on every draft change.
            is_ready: Memo::new(move |_| draft.read().is_some()).into(),
            has_errors: Memo::new(move |_| !errors.read().is_empty()).into(),
            is_saving: Signal::stored(false),
            field_values: fields,
            draft_model: draft,
            baseline,
            input_errors: errors,
            failure,
            owners: StoredValue::new(FieldOwners {
                form: Owner::current(),
                values: None,
            }),
            targets: RwSignal::new(HashMap::new()),
            error_summaries: StoredValue::new(0),
        }
    }

    /// Returns this state, which saves while `is_saving` is `true`.
    pub(super) fn saving_while(self, is_saving: Signal<bool>) -> Self {
        Self { is_saving, ..self }
    }

    /// Returns the reactive value of `field`. Tracks loads of other entities.
    pub fn field(&self, field: &F) -> Option<ReactiveField> {
        self.fields
            .get()
            .with_value(|fields| fields.get(field).copied())
    }

    /// Returns why `field` is invalid, if it is: its input error, or else the problems that the
    /// last save reported for it and its conflicts with changes of someone else.
    pub fn field_error(&self, field: F) -> Signal<Option<String>> {
        let (errors, problems, conflicts) = (self.errors, self.problems, self.conflicts);
        let texts = use_crud_texts();
        Signal::derive(move || {
            if let Some(error) = errors.read().get(&field) {
                return Some(error.clone());
            }
            let name = field.name();
            let mut messages = problems
                .read()
                .iter()
                .filter(|problem| problem.field == name)
                .map(|problem| at_path(problem.path_in_field(), &problem.detail))
                .collect::<Vec<_>>();
            texts.with(|texts| {
                messages.extend(
                    conflicts
                        .read()
                        .iter()
                        .filter(|conflict| conflict.field == name)
                        .map(|conflict| {
                            at_path(conflict.path_in_field(), &conflict_message(texts, conflict))
                        }),
                );
            });
            (!messages.is_empty()).then(|| messages.join(" "))
        })
    }

    /// Applies user input for `field`.
    ///
    /// A value updates both the draft model and the field's reactive value and clears the field's
    /// input error. An error records the input error and leaves the draft unchanged. Either way,
    /// the problems and conflicts of the field no longer apply.
    pub fn set_field(&self, field: F, value: Result<Value, String>) {
        let name = field.name();
        let concerns_field = self.failure.with_untracked(|failure| {
            failure.problems.iter().any(|problem| problem.field == name)
                || failure
                    .conflicts
                    .iter()
                    .any(|conflict| conflict.field == name)
        });
        if concerns_field {
            self.failure.update(|failure| {
                failure.problems.retain(|problem| problem.field != name);
                failure.conflicts.retain(|conflict| conflict.field != name);
            });
        }
        match value {
            Ok(value) => {
                self.draft_model.update(|draft| {
                    if let Some(draft) = draft {
                        field.set_value(draft, value.clone());
                    }
                });
                self.input_errors.update(|errors| {
                    errors.remove(&field);
                });
                if let Some(reactive) = self.field(&field) {
                    reactive.set(value);
                } else {
                    tracing::error!(?field, "form has no reactive value for field");
                }
            }
            Err(err) => self.input_errors.update(|errors| {
                errors.insert(field, err);
            }),
        }
    }

    /// Replaces draft, baseline, and field values with `model`, whose fields have the `values`.
    ///
    /// The values belong to an owner of their own below the form's, so that they outlive the
    /// effect that loads them. Loading the next entity disposes them.
    pub(super) fn load(&self, model: F::Model, values: Vec<(F, Value)>) {
        let owner = self
            .owners
            .with_value(|owners| owners.form.as_ref().map(Owner::child));
        let create = move || StoredValue::new(reactive_fields(values));
        let fields = match &owner {
            Some(owner) => owner.with(create),
            None => create(),
        };
        self.field_values.set(fields);
        self.baseline.set(Some(model.clone()));
        self.draft_model.set(Some(model));
        self.input_errors.update(HashMap::clear);
        self.record_save_error(None);
        let previous = self
            .owners
            .try_update_value(|owners| std::mem::replace(&mut owners.values, owner))
            .flatten();
        if let Some(previous) = previous {
            previous.cleanup();
        }
    }

    /// Replaces draft, baseline, and field values with `model`, whose fields have the `values`.
    /// Existing reactive values take the new values, so that rendered fields update in place.
    pub(super) fn sync(&self, model: F::Model, values: Vec<(F, Value)>) {
        let current = self.fields.get_untracked();
        let owner = self
            .owners
            .with_value(|owners| owners.values.clone().or_else(|| owners.form.clone()));
        for (field, value) in values {
            if let Some(existing) = current.with_value(|current| current.get(&field).copied()) {
                existing.set(value);
                continue;
            }
            let create = move || ReactiveField::new(value);
            let reactive = match &owner {
                Some(owner) => owner.with(create),
                None => create(),
            };
            current.update_value(|current| {
                current.insert(field, reactive);
            });
        }
        self.baseline.set(Some(model.clone()));
        self.draft_model.set(Some(model));
        self.input_errors.update(HashMap::clear);
        self.record_save_error(None);
    }

    /// Records why the last save failed, or `None` after a successful save, which also resolves
    /// the conflicts. The problems of fields that `error` reports replace those of earlier saves.
    pub(crate) fn record_save_error(&self, error: Option<RequestError>) {
        let failure = match error {
            None => SaveFailure::default(),
            Some(error) => SaveFailure {
                problems: match &error {
                    RequestError::InvalidFields { problems, .. } => problems.clone(),
                    _ => Vec::new(),
                },
                conflicts: self
                    .failure
                    .with_untracked(|failure| failure.conflicts.clone()),
                error: Some(error),
            },
        };
        if self.failure.with_untracked(|current| *current != failure) {
            self.failure.set(failure);
        }
    }

    /// Forgets a save refused as outdated, e.g. once the entity is reloaded.
    pub(super) fn clear_outdated(&self) {
        let is_outdated = self
            .failure
            .with_untracked(|failure| matches!(failure.error, Some(RequestError::Outdated(_))));
        if is_outdated {
            self.failure.update(|failure| failure.error = None);
        }
    }

    /// Records where the field named `name` is rendered, until the current owner is cleaned up.
    /// Field atoms call it, so that the form's error summary links to them.
    pub(crate) fn register_field(&self, name: String, target: FieldTarget) {
        let dom_id = target.dom_id.clone();
        self.targets.update(|targets| {
            targets.insert(name.clone(), target);
        });
        let targets = self.targets;
        on_cleanup(move || {
            targets.try_update(|targets| {
                if targets
                    .get(&name)
                    .is_some_and(|target| target.dom_id == dom_id)
                {
                    targets.remove(&name);
                }
            });
        });
    }

    /// Returns where the fields of the form are rendered, by the names of the fields.
    pub(crate) fn field_targets(&self) -> Signal<HashMap<String, FieldTarget>> {
        self.targets.into()
    }

    /// Returns the counter of the error summaries that show why the form's saves failed.
    pub(crate) fn error_summaries(&self) -> StoredValue<usize> {
        self.error_summaries
    }

    /// Returns whether a mounted error summary shows why the form's saves fail, so that
    /// notifications need not announce it again. `false` once the form is gone.
    pub(super) fn shows_failures(&self) -> bool {
        self.error_summaries
            .try_with_value(|count| *count > 0)
            .unwrap_or(false)
    }
}

impl CrudFormState<DynUpdateField> {
    /// Makes `model` the version the draft is compared against, and applies the user's changes to
    /// it, so that the form shows the current state of the entity and keeps the user's input.
    ///
    /// The user's changes are the JSON merge patch from the old baseline to the draft, both
    /// serialized as the model serializes them, so a change of one member of an object field
    /// keeps the changes of others to the other members. The patch applies to `model`, and the
    /// result becomes the draft, also what the model holds besides its fields, e.g. its version.
    /// Members that someone else changed to another value become [conflicts](Self::conflicts),
    /// and keep the user's value. A field whose input could not be read takes the value of
    /// `model`, and keeps its input error. A save refused as outdated no longer applies.
    pub(super) fn rebase(&self, model: DynUpdateModel, models: ModelHandler) {
        let (Some(draft), Some(baseline)) = (
            self.draft_model.get_untracked(),
            self.baseline.get_untracked(),
        ) else {
            self.baseline.set(Some(model));
            return;
        };
        let json = |model: &DynUpdateModel| model.to_untagged_json();
        let (Ok(base), Ok(mine), Ok(theirs)) = (json(&baseline), json(&draft), json(&model)) else {
            tracing::error!("cannot serialize the form's models; keeping the draft as it is");
            self.baseline.set(Some(model));
            self.clear_outdated();
            return;
        };
        let mut patch = merge_patch(&base, &mine);
        if let serde_json::Value::Object(members) = &mut patch {
            self.input_errors.with_untracked(|input_errors| {
                for field in input_errors.keys() {
                    members.remove(field.name().as_ref());
                }
            });
        }
        let mut merged = theirs.clone();
        apply_merge_patch(&mut merged, &patch);
        let rebased = match models.deserialize_update_model(merged.clone()) {
            Ok(rebased) => rebased,
            Err(err) => {
                tracing::error!(%err, "cannot apply the draft's changes to the reloaded entity");
                self.baseline.set(Some(model));
                self.clear_outdated();
                return;
            }
        };
        let found = merge_patch_conflicts(&patch, &base, &theirs)
            .into_iter()
            .filter_map(|conflict| {
                let (field, _) = split_first_token(&conflict.pointer)?;
                Some(CrudFieldConflict {
                    field,
                    pointer: conflict.pointer,
                    theirs: conflict.theirs,
                })
            })
            .collect::<Vec<_>>();
        let changes = |field: &str| patch.get(field).is_some();
        self.failure.update(|failure| {
            // Earlier conflicts stay while the user's change still applies to their field.
            failure.conflicts.retain(|conflict| {
                changes(&conflict.field)
                    && found.iter().all(|found| found.pointer != conflict.pointer)
            });
            failure.conflicts.extend(found);
            if matches!(failure.error, Some(RequestError::Outdated(_))) {
                failure.error = None;
            }
        });
        let shown = self.fields.get_untracked();
        for (field, value) in models.update_model_values(&rebased) {
            let name = field.name();
            if merged.get(name.as_ref()) == mine.get(name.as_ref()) {
                continue;
            }
            if let Some(reactive) = shown.with_value(|shown| shown.get(&field).copied()) {
                reactive.set(value);
            }
        }
        self.baseline.set(Some(model));
        self.draft_model.set(Some(rebased));
    }
}

/// Registers a mounted error summary of the form whose summaries `error_summaries` counts, until
/// the current owner is cleaned up.
pub(crate) fn register_error_summary(error_summaries: StoredValue<usize>) {
    error_summaries.update_value(|count| *count += 1);
    on_cleanup(move || {
        error_summaries.try_update_value(|count| *count -= 1);
    });
}

/// Returns `message` about the member at `path` below a field, e.g. `0/count: must be positive`,
/// or `message` alone for the field as a whole.
fn at_path(path: &str, message: &str) -> String {
    match path.strip_prefix('/') {
        Some(path) => format!("{path}: {message}"),
        None => message.to_owned(),
    }
}

/// Describes how someone else changed the member of `conflict`.
pub(crate) fn conflict_message(texts: &CrudUiTexts, conflict: &CrudFieldConflict) -> String {
    match &conflict.theirs {
        Some(serde_json::Value::String(text)) => (texts.changed_meanwhile)(text),
        Some(value) => (texts.changed_meanwhile)(&value.to_string()),
        None => texts.removed_meanwhile.to_string(),
    }
}
