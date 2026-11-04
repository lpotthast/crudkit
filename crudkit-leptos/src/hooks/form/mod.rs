//! Create, edit, and read forms.

mod state;

pub use state::{CrudFieldConflict, CrudFormState};
pub(crate) use state::{FieldTarget, conflict_message, register_error_summary};

use crate::config::{CrudEntityViewKind, CrudText};
use crate::hooks::instance::use_crud_instance;
use crate::hooks::notify::{CrudNotification, CrudNotifier};
use crate::hooks::texts::{CrudTexts, use_crud_texts};
use crate::instance::CrudInstanceContext;
use crate::instance::CrudNavigation;
use crudkit_core::id::SerializableId;
use crudkit_core::validation::PartialSerializableAggregateViolations;
use crudkit_web::data_source::{
    CrudCompletion, CrudCreateRequest, CrudFieldProblem, CrudReadRequest, CrudSaved,
    CrudUpdateRequest, CrudVersion,
};
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
    /// Why the last save failed, while it applies, see [`CrudFormState::save_error`]. Always
    /// `None` in details.
    pub save_error: Signal<Option<RequestError>>,
    /// The problems of fields that the last save reported, see [`CrudFormState::problems`].
    pub problems: Signal<Vec<CrudFieldProblem>>,
    /// The members that someone else changed too, see [`CrudFormState::conflicts`].
    pub conflicts: Signal<Vec<CrudFieldConflict>>,
    /// Whether the last reload could not apply the user's changes, see
    /// [`CrudFormState::rebase_failed`].
    pub rebase_failed: Signal<bool>,
    /// Grows whenever the form asks for the user's attention, see [`CrudFormState::alerts`].
    pub alerts: Signal<u64>,
    /// Reloads the form's entity to show its current state, e.g. after a save was refused as
    /// [outdated](RequestError::Outdated). A form with unsaved changes keeps them. `None` in create
    /// forms.
    pub reload: Option<Callback<()>>,
    /// Where the form's fields are rendered, by the names of the fields, once for every mounted
    /// field atom.
    pub(crate) field_targets: Signal<HashMap<String, Vec<FieldTarget>>>,
    /// The number of mounted error summaries of the form.
    pub(crate) error_summaries: StoredValue<usize>,
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
    pub on_saved: Option<Callback<CrudSaved>>,
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

    let form = CrudFormState::<DynCreateField>::new();
    let save_action = create_action(&ctx, notifications, &form);
    let is_saving: Signal<bool> = save_action.pending().into();
    let form = form.saving_while(is_saving);
    let default_model = default_create_model(&ctx);
    let values = ctx
        .static_config
        .read_value()
        .model_handler
        .create_model_values(&default_model);
    form.load(default_model, values);
    navigation.guard(form.is_dirty);

    let save_result = save_action.value();
    let reset = move || {
        let model = default_create_model(&ctx);
        let values = ctx
            .static_config
            .read_value()
            .model_handler
            .create_model_values(&model);
        form.sync(model, values);
    };
    // The outcome was already notified by the request. The form applies it while it is mounted.
    Effect::new(move |_prev| {
        let Some(SaveOutcome {
            result, follow_up, ..
        }) = save_result.get()
        else {
            return;
        };
        // Callbacks may read signals. They must not become dependencies of this effect, or a
        // change would replay the save result.
        untrack(move || match result {
            Ok(saved) => {
                form.record_save_error(None);
                violations.set(Some(saved.violations.clone()));
                let id = saved.entity.id();
                if let Some(on_saved) = on_saved {
                    on_saved.run(saved);
                }
                follow_up.perform(ctx.navigation, id, CrudEntityViewKind::Create, reset);
            }
            Err(request_error) => {
                form.record_save_error(Some(request_error.clone()));
                if let Some(on_save_failed) = on_save_failed {
                    on_save_failed.run(request_error);
                }
            }
        });
    });

    let save = Callback::new(move |follow_up: CrudSaveFollowUp| {
        if let Some(draft) = savable_draft(&form) {
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
    form: &CrudFormState<F>,
) -> Signal<CrudEntityLoadStatus> {
    let form = *form;
    // A memo, so that views keyed on the status re-render only when it changes.
    Memo::new(move |_| match &*entity.read() {
        LoadState::Loaded(_) if form.is_ready.get() => CrudEntityLoadStatus::Ready,
        LoadState::Loading | LoadState::Loaded(_) => CrudEntityLoadStatus::Loading,
        LoadState::NotFound => CrudEntityLoadStatus::NotFound,
        LoadState::Failed(error) => CrudEntityLoadStatus::Failed(error.clone()),
    })
    .into()
}

/// The outcome of a save request.
#[derive(Debug, Clone)]
struct SaveOutcome {
    result: Result<CrudSaved, RequestError>,
    /// What the form does after a successful save.
    follow_up: CrudSaveFollowUp,
    /// Whether the outcome no longer applies: a refusal as outdated of a version that a reload
    /// replaced while the request was in flight. Saving again is based on the reloaded version.
    stale: bool,
}

/// Creates the action sending create requests for the instance of `ctx` from `form`. It notifies
/// the outcome as `notifications` ask, also when the form is gone by then, but no failure while
/// the form's error summary shows it.
// TODO: Can we get rid of new_local?
fn create_action(
    ctx: &CrudInstanceContext,
    notifications: CrudSaveNotifications,
    form: &CrudFormState<DynCreateField>,
) -> Action<(DynCreateModel, CrudSaveFollowUp), SaveOutcome> {
    let (ctx, form) = (*ctx, *form);
    let notify = SaveNotifier::new(&ctx, notifications);
    Action::new_local(
        move |(create_model, follow_up): &(DynCreateModel, CrudSaveFollowUp)| {
            let entity = create_model.clone();
            let follow_up = *follow_up;
            let data_source = ctx.data_source();
            let scope = ctx.scope.get_untracked();
            async move {
                let result = data_source
                    .create(&scope, CrudCreateRequest::new(entity))
                    .await;
                notify.outcome(&result, SaveKind::Create, form.shows_failures());
                SaveOutcome {
                    result,
                    follow_up,
                    stale: false,
                }
            }
        },
    )
}

/// Creates the action sending update requests for the instance of `ctx` from `form`. It notifies
/// the outcome as `notifications` ask, also when the form is gone by then, but no failure while
/// the form's error summary shows it, and no stale outcome.
fn update_action(
    ctx: &CrudInstanceContext,
    notifications: CrudSaveNotifications,
    form: &CrudFormState<DynUpdateField>,
) -> Action<(CrudUpdateRequest, CrudSaveFollowUp), SaveOutcome> {
    let (ctx, form) = (*ctx, *form);
    let notify = SaveNotifier::new(&ctx, notifications);
    Action::new_local(
        move |(request, follow_up): &(CrudUpdateRequest, CrudSaveFollowUp)| {
            let request = request.clone();
            let based_on = request.version().cloned();
            let follow_up = *follow_up;
            let data_source = ctx.data_source();
            let scope = ctx.scope.get_untracked();
            async move {
                let result = data_source.update(&scope, request).await;
                let stale = matches!(result, Err(RequestError::Outdated(_)))
                    && is_replaced(&form, based_on.as_ref());
                if !stale {
                    notify.outcome(&result, SaveKind::Update, form.shows_failures());
                }
                SaveOutcome {
                    result,
                    follow_up,
                    stale,
                }
            }
        },
    )
}

/// Returns whether `form` holds another version of its entity than `version` by now, because a
/// reload replaced it. `false` once the form is gone.
fn is_replaced(form: &CrudFormState<DynUpdateField>, version: Option<&CrudVersion>) -> bool {
    form.baseline
        .try_with_untracked(|baseline| {
            baseline
                .as_ref()
                .is_some_and(|baseline| baseline.version() != version)
        })
        .unwrap_or(false)
}

/// What a save does, which chooses the messages of its outcomes.
#[derive(Clone, Copy)]
enum SaveKind {
    Create,
    Update,
}

impl SaveKind {
    /// The title and the message after a completed save.
    fn completed(self) -> (CrudText<'static>, CrudText<'static>) {
        match self {
            Self::Create => (CrudText::SaveCreatedTitle, CrudText::SaveCreated),
            Self::Update => (CrudText::SaveUpdatedTitle, CrudText::SaveUpdated),
        }
    }

    /// The title and the message after a save that the data source accepted, and still applies.
    fn accepted(self) -> (CrudText<'static>, CrudText<'static>) {
        match self {
            Self::Create => (CrudText::SaveCreatedTitle, CrudText::SaveCreateAccepted),
            Self::Update => (
                CrudText::SaveUpdateAcceptedTitle,
                CrudText::SaveUpdateAccepted,
            ),
        }
    }

    /// The title after a failed save, whose message is the error.
    fn failed(self) -> CrudText<'static> {
        match self {
            Self::Create => CrudText::SaveCreateFailedTitle,
            Self::Update => CrudText::SaveUpdateFailedTitle,
        }
    }
}

/// Notifies the outcomes of save requests through the instance's notifier.
#[derive(Clone, Copy)]
struct SaveNotifier {
    notifier: CrudNotifier,
    texts: CrudTexts,
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

    /// Notifies `result` of a save of `kind`, as the notifications ask. A save refused as
    /// outdated is a warning, which asks the user to look at the current state. A failure that
    /// the form's error summary shows (`form_shows_failures`) is not notified, so that screen
    /// readers do not announce it twice.
    fn outcome(
        self,
        result: &Result<CrudSaved, RequestError>,
        kind: SaveKind,
        form_shows_failures: bool,
    ) {
        if result.is_err() && form_shows_failures {
            if let Err(error) = result {
                tracing::warn!(%error, "could not save the entity");
            }
            return;
        }
        let text = |text: CrudText<'_>| self.texts.get_untracked(text);
        let notification = match result {
            Ok(saved) if self.notifications == CrudSaveNotifications::All => {
                let (title, message) = match saved.completion {
                    CrudCompletion::Completed => kind.completed(),
                    CrudCompletion::Accepted => kind.accepted(),
                };
                CrudNotification::success(text(title), text(message))
            }
            Err(error @ RequestError::Outdated(_))
                if self.notifications != CrudSaveNotifications::None =>
            {
                tracing::warn!(%error, "could not save the entity");
                CrudNotification::warning(
                    text(CrudText::SaveOutdatedTitle),
                    text(CrudText::FormOutdated),
                )
            }
            Err(error) if self.notifications != CrudSaveNotifications::None => {
                tracing::warn!(%error, "could not save the entity");
                CrudNotification::error(
                    text(kind.failed()),
                    self.texts.request_error_untracked(error),
                )
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
fn savable_draft<F: TypeErasedField>(form: &CrudFormState<F>) -> Option<F::Model> {
    if form.has_errors.get_untracked() {
        tracing::warn!("not saving a draft with input errors");
        return None;
    }
    form.draft.get_untracked()
}

/// Returns the create model's default, with the IDs of the scope's parents in the fields that
/// refer to them. Data sources that place new entities below their parents, e.g. in a path, need
/// no such fields, so the create model may lack them.
fn default_create_model(ctx: &CrudInstanceContext) -> DynCreateModel {
    let model_handler = ctx.static_config.read_value().model_handler;
    let mut entity = model_handler.default_create_model();
    for parent in ctx.scope.get_untracked().parents {
        if let Some(field) = model_handler.create_model_field(&parent.field_name) {
            field.set_value(&mut entity, parent.value.into());
        } else {
            tracing::debug!(
                field = %parent.field_name,
                "the create model has no field referring to the parent; not prefilling it"
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
    pub on_saved: Option<Callback<CrudSaved>>,
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
    /// Reloads the entity to show its current state, e.g. after a save was refused as
    /// [outdated](RequestError::Outdated). The draft keeps its unsaved changes, which then apply
    /// to the reloaded version.
    pub reload: Callback<()>,
}

/// Loads an entity of the surrounding instance and creates a form editing it.
///
/// A changed draft guards the current navigation. Saving sends the draft and the baseline it
/// started from, the version that was loaded or saved last. A failed save keeps the draft, and the
/// form's [`save_error`](CrudFormState::save_error) tells why: a save refused as
/// [outdated](RequestError::Outdated) asks for [`UseCrudEditFormReturn::reload`], and the problems
/// of [invalid fields](RequestError::InvalidFields) reach their fields.
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
    let form = CrudFormState::<DynUpdateField>::new();
    let save_action = update_action(&ctx, notifications, &form);
    let is_saving: Signal<bool> = save_action.pending().into();
    let form = form.saving_while(is_saving);
    load_into_form(&ctx, entity, &form);
    navigation.guard(form.is_dirty);

    let save_result = save_action.value();
    // The outcome was already notified by the request. The form applies it while it is mounted.
    Effect::new(move |_prev| {
        let Some(SaveOutcome {
            result,
            follow_up,
            stale,
        }) = save_result.get()
        else {
            return;
        };
        if stale {
            return;
        }
        // Callbacks may read signals. They must not become dependencies of this effect, or a
        // change would replay the save result.
        untrack(move || match result {
            Ok(saved) => {
                violations.set(Some(saved.violations.clone()));
                // The server may have normalized the entity. The form continues from its version.
                let values = ctx
                    .static_config
                    .read_value()
                    .model_handler
                    .update_model_values(&saved.entity);
                let saved_id = saved.entity.id();
                form.sync(saved.entity.clone(), values);
                if let Some(on_saved) = on_saved {
                    on_saved.run(saved);
                }
                follow_up.perform(navigation, saved_id, CrudEntityViewKind::Update, || {});
            }
            Err(request_error) => {
                // The draft stays as it is, also when the entity changed meanwhile: reloading it
                // applies the draft's changes to the current version.
                form.record_save_error(Some(request_error.clone()));
                if let Some(on_save_failed) = on_save_failed {
                    on_save_failed.run(request_error);
                }
            }
        });
    });

    let is_dirty = form.is_dirty;
    let save = Callback::new(move |follow_up: CrudSaveFollowUp| {
        let (Some(draft), Some(baseline)) = (savable_draft(&form), form.baseline.get_untracked())
        else {
            return;
        };
        let request = CrudUpdateRequest::new(id.get_untracked(), draft, baseline);
        save_action.dispatch((request, follow_up));
    });
    // Deletes the entity as the data source last returned it, the baseline: the draft may hold
    // unsaved changes, even of its ID fields, and the baseline holds the version that the
    // deletion is based on, also after a save.
    let delete = Callback::new(move |()| {
        if let Some(entity) = form.baseline.get_untracked() {
            ctx.deletion.for_navigation(navigation).request(entity);
        }
    });
    let reload = Callback::new(move |()| ctx.reload());

    UseCrudEditFormReturn {
        entity,
        status: entity_status(entity, &form),
        violations: violations.into(),
        form,
        save,
        is_saving,
        can_save: Signal::derive(move || {
            !is_saving.get() && is_dirty.get() && !form.has_errors.get()
        }),
        delete,
        can_delete: Signal::derive(move || !is_saving.get() && entity.read().loaded().is_some()),
        reload,
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
    /// Reloads the entity to show its current state.
    pub reload: Callback<()>,
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
    load_into_form(&ctx, entity, &form);

    let delete = Callback::new(move |()| {
        if let Some(entity) = entity.get_untracked().into_loaded() {
            ctx.deletion.for_navigation(navigation).request(entity);
        }
    });

    UseCrudReadReturn {
        entity,
        status: entity_status(entity, &form),
        form,
        delete,
        can_delete: Signal::derive(move || entity.read().loaded().is_some()),
        reload: Callback::new(move |()| ctx.reload()),
    }
}

/// Loads the entity identified by `id` in the instance's scope.
fn use_entity(
    ctx: &CrudInstanceContext,
    id: Signal<SerializableId>,
) -> Signal<LoadState<DynUpdateModel>> {
    let ctx = *ctx;
    // TODO: Do not use LocalResource, allow loading on the server.
    let resource = LocalResource::new(move || async move {
        let _ = ctx.reload.get();
        let id = id.get();
        let scope = ctx.scope.get();
        ctx.data_source()
            .read(&scope, CrudReadRequest::new(id))
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
/// A reload of the entity the form holds unsaved changes or unreadable input of keeps them: the
/// changes apply to the reloaded version, which becomes what they are compared against, see
/// [`CrudFormState::rebase`]. Discarding them is up to the user, e.g. by leaving the view. A clean
/// form takes the reloaded version, and its rendered fields update in place. Another entity
/// replaces the form's field values.
fn load_into_form(
    ctx: &CrudInstanceContext,
    entity: Signal<LoadState<DynUpdateModel>>,
    form: &CrudFormState<DynUpdateField>,
) {
    let (ctx, form) = (*ctx, *form);
    Effect::new(move |_prev| {
        if let LoadState::Loaded(model) = entity.get() {
            let models = ctx.static_config.read_value().model_handler;
            // The baseline, not the draft, because the user may have changed ID fields.
            let shows_this_entity = form.baseline.with_untracked(|baseline| {
                baseline
                    .as_ref()
                    .is_some_and(|baseline| baseline.id() == model.id())
            });
            let holds_input = form.is_dirty.get_untracked() || form.has_errors.get_untracked();
            if shows_this_entity && holds_input {
                form.rebase(model, models);
            } else if shows_this_entity {
                let values = models.update_model_values(&model);
                form.sync(model, values);
            } else {
                let values = models.update_model_values(&model);
                form.load(model, values);
            }
        }
    });
}
