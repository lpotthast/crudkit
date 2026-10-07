//! Create, edit, and read forms.

use crate::config::CrudUiTexts;
use crate::config::{CrudBuiltinViewControls, CrudCreateSaveTarget};
use crate::hooks::field::CrudFieldMap;
use crate::hooks::field::{ReactiveField, reactive_fields};
use crate::hooks::instance::use_crud_instance;
use crate::hooks::notify::{CrudNotification, CrudNotifier};
use crate::hooks::texts::use_crud_texts;
use crate::instance::CrudInstanceContext;
use crate::instance::CrudNavigation;
use crudkit_core::condition::{TryIntoAllEqualCondition, merge_conditions};
use crudkit_core::id::{SerializableId, SerializableIdEntry};
use crudkit_core::validation::PartialSerializableAggregateViolations;
use crudkit_core::{Saved, Value};
use crudkit_web::http::RequestError;
use crudkit_web::load_state::LoadState;
use crudkit_web::prelude::*;
use crudkit_web::view::CrudView;
use leptos::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// What happens after an entity was saved successfully.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum CrudSaveFollowUp {
    /// The configured default: the instance's `create_save_target` after creating, staying after
    /// editing.
    #[default]
    Default,
    /// Keep the current view.
    Stay,
    /// Perform the navigation's return action.
    Return,
    /// Open a fresh create view.
    CreateAnother,
}

/// Draft state of a form editing one entity, with one [`ReactiveField`] per field.
///
/// The form keeps the complete draft model for saving and dirty checks, and one reactive value per
/// field for fine-grained rendering. [`Self::set_field`] keeps both in sync.
pub struct CrudFormState<F: TypeErasedField> {
    /// The reactive values of all fields. Changes when an entity (re)loads.
    pub fields: Signal<CrudFieldMap<F>>,
    /// The current draft, or `None` while no entity is loaded.
    pub draft: Signal<Option<F::Model>>,
    /// Input errors keyed by field.
    pub errors: Signal<HashMap<F, String>>,
    field_values: RwSignal<CrudFieldMap<F>>,
    draft_model: RwSignal<Option<F::Model>>,
    baseline: RwSignal<Option<F::Model>>,
    input_errors: RwSignal<HashMap<F, String>>,
    /// Whether the draft differs from the loaded or default entity.
    pub is_dirty: Signal<bool>,
    /// Whether the form holds an entity. Field values exist only while this is `true`.
    pub is_ready: Signal<bool>,
    /// Whether some field holds input that could not be converted into a value. Such a draft is
    /// not saved, because it does not reflect what the user sees.
    pub has_errors: Signal<bool>,
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
    fn new() -> Self {
        let draft = RwSignal::new(None::<F::Model>);
        let baseline = RwSignal::new(None::<F::Model>);
        let errors = RwSignal::new(HashMap::new());
        let fields = RwSignal::new(StoredValue::new(HashMap::new()));
        Self {
            fields: fields.into(),
            draft: draft.into(),
            errors: errors.into(),
            field_values: fields,
            draft_model: draft,
            baseline,
            input_errors: errors,
            is_dirty: Memo::new(move |_| match (&*draft.read(), &*baseline.read()) {
                (Some(draft), Some(baseline)) => draft != baseline,
                _ => false,
            })
            .into(),
            // A memo, so that views keyed on readiness do not re-render on every draft change.
            is_ready: Memo::new(move |_| draft.read().is_some()).into(),
            has_errors: Memo::new(move |_| !errors.read().is_empty()).into(),
        }
    }

    /// Returns the reactive value of `field`. Tracks entity (re)loads.
    pub fn field(&self, field: &F) -> Option<ReactiveField> {
        self.fields
            .get()
            .with_value(|fields| fields.get(field).copied())
    }

    /// Returns the input error of `field`, if any.
    pub fn field_error(&self, field: F) -> Signal<Option<String>> {
        let errors = self.errors;
        Signal::derive(move || errors.read().get(&field).cloned())
    }

    /// Applies user input for `field`.
    ///
    /// A value updates both the draft model and the field's reactive value and clears the field's
    /// input error. An error records the input error and leaves the draft unchanged.
    pub fn set_field(&self, field: F, value: Result<Value, String>) {
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

    /// Replaces draft, baseline, and field values with `model`.
    fn load(&self, model: F::Model, fields: HashMap<F, ReactiveField>) {
        self.field_values.set(StoredValue::new(fields));
        self.baseline.set(Some(model.clone()));
        self.draft_model.set(Some(model));
        self.input_errors.update(HashMap::clear);
    }

    /// Replaces draft, baseline, and field values with `model`, keeping the existing reactive
    /// values, so that rendered fields update in place.
    fn sync(&self, model: F::Model, fields: HashMap<F, ReactiveField>) {
        let current = self.fields.get_untracked();
        for (field, reactive) in fields {
            let existing = current.with_value(|current| current.get(&field).copied());
            match existing {
                Some(existing) => existing.set(reactive.get_untracked()),
                None => current.update_value(|current| {
                    current.insert(field, reactive);
                }),
            }
        }
        self.baseline.set(Some(model.clone()));
        self.draft_model.set(Some(model));
        self.input_errors.update(HashMap::clear);
    }
}

/// Input of [`use_crud_create_form`].
#[derive(Debug, Clone, Default)]
pub struct UseCrudCreateFormInput {
    /// Called after the entity was created.
    pub on_saved: Option<Callback<Saved<DynUpdateModel>>>,
    /// Called when creating the entity failed.
    pub on_save_failed: Option<Callback<RequestError>>,
    /// Suppresses the error notification that is otherwise sent when creating the entity fails.
    pub quiet: bool,
}

/// Output of [`use_crud_create_form`].
#[derive(Debug, Clone, Copy)]
pub struct UseCrudCreateFormReturn {
    /// The draft of the entity to create.
    pub form: CrudFormState<DynCreateField>,
    /// Creates the drafted entity, then performs the given follow-up.
    pub save: Callback<CrudSaveFollowUp>,
    /// Whether a save request is in flight.
    pub is_saving: Signal<bool>,
    /// Whether [`Self::save`] is currently allowed. An unchanged default draft may be saved.
    pub can_save: Signal<bool>,
    /// The save controls this form publishes to its instance, see [`CrudCreateActions`].
    pub actions: CrudCreateActions,
    /// Non-critical validation violations reported by the last successful save, if any.
    pub violations: Signal<Option<PartialSerializableAggregateViolations>>,
}

/// The save controls of a mounted create form.
///
/// While a create form is mounted, it publishes these through
/// [`CrudInstanceContext::create_actions`], so applications can place the save buttons outside of
/// the form, e.g. in a page header.
#[derive(Debug, Clone, Copy)]
pub struct CrudCreateActions {
    /// Creates the drafted entity, then performs the given follow-up.
    pub save: Callback<CrudSaveFollowUp>,
    /// Whether saving is currently allowed.
    pub can_save: Signal<bool>,
    /// Navigation of the create view, e.g. for a return button.
    pub navigation: CrudNavigation,
    /// The instance's built-in view controls, e.g. which save follow-ups to offer.
    pub controls: Signal<CrudBuiltinViewControls>,
}

/// Creates a form for a new entity of the surrounding instance.
///
/// The draft starts from the create model's default, with the parent reference filled in for nested
/// instances. A changed draft guards the current navigation. While mounted, the form publishes its
/// save controls to the instance.
///
/// # Panics
///
/// Panics when called outside of a CrudKit instance.
#[must_use]
// Hooks take their input by value, following Leptonic's `use_x(UseXInput) -> UseXReturn` convention.
#[allow(clippy::needless_pass_by_value)]
pub fn use_crud_create_form(input: UseCrudCreateFormInput) -> UseCrudCreateFormReturn {
    let UseCrudCreateFormInput {
        on_saved,
        on_save_failed,
        quiet,
    } = input;
    let ctx = use_crud_instance();
    let navigation = ctx.navigation;
    let texts = StoredValue::new(use_crud_texts());
    let violations = RwSignal::new(None::<PartialSerializableAggregateViolations>);

    let form = CrudFormState::<DynCreateField>::new();
    let default_model = default_create_model(&ctx);
    let fields = reactive_fields(
        ctx.static_config
            .read_value()
            .model_handler
            .create_model_values(&default_model),
    );
    form.load(default_model, fields);
    navigation.guard(form.is_dirty);

    let save_action = create_action(&ctx);

    let save_result = save_action.value();
    let reset = move || {
        let model = default_create_model(&ctx);
        let fields = reactive_fields(
            ctx.static_config
                .read_value()
                .model_handler
                .create_model_values(&model),
        );
        form.sync(model, fields);
    };
    Effect::new(move |_prev| {
        let Some((result, follow_up)) = save_result.get() else {
            return;
        };
        // Callbacks may read signals. They must not become dependencies of this effect, or a
        // change would replay the save result.
        untrack(move || match result {
            Ok(saved) => {
                violations.set(Some(saved.violations.clone()));
                let id = saved.entity.id();
                if let Some(on_saved) = on_saved {
                    on_saved.run(saved);
                }
                follow_up_after_create(&ctx, follow_up, id, reset);
            }
            Err(request_error) => {
                tracing::warn!("Could not create entity due to error: {request_error}");
                let failure = SaveFailure {
                    notifier: ctx.notifier,
                    texts,
                    quiet,
                    on_save_failed,
                };
                failure.report(request_error, |texts| texts.create_failed.as_ref());
            }
        });
    });

    let is_saving: Signal<bool> = save_action.pending().into();
    let save = Callback::new(move |follow_up: CrudSaveFollowUp| {
        if let Some(draft) = savable_draft(form) {
            save_action.dispatch((draft, follow_up));
        }
    });

    let can_save = Signal::derive(move || !is_saving.get() && !form.has_errors.get());
    let actions = CrudCreateActions {
        save,
        can_save,
        navigation,
        controls: Signal::derive(move || ctx.builtin_view_controls()),
    };
    ctx.set_create_actions(Some(actions));
    on_cleanup(move || ctx.set_create_actions(None));

    UseCrudCreateFormReturn {
        form,
        save,
        is_saving,
        can_save,
        actions,
        violations: violations.into(),
    }
}

/// Performs `follow_up` after creating the entity `id`. `reset` starts a fresh draft.
fn follow_up_after_create(
    ctx: &CrudInstanceContext,
    follow_up: CrudSaveFollowUp,
    id: SerializableId,
    reset: impl Fn(),
) {
    let navigation = ctx.navigation;
    match follow_up {
        CrudSaveFollowUp::Default => match ctx.builtin_view_controls().create_save_target {
            CrudCreateSaveTarget::EditView => {
                navigation.navigate_committed(CrudView::edit(id));
            }
            CrudCreateSaveTarget::View(resolve) => {
                navigation.navigate_committed(resolve.run(id));
            }
            CrudCreateSaveTarget::Return => {
                navigation.return_committed();
            }
            // The entity exists now. Staying offers a fresh draft for the next one.
            CrudCreateSaveTarget::Stay => reset(),
        },
        CrudSaveFollowUp::Stay => reset(),
        CrudSaveFollowUp::Return => {
            navigation.return_committed();
        }
        CrudSaveFollowUp::CreateAnother => {
            navigation.navigate_committed(CrudView::create());
        }
    }
}

/// Whether an edited or displayed entity can be shown.
#[derive(Debug, Clone, PartialEq)]
pub enum CrudEntityStatus {
    /// The entity is being loaded, or its field values are being prepared.
    Loading,
    /// The entity is loaded and the form holds its field values.
    Ready,
    /// No entity matches the requested ID.
    NotFound,
    /// Loading the entity failed.
    Failed(RequestError),
}

/// Returns the status of `entity`, shown through `form`.
fn entity_status<F: TypeErasedField>(
    entity: Signal<LoadState<DynUpdateModel>>,
    form: CrudFormState<F>,
) -> Signal<CrudEntityStatus> {
    // A memo, so that views keyed on the status re-render only when it changes.
    Memo::new(move |_| match &*entity.read() {
        LoadState::Loaded(_) if form.is_ready.get() => CrudEntityStatus::Ready,
        LoadState::Loading | LoadState::Loaded(_) => CrudEntityStatus::Loading,
        LoadState::NotFound => CrudEntityStatus::NotFound,
        LoadState::Failed(error) => CrudEntityStatus::Failed(error.clone()),
    })
    .into()
}

/// Result of a save request, together with the follow-up requested for it.
type SaveOutcome = (
    Result<Saved<DynUpdateModel>, RequestError>,
    CrudSaveFollowUp,
);

/// Creates the action sending create requests for the instance of `ctx`.
// TODO: Can we get rid of new_local?
fn create_action(
    ctx: &CrudInstanceContext,
) -> Action<(DynCreateModel, CrudSaveFollowUp), SaveOutcome> {
    let ctx = *ctx;
    Action::new_local(
        move |(create_model, follow_up): &(DynCreateModel, CrudSaveFollowUp)| {
            let entity = create_model.clone();
            let follow_up = *follow_up;
            let data_provider = ctx.data_provider.get_untracked();
            async move {
                let result = data_provider.create_one(CreateOne { entity }).await;
                (result, follow_up)
            }
        },
    )
}

/// Creates the action sending update requests for the entity `id` of the instance of `ctx`.
fn update_action(
    ctx: &CrudInstanceContext,
    id: Signal<SerializableId>,
) -> Action<(DynUpdateModel, CrudSaveFollowUp), SaveOutcome> {
    let ctx = *ctx;
    Action::new_local(
        move |(entity, follow_up): &(DynUpdateModel, CrudSaveFollowUp)| {
            let entity = entity.clone();
            let follow_up = *follow_up;
            let data_provider = ctx.data_provider.get_untracked();
            let id = id.get_untracked();
            let base_condition = ctx.base_condition.get_untracked();
            async move {
                let id_condition = match id.0.into_iter().try_into_all_equal_condition() {
                    Ok(condition) => condition,
                    Err(e) => {
                        return (
                            Err(RequestError::InvalidRequest(format!(
                                "ID contains unsupported field types: {e:?}"
                            ))),
                            follow_up,
                        );
                    }
                };
                let result = data_provider
                    .update_one(UpdateOne {
                        entity,
                        condition: merge_conditions(base_condition, Some(id_condition)),
                    })
                    .await;
                (result, follow_up)
            }
        },
    )
}

/// Reports a failed save: as a notification unless `quiet`, and to `on_save_failed`.
struct SaveFailure {
    notifier: CrudNotifier,
    texts: StoredValue<Arc<CrudUiTexts>>,
    quiet: bool,
    on_save_failed: Option<Callback<RequestError>>,
}

impl SaveFailure {
    fn report(self, error: RequestError, message: fn(&CrudUiTexts) -> &str) {
        if !self.quiet {
            let (title, message) = self.texts.with_value(|texts| {
                (
                    texts.error.to_string(),
                    format!("{} {error}", message(texts)),
                )
            });
            self.notifier
                .notify(CrudNotification::error(title, message));
        }
        if let Some(on_save_failed) = self.on_save_failed {
            on_save_failed.run(error);
        }
    }
}

/// Returns the draft of `form`, unless it is missing or holds rejected input.
fn savable_draft<F: TypeErasedField>(form: CrudFormState<F>) -> Option<F::Model> {
    if form.has_errors.get_untracked() {
        tracing::warn!("not saving a draft with input errors");
        return None;
    }
    form.draft.get_untracked()
}

fn default_create_model(ctx: &CrudInstanceContext) -> DynCreateModel {
    let mut entity = ctx
        .static_config
        .read_value()
        .model_handler
        .default_create_model();

    if let Some(parent) = ctx.parent.get_value() {
        if let Some(parent_id) = ctx.parent_id.get_untracked() {
            let SerializableIdEntry {
                field_name: _,
                value,
            } = parent_id
                .entries()
                .find(|entry| entry.field_name.as_str() == parent.referenced_field.as_ref())
                .expect("related parent field must be part of the parents id!");
            let referencing_field = ctx
                .static_config
                .read_value()
                .model_handler
                .create_model_field(&parent.referencing_field);
            if let Some(field) = referencing_field {
                field.set_value(&mut entity, value.clone().into());
            } else {
                tracing::error!(
                    field = %parent.referencing_field,
                    "the parent's referencing field is not a field of the create model; not prefilling it"
                );
            }
        } else {
            tracing::error!(
                "CrudInstance is configured to be a nested instance but no parent id was passed down!"
            );
        }
    }
    entity
}

/// Input of [`use_crud_edit_form`].
#[derive(Debug, Clone)]
pub struct UseCrudEditFormInput {
    /// ID of the edited entity.
    pub id: Signal<SerializableId>,
    /// Called after the entity was saved.
    pub on_saved: Option<Callback<Saved<DynUpdateModel>>>,
    /// Called when saving the entity failed.
    pub on_save_failed: Option<Callback<RequestError>>,
    /// Suppresses the error notification that is otherwise sent when saving the entity fails.
    pub quiet: bool,
}

impl UseCrudEditFormInput {
    /// Creates an input editing the entity identified by `id`.
    pub fn new(id: impl Into<Signal<SerializableId>>) -> Self {
        Self {
            id: id.into(),
            on_saved: None,
            on_save_failed: None,
            quiet: false,
        }
    }
}

/// Output of [`use_crud_edit_form`].
#[derive(Debug, Clone, Copy)]
pub struct UseCrudEditFormReturn {
    /// The entity as loaded from the server.
    pub entity: Signal<LoadState<DynUpdateModel>>,
    /// Whether the entity can be shown, see [`CrudEntityStatus`].
    pub status: Signal<CrudEntityStatus>,
    /// Non-critical validation violations reported by the last successful save, if any.
    pub violations: Signal<Option<PartialSerializableAggregateViolations>>,
    /// The draft of the edited entity. Reset whenever the entity (re)loads.
    pub form: CrudFormState<DynUpdateField>,
    /// Saves the draft, then performs the given follow-up.
    pub save: Callback<CrudSaveFollowUp>,
    /// Whether a save request is in flight.
    pub is_saving: Signal<bool>,
    /// Whether [`Self::save`] is currently useful: the draft is changed and no save is in flight.
    pub can_save: Signal<bool>,
    /// Asks the user to confirm deleting the entity.
    pub delete: Callback<()>,
    /// Whether [`Self::delete`] is currently allowed.
    pub can_delete: Signal<bool>,
}

/// Loads an entity of the surrounding instance and creates a form editing it.
///
/// A changed draft guards the current navigation.
///
/// # Panics
///
/// Panics when called outside of a CrudKit instance.
#[must_use]
// Hooks take their input by value, following Leptonic's `use_x(UseXInput) -> UseXReturn` convention.
#[allow(clippy::needless_pass_by_value)]
pub fn use_crud_edit_form(input: UseCrudEditFormInput) -> UseCrudEditFormReturn {
    let UseCrudEditFormInput {
        id,
        on_saved,
        on_save_failed,
        quiet,
    } = input;
    let ctx = use_crud_instance();
    let navigation = ctx.navigation;
    let texts = StoredValue::new(use_crud_texts());
    let violations = RwSignal::new(None::<PartialSerializableAggregateViolations>);

    let entity = use_entity(&ctx, id);
    let form = CrudFormState::<DynUpdateField>::new();
    load_into_form(&ctx, entity, form);
    navigation.guard(form.is_dirty);

    let save_action = update_action(&ctx, id);

    let save_result = save_action.value();
    Effect::new(move |_prev| {
        let Some((result, follow_up)) = save_result.get() else {
            return;
        };
        // Callbacks may read signals. They must not become dependencies of this effect, or a
        // change would replay the save result.
        untrack(move || match result {
            Ok(saved) => {
                violations.set(Some(saved.violations.clone()));
                // The server may have normalized the entity. The form continues from its version.
                let fields = reactive_fields(
                    ctx.static_config
                        .read_value()
                        .model_handler
                        .update_model_values(&saved.entity),
                );
                form.sync(saved.entity.clone(), fields);
                if let Some(on_saved) = on_saved {
                    on_saved.run(saved);
                }
                match follow_up {
                    CrudSaveFollowUp::Default | CrudSaveFollowUp::Stay => {}
                    CrudSaveFollowUp::Return => {
                        navigation.return_committed();
                    }
                    CrudSaveFollowUp::CreateAnother => {
                        navigation.navigate_committed(CrudView::create());
                    }
                }
            }
            Err(request_error) => {
                tracing::warn!("Could not update entity due to error: {request_error}");
                let failure = SaveFailure {
                    notifier: ctx.notifier,
                    texts,
                    quiet,
                    on_save_failed,
                };
                failure.report(request_error, |texts| texts.update_failed.as_ref());
            }
        });
    });

    let is_saving: Signal<bool> = save_action.pending().into();
    let is_dirty = form.is_dirty;
    let save = Callback::new(move |follow_up: CrudSaveFollowUp| {
        if let Some(draft) = savable_draft(form) {
            save_action.dispatch((draft, follow_up));
        }
    });
    let delete = Callback::new(move |()| {
        if let Some(draft) = form.draft.get_untracked() {
            ctx.deletion.for_navigation(navigation).request(draft);
        }
    });

    UseCrudEditFormReturn {
        entity,
        status: entity_status(entity, form),
        violations: violations.into(),
        form,
        save,
        is_saving,
        can_save: Signal::derive(move || {
            !is_saving.get() && is_dirty.get() && !form.has_errors.get()
        }),
        delete,
        can_delete: Signal::derive(move || !is_saving.get() && form.draft.read().is_some()),
    }
}

/// Output of [`use_crud_read`].
#[derive(Debug, Clone, Copy)]
pub struct UseCrudReadReturn {
    /// The entity as loaded from the server.
    pub entity: Signal<LoadState<DynUpdateModel>>,
    /// Whether the entity can be shown, see [`CrudEntityStatus`].
    pub status: Signal<CrudEntityStatus>,
    /// Reactive values of the entity's fields. Not meant to be edited.
    pub form: CrudFormState<DynUpdateField>,
    /// Asks the user to confirm deleting the entity.
    pub delete: Callback<()>,
    /// Whether [`Self::delete`] is currently allowed.
    pub can_delete: Signal<bool>,
}

/// Loads an entity of the surrounding instance for read-only display.
///
/// # Panics
///
/// Panics when called outside of a CrudKit instance.
pub fn use_crud_read(id: impl Into<Signal<SerializableId>>) -> UseCrudReadReturn {
    let ctx = use_crud_instance();
    let navigation = ctx.navigation;

    let entity = use_entity(&ctx, id.into());
    let form = CrudFormState::<DynUpdateField>::new();
    load_into_form(&ctx, entity, form);

    let delete = Callback::new(move |()| {
        if let Some(entity) = entity.get_untracked().into_loaded() {
            ctx.deletion.for_navigation(navigation).request(entity);
        }
    });

    UseCrudReadReturn {
        entity,
        status: entity_status(entity, form),
        form,
        delete,
        can_delete: Signal::derive(move || entity.read().loaded().is_some()),
    }
}

/// Loads the entity identified by `id` within the instance's base condition.
fn use_entity(
    ctx: &CrudInstanceContext,
    id: Signal<SerializableId>,
) -> Signal<LoadState<DynUpdateModel>> {
    let ctx = *ctx;
    // TODO: Do not use LocalResource, allow loading on the server.
    let resource = LocalResource::new(move || async move {
        let _ = ctx.reload.get();
        let equals_id_condition = match id.get().into_entries().try_into_all_equal_condition() {
            Ok(condition) => condition,
            Err(e) => {
                return Err(RequestError::InvalidRequest(format!(
                    "ID contains unsupported field types: {e:?}"
                )));
            }
        };
        ctx.data_provider
            .read()
            .read_one(ReadOne {
                skip: None,
                order_by: None,
                condition: merge_conditions(ctx.base_condition.get(), Some(equals_id_condition)),
            })
            .await
    });

    Memo::new(move |_prev| match resource.get() {
        Some(result) => LoadState::from_optional_result(result).map(|read_model| {
            ctx.static_config
                .read_value()
                .model_handler
                .read_model_to_update_model(read_model)
        }),
        None => LoadState::Loading,
    })
    .into()
}

/// Resets `form` to every newly loaded version of `entity`.
fn load_into_form(
    ctx: &CrudInstanceContext,
    entity: Signal<LoadState<DynUpdateModel>>,
    form: CrudFormState<DynUpdateField>,
) {
    let ctx = *ctx;
    Effect::new(move |_prev| {
        if let LoadState::Loaded(model) = entity.get() {
            let fields = reactive_fields(
                ctx.static_config
                    .read_value()
                    .model_handler
                    .update_model_values(&model),
            );
            form.load(model, fields);
        }
    });
}
