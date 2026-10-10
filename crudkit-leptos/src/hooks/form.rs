//! Create, edit, and read forms.

use crate::config::{CrudEntityViewKind, CrudUiTexts};
use crate::hooks::field::CrudFieldMap;
use crate::hooks::field::{ReactiveField, reactive_fields};
use crate::hooks::instance::use_crud_instance;
use crate::hooks::notify::{CrudNotification, CrudNotifier};
use crate::hooks::texts::{use_crud_texts, with_texts};
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
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;

/// What happens after an entity was saved successfully.
#[derive(Debug, Clone, Copy)]
pub enum CrudSaveFollowUp {
    /// Keep the current view. An edit form goes on editing the saved entity, a create form starts
    /// a fresh draft.
    Stay,
    /// Return from the current view, see [`CrudNavigation::return_from_current`].
    Return,
    /// Open the saved entity in the edit view. After editing, this stays.
    Edit,
    /// Start another entity: a create form starts a fresh draft, an edit form opens a create view.
    CreateAnother,
    /// Open the view returned for the saved entity's id.
    View(Callback<SerializableId, CrudView>),
}

impl CrudSaveFollowUp {
    /// Performs this follow-up on `navigation` after a form of `kind` saved the entity `id`.
    /// `stay` runs for [`Self::Stay`], and where the form already shows what the follow-up asks
    /// for: [`Self::Edit`] in an edit form, [`Self::CreateAnother`] in a create form.
    fn perform(
        self,
        navigation: CrudNavigation,
        id: SerializableId,
        kind: CrudEntityViewKind,
        stay: impl Fn(),
    ) {
        match self {
            Self::Stay => stay(),
            Self::Edit if kind == CrudEntityViewKind::Update => stay(),
            Self::CreateAnother if kind == CrudEntityViewKind::Create => stay(),
            Self::Return => {
                navigation.return_committed();
            }
            Self::Edit => {
                navigation.navigate_committed(CrudView::edit(id));
            }
            Self::CreateAnother => {
                navigation.navigate_committed(CrudView::create());
            }
            Self::View(resolve) => {
                navigation.navigate_committed(resolve.run(id));
            }
        }
    }
}

/// The form surrounding the caller: a [`CrudCreateForm`](crate::atoms::CrudCreateForm),
/// [`CrudEditForm`](crate::atoms::CrudEditForm), or [`CrudDetails`](crate::atoms::CrudDetails).
///
/// Read it with [`use_crud_form`], e.g. to render a form's status, dirty state, or save controls
/// in markup the atoms do not cover.
#[derive(Clone, Copy)]
pub struct CrudFormHandle {
    /// Whether the form creates, edits, or shows an entity.
    pub kind: CrudEntityViewKind,
    /// Whether the form's entity is shown. Always ready in create forms.
    pub status: Signal<CrudEntityLoadStatus>,
    /// Whether the form holds unsaved changes.
    pub is_dirty: Signal<bool>,
    /// Whether some field holds input that could not be read, which keeps the form from saving.
    pub has_errors: Signal<bool>,
    /// Whether a save request is in flight.
    pub is_saving: Signal<bool>,
    /// Whether the form can be saved now.
    pub can_save: Signal<bool>,
    /// Saves the form, then performs the given follow-up. `None` in details, which do not save.
    pub save: Option<Callback<CrudSaveFollowUp>>,
    /// Non-critical validation violations the server reported for the last successful save, if
    /// any. Always `None` in details.
    pub violations: Signal<Option<PartialSerializableAggregateViolations>>,
    /// What saving does when the saving control does not say, e.g. on submit.
    pub follow_up: CrudSaveFollowUp,
    /// The shown entity including unsaved changes, which entity actions act on. `None` in create
    /// forms and while the entity loads.
    pub draft: Signal<Option<DynUpdateModel>>,
    /// The `TypeId` of the form's field type, e.g. `DynCreateField`.
    pub(crate) field_type: TypeId,
    /// Whether the form is a `<form>` element, which its save button submits. A form nested in
    /// another one is not.
    pub(crate) submits: bool,
}

impl fmt::Debug for CrudFormHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrudFormHandle")
            .field("kind", &self.kind)
            .field("follow_up", &self.follow_up)
            .finish_non_exhaustive()
    }
}

/// Returns the surrounding form.
///
/// # Panics
///
/// Panics when called outside of a form, or inside an instance nested in a form.
#[must_use]
pub fn use_crud_form() -> CrudFormHandle {
    try_use_crud_form().expect("`use_crud_form` must be called inside a CrudKit form")
}

/// Returns the surrounding form, unless an instance lies in between.
pub(crate) fn try_use_crud_form() -> Option<CrudFormHandle> {
    use_context::<Option<CrudFormHandle>>().flatten()
}

/// Returns the form surrounding `atom`.
///
/// # Panics
///
/// Panics when `atom` is rendered outside of a form, or inside an instance nested in a form.
pub(crate) fn expect_crud_form(atom: &'static str) -> CrudFormHandle {
    try_use_crud_form().unwrap_or_else(|| {
        panic!("`{atom}` must be rendered inside a CrudKit form, e.g. a `CrudEditForm`")
    })
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
    /// Whether a save request is in flight. Fields are read-only meanwhile, so that the saved
    /// entity, which becomes the form's new state, cannot overwrite input made during the request.
    pub is_saving: Signal<bool>,
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
    fn new(is_saving: Signal<bool>) -> Self {
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
            is_saving,
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

    /// Makes `model` the version the draft is compared against, keeping the draft and its input.
    fn rebase(&self, model: F::Model) {
        self.baseline.set(Some(model));
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

/// Which outcomes of saving a form notify the user, through the instance's [`CrudNotifier`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CrudSaveNotifications {
    /// Successes and failures.
    #[default]
    All,
    /// Failures only, e.g. when the application reports successes itself.
    Failures,
    /// None.
    None,
}

/// Input of [`use_crud_create_form`].
#[derive(Debug, Clone, Default)]
pub struct UseCrudCreateFormInput {
    /// Called after the entity was created, while the form is mounted.
    pub on_saved: Option<Callback<Saved<DynUpdateModel>>>,
    /// Called when creating the entity failed, while the form is mounted.
    pub on_save_failed: Option<Callback<RequestError>>,
    /// Which outcomes notify the user. Notifications are sent even when the form is gone by then.
    pub notifications: CrudSaveNotifications,
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
    /// Non-critical validation violations reported by the last successful save, if any.
    pub violations: Signal<Option<PartialSerializableAggregateViolations>>,
}

/// Creates a form for a new entity of the surrounding instance.
///
/// The draft starts from the create model's default, with the parent reference filled in for nested
/// instances. A changed draft guards the current navigation.
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
        notifications,
    } = input;
    let ctx = use_crud_instance();
    let navigation = ctx.navigation;
    let violations = RwSignal::new(None::<PartialSerializableAggregateViolations>);

    let save_action = create_action(&ctx, notifications);
    let is_saving: Signal<bool> = save_action.pending().into();
    let form = CrudFormState::<DynCreateField>::new(is_saving);
    let default_model = default_create_model(&ctx);
    let fields = reactive_fields(
        ctx.static_config
            .read_value()
            .model_handler
            .create_model_values(&default_model),
    );
    form.load(default_model, fields);
    navigation.guard(form.is_dirty);

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
    // The outcome was already notified by the request. The form applies it while it is mounted.
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
                follow_up.perform(ctx.navigation, id, CrudEntityViewKind::Create, reset);
            }
            Err(request_error) => {
                if let Some(on_save_failed) = on_save_failed {
                    on_save_failed.run(request_error);
                }
            }
        });
    });

    let save = Callback::new(move |follow_up: CrudSaveFollowUp| {
        if let Some(draft) = savable_draft(form) {
            save_action.dispatch((draft, follow_up));
        }
    });

    let can_save = Signal::derive(move || !is_saving.get() && !form.has_errors.get());

    UseCrudCreateFormReturn {
        form,
        save,
        is_saving,
        can_save,
        violations: violations.into(),
    }
}

/// Whether an edited or displayed entity can be shown.
#[derive(Debug, Clone, PartialEq)]
pub enum CrudEntityLoadStatus {
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
) -> Signal<CrudEntityLoadStatus> {
    // A memo, so that views keyed on the status re-render only when it changes.
    Memo::new(move |_| match &*entity.read() {
        LoadState::Loaded(_) if form.is_ready.get() => CrudEntityLoadStatus::Ready,
        LoadState::Loading | LoadState::Loaded(_) => CrudEntityLoadStatus::Loading,
        LoadState::NotFound => CrudEntityLoadStatus::NotFound,
        LoadState::Failed(error) => CrudEntityLoadStatus::Failed(error.clone()),
    })
    .into()
}

/// Result of a save request, together with the follow-up requested for it.
type SaveOutcome = (
    Result<Saved<DynUpdateModel>, RequestError>,
    CrudSaveFollowUp,
);

/// Creates the action sending create requests for the instance of `ctx`. It notifies the outcome
/// as `notifications` ask, also when the form is gone by then.
// TODO: Can we get rid of new_local?
fn create_action(
    ctx: &CrudInstanceContext,
    notifications: CrudSaveNotifications,
) -> Action<(DynCreateModel, CrudSaveFollowUp), SaveOutcome> {
    let ctx = *ctx;
    let notify = SaveNotifier::new(&ctx, notifications);
    Action::new_local(
        move |(create_model, follow_up): &(DynCreateModel, CrudSaveFollowUp)| {
            let entity = create_model.clone();
            let follow_up = *follow_up;
            let data_provider = ctx.data_provider.get_untracked();
            async move {
                let result = data_provider.create_one(CreateOne { entity }).await;
                notify.outcome(
                    &result,
                    |texts| &texts.created,
                    |texts| &texts.create_failed,
                );
                (result, follow_up)
            }
        },
    )
}

/// Creates the action sending update requests for the entity `id` of the instance of `ctx`. It
/// notifies the outcome as `notifications` ask, also when the form is gone by then.
fn update_action(
    ctx: &CrudInstanceContext,
    id: Signal<SerializableId>,
    notifications: CrudSaveNotifications,
) -> Action<(DynUpdateModel, CrudSaveFollowUp), SaveOutcome> {
    let ctx = *ctx;
    let notify = SaveNotifier::new(&ctx, notifications);
    Action::new_local(
        move |(entity, follow_up): &(DynUpdateModel, CrudSaveFollowUp)| {
            let entity = entity.clone();
            let follow_up = *follow_up;
            let data_provider = ctx.data_provider.get_untracked();
            let id = id.get_untracked();
            let base_condition = ctx.base_condition.get_untracked();
            async move {
                let result = match id.0.into_iter().try_into_all_equal_condition() {
                    Ok(id_condition) => {
                        data_provider
                            .update_one(UpdateOne {
                                entity,
                                condition: merge_conditions(base_condition, Some(id_condition)),
                            })
                            .await
                    }
                    Err(e) => Err(RequestError::InvalidRequest(format!(
                        "ID contains unsupported field types: {e:?}"
                    ))),
                };
                notify.outcome(&result, |texts| &texts.saved, |texts| &texts.update_failed);
                (result, follow_up)
            }
        },
    )
}

/// Notifies the outcomes of save requests through the instance's notifier.
#[derive(Clone, Copy)]
struct SaveNotifier {
    notifier: CrudNotifier,
    texts: Signal<CrudUiTexts>,
    notifications: CrudSaveNotifications,
}

impl SaveNotifier {
    fn new(ctx: &CrudInstanceContext, notifications: CrudSaveNotifications) -> Self {
        Self {
            notifier: ctx.notifier,
            texts: use_crud_texts(),
            notifications,
        }
    }

    /// Notifies `result` with the message `saved` or `failed`, as the notifications ask.
    fn outcome<T>(
        self,
        result: &Result<T, RequestError>,
        saved: fn(&CrudUiTexts) -> &str,
        failed: fn(&CrudUiTexts) -> &str,
    ) {
        let notification = match result {
            Ok(_) if self.notifications == CrudSaveNotifications::All => {
                with_texts(self.texts, |texts| {
                    CrudNotification::success(texts.save.to_string(), saved(texts))
                })
            }
            Err(error) if self.notifications != CrudSaveNotifications::None => {
                tracing::warn!(%error, "could not save the entity");
                with_texts(self.texts, |texts| {
                    CrudNotification::error(
                        texts.error.to_string(),
                        format!("{} {error}", failed(texts)),
                    )
                })
            }
            Ok(_) => return,
            Err(error) => {
                tracing::warn!(%error, "could not save the entity");
                return;
            }
        };
        self.notifier.notify(notification);
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
    /// Called after the entity was saved, while the form is mounted.
    pub on_saved: Option<Callback<Saved<DynUpdateModel>>>,
    /// Called when saving the entity failed, while the form is mounted.
    pub on_save_failed: Option<Callback<RequestError>>,
    /// Which outcomes notify the user. Notifications are sent even when the form is gone by then.
    pub notifications: CrudSaveNotifications,
}

/// Output of [`use_crud_edit_form`].
#[derive(Debug, Clone, Copy)]
pub struct UseCrudEditFormReturn {
    /// The entity as loaded from the server.
    pub entity: Signal<LoadState<DynUpdateModel>>,
    /// Whether the entity can be shown, see [`CrudEntityLoadStatus`].
    pub status: Signal<CrudEntityLoadStatus>,
    /// Non-critical validation violations reported by the last successful save, if any.
    pub violations: Signal<Option<PartialSerializableAggregateViolations>>,
    /// The draft of the edited entity. Reset whenever the entity (re)loads, unless it holds
    /// unsaved changes of that entity.
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
        notifications,
    } = input;
    let ctx = use_crud_instance();
    let navigation = ctx.navigation;
    let violations = RwSignal::new(None::<PartialSerializableAggregateViolations>);

    let entity = use_entity(&ctx, id);
    let save_action = update_action(&ctx, id, notifications);
    let is_saving: Signal<bool> = save_action.pending().into();
    let form = CrudFormState::<DynUpdateField>::new(is_saving);
    load_into_form(&ctx, entity, form);
    navigation.guard(form.is_dirty);

    let save_result = save_action.value();
    // The outcome was already notified by the request. The form applies it while it is mounted.
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
                let saved_id = saved.entity.id();
                form.sync(saved.entity.clone(), fields);
                if let Some(on_saved) = on_saved {
                    on_saved.run(saved);
                }
                follow_up.perform(navigation, saved_id, CrudEntityViewKind::Update, || {});
            }
            Err(request_error) => {
                if let Some(on_save_failed) = on_save_failed {
                    on_save_failed.run(request_error);
                }
            }
        });
    });

    let is_dirty = form.is_dirty;
    let save = Callback::new(move |follow_up: CrudSaveFollowUp| {
        if let Some(draft) = savable_draft(form) {
            save_action.dispatch((draft, follow_up));
        }
    });
    // Deletes the entity as loaded: the draft may hold unsaved changes, even of its ID fields.
    let delete = Callback::new(move |()| {
        if let Some(entity) = entity.get_untracked().into_loaded() {
            ctx.deletion.for_navigation(navigation).request(entity);
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
        can_delete: Signal::derive(move || !is_saving.get() && entity.read().loaded().is_some()),
    }
}

/// Output of [`use_crud_read`].
#[derive(Debug, Clone, Copy)]
pub struct UseCrudReadReturn {
    /// The entity as loaded from the server.
    pub entity: Signal<LoadState<DynUpdateModel>>,
    /// Whether the entity can be shown, see [`CrudEntityLoadStatus`].
    pub status: Signal<CrudEntityLoadStatus>,
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
    let form = CrudFormState::<DynUpdateField>::new(Signal::stored(false));
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
///
/// A reload of the entity the form holds unsaved changes of keeps those changes: the reloaded
/// version only becomes what they are compared against. Discarding them is up to the user, e.g.
/// by leaving the view.
fn load_into_form(
    ctx: &CrudInstanceContext,
    entity: Signal<LoadState<DynUpdateModel>>,
    form: CrudFormState<DynUpdateField>,
) {
    let ctx = *ctx;
    Effect::new(move |_prev| {
        if let LoadState::Loaded(model) = entity.get() {
            let edits_this_entity = form.is_dirty.get_untracked()
                && form.draft.with_untracked(|draft| {
                    draft.as_ref().is_some_and(|draft| draft.id() == model.id())
                });
            if edits_this_entity {
                form.rebase(model);
                return;
            }
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
